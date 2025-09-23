# MCP Tool Issues Analysis & Fix Proposals

## Issues Discovered During Phase 2 Refactoring

### 1. **.bak File Creation Issue**

**Problem:** Every `apply_workspace_edit` operation creates backup files (.bak)
**Root Cause:** In `src/core/file-operations/editor.ts:70`
```typescript
createBackupFiles = validateBeforeApply  // Defaults to true
```

**Current Behavior:**
- When `validateBeforeApply` is true (default), backup files are created
- MCP handlers don't explicitly set `createBackupFiles: false`
- Results in .bak files littering the codebase

**Proposed Fix:**
```typescript
// In src/mcp/handlers/advanced-handlers.ts:496
const result = await applyWorkspaceEdit(workspaceEdit, {
  validateBeforeApply: true,
  createBackupFiles: false,  // Add this line
  lspClient,
});
```

### 2. **Corrupted Import Lines Issue**

**Problem:** `apply_workspace_edit` corrupted imports like:
```typescript
// Before
import { isProcessRunning } from '../../utils/platform-utils.js';

// After (corrupted)
import { isProcessRunning } from '../../utils/platform/process.js';rver-utils.js';
```

**Root Cause:** Incorrect character ranges in workspace edits
- When specifying `end: { line: X, character: 100 }` on a line with only 90 characters
- The edit partially replaces the line, leaving remnants

**Analysis:**
- The MCP tool validates line numbers but not character positions
- When character position exceeds line length, it should fail gracefully

**Proposed Fix:**
```typescript
// In src/core/file-operations/editor.ts - Add validation
function validateTextEdit(content: string, edit: TextEdit): void {
  const lines = content.split('\n');
  const { start, end } = edit.range;
  
  // Validate line numbers
  if (start.line >= lines.length || end.line >= lines.length) {
    throw new Error(`Invalid line number`);
  }
  
  // Validate character positions
  const startLineLength = lines[start.line].length;
  const endLineLength = lines[end.line].length;
  
  if (start.character > startLineLength) {
    throw new Error(`Invalid start character ${start.character} on line ${start.line} (line has ${startLineLength} characters)`);
  }
  
  if (end.character > endLineLength) {
    throw new Error(`Invalid end character ${end.character} on line ${end.line} (line has ${endLineLength} characters)`);
  }
}
```

### 3. **Relative Path Issues After Moving Files**

**Problem:** After using `rename_file` to move services:
- `src/services/symbol-service.ts` → `src/services/lsp/symbol-service.ts`
- Internal imports weren't adjusted: `../core/` should become `../../core/`

**Root Cause:** The `rename_file` tool updates external imports but not internal relative imports within the moved file

**Current Behavior:**
- External imports TO the file are updated ✅
- Internal imports FROM the file are NOT updated ❌

**Proposed Fix:**
```typescript
// In file rename handler - Also update internal imports
async function handleRenameFile(oldPath: string, newPath: string) {
  // 1. Update all imports TO this file (current behavior) ✅
  
  // 2. NEW: Update imports WITHIN the moved file
  const depthChange = calculateDepthChange(oldPath, newPath);
  if (depthChange !== 0) {
    await updateRelativeImports(newPath, depthChange);
  }
}

function calculateDepthChange(oldPath: string, newPath: string): number {
  const oldDepth = oldPath.split('/').length;
  const newDepth = newPath.split('/').length;
  return newDepth - oldDepth;
}

function updateRelativeImports(filePath: string, depthChange: number) {
  // Add or remove '../' based on depth change
  // '../core/' becomes '../../core/' if depthChange = 1
}
```

## Summary of Fixes

### Quick Fixes (Configuration)
1. **Disable .bak files**: Set `createBackupFiles: false` in MCP handlers
2. **Add environment variable**: `MCP_CREATE_BACKUPS=false`

### Code Fixes (Validation)
1. **Character validation**: Validate character positions before applying edits
2. **Better error messages**: Include line content and length in errors
3. **Automatic range adjustment**: Clamp character positions to line length

### Advanced Fixes (Intelligence)
1. **Smart import updates**: Update relative imports when moving files
2. **Preview mode**: Show what will change before applying
3. **Atomic rollback**: Keep backups in memory, not .bak files

## Recommendations

1. **Immediate**: Add `createBackupFiles: false` to all MCP handlers
2. **Short-term**: Implement character position validation
3. **Long-term**: Enhance `rename_file` to update internal imports

These fixes would make MCP tools more robust and prevent the issues we encountered during Phase 2 refactoring.