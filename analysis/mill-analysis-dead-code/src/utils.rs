// analysis/mill-analysis-dead-code/src/utils.rs

/// Convert LSP SymbolKind number to human-readable string
pub(crate) fn lsp_kind_to_string(kind: u64) -> String {
    match kind {
        1 => "file",
        2 => "module",
        3 => "namespace",
        4 => "package",
        5 => "class",
        6 => "method",
        7 => "property",
        8 => "field",
        9 => "constructor",
        10 => "enum",
        11 => "interface",
        12 => "function",
        13 => "variable",
        14 => "constant",
        15 => "string",
        16 => "number",
        17 => "boolean",
        18 => "array",
        19 => "object",
        20 => "key",
        21 => "null",
        22 => "enum_member",
        23 => "struct",
        24 => "event",
        25 => "operator",
        26 => "type_parameter",
        _ => "unknown",
    }
    .to_string()
}

/// Check if a symbol appears to be exported based on heuristic analysis.
pub(crate) fn is_symbol_exported(file_path: &str, line: u32) -> bool {
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    let file = match File::open(file_path) {
        Ok(f) => f,
        Err(_) => return false,
    };

    let reader = BufReader::new(file);
    if let Some(Ok(line_content)) = reader.lines().nth(line as usize) {
        let line_lower = line_content.to_lowercase();
        return line_lower.contains("export ")
            || line_lower.contains("pub ")
            || line_lower.contains("public ");
    }

    false
}
