import { copyFileSync, existsSync, mkdirSync, readdirSync, statSync } from 'node:fs';
import { dirname, join } from 'node:path';

export interface BackupEntry {
  originalPath: string;
  backupPath: string;
  isDirectory: boolean;
}

export class FileBackupManager {
  private backups: BackupEntry[] = [];
  private backupDir: string;

  constructor(backupDir = '/tmp/cclsp-test-backups') {
    this.backupDir = backupDir;
    this.ensureBackupDir();
  }

  private ensureBackupDir(): void {
    if (!existsSync(this.backupDir)) {
      mkdirSync(this.backupDir, { recursive: true });
    }
  }

  /**
   * Create backup of a single file
   */
  backupFile(filePath: string): string {
    if (!existsSync(filePath)) {
      throw new Error(`File not found: ${filePath}`);
    }

    const timestamp = Date.now();
    const fileName = filePath.replace(/[/\\]/g, '_');
    const backupPath = join(this.backupDir, `${fileName}_${timestamp}.backup`);

    copyFileSync(filePath, backupPath);

    this.backups.push({
      originalPath: filePath,
      backupPath,
      isDirectory: false,
    });

    return backupPath;
  }

  /**
   * Create backups of all files in a directory (non-recursive)
   */
  backupDirectory(directoryPath: string, extensions?: string[]): string[] {
    if (!existsSync(directoryPath)) {
      throw new Error(`Directory not found: ${directoryPath}`);
    }

    const files = readdirSync(directoryPath);
    const backedUpFiles: string[] = [];

    for (const file of files) {
      const fullPath = join(directoryPath, file);
      const stat = statSync(fullPath);

      if (stat.isFile()) {
        // Check extension filter if provided
        if (extensions) {
          const ext = file.split('.').pop()?.toLowerCase();
          if (!ext || !extensions.includes(ext)) {
            continue;
          }
        }

        const backupPath = this.backupFile(fullPath);
        backedUpFiles.push(backupPath);
      }
    }

    return backedUpFiles;
  }

  /**
   * Recursively backup all files in a directory tree
   */
  backupDirectoryRecursive(directoryPath: string, extensions?: string[]): string[] {
    if (!existsSync(directoryPath)) {
      throw new Error(`Directory not found: ${directoryPath}`);
    }

    const backedUpFiles: string[] = [];

    const processDirectory = (currentPath: string) => {
      const items = readdirSync(currentPath);

      for (const item of items) {
        const fullPath = join(currentPath, item);
        const stat = statSync(fullPath);

        if (stat.isDirectory()) {
          // Skip node_modules and other common directories
          if (['node_modules', '.git', 'dist', 'build'].includes(item)) {
            continue;
          }
          processDirectory(fullPath);
        } else if (stat.isFile()) {
          // Check extension filter if provided
          if (extensions) {
            const ext = item.split('.').pop()?.toLowerCase();
            if (!ext || !extensions.includes(ext)) {
              continue;
            }
          }

          const backupPath = this.backupFile(fullPath);
          backedUpFiles.push(backupPath);
        }
      }
    };

    processDirectory(directoryPath);
    return backedUpFiles;
  }

  /**
   * Restore a specific file from its backup
   */
  restoreFile(originalPath: string): boolean {
    const backup = this.backups.find((b) => b.originalPath === originalPath);
    if (!backup) {
      return false;
    }

    if (existsSync(backup.backupPath)) {
      // Ensure target directory exists
      const targetDir = dirname(originalPath);
      if (!existsSync(targetDir)) {
        mkdirSync(targetDir, { recursive: true });
      }

      copyFileSync(backup.backupPath, originalPath);
      return true;
    }

    return false;
  }

  /**
   * Restore all backed up files
   */
  restoreAll(): number {
    let restored = 0;

    for (const backup of this.backups) {
      if (this.restoreFile(backup.originalPath)) {
        restored++;
      }
    }

    return restored;
  }

  /**
   * Clean up all backup files
   */
  cleanup(): void {
    // Note: We don't delete backup files automatically to allow manual inspection
    // In a real scenario, you might want to clean them up after successful tests
    this.backups = [];
  }

  /**
   * Get list of all backed up files
   */
  getBackedUpFiles(): string[] {
    return this.backups.map((b) => b.originalPath);
  }

  /**
   * Check if a file has been backed up
   */
  hasBackup(filePath: string): boolean {
    return this.backups.some((b) => b.originalPath === filePath);
  }
}
