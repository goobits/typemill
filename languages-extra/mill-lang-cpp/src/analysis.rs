//! Analysis capabilities for C++ code
//!
//! Provides stub implementations for code analysis operations.

use mill_foundation::protocol::{
    ImportGraph, ImportGraphMetadata, ImportInfo, ImportType, SourceLocation,
};
use mill_plugin_api::{
    capabilities::{ImportAnalyzer, ModuleReferenceScanner},
    ModuleReference, PluginResult, ReferenceKind, ScanScope,
};
use std::path::Path;

use crate::constants::{module_include_pattern, INCLUDE_PATTERN};

pub struct CppAnalysisProvider;

impl ModuleReferenceScanner for CppAnalysisProvider {
    fn scan_references(
        &self,
        content: &str,
        module_name: &str,
        _scope: ScanScope,
    ) -> PluginResult<Vec<ModuleReference>> {
        let re = module_include_pattern(module_name);
        let references = re
            .captures_iter(content)
            .map(|caps| {
                let m = caps.get(0).unwrap();
                let line = content[..m.start()].lines().count();
                // Calculate column safely: if line is 0, sum is 0
                let line_start_offset = if line == 0 {
                    0
                } else {
                    content
                        .lines()
                        .take(line.saturating_sub(1))
                        .map(|l| l.len() + 1)
                        .sum::<usize>()
                };
                let column = m.start().saturating_sub(line_start_offset);
                ModuleReference {
                    line,
                    column,
                    length: m.len(),
                    text: caps.get(1).unwrap().as_str().to_string(),
                    kind: ReferenceKind::Declaration,
                }
            })
            .collect();
        Ok(references)
    }
}

impl ImportAnalyzer for CppAnalysisProvider {
    fn build_import_graph(&self, file_path: &Path) -> PluginResult<ImportGraph> {
        let content = std::fs::read_to_string(file_path).map_err(|e| {
            mill_plugin_api::PluginApiError::internal(format!("Failed to read file: {}", e))
        })?;
        let imports = INCLUDE_PATTERN
            .captures_iter(&content)
            .map(|caps| {
                let m = caps.get(0).unwrap();
                let start_byte = m.start();
                let mut line_number = 0;
                let mut last_line_start = 0;
                for (i, line) in content.lines().enumerate() {
                    if last_line_start + line.len() >= start_byte {
                        line_number = i;
                        break;
                    }
                    last_line_start += line.len() + 1;
                }
                let start_column = start_byte - last_line_start;

                ImportInfo {
                    module_path: caps.get(1).unwrap().as_str().to_string(),
                    import_type: ImportType::CInclude,
                    named_imports: vec![],
                    default_import: None,
                    namespace_import: None,
                    type_only: false,
                    location: SourceLocation {
                        start_line: line_number as u32,
                        start_column: start_column as u32,
                        end_line: line_number as u32,
                        end_column: (start_column + m.len()) as u32,
                    },
                }
            })
            .collect();

        Ok(ImportGraph {
            source_file: file_path.to_string_lossy().to_string(),
            imports,
            importers: vec![],
            metadata: ImportGraphMetadata {
                language: "C++".to_string(),
                parsed_at: chrono::Utc::now(),
                parser_version: "0.1.0".to_string(),
                circular_dependencies: vec![],
                external_dependencies: vec![],
            },
        })
    }
}
