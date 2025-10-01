# Proposal: Standardized Error Reporting

## 1. Problem

The current error handling within the MCP server is inconsistent. Many tool failures return simple string-based errors, making it difficult for API clients to parse, handle, or display them effectively.

- **Inconsistent Format:** Errors lack a uniform structure.
- **Opaque Messages:** A simple string like "File not found" lacks the context (which file?) needed for programmatic handling or user feedback.
- **Difficult Debugging:** Clients cannot reliably distinguish between different types of errors (e.g., invalid input vs. an internal server issue).

**Before:**
```json
{
  "error": "File does not exist: /path/to/missing.ts"
}
```

## 2. Proposed Solution

We will introduce a standardized, structured error format for all tool-related failures.

1.  **Define a Standard `ApiError` Struct:** A new error struct will be added to `cb-core` with the following fields:
    - `code`: A unique, machine-readable error code (e.g., `E1001_FILE_NOT_FOUND`).
    - `message`: A clear, human-readable description of the error.
    - `details`: An optional JSON object containing relevant context (e.g., file paths, line numbers).

2.  **Refactor Tool Handlers:** All tool handlers in `plugin_dispatcher.rs` and its services will be updated to return this structured error on failure.

3.  **Document Error Codes:** A new "Error Reference" section will be added to `MCP_API.md` to document all possible error codes.

**After:**
```json
{
  "error": {
    "code": "E1001_FILE_NOT_FOUND",
    "message": "The specified file could not be found.",
    "details": {
      "file_path": "/path/to/missing.ts"
    }
  }
}
```

## 3. Benefits

- **Predictability:** Clients can build robust logic based on the stable `code` field.
- **Improved UX:** UIs can display user-friendly messages while having access to rich contextual data for debugging.
- **Maintainability:** Centralizes error definitions, making the system easier to manage and extend.
