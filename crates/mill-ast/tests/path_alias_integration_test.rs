//! Integration test for path alias resolution
//!
//! This test verifies that the ImportPathResolver correctly uses
//! language plugins to resolve path aliases like $lib/*, @/*, etc.

use mill_ast::import_updater::path_resolver::ImportPathResolver;
use std::sync::Arc;
use tempfile::TempDir;

#[cfg(feature = "lang-typescript")]
#[test]
fn test_resolve_sveltekit_lib_alias() {
    // Setup: Create temp project with tsconfig.json
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();

    // Create tsconfig.json with $lib alias
    std::fs::write(
        project_root.join("tsconfig.json"),
        r#"{
            "compilerOptions": {
                "baseUrl": ".",
                "paths": {
                    "$lib/*": ["src/lib/*"]
                }
            }
        }"#,
    )
    .unwrap();

    // Create source file structure
    let src_dir = project_root.join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    let importing_file = src_dir.join("app.ts");
    std::fs::write(&importing_file, "import { foo } from '$lib/utils';").unwrap();

    // Create target file
    std::fs::create_dir_all(project_root.join("src/lib")).unwrap();
    let target_file = project_root.join("src/lib/utils.ts");
    std::fs::write(&target_file, "export const foo = 1;").unwrap();

    // Setup resolver with TypeScript plugin
    let typescript_plugin: Arc<dyn mill_plugin_api::LanguagePlugin> =
        Arc::from(mill_lang_typescript::TypeScriptPlugin::new());
    let plugins = vec![typescript_plugin];
    let resolver = ImportPathResolver::with_plugins(project_root, plugins);

    // Get canonical paths for project_files
    let canonical_target = target_file.canonicalize().unwrap();
    let project_files = vec![canonical_target.clone()];

    // Test: Resolve $lib/utils
    let resolved = resolver.resolve_import_to_file("$lib/utils", &importing_file, &project_files);

    assert!(
        resolved.is_some(),
        "Path alias $lib/utils should resolve to a file"
    );
    let resolved_path = resolved.unwrap();
    assert!(
        resolved_path.ends_with("src/lib/utils.ts"),
        "Resolved path should point to src/lib/utils.ts, got: {:?}",
        resolved_path
    );
}

#[cfg(feature = "lang-typescript")]
#[test]
fn test_resolve_nextjs_at_alias() {
    // Setup: Create temp project with tsconfig.json
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();

    // Create tsconfig.json with @/* alias
    std::fs::write(
        project_root.join("tsconfig.json"),
        r#"{
            "compilerOptions": {
                "baseUrl": ".",
                "paths": {
                    "@/*": ["src/*"]
                }
            }
        }"#,
    )
    .unwrap();

    // Create source files
    let src_dir = project_root.join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    let importing_file = src_dir.join("page.tsx");
    std::fs::write(&importing_file, "import Button from '@/components/Button';").unwrap();

    // Create target file
    std::fs::create_dir_all(project_root.join("src/components")).unwrap();
    let target_file = project_root.join("src/components/Button.tsx");
    std::fs::write(&target_file, "export default function Button() {}").unwrap();

    // Setup resolver with TypeScript plugin
    let typescript_plugin: Arc<dyn mill_plugin_api::LanguagePlugin> =
        Arc::from(mill_lang_typescript::TypeScriptPlugin::new());
    let plugins = vec![typescript_plugin];
    let resolver = ImportPathResolver::with_plugins(project_root, plugins);

    // Get canonical paths
    let canonical_target = target_file.canonicalize().unwrap();
    let project_files = vec![canonical_target.clone()];

    // Test: Resolve @/components/Button
    let resolved =
        resolver.resolve_import_to_file("@/components/Button", &importing_file, &project_files);

    assert!(
        resolved.is_some(),
        "Path alias @/components/Button should resolve to a file"
    );
    let resolved_path = resolved.unwrap();
    assert!(
        resolved_path.ends_with("src/components/Button.tsx"),
        "Resolved path should point to src/components/Button.tsx, got: {:?}",
        resolved_path
    );
}

#[test]
fn test_resolve_relative_import_without_alias() {
    // Test that regular relative imports still work without path alias resolution
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();

    // Create source files
    let src_dir = project_root.join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    let importing_file = src_dir.join("app.ts");
    std::fs::write(&importing_file, "import { foo } from './utils';").unwrap();

    let target_file = src_dir.join("utils.ts");
    std::fs::write(&target_file, "export const foo = 1;").unwrap();

    // Setup resolver WITHOUT plugins (simulates fallback behavior)
    let resolver = ImportPathResolver::new(project_root);

    // Get canonical paths
    let canonical_target = target_file.canonicalize().unwrap();
    let project_files = vec![canonical_target.clone()];

    // Test: Resolve ./utils
    let resolved = resolver.resolve_import_to_file("./utils", &importing_file, &project_files);

    assert!(
        resolved.is_some(),
        "Relative import ./utils should resolve without plugins"
    );
    let resolved_path = resolved.unwrap();
    assert!(
        resolved_path.ends_with("src/utils.ts"),
        "Resolved path should point to src/utils.ts, got: {:?}",
        resolved_path
    );
}
