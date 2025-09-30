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
    RenameSymbolTestCase {
        test_name: "multi_file_with_multiple_references",
        initial_files: &[
            (
                "src/exporter.ts",
                r#"export function oldFunctionName(value: string): string {
    return value.toUpperCase();
}

export class OldClassName {
    private data: string;

    constructor(data: string) {
        this.data = data;
    }

    getData(): string {
        return this.data;
    }
}

export const OLD_CONSTANT = 'constant_value';
"#,
            ),
            (
                "src/consumer.ts",
                r#"import { oldFunctionName, OldClassName, OLD_CONSTANT } from './exporter';

export function useImports() {
    const result = oldFunctionName('test');
    const instance = new OldClassName('data');
    const data = instance.getData();

    console.log(result, data, OLD_CONSTANT);

    // Multiple references to test thorough renaming
    const anotherCall = oldFunctionName('another');
    const anotherInstance = new OldClassName('more');
}
"#,
            ),
            (
                "src/another-consumer.ts",
                r#"import { oldFunctionName as renamed, OldClassName } from './exporter';

export class ConsumerClass {
    private helper = new OldClassName('helper');

    process(input: string): string {
        return renamed(input);
    }

    getHelper(): OldClassName {
        return this.helper;
    }
}
"#,
            ),
        ],
        file_path: "src/exporter.ts",
        old_name: "oldFunctionName",
        new_name: "newFunctionName",
        expect_success: true,
    },
    RenameSymbolTestCase {
        test_name: "multi_file_class_rename",
        initial_files: &[
            (
                "src/services/DataService.ts",
                r#"export class DataService {
    private data: string[] = [];

    addData(item: string): void {
        this.data.push(item);
    }

    getData(): string[] {
        return this.data;
    }
}
"#,
            ),
            (
                "src/controllers/MainController.ts",
                r#"import { DataService } from '../services/DataService';

export class MainController {
    private service: DataService;

    constructor() {
        this.service = new DataService();
    }

    processData(input: string): void {
        this.service.addData(input);
    }
}
"#,
            ),
            (
                "src/utils/helper.ts",
                r#"import { DataService } from '../services/DataService';

export function createService(): DataService {
    return new DataService();
}

export function processWithService(service: DataService, data: string): void {
    service.addData(data);
}
"#,
            ),
        ],
        file_path: "src/services/DataService.ts",
        old_name: "DataService",
        new_name: "DataManager",
        expect_success: true,
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

// =============================================================================
// FIND DEAD CODE TEST CASES
// =============================================================================

/// Test fixture for find_dead_code operations
#[derive(Debug, Clone)]
pub struct FindDeadCodeTestCase {
    pub test_name: &'static str,
    pub initial_files: &'static [(&'static str, &'static str)],
    pub workspace_path: &'static str, // Relative to workspace root
    pub expected_dead_symbols: &'static [&'static str], // Names of dead symbols expected
    pub expect_success: bool,
}

pub const FIND_DEAD_CODE_TESTS: &[FindDeadCodeTestCase] = &[
    FindDeadCodeTestCase {
        test_name: "detect_unused_exports",
        initial_files: &[
            (
                "src/utils.ts",
                r#"export function usedFunction(x: number): number {
    return x * 2;
}

export function unusedFunction(x: number): number {
    return x * 3;
}

export class UnusedClass {
    value: number = 0;
}
"#,
            ),
            (
                "src/main.ts",
                r#"import { usedFunction } from './utils';

export function main() {
    const result = usedFunction(5);
    console.log(result);
}
"#,
            ),
        ],
        workspace_path: "",
        expected_dead_symbols: &["unusedFunction", "UnusedClass"],
        expect_success: true,
    },
    FindDeadCodeTestCase {
        test_name: "no_dead_code",
        initial_files: &[
            (
                "src/module.ts",
                r#"export function activeFunction(): void {
    console.log("active");
}
"#,
            ),
            (
                "src/app.ts",
                r#"import { activeFunction } from './module';

activeFunction();
"#,
            ),
        ],
        workspace_path: "",
        expected_dead_symbols: &[],
        expect_success: true,
    },
    FindDeadCodeTestCase {
        test_name: "empty_workspace",
        initial_files: &[],
        workspace_path: "",
        expected_dead_symbols: &[],
        expect_success: true,
    },
];

// =============================================================================
// RENAME DIRECTORY TEST CASES
// =============================================================================

/// Test fixture for rename_directory operations
#[derive(Debug, Clone)]
pub struct RenameDirectoryTestCase {
    pub test_name: &'static str,
    pub initial_files: &'static [(&'static str, &'static str)],
    pub dir_to_rename: &'static str,
    pub new_dir_name: &'static str,
    pub update_imports: bool,
    pub expect_success: bool,
}

pub const RENAME_DIRECTORY_TESTS: &[RenameDirectoryTestCase] = &[
    RenameDirectoryTestCase {
        test_name: "simple_rename",
        initial_files: &[
            ("olddir/file1.ts", "export const value = 1;"),
            ("olddir/file2.ts", "export const value = 2;"),
        ],
        dir_to_rename: "olddir",
        new_dir_name: "newdir",
        update_imports: false,
        expect_success: true,
    },
    RenameDirectoryTestCase {
        test_name: "rename_with_import_updates",
        initial_files: &[
            ("components/Button.tsx", "export const Button = () => <button />;"),
            (
                "app.tsx",
                r#"import { Button } from './components/Button';

export default function App() {
    return <Button />;
}
"#,
            ),
        ],
        dir_to_rename: "components",
        new_dir_name: "ui",
        update_imports: true,
        expect_success: true,
    },
    RenameDirectoryTestCase {
        test_name: "nonexistent_directory",
        initial_files: &[("src/file.ts", "export const value = 1;")],
        dir_to_rename: "nonexistent",
        new_dir_name: "newdir",
        update_imports: false,
        expect_success: false,
    },
];

// =============================================================================
// RENAME FILE TEST CASES
// =============================================================================

/// Test fixture for rename_file operations
#[derive(Debug, Clone)]
pub struct RenameFileTestCase {
    pub test_name: &'static str,
    pub initial_files: &'static [(&'static str, &'static str)],
    pub old_file_path: &'static str,
    pub new_file_path: &'static str,
    pub expect_success: bool,
    pub expected_import_updates: &'static [(&'static str, &'static str)], // (file_path, expected_content_substring)
}

pub const RENAME_FILE_TESTS: &[RenameFileTestCase] = &[
    RenameFileTestCase {
        test_name: "basic_rename_with_import_updates",
        initial_files: &[
            (
                "src/utils.ts",
                r#"export const myUtil = () => {
    return "utility function";
};

export function helperFunc(data: string): string {
    return data.toUpperCase();
}
"#,
            ),
            (
                "src/main.ts",
                r#"import { myUtil, helperFunc } from './utils';

export function main() {
    const result = myUtil();
    const processed = helperFunc(result);
    console.log(processed);
}
"#,
            ),
        ],
        old_file_path: "src/utils.ts",
        new_file_path: "src/renamed_utils.ts",
        expect_success: true,
        expected_import_updates: &[("src/main.ts", "from './renamed_utils'")],
    },
    RenameFileTestCase {
        test_name: "nested_import_path_resolution",
        initial_files: &[
            (
                "src/core/types.ts",
                r#"export interface User {
    id: number;
    name: string;
}

export type Status = 'active' | 'inactive';
"#,
            ),
            (
                "src/core/models/UserModel.ts",
                r#"import { User, Status } from '../types';

export class UserModel implements User {
    constructor(
        public id: number,
        public name: string,
        public status: Status = 'active'
    ) {}
}
"#,
            ),
            (
                "src/features/users/UserService.ts",
                r#"import { UserModel } from '../../core/models/UserModel';
import { Status } from '../../core/types';

export class UserService {
    private users: UserModel[] = [];

    addUser(name: string): UserModel {
        const user = new UserModel(Date.now(), name);
        this.users.push(user);
        return user;
    }

    setUserStatus(id: number, status: Status): void {
        const user = this.users.find(u => u.id === id);
        if (user) {
            user.status = status;
        }
    }
}
"#,
            ),
        ],
        old_file_path: "src/core/types.ts",
        new_file_path: "src/shared/types.ts",
        expect_success: true,
        expected_import_updates: &[
            ("src/core/models/UserModel.ts", "from '../../shared/types'"),
            ("src/features/users/UserService.ts", "from '../../shared/types'"),
        ],
    },
    RenameFileTestCase {
        test_name: "rename_to_subdirectory",
        initial_files: &[
            ("config.ts", "export const API_URL = 'https://api.example.com';"),
            (
                "app.ts",
                r#"import { API_URL } from './config';

console.log(API_URL);
"#,
            ),
        ],
        old_file_path: "config.ts",
        new_file_path: "settings/config.ts",
        expect_success: true,
        expected_import_updates: &[("app.ts", "from './settings/config'")],
    },
];
