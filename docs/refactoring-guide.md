# CodeFlow Buddy: Advanced Refactoring Guide

This guide explains how to use CodeFlow Buddy's powerful orchestration tools for safe, atomic refactoring operations.

## Overview

CodeFlow Buddy provides three sophisticated orchestration tools that work together to make complex refactoring operations safe and predictable:

- **`analyze_refactor_impact`** - Analyze the impact of planned changes before executing them
- **`preview_batch_operation`** - Preview exactly what changes will be made without applying them
- **`batch_move_files`** - Execute multiple file moves atomically with automatic rollback on failure

These tools leverage your existing Language Server Protocol (LSP) infrastructure to provide intelligent dependency analysis and automatic import updates.

## Key Concepts

### Atomic Operations
All batch operations are truly atomic - either all changes succeed, or all changes are rolled back to the original state. This prevents your codebase from being left in a broken, partially-refactored state.

### Intelligent Analysis
The tools use your language server to understand dependencies, analyze symbol usage, and calculate precise impact assessments rather than relying on simple text matching.

### Safe-by-Default
The default strategy is "safe" - operations abort on the first error. You can optionally use "force" strategy to continue despite non-critical errors.

## Complete Refactoring Workflow

Here's the recommended workflow for complex refactoring operations:

### Step 1: Plan Your Changes

First, identify the operations you want to perform. Operations can include:

- **File moves**: Moving files to new locations
- **Symbol renames**: Renaming classes, functions, variables, etc.

### Step 2: Analyze Impact

Use `analyze_refactor_impact` to understand the scope and risk of your planned changes:

```json
{
  "operations": [
    {
      "type": "move_file",
      "old_path": "src/utils/string-helpers.ts",
      "new_path": "lib/utilities/string-utils.ts"
    },
    {
      "type": "move_file",
      "old_path": "src/data/processor.ts",
      "new_path": "lib/services/data-processor.ts"
    },
    {
      "type": "rename_symbol",
      "file_path": "lib/services/data-processor.ts",
      "symbol_name": "DataProcessor",
      "new_name": "EnhancedDataProcessor"
    }
  ],
  "include_recommendations": true
}
```

The analysis will show you:
- **Risk level** for each operation (low/medium/high)
- **Estimated number of files** that will be modified
- **Dependent files** that import the files you're moving
- **Recommendations** for safe execution

Example output:
```
## Refactoring Impact Analysis

**Operations**: 3
**Estimated file changes**: 12
**Unique files affected**: 8
**Risk assessment**: 1 high, 1 medium, 1 low

### Recommendations

⚠️ **1 high-risk operation(s)** - consider breaking into smaller batches
⚠️ **High impact** - 12 estimated changes across 8 files

### Execution Strategy
- Use `dry_run: true` first to preview changes
- Consider executing high-risk operations individually
- Ensure you have backup/version control
```

### Step 3: Preview Changes

Use `preview_batch_operation` to see exactly what will happen:

```json
{
  "operations": [
    {
      "type": "move_file",
      "old_path": "src/utils/string-helpers.ts",
      "new_path": "lib/utilities/string-utils.ts"
    }
  ],
  "detailed": true
}
```

This shows you:
- **Validation status** for each operation
- **Import updates** that will be made
- **File-by-file preview** of changes (when detailed=true)

### Step 4: Execute with Dry Run

Before making real changes, always test with `dry_run: true`:

```json
{
  "moves": [
    {
      "old_path": "src/utils/string-helpers.ts",
      "new_path": "lib/utilities/string-utils.ts"
    }
  ],
  "dry_run": true,
  "strategy": "safe"
}
```

This will show you exactly what would happen without making any actual changes.

### Step 5: Execute the Refactoring

Finally, execute the actual refactoring:

```json
{
  "moves": [
    {
      "old_path": "src/utils/string-helpers.ts",
      "new_path": "lib/utilities/string-utils.ts"
    }
  ],
  "dry_run": false,
  "strategy": "safe"
}
```

## Detailed Examples

### Example 1: Simple File Reorganization

**Scenario**: Moving utility files from `src/utils/` to `lib/common/`

**Step 1 - Analyze Impact**:
```json
{
  "operations": [
    {
      "type": "move_file",
      "old_path": "src/utils/string-utils.ts",
      "new_path": "lib/common/string-utils.ts"
    },
    {
      "type": "move_file",
      "old_path": "src/utils/date-utils.ts",
      "new_path": "lib/common/date-utils.ts"
    }
  ]
}
```

**Step 2 - Preview Changes**:
```json
{
  "operations": [
    {
      "type": "move_file",
      "old_path": "src/utils/string-utils.ts",
      "new_path": "lib/common/string-utils.ts"
    },
    {
      "type": "move_file",
      "old_path": "src/utils/date-utils.ts",
      "new_path": "lib/common/date-utils.ts"
    }
  ],
  "detailed": false
}
```

**Step 3 - Execute**:
```json
{
  "moves": [
    {
      "old_path": "src/utils/string-utils.ts",
      "new_path": "lib/common/string-utils.ts"
    },
    {
      "old_path": "src/utils/date-utils.ts",
      "new_path": "lib/common/date-utils.ts"
    }
  ],
  "strategy": "safe"
}
```

### Example 2: Component Refactoring with Symbol Rename

**Scenario**: Moving a React component and renaming it

**Step 1 - Analyze Combined Impact**:
```json
{
  "operations": [
    {
      "type": "move_file",
      "old_path": "src/components/Button.tsx",
      "new_path": "src/ui/ActionButton.tsx"
    },
    {
      "type": "rename_symbol",
      "file_path": "src/ui/ActionButton.tsx",
      "symbol_name": "Button",
      "new_name": "ActionButton"
    }
  ]
}
```

**Note**: For symbol renames, execute them separately after file moves:

1. First, move the file using `batch_move_files`
2. Then, rename the symbol using the regular `rename_symbol` tool

### Example 3: Large-Scale Reorganization

**Scenario**: Reorganizing an entire feature module

**Strategy**: Break into smaller, safer batches:

**Batch 1 - Move core files**:
```json
{
  "moves": [
    {
      "old_path": "src/features/auth/types.ts",
      "new_path": "src/modules/authentication/types.ts"
    },
    {
      "old_path": "src/features/auth/constants.ts",
      "new_path": "src/modules/authentication/constants.ts"
    }
  ]
}
```

**Batch 2 - Move service files**:
```json
{
  "moves": [
    {
      "old_path": "src/features/auth/service.ts",
      "new_path": "src/modules/authentication/auth-service.ts"
    }
  ]
}
```

**Batch 3 - Move UI components**:
```json
{
  "moves": [
    {
      "old_path": "src/features/auth/components/LoginForm.tsx",
      "new_path": "src/modules/authentication/components/LoginForm.tsx"
    }
  ]
}
```

## Error Handling and Recovery

### Rollback Mechanism

If any operation fails during a batch move, the system automatically:

1. **Stops execution** immediately
2. **Rolls back file moves** that were already completed
3. **Restores original file content** for any import updates that were applied
4. **Reports the exact failure** and what was rolled back

Example failure output:
```
## Batch Move Validation Failed

**Total operations**: 3
**Validation errors**: 1

### ❌ Validation Errors

1. **src/old-file.ts → src/new-file.ts**
   Error: Target file already exists: src/new-file.ts

**Strategy**: safe - aborting due to validation errors
```

### Common Issues and Solutions

**Issue**: "Target file already exists"
- **Solution**: Check if the target location already has a file with that name
- **Prevention**: Use `preview_batch_operation` first to catch conflicts

**Issue**: "Source file does not exist"
- **Solution**: Verify the source path is correct
- **Prevention**: Double-check file paths before execution

**Issue**: "High-risk operation detected"
- **Solution**: Break the operation into smaller batches
- **Prevention**: Use `analyze_refactor_impact` to identify high-risk operations

## Best Practices

### 1. Always Use the Three-Step Process

1. **Analyze** → 2. **Preview** → 3. **Execute**

Never skip the analysis and preview steps for non-trivial changes.

### 2. Start with Dry Runs

Always test with `dry_run: true` before executing real changes:

```json
{
  "dry_run": true,  // ← Always start with this
  "strategy": "safe"
}
```

### 3. Use Safe Strategy by Default

The "safe" strategy aborts on any error, preventing partial failures:

```json
{
  "strategy": "safe"  // ← Recommended default
}
```

Only use `"force"` strategy when you understand the risks and want to continue despite non-critical errors.

### 4. Break Large Operations into Batches

For high-risk or high-impact operations:
- Move files in logical groups
- Execute symbol renames separately
- Verify each batch before proceeding

### 5. Commit Frequently

- Commit your changes before starting refactoring
- Commit after each successful batch
- This provides additional safety and rollback options

### 6. Monitor Dependencies

Pay attention to the dependency analysis:
- **Low risk**: 0-1 dependent files
- **Medium risk**: 2-5 dependent files
- **High risk**: 6+ dependent files

High-risk operations should be executed with extra caution.

## Advanced Usage

### Custom Import Path Patterns

The system automatically handles various import patterns:
- Relative imports: `./utils`, `../services`
- Absolute imports: `/src/components`
- Module imports: `@/utils`, `~/components`

### Symbol Rename Integration

While `batch_move_files` only handles file operations, you can sequence it with symbol renames:

1. Use `batch_move_files` to move files
2. Use `rename_symbol` to rename symbols in the moved files
3. Use `analyze_refactor_impact` to plan the sequence

### TypeScript Project Support

The tools work seamlessly with TypeScript projects and respect:
- `tsconfig.json` path mapping
- Module resolution settings
- Import/export syntax variations

## Troubleshooting

### Language Server Issues

If you encounter errors like "LSP server not responding":

1. Check that your language server is properly configured
2. Ensure the file types are supported by your LSP setup
3. Try restarting the LSP servers: `restart_server`

### Permission Issues

If you see "Failed to backup file" errors:
- Check file permissions in your project directory
- Ensure you have write access to both source and target locations

### Large Project Performance

For very large projects (1000+ files):
- Break operations into smaller batches (10-20 files at a time)
- Use `include_recommendations: false` to speed up impact analysis
- Consider running operations during low-activity periods

## Integration with Development Workflow

### IDE Integration

CodeFlow Buddy integrates with any MCP-compatible environment:
- Claude Desktop
- VS Code (with MCP extension)
- Other MCP-enabled editors

### CI/CD Integration

For automated refactoring in CI/CD:
- Always use `dry_run: true` first in CI
- Require manual approval for actual execution
- Use `strategy: "safe"` to prevent partial failures

### Version Control

Recommended git workflow:
```bash
# Before refactoring
git add .
git commit -m "Pre-refactoring checkpoint"

# Execute refactoring with CodeFlow Buddy
# (files are moved and imports updated automatically)

# After successful refactoring
git add .
git commit -m "Refactor: Move utilities to lib/common"
```

## Conclusion

CodeFlow Buddy's orchestration tools provide enterprise-grade safety and intelligence for complex refactoring operations. By following the analyze → preview → execute workflow and using atomic operations with automatic rollback, you can confidently restructure large codebases without the risk of introducing breaking changes.

The key to successful refactoring is taking a methodical approach: understand the impact, preview the changes, test with dry runs, and execute in controlled batches. These tools make it possible to perform refactoring operations that would be extremely risky or time-consuming to do manually.