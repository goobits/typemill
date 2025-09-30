use crate::error::{AstError, AstResult};
use syn::{visit::Visit, File};

// A visitor that will walk the AST and collect the names of all functions it finds.
struct FunctionVisitor {
    functions: Vec<String>,
}

impl<'ast> Visit<'ast> for FunctionVisitor {
    // This method is called for every function, method, or function-like item.
    fn visit_item_fn(&mut self, i: &'ast syn::ItemFn) {
        // Add the function's identifier (its name) to our list.
        self.functions.push(i.sig.ident.to_string());

        // Continue visiting to find nested functions inside this one.
        syn::visit::visit_item_fn(self, i);
    }

    // This method is called for every impl method.
    fn visit_impl_item_fn(&mut self, i: &'ast syn::ImplItemFn) {
        // Add the method's identifier (its name) to our list.
        self.functions.push(i.sig.ident.to_string());

        // Continue visiting to find nested functions inside this method.
        syn::visit::visit_impl_item_fn(self, i);
    }
}

/// Parses Rust source code and returns a list of all function and method names.
pub fn list_functions(source: &str) -> AstResult<Vec<String>> {
    // Parse the source code into a syn::File, which is the root of the AST.
    let ast: File =
        syn::parse_file(source).map_err(|e| AstError::analysis(format!("Failed to parse Rust code: {}", e)))?;

    // Create an instance of our visitor.
    let mut visitor = FunctionVisitor {
        functions: Vec::new(),
    };

    // Walk the AST, starting from the root file node.
    visitor.visit_file(&ast);

    // Return the collected function names.
    Ok(visitor.functions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_rust_functions_and_methods() {
        let source = r#"
fn top_level() {}

struct MyStruct;

impl MyStruct {
    fn my_method() {}
    fn another_method(&self) {}
}

mod my_mod {
    fn function_in_mod() {}
}
"#;
        let functions = list_functions(source).unwrap();
        assert_eq!(functions.len(), 4);
        assert!(functions.contains(&"top_level".to_string()));
        assert!(functions.contains(&"my_method".to_string()));
        assert!(functions.contains(&"another_method".to_string()));
        assert!(functions.contains(&"function_in_mod".to_string()));
    }

    #[test]
    fn test_list_rust_nested_functions() {
        let source = r#"
fn outer() {
    fn inner() {}
}
"#;
        let functions = list_functions(source).unwrap();
        assert_eq!(functions.len(), 2);
        assert!(functions.contains(&"outer".to_string()));
        assert!(functions.contains(&"inner".to_string()));
    }

    #[test]
    fn test_list_rust_syntax_error() {
        let source = "fn my_func {";
        let result = list_functions(source);
        assert!(result.is_err());
    }
}
