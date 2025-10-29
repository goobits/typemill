//! PHP AST parsing functionality using tree-sitter.
//!
//! This module provides PHP source code parsing to extract symbols
//! like classes, functions, interfaces, and traits.

use mill_plugin_api::{Symbol, SymbolKind, SourceLocation, PluginResult};
use tree_sitter::{Parser, Query, QueryCursor};

/// Extracts symbols from PHP source code.
pub fn extract_symbols(source: &str) -> PluginResult<Vec<Symbol>> {
    let mut parser = Parser::new();
    let language_fn = tree_sitter_php::LANGUAGE_PHP;
    parser
        .set_language(language_fn.into())
        .map_err(|e| mill_plugin_api::PluginError::internal(e.to_string()))?;

    let tree = parser.parse(source, None).ok_or_else(|| {
        mill_plugin_api::PluginError::internal("Failed to parse PHP code".to_string())
    })?;

    let query_str = "
        (class_declaration name: (name) @name) @class
        (interface_declaration name: (name) @name) @interface
        (trait_declaration name: (name) @name) @trait
        (function_definition name: (name) @name) @function
        (method_declaration name: (name) @name) @function
    ";

    let query = Query::new(language_fn.into(), query_str)
        .map_err(|e| mill_plugin_api::PluginError::internal(e.to_string()))?;

    let mut cursor = QueryCursor::new();
    let matches = cursor.matches(&query, tree.root_node(), source.as_bytes());

    let mut symbols = Vec::new();

    for mat in matches {
        let node = mat.captures[0].node;
        let name_node = mat.captures[1].node;

        let name = name_node.utf8_text(source.as_bytes()).unwrap_or("").to_string();
        let kind = match node.kind() {
            "class_declaration" => SymbolKind::Class,
            "interface_declaration" => SymbolKind::Interface,
            "trait_declaration" => SymbolKind::Interface, // No Trait kind, using Interface
            "function_definition" | "method_declaration" => SymbolKind::Function,
            _ => SymbolKind::Other,
        };

        symbols.push(Symbol {
            name,
            kind,
            location: SourceLocation {
                line: node.start_position().row,
                column: node.start_position().column,
            },
            documentation: None,
        });
    }

    Ok(symbols)
}