//! Logic for the extract_module_to_package refactoring tool.
//!
//! This module provides language-agnostic package extraction capabilities
//! using a trait-based adapter pattern to support multiple languages.

use crate::error::AstResult;
use async_trait::async_trait;
use cb_api::EditPlan;
use cb_core::language::ProjectLanguage;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct ExtractModuleToPackageParams {
    pub source_package: String,
    pub module_path: String,
    pub target_package_path: String,
    pub target_package_name: String,
    pub update_imports: Option<bool>,
    pub create_manifest: Option<bool>,
    pub dry_run: Option<bool>,
}

/// Language-specific adapter for package extraction operations
///
/// This trait abstracts language-specific operations needed for extracting
/// modules to packages, enabling support for multiple programming languages.
#[async_trait]
pub trait LanguageAdapter: Send + Sync {
    /// Get the language this adapter supports
    fn language(&self) -> ProjectLanguage;

    /// Get the package manifest filename (e.g., "Cargo.toml", "package.json")
    fn manifest_filename(&self) -> &'static str;

    /// Get the source directory name (e.g., "src" for Rust/TS, "" for Python)
    fn source_dir(&self) -> &'static str;

    /// Get the entry point filename (e.g., "lib.rs", "index.ts", "__init__.py")
    fn entry_point(&self) -> &'static str;

    /// Get the module path separator (e.g., "::" for Rust, "." for Python/TS)
    fn module_separator(&self) -> &'static str;

    /// Locate module files given a module path within a package
    ///
    /// # Arguments
    ///
    /// * `package_path` - Path to the source package
    /// * `module_path` - Dotted module path (e.g., "services.planner")
    ///
    /// # Returns
    ///
    /// Vector of file paths that comprise the module
    async fn locate_module_files(
        &self,
        package_path: &Path,
        module_path: &str,
    ) -> AstResult<Vec<std::path::PathBuf>>;

    /// Parse imports/dependencies from a file
    ///
    /// # Arguments
    ///
    /// * `file_path` - Path to the file to analyze
    ///
    /// # Returns
    ///
    /// Vector of import statements/paths found in the file
    async fn parse_imports(&self, file_path: &Path) -> AstResult<Vec<String>>;

    /// Generate a package manifest for a new package
    ///
    /// # Arguments
    ///
    /// * `package_name` - Name of the new package
    /// * `dependencies` - List of dependencies the package needs
    ///
    /// # Returns
    ///
    /// String containing the manifest file content
    fn generate_manifest(&self, package_name: &str, dependencies: &[String]) -> String;

    /// Update an import statement from internal to external
    ///
    /// # Arguments
    ///
    /// * `old_import` - Original import path (e.g., "crate::services::planner")
    /// * `new_package_name` - New package name (e.g., "cb_planner")
    ///
    /// # Returns
    ///
    /// Updated import statement
    fn rewrite_import(&self, old_import: &str, new_package_name: &str) -> String;
}

/// Rust language adapter
pub struct RustAdapter;

#[async_trait]
impl LanguageAdapter for RustAdapter {
    fn language(&self) -> ProjectLanguage {
        ProjectLanguage::Rust
    }

    fn manifest_filename(&self) -> &'static str {
        "Cargo.toml"
    }

    fn source_dir(&self) -> &'static str {
        "src"
    }

    fn entry_point(&self) -> &'static str {
        "lib.rs"
    }

    fn module_separator(&self) -> &'static str {
        "::"
    }

    async fn locate_module_files(
        &self,
        package_path: &Path,
        module_path: &str,
    ) -> AstResult<Vec<std::path::PathBuf>> {
        use std::path::PathBuf;
        use tracing::debug;

        debug!(
            package_path = %package_path.display(),
            module_path = %module_path,
            "Locating Rust module files"
        );

        // Start at the crate's source root (e.g., package_path/src)
        let src_root = package_path.join(self.source_dir());

        if !src_root.exists() {
            return Err(crate::error::AstError::Analysis {
                message: format!("Source directory not found: {}", src_root.display()),
            });
        }

        // Split module path by either "::" or "." into segments
        let segments: Vec<&str> = module_path
            .split(|c| c == ':' || c == '.')
            .filter(|s| !s.is_empty())
            .collect();

        if segments.is_empty() {
            return Err(crate::error::AstError::Analysis {
                message: "Module path cannot be empty".to_string(),
            });
        }

        // Build path by joining segments
        let mut current_path = src_root.clone();

        // Navigate through all segments except the last
        for segment in &segments[..segments.len() - 1] {
            current_path = current_path.join(segment);
        }

        // For the final segment, check both naming conventions
        let final_segment = segments[segments.len() - 1];
        let mut found_files = Vec::new();

        // Check for module_name.rs
        let file_path = current_path.join(format!("{}.rs", final_segment));
        if file_path.exists() && file_path.is_file() {
            debug!(file_path = %file_path.display(), "Found module file");
            found_files.push(file_path);
        }

        // Check for module_name/mod.rs
        let mod_path = current_path.join(final_segment).join("mod.rs");
        if mod_path.exists() && mod_path.is_file() {
            debug!(file_path = %mod_path.display(), "Found mod.rs file");
            found_files.push(mod_path);
        }

        if found_files.is_empty() {
            return Err(crate::error::AstError::Analysis {
                message: format!(
                    "Module '{}' not found at {} (checked both {}.rs and {}/mod.rs)",
                    module_path,
                    current_path.display(),
                    final_segment,
                    final_segment
                ),
            });
        }

        debug!(
            files_count = found_files.len(),
            "Successfully located module files"
        );

        Ok(found_files)
    }

    async fn parse_imports(&self, _file_path: &Path) -> AstResult<Vec<String>> {
        unimplemented!("RustAdapter::parse_imports not yet implemented")
    }

    fn generate_manifest(&self, _package_name: &str, _dependencies: &[String]) -> String {
        unimplemented!("RustAdapter::generate_manifest not yet implemented")
    }

    fn rewrite_import(&self, _old_import: &str, _new_package_name: &str) -> String {
        unimplemented!("RustAdapter::rewrite_import not yet implemented")
    }
}

/// TypeScript/JavaScript language adapter
pub struct TypeScriptAdapter;

#[async_trait]
impl LanguageAdapter for TypeScriptAdapter {
    fn language(&self) -> ProjectLanguage {
        ProjectLanguage::TypeScript
    }

    fn manifest_filename(&self) -> &'static str {
        "package.json"
    }

    fn source_dir(&self) -> &'static str {
        "src"
    }

    fn entry_point(&self) -> &'static str {
        "index.ts"
    }

    fn module_separator(&self) -> &'static str {
        "."
    }

    async fn locate_module_files(
        &self,
        _package_path: &Path,
        _module_path: &str,
    ) -> AstResult<Vec<std::path::PathBuf>> {
        unimplemented!("TypeScriptAdapter::locate_module_files not yet implemented")
    }

    async fn parse_imports(&self, _file_path: &Path) -> AstResult<Vec<String>> {
        unimplemented!("TypeScriptAdapter::parse_imports not yet implemented")
    }

    fn generate_manifest(&self, _package_name: &str, _dependencies: &[String]) -> String {
        unimplemented!("TypeScriptAdapter::generate_manifest not yet implemented")
    }

    fn rewrite_import(&self, _old_import: &str, _new_package_name: &str) -> String {
        unimplemented!("TypeScriptAdapter::rewrite_import not yet implemented")
    }
}

/// Python language adapter
pub struct PythonAdapter;

#[async_trait]
impl LanguageAdapter for PythonAdapter {
    fn language(&self) -> ProjectLanguage {
        ProjectLanguage::Python
    }

    fn manifest_filename(&self) -> &'static str {
        "pyproject.toml"
    }

    fn source_dir(&self) -> &'static str {
        ""
    }

    fn entry_point(&self) -> &'static str {
        "__init__.py"
    }

    fn module_separator(&self) -> &'static str {
        "."
    }

    async fn locate_module_files(
        &self,
        _package_path: &Path,
        _module_path: &str,
    ) -> AstResult<Vec<std::path::PathBuf>> {
        unimplemented!("PythonAdapter::locate_module_files not yet implemented")
    }

    async fn parse_imports(&self, _file_path: &Path) -> AstResult<Vec<String>> {
        unimplemented!("PythonAdapter::parse_imports not yet implemented")
    }

    fn generate_manifest(&self, _package_name: &str, _dependencies: &[String]) -> String {
        unimplemented!("PythonAdapter::generate_manifest not yet implemented")
    }

    fn rewrite_import(&self, _old_import: &str, _new_package_name: &str) -> String {
        unimplemented!("PythonAdapter::rewrite_import not yet implemented")
    }
}

/// Main entry point for extracting a module to a package
///
/// This function orchestrates the extraction process by:
/// 1. Detecting the source package language
/// 2. Selecting the appropriate adapter
/// 3. Generating an EditPlan for the refactoring
pub async fn plan_extract_module_to_package(
    params: ExtractModuleToPackageParams,
) -> AstResult<EditPlan> {
    use cb_api::{EditPlanMetadata, ValidationRule, ValidationType};
    use cb_core::language::detect_project_language;
    use serde_json::json;
    use std::collections::HashMap;
    use tracing::{debug, info};

    info!(
        source_package = %params.source_package,
        module_path = %params.module_path,
        target_package = %params.target_package_path,
        "Planning extract_module_to_package operation"
    );

    // Step 1: Detect language from source package
    let source_path = Path::new(&params.source_package);
    let detected_language = detect_project_language(source_path);

    debug!(
        language = %detected_language.as_str(),
        "Detected project language"
    );

    // Step 2: Create appropriate language adapter based on detection
    let adapter: Box<dyn LanguageAdapter> = match detected_language {
        ProjectLanguage::Rust => {
            info!("Selected RustAdapter for extraction");
            Box::new(RustAdapter)
        }
        ProjectLanguage::TypeScript => {
            info!("Selected TypeScriptAdapter for extraction");
            Box::new(TypeScriptAdapter)
        }
        ProjectLanguage::Python => {
            info!("Selected PythonAdapter for extraction");
            Box::new(PythonAdapter)
        }
        ProjectLanguage::Go | ProjectLanguage::Java => {
            return Err(crate::error::AstError::UnsupportedSyntax {
                feature: format!(
                    "{} language not yet supported for extract_module_to_package",
                    detected_language.as_str()
                ),
            });
        }
        ProjectLanguage::Unknown => {
            return Err(crate::error::AstError::Analysis {
                message: "Could not detect project language - no manifest files found".to_string(),
            });
        }
    };

    // Step 3: Locate module files using the adapter
    let located_files = adapter
        .locate_module_files(source_path, &params.module_path)
        .await?;

    debug!(
        files_count = located_files.len(),
        "Located module files"
    );

    // Step 4: Generate EditPlan with located files in metadata
    // Convert PathBuf to strings for JSON serialization
    let located_files_strings: Vec<String> = located_files
        .iter()
        .map(|p| p.display().to_string())
        .collect();

    let edit_plan = EditPlan {
        source_file: params.source_package.clone(),
        edits: vec![],
        dependency_updates: vec![],
        validations: vec![ValidationRule {
            rule_type: ValidationType::SyntaxCheck,
            description: "Verify syntax is valid after extraction".to_string(),
            parameters: HashMap::new(),
        }],
        metadata: EditPlanMetadata {
            intent_name: "extract_module_to_package".to_string(),
            intent_arguments: json!({
                "source_package": params.source_package,
                "module_path": params.module_path,
                "target_package_path": params.target_package_path,
                "target_package_name": params.target_package_name,
                "adapter_selected": adapter.language().as_str(),
                "located_files": located_files_strings,
            }),
            created_at: chrono::Utc::now(),
            complexity: 1,
            impact_areas: vec!["package_extraction".to_string()],
        },
    };

    info!(
        adapter = %adapter.language().as_str(),
        files_count = located_files.len(),
        "Successfully created EditPlan with located files"
    );

    Ok(edit_plan)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_locate_module_files_single_file() {
        // Create a temporary Rust project structure
        let temp_dir = tempdir().unwrap();
        let src_dir = temp_dir.path().join("src");
        fs::create_dir(&src_dir).unwrap();

        // Create lib.rs
        fs::write(src_dir.join("lib.rs"), "// lib.rs").unwrap();

        // Create a module as a single file: src/my_module.rs
        fs::write(src_dir.join("my_module.rs"), "// my_module.rs").unwrap();

        let adapter = RustAdapter;
        let result = adapter
            .locate_module_files(temp_dir.path(), "my_module")
            .await;

        assert!(result.is_ok());
        let files = result.unwrap();
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("my_module.rs"));
    }

    #[tokio::test]
    async fn test_locate_module_files_mod_rs() {
        // Create a temporary Rust project structure
        let temp_dir = tempdir().unwrap();
        let src_dir = temp_dir.path().join("src");
        fs::create_dir(&src_dir).unwrap();

        // Create lib.rs
        fs::write(src_dir.join("lib.rs"), "// lib.rs").unwrap();

        // Create a module as a directory with mod.rs: src/my_module/mod.rs
        let module_dir = src_dir.join("my_module");
        fs::create_dir(&module_dir).unwrap();
        fs::write(module_dir.join("mod.rs"), "// mod.rs").unwrap();

        let adapter = RustAdapter;
        let result = adapter
            .locate_module_files(temp_dir.path(), "my_module")
            .await;

        assert!(result.is_ok());
        let files = result.unwrap();
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("my_module/mod.rs") || files[0].ends_with("my_module\\mod.rs"));
    }

    #[tokio::test]
    async fn test_locate_module_files_nested_module() {
        // Create a temporary Rust project structure
        let temp_dir = tempdir().unwrap();
        let src_dir = temp_dir.path().join("src");
        fs::create_dir(&src_dir).unwrap();

        // Create lib.rs
        fs::write(src_dir.join("lib.rs"), "// lib.rs").unwrap();

        // Create nested module structure: src/services/planner.rs
        let services_dir = src_dir.join("services");
        fs::create_dir(&services_dir).unwrap();
        fs::write(services_dir.join("planner.rs"), "// planner.rs").unwrap();

        let adapter = RustAdapter;
        let result = adapter
            .locate_module_files(temp_dir.path(), "services::planner")
            .await;

        assert!(result.is_ok());
        let files = result.unwrap();
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("services/planner.rs") || files[0].ends_with("services\\planner.rs"));
    }

    #[tokio::test]
    async fn test_locate_module_files_dot_separator() {
        // Test that the function accepts both :: and . as separators
        let temp_dir = tempdir().unwrap();
        let src_dir = temp_dir.path().join("src");
        fs::create_dir(&src_dir).unwrap();

        // Create lib.rs
        fs::write(src_dir.join("lib.rs"), "// lib.rs").unwrap();

        // Create nested module structure: src/services/planner.rs
        let services_dir = src_dir.join("services");
        fs::create_dir(&services_dir).unwrap();
        fs::write(services_dir.join("planner.rs"), "// planner.rs").unwrap();

        let adapter = RustAdapter;
        let result = adapter
            .locate_module_files(temp_dir.path(), "services.planner")
            .await;

        assert!(result.is_ok());
        let files = result.unwrap();
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("services/planner.rs") || files[0].ends_with("services\\planner.rs"));
    }

    #[tokio::test]
    async fn test_locate_module_files_not_found() {
        // Create a temporary Rust project structure
        let temp_dir = tempdir().unwrap();
        let src_dir = temp_dir.path().join("src");
        fs::create_dir(&src_dir).unwrap();

        // Create lib.rs but no module files
        fs::write(src_dir.join("lib.rs"), "// lib.rs").unwrap();

        let adapter = RustAdapter;
        let result = adapter
            .locate_module_files(temp_dir.path(), "nonexistent")
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, crate::error::AstError::Analysis { .. }));
    }

    #[tokio::test]
    async fn test_locate_module_files_no_src_dir() {
        // Create a temporary directory without src/
        let temp_dir = tempdir().unwrap();

        let adapter = RustAdapter;
        let result = adapter
            .locate_module_files(temp_dir.path(), "my_module")
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, crate::error::AstError::Analysis { .. }));
    }

    #[tokio::test]
    async fn test_locate_module_files_empty_module_path() {
        let temp_dir = tempdir().unwrap();
        let src_dir = temp_dir.path().join("src");
        fs::create_dir(&src_dir).unwrap();

        let adapter = RustAdapter;
        let result = adapter.locate_module_files(temp_dir.path(), "").await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, crate::error::AstError::Analysis { .. }));
    }
}
