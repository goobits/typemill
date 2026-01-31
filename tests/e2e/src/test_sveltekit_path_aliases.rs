//! SvelteKit Path Alias Integration Tests
//!
//! Tests that $lib and other path aliases are correctly handled during file moves
//! in SvelteKit/Vite projects. Verifies:
//! - Moving files within $lib preserves alias imports
//! - Moving files out of $lib converts to relative imports
//! - Moving files into $lib converts to alias imports
//! - Cross-directory moves update imports correctly

use crate::harness::{TestClient, TestWorkspace};
use serde_json::json;

/// Create a SvelteKit-like project structure with tsconfig.json
fn setup_sveltekit_workspace(workspace: &TestWorkspace) {
    // Create tsconfig.json with $lib alias (SvelteKit pattern)
    workspace.create_file(
        "tsconfig.json",
        r#"{
  "compilerOptions": {
    "baseUrl": ".",
    "paths": {
      "$lib": ["src/lib"],
      "$lib/*": ["src/lib/*"]
    }
  }
}"#,
    );

    // Create package.json
    workspace.create_file(
        "package.json",
        r#"{
  "name": "test-sveltekit",
  "type": "module"
}"#,
    );

    // Create src/lib structure (the $lib directory)
    workspace.create_directory("src/lib");
    workspace.create_directory("src/lib/components");
    workspace.create_directory("src/lib/utils");
    workspace.create_directory("src/routes");
}

/// Test 1: Move file within $lib - imports should preserve $lib alias
#[tokio::test]
async fn test_move_within_lib_preserves_alias() {
    let workspace = TestWorkspace::new();
    setup_sveltekit_workspace(&workspace);

    // Create a utility file in $lib/utils
    workspace.create_file(
        "src/lib/utils/helpers.ts",
        r#"export function formatDate(date: Date): string {
    return date.toISOString();
}

export function capitalize(str: string): string {
    return str.charAt(0).toUpperCase() + str.slice(1);
}"#,
    );

    // Create a component that imports from $lib/utils
    workspace.create_file(
        "src/lib/components/DateDisplay.svelte",
        r#"<script lang="ts">
import { formatDate } from '$lib/utils/helpers';

export let date: Date;
</script>

<span>{formatDate(date)}</span>"#,
    );

    // Create a route that imports from $lib
    workspace.create_file(
        "src/routes/+page.svelte",
        r#"<script lang="ts">
import { capitalize } from '$lib/utils/helpers';
import DateDisplay from '$lib/components/DateDisplay.svelte';

let name = capitalize('world');
</script>

<h1>Hello {name}</h1>
<DateDisplay date={new Date()} />"#,
    );

    let mut client = TestClient::new(workspace.path());

    // Move helpers.ts to a new location within $lib
    let old_path = workspace.path().join("src/lib/utils/helpers.ts");
    let new_path = workspace.path().join("src/lib/utils/string/helpers.ts");

    // Create target directory
    workspace.create_directory("src/lib/utils/string");

    let params = json!({
        "target": {
            "kind": "file",
            "filePath": old_path.to_string_lossy()
        },
        "destination": {
            "filePath": new_path.to_string_lossy()
        },
        "options": { "dryRun": false }
    });

    let result = client
        .call_tool("relocate", params)
        .await
        .expect("relocate should succeed");

    let content = result
        .get("result")
        .and_then(|r| r.get("content"))
        .expect("Should have result.content");

    assert_eq!(
        content.get("status").and_then(|v| v.as_str()),
        Some("success"),
        "Move should succeed"
    );

    // Verify file was moved
    assert!(!workspace.file_exists("src/lib/utils/helpers.ts"));
    assert!(workspace.file_exists("src/lib/utils/string/helpers.ts"));

    // Verify imports in component were updated (should still use $lib)
    let component_content = workspace.read_file("src/lib/components/DateDisplay.svelte");
    assert!(
        component_content.contains("$lib/utils/string/helpers"),
        "Component should have updated $lib import path.\nActual content:\n{}",
        component_content
    );

    // Verify imports in route were updated
    let route_content = workspace.read_file("src/routes/+page.svelte");
    assert!(
        route_content.contains("$lib/utils/string/helpers"),
        "Route should have updated $lib import path.\nActual content:\n{}",
        route_content
    );

    println!("✅ Move within $lib preserves alias imports");
}

/// Test 2: Move file from $lib to outside - should convert to relative imports
#[tokio::test]
async fn test_move_out_of_lib_converts_to_relative() {
    let workspace = TestWorkspace::new();
    setup_sveltekit_workspace(&workspace);

    // Create a utility in $lib
    workspace.create_file(
        "src/lib/utils/api.ts",
        r#"export async function fetchData(url: string) {
    const response = await fetch(url);
    return response.json();
}"#,
    );

    // Create a route that imports from $lib
    workspace.create_file(
        "src/routes/api/+server.ts",
        r#"import { fetchData } from '$lib/utils/api';

export async function GET() {
    const data = await fetchData('/api/data');
    return new Response(JSON.stringify(data));
}"#,
    );

    let mut client = TestClient::new(workspace.path());

    // Move api.ts outside of $lib to src/server/
    let old_path = workspace.path().join("src/lib/utils/api.ts");
    let new_path = workspace.path().join("src/server/api.ts");

    workspace.create_directory("src/server");

    let params = json!({
        "target": {
            "kind": "file",
            "filePath": old_path.to_string_lossy()
        },
        "destination": {
            "filePath": new_path.to_string_lossy()
        },
        "options": { "dryRun": false }
    });

    let result = client
        .call_tool("relocate", params)
        .await
        .expect("relocate should succeed");

    let content = result
        .get("result")
        .and_then(|r| r.get("content"))
        .expect("Should have result.content");

    assert_eq!(
        content.get("status").and_then(|v| v.as_str()),
        Some("success"),
        "Move should succeed"
    );

    // Verify file was moved
    assert!(!workspace.file_exists("src/lib/utils/api.ts"));
    assert!(workspace.file_exists("src/server/api.ts"));

    // Verify imports were converted to relative paths
    let server_content = workspace.read_file("src/routes/api/+server.ts");

    // Should no longer have $lib import
    assert!(
        !server_content.contains("$lib/utils/api"),
        "Should NOT have old $lib import.\nActual content:\n{}",
        server_content
    );

    // Should have relative import (../../server/api)
    assert!(
        server_content.contains("../") || server_content.contains("../../server/api"),
        "Should have relative import path.\nActual content:\n{}",
        server_content
    );

    println!("✅ Move out of $lib converts to relative imports");
}

/// Test 3: Move directory within $lib
#[tokio::test]
async fn test_move_directory_within_lib() {
    let workspace = TestWorkspace::new();
    setup_sveltekit_workspace(&workspace);

    // Create multiple files in a directory
    workspace.create_file(
        "src/lib/components/Button.svelte",
        r#"<script lang="ts">
export let label: string;
</script>
<button>{label}</button>"#,
    );

    workspace.create_file(
        "src/lib/components/Input.svelte",
        r#"<script lang="ts">
export let value: string;
</script>
<input bind:value />"#,
    );

    // Create an index file that re-exports
    workspace.create_file(
        "src/lib/components/index.ts",
        r#"export { default as Button } from './Button.svelte';
export { default as Input } from './Input.svelte';"#,
    );

    // Create a route that imports from the components
    workspace.create_file(
        "src/routes/+page.svelte",
        r#"<script lang="ts">
import { Button, Input } from '$lib/components';
</script>

<Input value="" />
<Button label="Submit" />"#,
    );

    let mut client = TestClient::new(workspace.path());

    // Move components directory to ui/components
    let old_path = workspace.path().join("src/lib/components");
    let new_path = workspace.path().join("src/lib/ui/components");

    workspace.create_directory("src/lib/ui");

    let params = json!({
        "target": {
            "kind": "directory",
            "filePath": old_path.to_string_lossy()
        },
        "newName": new_path.to_string_lossy(),
        "options": { "dryRun": false }
    });

    let result = client
        .call_tool("rename_all", params)
        .await
        .expect("rename_all should succeed");

    let content = result
        .get("result")
        .and_then(|r| r.get("content"))
        .expect("Should have result.content");

    assert_eq!(
        content.get("status").and_then(|v| v.as_str()),
        Some("success"),
        "Directory move should succeed"
    );

    // Verify directory was moved
    assert!(!workspace.file_exists("src/lib/components/Button.svelte"));
    assert!(workspace.file_exists("src/lib/ui/components/Button.svelte"));
    assert!(workspace.file_exists("src/lib/ui/components/Input.svelte"));
    assert!(workspace.file_exists("src/lib/ui/components/index.ts"));

    // Verify route imports were updated
    let route_content = workspace.read_file("src/routes/+page.svelte");
    assert!(
        route_content.contains("$lib/ui/components"),
        "Route should have updated import path.\nActual content:\n{}",
        route_content
    );

    println!("✅ Directory move within $lib updates imports correctly");
}

/// Test 4: Verify @alias pattern (Next.js style)
#[tokio::test]
async fn test_at_alias_pattern() {
    let workspace = TestWorkspace::new();

    // Create tsconfig.json with @ alias (Next.js pattern)
    workspace.create_file(
        "tsconfig.json",
        r#"{
  "compilerOptions": {
    "baseUrl": ".",
    "paths": {
      "@/*": ["src/*"]
    }
  }
}"#,
    );

    workspace.create_file(
        "package.json",
        r#"{"name": "test-nextjs", "type": "module"}"#,
    );

    workspace.create_directory("src/components");
    workspace.create_directory("src/utils");
    workspace.create_directory("src/pages");

    // Create a utility
    workspace.create_file(
        "src/utils/format.ts",
        r#"export function formatCurrency(amount: number): string {
    return `$${amount.toFixed(2)}`;
}"#,
    );

    // Create a component that imports with @/
    workspace.create_file(
        "src/components/Price.tsx",
        r#"import { formatCurrency } from '@/utils/format';

export function Price({ amount }: { amount: number }) {
    return <span>{formatCurrency(amount)}</span>;
}"#,
    );

    // Create a page that imports with @/
    workspace.create_file(
        "src/pages/index.tsx",
        r#"import { Price } from '@/components/Price';
import { formatCurrency } from '@/utils/format';

export default function Home() {
    return <Price amount={99.99} />;
}"#,
    );

    let mut client = TestClient::new(workspace.path());

    // Move format.ts to a subdirectory
    let old_path = workspace.path().join("src/utils/format.ts");
    let new_path = workspace.path().join("src/utils/currency/format.ts");

    workspace.create_directory("src/utils/currency");

    let params = json!({
        "target": {
            "kind": "file",
            "filePath": old_path.to_string_lossy()
        },
        "destination": {
            "filePath": new_path.to_string_lossy()
        },
        "options": { "dryRun": false }
    });

    let result = client
        .call_tool("relocate", params)
        .await
        .expect("relocate should succeed");

    let content = result
        .get("result")
        .and_then(|r| r.get("content"))
        .expect("Should have result.content");

    assert_eq!(
        content.get("status").and_then(|v| v.as_str()),
        Some("success"),
        "Move should succeed"
    );

    // Verify imports were updated with @ alias
    let component_content = workspace.read_file("src/components/Price.tsx");
    assert!(
        component_content.contains("@/utils/currency/format"),
        "Component should have updated @/ import.\nActual content:\n{}",
        component_content
    );

    let page_content = workspace.read_file("src/pages/index.tsx");
    assert!(
        page_content.contains("@/utils/currency/format"),
        "Page should have updated @/ import.\nActual content:\n{}",
        page_content
    );

    println!("✅ @/ alias pattern works correctly");
}

/// Test 5: Dry run shows correct changes for alias moves
#[tokio::test]
async fn test_dry_run_shows_alias_updates() {
    let workspace = TestWorkspace::new();
    setup_sveltekit_workspace(&workspace);

    workspace.create_file(
        "src/lib/stores/user.ts",
        r#"import { writable } from 'svelte/store';
export const user = writable(null);"#,
    );

    workspace.create_file(
        "src/routes/+layout.svelte",
        r#"<script lang="ts">
import { user } from '$lib/stores/user';
</script>
<slot />"#,
    );

    let mut client = TestClient::new(workspace.path());

    let old_path = workspace.path().join("src/lib/stores/user.ts");
    let new_path = workspace.path().join("src/lib/stores/auth/user.ts");

    workspace.create_directory("src/lib/stores/auth");

    // Dry run first
    let params = json!({
        "target": {
            "kind": "file",
            "filePath": old_path.to_string_lossy()
        },
        "destination": {
            "filePath": new_path.to_string_lossy()
        },
        "options": { "dryRun": true }
    });

    let result = client
        .call_tool("relocate", params)
        .await
        .expect("dry run should succeed");

    let content = result
        .get("result")
        .and_then(|r| r.get("content"))
        .expect("Should have result.content");

    // Check that the plan shows file edits
    let edits = content.get("fileEdits").and_then(|e| e.as_array());
    assert!(
        edits.is_some() && !edits.unwrap().is_empty(),
        "Dry run should show file edits.\nPlan content:\n{:?}",
        content
    );

    // Original file should still exist (dry run)
    assert!(
        workspace.file_exists("src/lib/stores/user.ts"),
        "File should still exist after dry run"
    );

    println!("✅ Dry run correctly shows alias update plan");
}
