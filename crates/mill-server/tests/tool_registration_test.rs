use mill_server::handlers::plugin_dispatcher::create_test_dispatcher;

#[tokio::test]
async fn test_all_public_tools_are_registered() {
    let dispatcher = create_test_dispatcher().await;
    dispatcher.initialize().await.unwrap();

    let registry = dispatcher.tool_registry.lock().await;
    let registered_tools = registry.list_tools();

    // Magnificent Seven - the new public API
    const EXPECTED_TOOLS: [&str; 7] = [
        // Code Intelligence (2)
        "inspect_code",   // Aggregates find_definition, find_references, etc.
        "search_code",    // Replaces search_symbols
        // Refactoring & Editing (4)
        "rename_all",     // Replaces rename
        "relocate",       // Replaces move
        "prune",          // Replaces delete
        "refactor",       // Replaces extract, inline
        // Workspace Management (1)
        "workspace",      // Replaces workspace.create_package, workspace.extract_dependencies, etc.
    ];

    fn find_missing<'a>(expected: &'a [&str], actual: &[String]) -> Vec<&'a str> {
        expected
            .iter()
            .filter(|tool| !actual.contains(&tool.to_string()))
            .copied()
            .collect()
    }
    fn find_extra(expected: &[&str], actual: &[String]) -> Vec<String> {
        actual
            .iter()
            .filter(|tool| !expected.contains(&tool.as_str()))
            .cloned()
            .collect()
    }

    assert_eq!(
        registered_tools.len(),
        EXPECTED_TOOLS.len(),
        "Expected {} tools, found {}.\nMissing: {:?}\nExtra: {:?}",
        EXPECTED_TOOLS.len(),
        registered_tools.len(),
        find_missing(&EXPECTED_TOOLS, &registered_tools),
        find_extra(&EXPECTED_TOOLS, &registered_tools)
    );

    let missing = find_missing(&EXPECTED_TOOLS, &registered_tools);
    assert!(
        missing.is_empty(),
        "The following tools are missing: {:?}",
        missing
    );
}

#[tokio::test]
async fn test_all_internal_tools_are_registered_and_hidden() {
    let dispatcher = create_test_dispatcher().await;
    dispatcher.initialize().await.unwrap();

    let registry = dispatcher.tool_registry.lock().await;

    // All legacy tools are now internal, plus existing internal tools
    const EXPECTED_INTERNAL_TOOLS: [&str; 41] = [
        // Legacy Navigation (10) - Now internal, use inspect_code/search_code
        "find_definition",
        "find_references",
        "find_implementations",
        "find_type_definition",
        "search_symbols",
        "find_symbol",
        "find_referencing_symbols",
        "get_symbol_info",
        "get_diagnostics",
        "get_call_hierarchy",
        // Legacy Refactoring (5) - Now internal, use rename_all/relocate/prune/refactor
        "rename",
        "extract",
        "inline",
        "move",
        "delete",
        // Legacy Workspace (3) - Now internal, use workspace action
        "workspace.create_package",
        "workspace.extract_dependencies",
        "workspace.find_replace",
        // System (1) - Now internal, use workspace action:verify_project
        "health_check",
        // Lifecycle (3)
        "notify_file_opened",
        "notify_file_saved",
        "notify_file_closed",
        // Internal Editing (3)
        "rename_symbol_with_imports",
        "edit_file",
        "insert_after_symbol",
        // Internal Workspace (1)
        "apply_workspace_edit",
        // Internal Intelligence (2)
        "get_completions",
        "get_signature_help",
        // Workspace Tools (3)
        "move_directory",
        "update_dependencies",
        "update_dependency",
        // File Operations (4)
        "create_file",
        "delete_file",
        "rename_file",
        "rename_directory",
        // File Utilities (3)
        "read_file",
        "write_file",
        "list_files",
        // Document Symbols (1)
        "get_document_symbols",
        // Advanced (2)
        "execute_edits",
        "execute_batch",
    ];

    // 1. Verify they are NOT in the public list
    let public_tools = registry.list_tools();
    for internal_tool in &EXPECTED_INTERNAL_TOOLS {
        assert!(
            !public_tools.contains(&internal_tool.to_string()),
            "Internal tool '{}' should not be in public tool list",
            internal_tool
        );
    }

    // 2. Verify they ARE in the internal list
    let internal_tools = registry.list_internal_tools();
    for expected in &EXPECTED_INTERNAL_TOOLS {
        assert!(
            internal_tools.contains(&expected.to_string()),
            "Expected internal tool '{}' not found in internal tool list. Found: {:?}",
            expected,
            internal_tools
        );
    }

    // 3. Verify they are still registered in the main registry
    for tool_name in &EXPECTED_INTERNAL_TOOLS {
        assert!(
            registry.has_tool(tool_name),
            "Internal tool '{}' should still be registered in the system",
            tool_name
        );
    }
}
