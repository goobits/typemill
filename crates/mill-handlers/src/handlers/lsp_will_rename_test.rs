//! Test for LSP workspace/willRenameFiles functionality
//!
//! This module tests:
//! 1. Helper functions for extracting files from WorkspaceEdit responses
//! 2. (Integration) The actual LSP willRenameFiles behavior (requires LSP server)

#[cfg(test)]
mod tests {
    use serde_json::json;
    use std::path::PathBuf;

    /// Extract file paths from a WorkspaceEdit response
    ///
    /// WorkspaceEdit can have two formats:
    /// 1. "changes": { uri -> TextEdit[] }
    /// 2. "documentChanges": TextDocumentEdit[]
    ///
    /// This function handles both and returns unique file paths.
    pub fn extract_affected_files_from_workspace_edit(
        workspace_edit: &serde_json::Value,
    ) -> Vec<PathBuf> {
        let mut files = std::collections::HashSet::new();

        // Format 1: "changes" (uri -> edits[])
        if let Some(changes) = workspace_edit.get("changes").and_then(|c| c.as_object()) {
            for uri in changes.keys() {
                if let Some(path) = uri_to_path(uri) {
                    files.insert(path);
                }
            }
        }

        // Format 2: "documentChanges" (array of TextDocumentEdit)
        if let Some(doc_changes) = workspace_edit
            .get("documentChanges")
            .and_then(|d| d.as_array())
        {
            for change in doc_changes {
                // TextDocumentEdit has textDocument.uri
                if let Some(uri) = change
                    .get("textDocument")
                    .and_then(|td| td.get("uri"))
                    .and_then(|u| u.as_str())
                {
                    if let Some(path) = uri_to_path(uri) {
                        files.insert(path);
                    }
                }
            }
        }

        files.into_iter().collect()
    }

    /// Convert a file:// URI to a PathBuf
    fn uri_to_path(uri: &str) -> Option<PathBuf> {
        if !uri.starts_with("file://") {
            return None;
        }
        let path_str = uri.trim_start_matches("file://");
        // Handle URL-encoded paths (spaces become %20, etc.)
        match urlencoding::decode(path_str) {
            Ok(decoded) => Some(PathBuf::from(decoded.as_ref())),
            Err(_) => Some(PathBuf::from(path_str)),
        }
    }

    // ============== Unit Tests ==============

    /// Test extracting files from "changes" format (uri -> edits[])
    #[test]
    fn test_extract_from_changes_format() {
        let workspace_edit = json!({
            "changes": {
                "file:///project/consumer.ts": [
                    {"range": {"start": {"line": 0, "character": 30}, "end": {"line": 0, "character": 37}}, "newText": "./services/api"}
                ],
                "file:///project/other.ts": [
                    {"range": {"start": {"line": 0, "character": 26}, "end": {"line": 0, "character": 33}}, "newText": "./services/api"}
                ]
            }
        });

        let files = extract_affected_files_from_workspace_edit(&workspace_edit);
        assert_eq!(files.len(), 2);
        assert!(files
            .iter()
            .any(|p| p.to_string_lossy().contains("consumer.ts")));
        assert!(files
            .iter()
            .any(|p| p.to_string_lossy().contains("other.ts")));
    }

    /// Test extracting files from "documentChanges" format
    #[test]
    fn test_extract_from_document_changes_format() {
        let workspace_edit = json!({
            "documentChanges": [
                {
                    "textDocument": {"uri": "file:///project/consumer.ts", "version": 1},
                    "edits": [{"range": {"start": {"line": 0, "character": 30}, "end": {"line": 0, "character": 37}}, "newText": "./services/api"}]
                },
                {
                    "textDocument": {"uri": "file:///project/other.ts", "version": 1},
                    "edits": [{"range": {"start": {"line": 0, "character": 26}, "end": {"line": 0, "character": 33}}, "newText": "./services/api"}]
                }
            ]
        });

        let files = extract_affected_files_from_workspace_edit(&workspace_edit);
        assert_eq!(files.len(), 2);
        assert!(files
            .iter()
            .any(|p| p.to_string_lossy().contains("consumer.ts")));
        assert!(files
            .iter()
            .any(|p| p.to_string_lossy().contains("other.ts")));
    }

    /// Test empty WorkspaceEdit returns empty list
    #[test]
    fn test_extract_from_empty_workspace_edit() {
        let workspace_edit = json!({});
        let files = extract_affected_files_from_workspace_edit(&workspace_edit);
        assert!(files.is_empty());
    }

    /// Test null WorkspaceEdit returns empty list
    #[test]
    fn test_extract_from_null_workspace_edit() {
        let workspace_edit = json!(null);
        let files = extract_affected_files_from_workspace_edit(&workspace_edit);
        assert!(files.is_empty());
    }

    /// Test URL-encoded paths are decoded correctly
    #[test]
    fn test_extract_url_encoded_paths() {
        let workspace_edit = json!({
            "changes": {
                "file:///project/My%20Documents/file.ts": [
                    {"range": {"start": {"line": 0, "character": 0}, "end": {"line": 0, "character": 10}}, "newText": "new"}
                ]
            }
        });

        let files = extract_affected_files_from_workspace_edit(&workspace_edit);
        assert_eq!(files.len(), 1);
        assert!(files
            .iter()
            .any(|p| p.to_string_lossy().contains("My Documents")));
    }

    /// Test filtering to project files only
    #[test]
    fn test_filter_to_project_files() {
        let project_root = PathBuf::from("/home/user/project");
        let workspace_edit = json!({
            "changes": {
                "file:///home/user/project/src/app.ts": [{"range": {}, "newText": ""}],
                "file:///opt/node/lib/typescript/lib.d.ts": [{"range": {}, "newText": ""}],
                "file:///home/user/project/src/utils.ts": [{"range": {}, "newText": ""}]
            }
        });

        let files = extract_affected_files_from_workspace_edit(&workspace_edit);

        // Filter to project files only
        let project_files: Vec<_> = files
            .into_iter()
            .filter(|f| f.starts_with(&project_root))
            .collect();

        assert_eq!(project_files.len(), 2);
        assert!(project_files.iter().any(|p| p.ends_with("app.ts")));
        assert!(project_files.iter().any(|p| p.ends_with("utils.ts")));
    }
}
