use tests::harness::{TestClient, TestWorkspace};
use serde_json::{json, Value};
use std::path::Path;

#[tokio::test]
async fn test_find_definition_function() {
    let workspace = TestWorkspace::new();
    let mut client = TestClient::new(workspace.path());

    // Create main file that imports from another file
    let main_file = workspace.path().join("main.ts");
    let util_file = workspace.path().join("utils.ts");

    std::fs::write(&util_file, r#"
export function calculateSum(a: number, b: number): number {
    return a + b;
}

export const PI = 3.14159;
"#).unwrap();

    std::fs::write(&main_file, r#"
import { calculateSum, PI } from './utils';

const result = calculateSum(5, 3);
console.log(`Result: ${result}, PI: ${PI}`);
"#).unwrap();

    // Give LSP time to process files
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

    // Find definition of calculateSum at the call site
    let response = client.call_tool("find_definition", json!({
        "file_path": main_file.to_string_lossy(),
        "line": 4,
        "character": 15
    })).await.unwrap();

    let locations = response["locations"].as_array().unwrap();
    assert!(!locations.is_empty());

    let definition = &locations[0];
    let def_uri = definition["uri"].as_str().unwrap();
    assert!(def_uri.contains("utils.ts"));

    let range = &definition["range"];
    let start_line = range["start"]["line"].as_u64().unwrap();
    assert_eq!(start_line, 1); // function is on line 2 (0-indexed)
}

#[tokio::test]
async fn test_find_definition_interface() {
    let workspace = TestWorkspace::new();
    let mut client = TestClient::new(workspace.path());

    let types_file = workspace.path().join("types.ts");
    let main_file = workspace.path().join("main.ts");

    std::fs::write(&types_file, r#"
export interface User {
    id: number;
    name: string;
    email: string;
}

export type UserRole = 'admin' | 'user' | 'guest';
"#).unwrap();

    std::fs::write(&main_file, r#"
import { User, UserRole } from './types';

const user: User = {
    id: 1,
    name: 'John Doe',
    email: 'john@example.com'
};

const role: UserRole = 'admin';
"#).unwrap();

    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

    // Find definition of User interface
    let response = client.call_tool("find_definition", json!({
        "file_path": main_file.to_string_lossy(),
        "line": 3,
        "character": 12
    })).await.unwrap();

    let locations = response["locations"].as_array().unwrap();
    assert!(!locations.is_empty());

    let definition = &locations[0];
    let def_uri = definition["uri"].as_str().unwrap();
    assert!(def_uri.contains("types.ts"));
}

#[tokio::test]
async fn test_find_references_function() {
    let workspace = TestWorkspace::new();
    let mut client = TestClient::new(workspace.path());

    let util_file = workspace.path().join("utils.ts");
    let main_file = workspace.path().join("main.ts");
    let test_file = workspace.path().join("test.ts");

    std::fs::write(&util_file, r#"
export function formatName(first: string, last: string): string {
    return `${first} ${last}`;
}
"#).unwrap();

    std::fs::write(&main_file, r#"
import { formatName } from './utils';

const fullName = formatName('John', 'Doe');
console.log(fullName);
"#).unwrap();

    std::fs::write(&test_file, r#"
import { formatName } from './utils';

describe('formatName', () => {
    it('should format name correctly', () => {
        const result = formatName('Jane', 'Smith');
        expect(result).toBe('Jane Smith');
    });
});
"#).unwrap();

    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

    // Find all references to formatName
    let response = client.call_tool("find_references", json!({
        "file_path": util_file.to_string_lossy(),
        "line": 1,
        "character": 17,
        "include_declaration": true
    })).await.unwrap();

    let references = response["references"].as_array().unwrap();
    assert!(references.len() >= 3); // declaration + 2 usages

    // Check that references include the declaration and usages
    let files_referenced: Vec<String> = references.iter()
        .map(|r| r["uri"].as_str().unwrap().to_string())
        .collect();

    assert!(files_referenced.iter().any(|f| f.contains("utils.ts")));
    assert!(files_referenced.iter().any(|f| f.contains("main.ts")));
    assert!(files_referenced.iter().any(|f| f.contains("test.ts")));
}

#[tokio::test]
async fn test_get_hover_function() {
    let workspace = TestWorkspace::new();
    let mut client = TestClient::new(workspace.path());

    let file_path = workspace.path().join("hover_test.ts");

    std::fs::write(&file_path, r#"
/**
 * Calculates the area of a rectangle
 * @param width The width of the rectangle
 * @param height The height of the rectangle
 * @returns The area in square units
 */
function calculateArea(width: number, height: number): number {
    return width * height;
}

const area = calculateArea(10, 5);
"#).unwrap();

    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

    // Get hover information for the function call
    let response = client.call_tool("get_hover", json!({
        "file_path": file_path.to_string_lossy(),
        "line": 10,
        "character": 15
    })).await.unwrap();

    let hover_content = response["contents"].as_str().unwrap();
    assert!(hover_content.contains("calculateArea"));
    assert!(hover_content.contains("number") || hover_content.contains("width") || hover_content.contains("height"));
}

#[tokio::test]
async fn test_get_hover_variable() {
    let workspace = TestWorkspace::new();
    let mut client = TestClient::new(workspace.path());

    let file_path = workspace.path().join("variable_hover.ts");

    std::fs::write(&file_path, r#"
interface Point {
    x: number;
    y: number;
}

const origin: Point = { x: 0, y: 0 };
const distance = Math.sqrt(origin.x ** 2 + origin.y ** 2);
"#).unwrap();

    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

    // Get hover information for the origin variable
    let response = client.call_tool("get_hover", json!({
        "file_path": file_path.to_string_lossy(),
        "line": 6,
        "character": 6
    })).await.unwrap();

    let hover_content = response["contents"].as_str().unwrap();
    assert!(hover_content.contains("Point") || hover_content.contains("origin"));
}

#[tokio::test]
async fn test_get_signature_help() {
    let workspace = TestWorkspace::new();
    let mut client = TestClient::new(workspace.path());

    let file_path = workspace.path().join("signature_test.ts");

    std::fs::write(&file_path, r#"
function greetUser(name: string, age: number, isActive: boolean = true): string {
    return `Hello ${name}, age ${age}, active: ${isActive}`;
}

const greeting = greetUser("Alice", 30,
"#).unwrap();

    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

    // Get signature help while typing function arguments
    let response = client.call_tool("get_signature_help", json!({
        "file_path": file_path.to_string_lossy(),
        "line": 5,
        "character": 35
    })).await.unwrap();

    let signatures = response["signatures"].as_array().unwrap();
    assert!(!signatures.is_empty());

    let signature = &signatures[0];
    let label = signature["label"].as_str().unwrap();
    assert!(label.contains("greetUser"));
    assert!(label.contains("name: string"));
    assert!(label.contains("age: number"));
}

#[tokio::test]
async fn test_search_workspace_symbols() {
    let workspace = TestWorkspace::new();

    // Setup proper TypeScript project structure with LSP config FIRST
    workspace.setup_typescript_project_with_lsp("workspace-symbols-test");

    // Create ALL TypeScript files in src/ directory BEFORE starting LSP client
    // (tsconfig.json includes only "src/**/*" files)
    workspace.create_file("src/models.ts", r#"
export class UserModel {
    constructor(public id: number, public name: string) {}

    getName(): string {
        return this.name;
    }
}

export interface UserData {
    id: number;
    name: string;
    email: string;
}
"#);

    workspace.create_file("src/services.ts", r#"
import { UserModel, UserData } from './models';

export class UserService {
    private users: UserModel[] = [];

    addUser(userData: UserData): UserModel {
        const user = new UserModel(userData.id, userData.name);
        this.users.push(user);
        return user;
    }

    findUserById(id: number): UserModel | undefined {
        return this.users.find(u => u.id === id);
    }
}
"#);

    // NOW create the TestClient after all files are in place
    let mut client = TestClient::new(workspace.path());

    println!("DEBUG: Test workspace path: {}", workspace.path().display());

    // Explicitly notify the LSP server about all TypeScript files
    // This ensures the server indexes them for symbol search
    let ts_files = ["src/models.ts", "src/services.ts", "tsconfig.json", "package.json"];
    for file in &ts_files {
        let file_path = workspace.path().join(file);
        if file_path.exists() {
            println!("DEBUG: Notifying LSP server about file: {}", file_path.display());
            client.call_tool("notify_file_opened", json!({
                "file_path": file_path.to_string_lossy()
            })).await.unwrap();
        }
    }

    // List all files recursively
    fn list_files_recursive(dir: &std::path::Path, prefix: &str) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let name = path.file_name().unwrap().to_string_lossy();
                if path.is_dir() {
                    println!("{}  {}/", prefix, name);
                    list_files_recursive(&path, &format!("{}  ", prefix));
                } else {
                    println!("{}  {}", prefix, name);
                }
            }
        }
    }

    println!("DEBUG: Workspace contents after all files created:");
    list_files_recursive(workspace.path(), "");

    // Verify tsconfig.json and package.json exist and print their contents
    let tsconfig_path = workspace.path().join("tsconfig.json");
    let package_path = workspace.path().join("package.json");
    if tsconfig_path.exists() {
        println!("DEBUG: tsconfig.json exists and contains:");
        println!("{}", std::fs::read_to_string(&tsconfig_path).unwrap_or_else(|e| format!("Error reading: {}", e)));
    } else {
        println!("DEBUG: tsconfig.json DOES NOT EXIST");
    }
    if package_path.exists() {
        println!("DEBUG: package.json exists and contains:");
        println!("{}", std::fs::read_to_string(&package_path).unwrap_or_else(|e| format!("Error reading: {}", e)));
    } else {
        println!("DEBUG: package.json DOES NOT EXIST");
    }

    // Give TypeScript language server time to initialize and scan the project
    // TypeScript server needs significant time to:
    // 1. Parse tsconfig.json and detect project structure
    // 2. Scan src/ directory and build internal index
    // 3. Be ready for workspace-level operations like symbol search
    println!("DEBUG: Waiting for TypeScript server project indexing...");
    tokio::time::sleep(tokio::time::Duration::from_millis(10000)).await;

    // Verify the server is ready by checking document symbols first
    // This is a simpler operation that should work if the project is recognized
    println!("DEBUG: Testing document symbols to verify project recognition...");
    let doc_symbols_test = client.call_tool("get_document_symbols", json!({
        "file_path": workspace.path().join("src/models.ts").to_string_lossy()
    })).await;

    match doc_symbols_test {
        Ok(response) => {
            println!("DEBUG: Document symbols test successful - file-level ops work");
        }
        Err(e) => {
            println!("DEBUG: Document symbols test failed: {}", e);
            // Continue anyway - this helps us understand if the issue is project-wide
        }
    }

    // BREAKTHROUGH: File-level operations work, workspace-level operations fail
    // This means TypeScript server needs explicit project configuration
    // Force TypeScript server to re-detect the project by opening tsconfig.json
    println!("DEBUG: Opening tsconfig.json to force project detection...");
    let _tsconfig_open = client.call_tool("notify_file_opened", json!({
        "file_path": workspace.path().join("tsconfig.json").to_string_lossy()
    })).await;

    // Give additional time for project re-detection
    println!("DEBUG: Waiting for project re-detection after tsconfig.json notification...");
    tokio::time::sleep(tokio::time::Duration::from_millis(5000)).await;

    // Search for symbols containing "User"
    let response = client.call_tool("search_workspace_symbols", json!({
        "query": "User"
    })).await.unwrap();

    // Print stderr logs to see server startup debug output
    let stderr_logs = client.get_stderr_logs();
    if !stderr_logs.is_empty() {
        println!("DEBUG: Server stderr logs:");
        for log in stderr_logs {
            println!("STDERR: {}", log);
        }
    }

    println!("DEBUG: Workspace symbols response: {:#}", response);

    // Check if there's an error first
    if let Some(error) = response.get("error") {
        panic!("Workspace symbols call failed: {}", error);
    }

    // Try multiple possible paths for the symbols
    let symbols = if let Some(content) = response.get("content") {
        if let Some(symbols) = content.get("symbols") {
            symbols.as_array()
        } else if let Some(symbols) = content.as_array() {
            Some(symbols)
        } else {
            None
        }
    } else if let Some(symbols) = response.get("symbols") {
        symbols.as_array()
    } else {
        None
    };

    let symbols = symbols.unwrap_or_else(|| {
        panic!("Could not find symbols in response. Available keys: {:?}",
               response.as_object().map(|o| o.keys().collect::<Vec<_>>()));
    });
    assert!(!symbols.is_empty());

    let symbol_names: Vec<String> = symbols.iter()
        .map(|s| s["name"].as_str().unwrap().to_string())
        .collect();

    // Should find UserModel, UserData, UserService, etc.
    assert!(symbol_names.iter().any(|name| name.contains("UserModel")));
    assert!(symbol_names.iter().any(|name| name.contains("UserService")));
}

#[tokio::test]
async fn test_get_document_symbols() {
    let workspace = TestWorkspace::new();
    let mut client = TestClient::new(workspace.path());

    let file_path = workspace.path().join("symbols.ts");

    std::fs::write(&file_path, r#"
export const API_URL = 'https://api.example.com';

export interface Config {
    timeout: number;
    retries: number;
}

export class ApiClient {
    private config: Config;

    constructor(config: Config) {
        this.config = config;
    }

    async get(endpoint: string): Promise<any> {
        return fetch(`${API_URL}/${endpoint}`);
    }

    async post(endpoint: string, data: any): Promise<any> {
        return fetch(`${API_URL}/${endpoint}`, {
            method: 'POST',
            body: JSON.stringify(data)
        });
    }
}

export function createClient(config: Config): ApiClient {
    return new ApiClient(config);
}
"#).unwrap();

    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

    let response = client.call_tool("get_document_symbols", json!({
        "file_path": file_path.to_string_lossy()
    })).await.unwrap();

    let symbols = response["symbols"].as_array().unwrap();
    assert!(!symbols.is_empty());

    let symbol_names: Vec<String> = symbols.iter()
        .map(|s| s["name"].as_str().unwrap().to_string())
        .collect();

    // Should find all top-level symbols
    assert!(symbol_names.contains(&"API_URL".to_string()));
    assert!(symbol_names.contains(&"Config".to_string()));
    assert!(symbol_names.contains(&"ApiClient".to_string()));
    assert!(symbol_names.contains(&"createClient".to_string()));

    // Check that class has methods as children
    let api_client_symbol = symbols.iter()
        .find(|s| s["name"].as_str().unwrap() == "ApiClient")
        .unwrap();

    if let Some(children) = api_client_symbol.get("children") {
        let children_array = children.as_array().unwrap();
        let method_names: Vec<String> = children_array.iter()
            .map(|c| c["name"].as_str().unwrap().to_string())
            .collect();

        assert!(method_names.contains(&"constructor".to_string()) ||
                method_names.contains(&"get".to_string()) ||
                method_names.contains(&"post".to_string()));
    }
}

#[tokio::test]
async fn test_cross_file_intelligence() {
    let workspace = TestWorkspace::new();
    let mut client = TestClient::new(workspace.path());

    // Create a more complex project structure
    let types_file = workspace.path().join("types.ts");
    let utils_file = workspace.path().join("utils.ts");
    let main_file = workspace.path().join("main.ts");

    std::fs::write(&types_file, r#"
export interface Product {
    id: string;
    name: string;
    price: number;
    category: string;
}

export type SortOrder = 'asc' | 'desc';
"#).unwrap();

    std::fs::write(&utils_file, r#"
import { Product, SortOrder } from './types';

export function sortProducts(products: Product[], order: SortOrder = 'asc'): Product[] {
    return products.sort((a, b) => {
        const multiplier = order === 'asc' ? 1 : -1;
        return (a.price - b.price) * multiplier;
    });
}

export function filterByCategory(products: Product[], category: string): Product[] {
    return products.filter(p => p.category === category);
}
"#).unwrap();

    std::fs::write(&main_file, r#"
import { Product } from './types';
import { sortProducts, filterByCategory } from './utils';

const products: Product[] = [
    { id: '1', name: 'Laptop', price: 999, category: 'Electronics' },
    { id: '2', name: 'Book', price: 29, category: 'Education' },
    { id: '3', name: 'Phone', price: 699, category: 'Electronics' }
];

const electronics = filterByCategory(products, 'Electronics');
const sortedElectronics = sortProducts(electronics, 'desc');

console.log(sortedElectronics);
"#).unwrap();

    tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;

    // Test 1: Find definition of Product interface from main.ts
    let response = client.call_tool("find_definition", json!({
        "file_path": main_file.to_string_lossy(),
        "line": 2,
        "character": 20
    })).await.unwrap();

    let locations = response["locations"].as_array().unwrap();
    assert!(!locations.is_empty());
    let def_uri = locations[0]["uri"].as_str().unwrap();
    assert!(def_uri.contains("types.ts"));

    // Test 2: Find references to sortProducts function
    let response = client.call_tool("find_references", json!({
        "file_path": utils_file.to_string_lossy(),
        "line": 3,
        "character": 17,
        "include_declaration": true
    })).await.unwrap();

    let references = response["references"].as_array().unwrap();
    assert!(references.len() >= 2); // declaration + usage in main.ts

    // Test 3: Get hover for filterByCategory in main.ts
    let response = client.call_tool("get_hover", json!({
        "file_path": main_file.to_string_lossy(),
        "line": 9,
        "character": 20
    })).await.unwrap();

    let hover_content = response["contents"].as_str().unwrap();
    assert!(hover_content.contains("filterByCategory") || hover_content.contains("Product"));

    // Test 4: Search for all Product-related symbols
    let response = client.call_tool("search_workspace_symbols", json!({
        "query": "Product"
    })).await.unwrap();

    let symbols = response["symbols"].as_array().unwrap();
    assert!(!symbols.is_empty());

    let symbol_names: Vec<String> = symbols.iter()
        .map(|s| s["name"].as_str().unwrap().to_string())
        .collect();

    assert!(symbol_names.iter().any(|name| name.contains("Product")));
}

#[tokio::test]
async fn test_lsp_intelligence_with_errors() {
    let workspace = TestWorkspace::new();
    let mut client = TestClient::new(workspace.path());

    let file_path = workspace.path().join("errors.ts");

    // Create file with intentional TypeScript errors
    std::fs::write(&file_path, r#"
interface User {
    id: number;
    name: string;
}

function processUser(user: User): void {
    console.log(user.name.toUpperCase());
    console.log(user.age); // Error: Property 'age' does not exist

    const invalidCall = nonExistentFunction(); // Error: function doesn't exist
}

const user: User = {
    id: 1,
    // Missing name property - error
    age: 30 // Error: Object literal may only specify known properties
};
"#).unwrap();

    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

    // Even with errors, LSP intelligence should still work for valid parts

    // Test 1: Find definition of User interface should work
    let response = client.call_tool("find_definition", json!({
        "file_path": file_path.to_string_lossy(),
        "line": 7,
        "character": 25
    })).await.unwrap();

    let locations = response["locations"].as_array().unwrap();
    assert!(!locations.is_empty());

    // Test 2: Get hover on valid property should work
    let response = client.call_tool("get_hover", json!({
        "file_path": file_path.to_string_lossy(),
        "line": 8,
        "character": 20
    })).await.unwrap();

    let hover_content = response["contents"].as_str().unwrap();
    assert!(hover_content.contains("name") || hover_content.contains("string"));

    // Test 3: Document symbols should still be extracted
    let response = client.call_tool("get_document_symbols", json!({
        "file_path": file_path.to_string_lossy()
    })).await.unwrap();

    let symbols = response["symbols"].as_array().unwrap();
    assert!(!symbols.is_empty());

    let symbol_names: Vec<String> = symbols.iter()
        .map(|s| s["name"].as_str().unwrap().to_string())
        .collect();

    assert!(symbol_names.contains(&"User".to_string()));
    assert!(symbol_names.contains(&"processUser".to_string()));
}