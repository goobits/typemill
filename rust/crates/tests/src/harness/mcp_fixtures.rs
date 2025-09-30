//! Data-driven test fixtures for MCP file operation handlers
//!
//! This module contains test data for all MCP file operation tools.
//! Each fixture struct represents a single test case with all necessary
//! setup data, operations, and expected outcomes.

/// Test fixture for create_file operations
#[derive(Debug, Clone)]
pub struct CreateFileTestCase {
    pub test_name: &'static str,
    pub file_to_create: &'static str,
    pub content: &'static str,
    pub initial_files: &'static [(&'static str, &'static str)], // (path, content)
    pub overwrite: bool,
    pub expect_success: bool,
}

/// Test fixture for read_file operations
#[derive(Debug, Clone)]
pub struct ReadFileTestCase {
    pub test_name: &'static str,
    pub file_to_read: &'static str,
    pub initial_files: &'static [(&'static str, &'static str)],
    pub expected_content: Option<&'static str>,
    pub start_line: Option<usize>,
    pub end_line: Option<usize>,
    pub expect_success: bool,
}

/// Test fixture for write_file operations
#[derive(Debug, Clone)]
pub struct WriteFileTestCase {
    pub test_name: &'static str,
    pub file_to_write: &'static str,
    pub content: &'static str,
    pub initial_files: &'static [(&'static str, &'static str)],
    pub expect_success: bool,
}

/// Test fixture for delete_file operations
#[derive(Debug, Clone)]
pub struct DeleteFileTestCase {
    pub test_name: &'static str,
    pub file_to_delete: &'static str,
    pub initial_files: &'static [(&'static str, &'static str)],
    pub expect_success: bool,
}

/// Test fixture for list_files operations
#[derive(Debug, Clone)]
pub struct ListFilesTestCase {
    pub test_name: &'static str,
    pub directory: &'static str, // Empty string means workspace root
    pub recursive: bool,
    pub pattern: Option<&'static str>,
    pub initial_files: &'static [&'static str],
    pub initial_dirs: &'static [&'static str],
    pub expected_contains: &'static [&'static str],
    pub expected_min_count: usize,
}

// =============================================================================
// CREATE FILE TEST CASES
// =============================================================================

pub const CREATE_FILE_TESTS: &[CreateFileTestCase] = &[
    CreateFileTestCase {
        test_name: "basic",
        file_to_create: "new_file.txt",
        content: "Hello, World!",
        initial_files: &[],
        overwrite: false,
        expect_success: true,
    },
    CreateFileTestCase {
        test_name: "with_directories",
        file_to_create: "nested/deep/new_file.js",
        content: "export const greeting = 'Hello from nested file!';",
        initial_files: &[],
        overwrite: false,
        expect_success: true,
    },
    CreateFileTestCase {
        test_name: "overwrite_protection",
        file_to_create: "existing.txt",
        content: "new content",
        initial_files: &[("existing.txt", "original content")],
        overwrite: false,
        expect_success: false,
    },
    CreateFileTestCase {
        test_name: "with_overwrite",
        file_to_create: "existing.txt",
        content: "new content",
        initial_files: &[("existing.txt", "original content")],
        overwrite: true,
        expect_success: true,
    },
];

// =============================================================================
// READ FILE TEST CASES
// =============================================================================

pub const READ_FILE_TESTS: &[ReadFileTestCase] = &[
    ReadFileTestCase {
        test_name: "basic",
        file_to_read: "test_file.txt",
        initial_files: &[(
            "test_file.txt",
            "This is test content\nwith multiple lines\nand unicode: 🚀",
        )],
        expected_content: Some("This is test content\nwith multiple lines\nand unicode: 🚀"),
        start_line: None,
        end_line: None,
        expect_success: true,
    },
    ReadFileTestCase {
        test_name: "nonexistent",
        file_to_read: "nonexistent.txt",
        initial_files: &[],
        expected_content: None,
        start_line: None,
        end_line: None,
        expect_success: false,
    },
];

// =============================================================================
// WRITE FILE TEST CASES
// =============================================================================

pub const WRITE_FILE_TESTS: &[WriteFileTestCase] = &[
    WriteFileTestCase {
        test_name: "basic",
        file_to_write: "write_test.txt",
        content: "Written content with special chars: @#$%^&*()",
        initial_files: &[],
        expect_success: true,
    },
    WriteFileTestCase {
        test_name: "overwrites_existing",
        file_to_write: "overwrite_test.txt",
        content: "completely new content",
        initial_files: &[("overwrite_test.txt", "original")],
        expect_success: true,
    },
];

// =============================================================================
// DELETE FILE TEST CASES
// =============================================================================

pub const DELETE_FILE_TESTS: &[DeleteFileTestCase] = &[
    DeleteFileTestCase {
        test_name: "basic",
        file_to_delete: "to_delete.txt",
        initial_files: &[("to_delete.txt", "content to be deleted")],
        expect_success: true,
    },
    DeleteFileTestCase {
        test_name: "nonexistent",
        file_to_delete: "nonexistent.txt",
        initial_files: &[],
        expect_success: false,
    },
];

// =============================================================================
// LIST FILES TEST CASES
// =============================================================================

pub const LIST_FILES_TESTS: &[ListFilesTestCase] = &[
    ListFilesTestCase {
        test_name: "basic",
        directory: "",
        recursive: false,
        pattern: None,
        initial_files: &["file1.txt", "file2.js", "file3.py", "subdir/nested.txt"],
        initial_dirs: &["subdir"],
        expected_contains: &["file1.txt", "file2.js", "file3.py", "subdir"],
        expected_min_count: 4,
    },
    ListFilesTestCase {
        test_name: "with_pattern",
        directory: "",
        recursive: false,
        pattern: Some("*.js"),
        initial_files: &["test.js", "test.ts", "test.py", "test.txt", "README.md"],
        initial_dirs: &[],
        expected_contains: &["test.js"],
        expected_min_count: 1,
    },
];

// =============================================================================
// RENAME SYMBOL TEST CASES
// =============================================================================

/// Test fixture for rename_symbol_with_imports operations
#[derive(Debug, Clone)]
pub struct RenameSymbolTestCase {
    pub test_name: &'static str,
    pub initial_files: &'static [(&'static str, &'static str)], // (path, content)
    pub file_path: &'static str,
    pub old_name: &'static str,
    pub new_name: &'static str,
    pub expect_success: bool,
}

pub const RENAME_SYMBOL_TESTS: &[RenameSymbolTestCase] = &[
    RenameSymbolTestCase {
        test_name: "simple_function_rename",
        initial_files: &[(
            "utils.ts",
            r#"export const oldName = () => {
    return "Hello from oldName!";
};

export const helper = () => {
    return oldName();
};
"#,
        )],
        file_path: "utils.ts",
        old_name: "oldName",
        new_name: "newName",
        expect_success: true,
    },
    RenameSymbolTestCase {
        test_name: "rename_with_imports",
        initial_files: &[
            (
                "utils.ts",
                r#"export const targetFunc = () => {
    return "test";
};
"#,
            ),
            (
                "main.ts",
                r#"import { targetFunc } from './utils';

console.log(targetFunc());
"#,
            ),
        ],
        file_path: "utils.ts",
        old_name: "targetFunc",
        new_name: "renamedFunc",
        expect_success: true,
    },
    RenameSymbolTestCase {
        test_name: "nonexistent_symbol",
        initial_files: &[(
            "test.ts",
            r#"export const existingFunc = () => {
    return "exists";
};
"#,
        )],
        file_path: "test.ts",
        old_name: "nonExistentFunc",
        new_name: "newFunc",
        expect_success: true, // Returns empty edit plan, not an error
    },
];

// =============================================================================
// ANALYZE IMPORTS TEST CASES
// =============================================================================

/// Test fixture for analyze_imports operations
#[derive(Debug, Clone)]
pub struct AnalyzeImportsTestCase {
    pub test_name: &'static str,
    pub file_path: &'static str,
    pub initial_files: &'static [(&'static str, &'static str)],
    pub expected_import_count: usize,
    pub expect_success: bool,
}

pub const ANALYZE_IMPORTS_TESTS: &[AnalyzeImportsTestCase] = &[
    AnalyzeImportsTestCase {
        test_name: "simple_imports",
        file_path: "main.ts",
        initial_files: &[(
            "main.ts",
            r#"import { foo, bar } from './utils';
import type { MyType } from './types';
import React from 'react';

console.log(foo, bar);
"#,
        )],
        expected_import_count: 3,
        expect_success: true,
    },
    AnalyzeImportsTestCase {
        test_name: "no_imports",
        file_path: "standalone.ts",
        initial_files: &[(
            "standalone.ts",
            r#"const value = 42;
console.log(value);
"#,
        )],
        expected_import_count: 0,
        expect_success: true,
    },
    AnalyzeImportsTestCase {
        test_name: "nonexistent_file",
        file_path: "does_not_exist.ts",
        initial_files: &[],
        expected_import_count: 0,
        expect_success: false,
    },
];
