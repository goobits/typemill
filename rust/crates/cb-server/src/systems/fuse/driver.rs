//! FUSE filesystem driver implementation

use cb_core::config::FuseConfig;
use fuser::{
    FileAttr, FileType, Filesystem, ReplyAttr, ReplyData, ReplyDirectory, ReplyEntry, Request,
    ReplyOpen,
};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tracing::{debug, error, info, warn};

const TTL: Duration = Duration::from_secs(1);

/// FUSE filesystem implementation for Codeflow Buddy
pub struct CodeflowFS {
    /// Path to the real workspace on disk
    workspace_path: PathBuf,
    /// Cache of file attributes to avoid repeated filesystem calls
    attr_cache: HashMap<u64, FileAttr>,
    /// Next available inode number
    next_inode: u64,
    /// Mapping from inode to real path
    inode_to_path: HashMap<u64, PathBuf>,
    /// Mapping from path to inode
    path_to_inode: HashMap<PathBuf, u64>,
}

impl CodeflowFS {
    /// Create a new CodeflowFS instance
    pub fn new(workspace_path: PathBuf) -> Self {
        let mut fs = Self {
            workspace_path: workspace_path.clone(),
            attr_cache: HashMap::new(),
            next_inode: 2, // Start at 2, as 1 is reserved for root
            inode_to_path: HashMap::new(),
            path_to_inode: HashMap::new(),
        };

        // Initialize root directory
        fs.inode_to_path.insert(1, workspace_path.clone());
        fs.path_to_inode.insert(workspace_path, 1);

        fs
    }

    /// Get or assign an inode for a given path
    fn get_or_assign_inode(&mut self, path: &Path) -> u64 {
        if let Some(&inode) = self.path_to_inode.get(path) {
            return inode;
        }

        let inode = self.next_inode;
        self.next_inode += 1;
        self.inode_to_path.insert(inode, path.to_path_buf());
        self.path_to_inode.insert(path.to_path_buf(), inode);
        inode
    }

    /// Convert a filesystem metadata to FUSE FileAttr
    fn metadata_to_attr(&self, metadata: &fs::Metadata, ino: u64) -> FileAttr {
        let file_type = if metadata.is_dir() {
            FileType::Directory
        } else if metadata.is_file() {
            FileType::RegularFile
        } else if metadata.file_type().is_symlink() {
            FileType::Symlink
        } else {
            FileType::RegularFile // Default fallback
        };

        let atime = metadata
            .accessed()
            .unwrap_or(UNIX_EPOCH)
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0));

        let mtime = metadata
            .modified()
            .unwrap_or(UNIX_EPOCH)
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0));

        let ctime = metadata
            .created()
            .unwrap_or(UNIX_EPOCH)
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0));

        FileAttr {
            ino,
            size: metadata.len(),
            blocks: (metadata.len() + 511) / 512, // 512-byte blocks
            atime: SystemTime::UNIX_EPOCH + atime,
            mtime: SystemTime::UNIX_EPOCH + mtime,
            ctime: SystemTime::UNIX_EPOCH + ctime,
            crtime: SystemTime::UNIX_EPOCH + ctime,
            kind: file_type,
            perm: if metadata.is_dir() { 0o755 } else { 0o644 },
            nlink: 1,
            uid: 1000, // Default user ID
            gid: 1000, // Default group ID
            rdev: 0,
            flags: 0,
            blksize: 4096,
        }
    }

    /// Resolve a path relative to the workspace
    fn resolve_path(&self, parent_ino: u64, name: &OsStr) -> Option<PathBuf> {
        let parent_path = self.inode_to_path.get(&parent_ino)?;
        Some(parent_path.join(name))
    }
}

impl Filesystem for CodeflowFS {
    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        debug!("lookup: parent={}, name={:?}", parent, name);

        let path = match self.resolve_path(parent, name) {
            Some(path) => path,
            None => {
                reply.error(libc::ENOENT);
                return;
            }
        };

        match fs::metadata(&path) {
            Ok(metadata) => {
                let ino = self.get_or_assign_inode(&path);
                let attr = self.metadata_to_attr(&metadata, ino);
                self.attr_cache.insert(ino, attr);
                reply.entry(&TTL, &attr, 0);
            }
            Err(err) => {
                warn!("Failed to get metadata for {:?}: {}", path, err);
                reply.error(libc::ENOENT);
            }
        }
    }

    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        debug!("getattr: ino={}", ino);

        // Check cache first
        if let Some(attr) = self.attr_cache.get(&ino) {
            reply.attr(&TTL, attr);
            return;
        }

        let path = match self.inode_to_path.get(&ino) {
            Some(path) => path.clone(),
            None => {
                reply.error(libc::ENOENT);
                return;
            }
        };

        match fs::metadata(&path) {
            Ok(metadata) => {
                let attr = self.metadata_to_attr(&metadata, ino);
                self.attr_cache.insert(ino, attr);
                reply.attr(&TTL, &attr);
            }
            Err(err) => {
                warn!("Failed to get metadata for {:?}: {}", path, err);
                reply.error(libc::ENOENT);
            }
        }
    }

    fn readdir(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        mut reply: ReplyDirectory,
    ) {
        debug!("readdir: ino={}, offset={}", ino, offset);

        let path = match self.inode_to_path.get(&ino) {
            Some(path) => path.clone(),
            None => {
                reply.error(libc::ENOENT);
                return;
            }
        };

        let entries = match fs::read_dir(&path) {
            Ok(entries) => entries,
            Err(err) => {
                warn!("Failed to read directory {:?}: {}", path, err);
                reply.error(libc::ENOENT);
                return;
            }
        };

        let mut dir_entries = Vec::new();

        // Add "." and ".." entries
        if offset == 0 {
            dir_entries.push((ino, FileType::Directory, ".".to_string()));
        }
        if offset <= 1 {
            // For simplicity, use parent inode as 1 (root) for ".."
            let parent_ino = if ino == 1 { 1 } else { 1 };
            dir_entries.push((parent_ino, FileType::Directory, "..".to_string()));
        }

        // Add actual directory entries
        for (i, entry) in entries.enumerate() {
            if (i as i64) < offset.saturating_sub(2) {
                continue;
            }

            let entry = match entry {
                Ok(entry) => entry,
                Err(err) => {
                    warn!("Failed to read directory entry: {}", err);
                    continue;
                }
            };

            let entry_path = entry.path();
            let file_name = match entry_path.file_name() {
                Some(name) => name.to_string_lossy().to_string(),
                None => continue,
            };

            let file_type = match entry.file_type() {
                Ok(ft) => {
                    if ft.is_dir() {
                        FileType::Directory
                    } else if ft.is_file() {
                        FileType::RegularFile
                    } else if ft.is_symlink() {
                        FileType::Symlink
                    } else {
                        FileType::RegularFile
                    }
                }
                Err(_) => FileType::RegularFile,
            };

            let entry_ino = self.get_or_assign_inode(&entry_path);
            dir_entries.push((entry_ino, file_type, file_name));
        }

        // Send entries to FUSE
        for (i, (entry_ino, file_type, name)) in dir_entries.iter().enumerate() {
            let offset = (i as i64) + 1;
            if reply.add(*entry_ino, offset, *file_type, name) {
                break; // Buffer is full
            }
        }

        reply.ok();
    }

    fn open(&mut self, _req: &Request, ino: u64, _flags: i32, reply: ReplyOpen) {
        debug!("open: ino={}", ino);

        let path = match self.inode_to_path.get(&ino) {
            Some(path) => path,
            None => {
                reply.error(libc::ENOENT);
                return;
            }
        };

        // For simplicity, just check if the file exists
        if path.exists() && path.is_file() {
            reply.opened(0, 0); // fh=0, flags=0
        } else {
            reply.error(libc::ENOENT);
        }
    }

    fn read(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        size: u32,
        _flags: i32,
        _lock: Option<u64>,
        reply: ReplyData,
    ) {
        debug!("read: ino={}, offset={}, size={}", ino, offset, size);

        let path = match self.inode_to_path.get(&ino) {
            Some(path) => path,
            None => {
                reply.error(libc::ENOENT);
                return;
            }
        };

        match fs::read(path) {
            Ok(data) => {
                let start = offset as usize;
                let end = std::cmp::min(start + size as usize, data.len());

                if start >= data.len() {
                    reply.data(&[]);
                } else {
                    reply.data(&data[start..end]);
                }
            }
            Err(err) => {
                warn!("Failed to read file {:?}: {}", path, err);
                reply.error(libc::EIO);
            }
        }
    }
}

/// Start the FUSE mount in a background thread
pub fn start_fuse_mount(config: &FuseConfig, workspace_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mount_point = config.mount_point.clone();
    let workspace_path = workspace_path.to_path_buf();

    info!("Starting FUSE mount: {} -> {}", workspace_path.display(), mount_point.display());

    // Validate mount point exists
    if !mount_point.exists() {
        return Err(format!("Mount point {:?} does not exist", mount_point).into());
    }

    if !mount_point.is_dir() {
        return Err(format!("Mount point {:?} is not a directory", mount_point).into());
    }

    // Validate workspace path exists
    if !workspace_path.exists() {
        return Err(format!("Workspace path {:?} does not exist", workspace_path).into());
    }

    if !workspace_path.is_dir() {
        return Err(format!("Workspace path {:?} is not a directory", workspace_path).into());
    }

    let filesystem = CodeflowFS::new(workspace_path.clone());

    // Spawn FUSE mount in a background thread
    let mount_point_clone = mount_point.clone();
    std::thread::spawn(move || {
        info!("Mounting FUSE filesystem at {:?}", mount_point_clone);

        let options = vec![
            fuser::MountOption::RO, // Read-only mount
            fuser::MountOption::FSName("codeflow-buddy".to_string()),
            fuser::MountOption::AutoUnmount,
        ];

        if let Err(err) = fuser::mount2(filesystem, &mount_point_clone, &options) {
            error!("FUSE mount failed: {}", err);
        } else {
            info!("FUSE filesystem unmounted");
        }
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_codeflow_fs_creation() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_path = temp_dir.path().to_path_buf();

        let fs = CodeflowFS::new(workspace_path.clone());

        assert_eq!(fs.workspace_path, workspace_path);
        assert_eq!(fs.next_inode, 2);
        assert!(fs.inode_to_path.contains_key(&1));
        assert!(fs.path_to_inode.contains_key(&workspace_path));
    }

    #[test]
    fn test_get_or_assign_inode() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_path = temp_dir.path().to_path_buf();
        let mut fs = CodeflowFS::new(workspace_path);

        let test_path = Path::new("/test/path");
        let inode1 = fs.get_or_assign_inode(test_path);
        let inode2 = fs.get_or_assign_inode(test_path);

        assert_eq!(inode1, inode2); // Should return same inode for same path
        assert_eq!(inode1, 2); // First assigned inode should be 2
    }

    #[test]
    fn test_metadata_to_attr() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_path = temp_dir.path().to_path_buf();
        let fs = CodeflowFS::new(workspace_path.clone());

        let metadata = fs::metadata(&workspace_path).unwrap();
        let attr = fs.metadata_to_attr(&metadata, 1);

        assert_eq!(attr.ino, 1);
        assert_eq!(attr.kind, FileType::Directory);
        assert_eq!(attr.perm, 0o755);
    }
}