//! Shared helper functions for reference updating
//!
//! This module contains utilities shared between reference_updater and move_service
//! to avoid code duplication.

use mill_foundation::protocol::{EditLocation, EditType, TextEdit};
use std::path::Path;

/// Compute line count and last line length from content
///
/// Returns (line_count, last_line_length) where:
/// - line_count is at least 1
/// - last_line_length is the length of the final line (0 if empty)
#[inline]
pub fn compute_line_info(content: &str) -> (usize, usize) {
    let line_count = content.lines().count().max(1);
    let last_line_len = content.lines().last().map(|l| l.len()).unwrap_or(0);
    (line_count, last_line_len)
}

/// Create a full-file replacement TextEdit
///
/// This is the common pattern used when a plugin rewrites file references.
/// The edit replaces the entire file content.
pub fn create_full_file_edit(
    file_path: &Path,
    original_content: String,
    new_content: String,
    edit_type: EditType,
    priority: u32,
    description: String,
) -> TextEdit {
    let (line_count, last_line_len) = compute_line_info(&original_content);

    TextEdit {
        file_path: Some(file_path.to_string_lossy().to_string()),
        edit_type,
        location: EditLocation {
            start_line: 0,
            start_column: 0,
            end_line: line_count.saturating_sub(1) as u32,
            end_column: last_line_len as u32,
        },
        original_text: original_content,
        new_text: new_content,
        priority,
        description,
    }
}

/// Create a TextEdit for import updates (priority 1)
pub fn create_import_update_edit(
    file_path: &Path,
    original_content: String,
    new_content: String,
    change_count: usize,
    context: &str,
) -> TextEdit {
    create_full_file_edit(
        file_path,
        original_content,
        new_content,
        EditType::UpdateImport,
        1,
        format!(
            "Update {} imports in {} for {}",
            change_count,
            file_path.display(),
            context
        ),
    )
}

/// Create a TextEdit for path reference updates (priority 0)
pub fn create_path_reference_edit(
    file_path: &Path,
    original_content: String,
    new_content: String,
    change_count: usize,
) -> TextEdit {
    create_full_file_edit(
        file_path,
        original_content,
        new_content,
        EditType::Replace,
        0,
        format!(
            "Update {} path references in {}",
            change_count,
            file_path.display()
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_line_info_single_line() {
        let (lines, last_len) = compute_line_info("hello world");
        assert_eq!(lines, 1);
        assert_eq!(last_len, 11);
    }

    #[test]
    fn test_compute_line_info_multi_line() {
        let (lines, last_len) = compute_line_info("line1\nline2\nend");
        assert_eq!(lines, 3);
        assert_eq!(last_len, 3);
    }

    #[test]
    fn test_compute_line_info_empty() {
        let (lines, last_len) = compute_line_info("");
        assert_eq!(lines, 1);
        assert_eq!(last_len, 0);
    }

    #[test]
    fn test_compute_line_info_trailing_newline() {
        let (lines, last_len) = compute_line_info("line1\nline2\n");
        assert_eq!(lines, 2);
        assert_eq!(last_len, 5); // "line2" is the last line
    }

    #[test]
    fn test_create_full_file_edit() {
        let edit = create_full_file_edit(
            Path::new("/test/file.rs"),
            "old content".to_string(),
            "new content".to_string(),
            EditType::Replace,
            1,
            "test edit".to_string(),
        );

        assert_eq!(edit.file_path, Some("/test/file.rs".to_string()));
        assert_eq!(edit.location.start_line, 0);
        assert_eq!(edit.location.end_line, 0);
        assert_eq!(edit.original_text, "old content");
        assert_eq!(edit.new_text, "new content");
    }
}
