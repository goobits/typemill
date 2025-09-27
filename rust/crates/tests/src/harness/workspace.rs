use std::fs;
use std::path::{Path, PathBuf};
use tempfile::{tempdir, TempDir};

/// Manages a temporary directory for a test scenario.
/// Cleans up automatically when dropped.
pub struct TestWorkspace {
    pub temp_dir: TempDir,
}

impl TestWorkspace {
    /// Creates a new empty workspace.
    pub fn new() -> Self {
        Self {
            temp_dir: tempdir().expect("Failed to create temp dir"),
        }
    }

    /// Returns the root path of the workspace.
    pub fn path(&self) -> &Path {
        self.temp_dir.path()
    }

    /// Creates a file with content within the workspace.
    /// Automatically creates parent directories.
    pub fn create_file(&self, rel_path: &str, content: &str) {
        let file_path = self.path().join(rel_path);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).expect("Failed to create parent dirs");
        }
        fs::write(file_path, content).expect("Failed to write file");
    }

    /// Creates a directory within the workspace.
    pub fn create_directory(&self, rel_path: &str) {
        let dir_path = self.path().join(rel_path);
        fs::create_dir_all(dir_path).expect("Failed to create directory");
    }

    /// Reads a file from the workspace.
    pub fn read_file(&self, rel_path: &str) -> String {
        let file_path = self.path().join(rel_path);
        fs::read_to_string(file_path).expect("Failed to read file")
    }

    /// Check if a file exists in the workspace.
    pub fn file_exists(&self, rel_path: &str) -> bool {
        self.path().join(rel_path).exists()
    }

    /// Get the absolute path to a file in the workspace.
    pub fn absolute_path(&self, rel_path: &str) -> PathBuf {
        self.path().join(rel_path)
    }

    /// Create a TypeScript configuration file.
    pub fn create_tsconfig(&self) {
        let tsconfig = serde_json::json!({
            "compilerOptions": {
                "target": "ES2022",
                "module": "ESNext",
                "moduleResolution": "node",
                "esModuleInterop": true,
                "allowSyntheticDefaultImports": true,
                "strict": true,
                "skipLibCheck": true,
                "forceConsistentCasingInFileNames": true,
                "resolveJsonModule": true,
                "isolatedModules": true,
                "noEmit": true
            },
            "include": ["src/**/*"],
            "exclude": ["node_modules"]
        });

        self.create_file(
            "tsconfig.json",
            &serde_json::to_string_pretty(&tsconfig).unwrap()
        );
    }

    /// Create a package.json file for a TypeScript/JavaScript project.
    pub fn create_package_json(&self, name: &str) {
        let package_json = serde_json::json!({
            "name": name,
            "version": "1.0.0",
            "type": "module",
            "dependencies": {},
            "devDependencies": {
                "typescript": "^5.0.0"
            }
        });

        self.create_file(
            "package.json",
            &serde_json::to_string_pretty(&package_json).unwrap()
        );
    }

    /// Create a basic TypeScript project structure.
    pub fn setup_typescript_project(&self, name: &str) {
        self.create_package_json(name);
        self.create_tsconfig();
        self.create_directory("src");
    }
}

impl Default for TestWorkspace {
    fn default() -> Self {
        Self::new()
    }
}