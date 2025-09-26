# CodeFlow Buddy: Advanced Refactoring Guide

This guide explains how to use CodeFlow Buddy's powerful batch execution system for safe, atomic refactoring operations.

## 🚨 Best Practices - READ THIS FIRST

### Critical Guidelines for Safe Refactoring

1. **ALWAYS use `batch_execute` for multiple operations**
   - `batch_execute` is the ONLY tool designed for running multiple operations safely
   - It provides atomic rollback if any operation fails
   - Other tools are designed for single operations only

2. **Use the correct tool for your task**:
   - **`rename_directory`** - For moving entire directories and all their contents
   - **`rename_file`** - For moving individual files
   - **`batch_execute`** - For combining multiple operations atomically

3. **Follow the three-step safety process**:
   - Step 1: Preview with `dry_run: true`
   - Step 2: Test execution logic
   - Step 3: Execute with `atomic: true`

4. **Large-scale refactoring example**:
```json
{
  "operations": [
    {
      "tool": "rename_directory",
      "args": {
        "old_path": "src/old-feature",
        "new_path": "lib/new-feature"
      }
    },
    {
      "tool": "rename_directory",
      "args": {
        "old_path": "src/utils",
        "new_path": "lib/shared"
      }
    }
  ],
  "options": {
    "atomic": true,
    "stop_on_error": true
  }
}
```

**⚠️ WARNING**: Never run individual file operations outside of `batch_execute` when you need to move multiple files. The individual tools lack the coordination needed for complex refactoring.

## Overview

CodeFlow Buddy provides a powerful `batch_execute` tool that enables you to perform complex refactoring operations safely and efficiently. This tool leverages your existing Language Server Protocol (LSP) infrastructure to provide intelligent dependency analysis and automatic import updates.

## Key Concepts

### Atomic Operations
All batch operations are truly atomic - either all changes succeed, or all changes are rolled back to the original state. This prevents your codebase from being left in a broken, partially-refactored state.

### Intelligent Analysis
The tools use your language server to understand dependencies, analyze symbol usage, and calculate precise impact assessments rather than relying on simple text matching.

### Safe-by-Default
The default strategy is "safe" - operations abort on the first error. You can optionally use "force" strategy to continue despite non-critical errors.

### Circular Dependency Prevention
Both `rename_file` and `rename_directory` include automatic circular dependency detection:

- **Before moving files or directories**, the tools analyze import relationships
- **Prevents moves** that would create circular dependencies between packages
- **Provides clear warnings** with suggestions for alternative approaches
- **Maintains code health** by preventing architectural problems

This safety feature addresses a common source of refactoring mistakes where moving code creates unexpected circular imports.

## Complete Refactoring Workflow

Here's the recommended workflow for complex refactoring operations using the `batch_execute` tool:

### Step 1: Plan Your Changes

First, identify the operations you want to perform. You can batch any MCP tool operations including:

- **File operations**: rename_file, rename_directory, create_file, delete_file
- **Symbol operations**: rename_symbol, find_references
- **Project management**: update_package_json
- **Diagnostics**: get_diagnostics
- **Code intelligence**: find_definition, get_symbols
- And any other available MCP tools

#### Directory vs File Operations

**Use `rename_file`** for:
- Moving individual files between directories
- Renaming single files in place
- When you need fine-grained control over specific files

**Use `rename_directory`** for:
- Moving entire packages/modules
- Restructuring directory hierarchies
- When all files in a directory should move together

### Step 2: Execute with Preview

Use `batch_execute` with preview mode to understand what will happen:

```json
{
  "operations": [
    {
      "tool": "rename_directory",
      "args": {
        "old_path": "src/utils",
        "new_path": "lib/utilities"
      }
    },
    {
      "tool": "update_package_json",
      "args": {
        "file_path": "./package.json",
        "add_dev_dependencies": {
          "@types/lodash": "^4.14.195"
        },
        "add_scripts": {
          "build:utils": "tsc --project lib/utilities/tsconfig.json"
        }
      }
    }
  ],
  "options": {
    "preview": true
  }
}
```

The preview will show you:
- **Operation count** and types
- **Validation status** for each operation
- **Estimated impact** without making changes

### Step 3: Execute with Dry Run

Test the actual execution logic with `dry_run: true`:

```json
{
  "operations": [
    {
      "tool": "rename_file",
      "args": {
        "old_path": "src/utils/string-helpers.ts",
        "new_path": "lib/utilities/string-utils.ts",
        "dry_run": true
      }
    }
  ],
  "options": {
    "dry_run": true
  }
}
```

### Step 4: Execute the Refactoring

Finally, execute the actual refactoring:

```json
{
  "operations": [
    {
      "tool": "rename_file",
      "args": {
        "old_path": "src/utils/string-helpers.ts",
        "new_path": "lib/utilities/string-utils.ts"
      }
    },
    {
      "tool": "rename_symbol",
      "args": {
        "file_path": "lib/utilities/string-utils.ts",
        "symbol_name": "formatString",
        "new_name": "formatText"
      }
    }
  ],
  "options": {
    "atomic": true,
    "stop_on_error": true
  }
}
```

## Detailed Examples

### Example 1: Simple File Reorganization

**Scenario**: Moving utility files from `src/utils/` to `lib/common/`

```json
{
  "operations": [
    {
      "tool": "rename_file",
      "args": {
        "old_path": "src/utils/string-utils.ts",
        "new_path": "lib/common/string-utils.ts"
      }
    },
    {
      "tool": "rename_file",
      "args": {
        "old_path": "src/utils/date-utils.ts",
        "new_path": "lib/common/date-utils.ts"
      }
    }
  ],
  "options": {
    "atomic": true,
    "stop_on_error": true
  }
}
```

### Example 2: Component Refactoring with Symbol Rename

**Scenario**: Moving a React component and renaming it

```json
{
  "operations": [
    {
      "tool": "rename_file",
      "args": {
        "old_path": "src/components/Button.tsx",
        "new_path": "src/ui/ActionButton.tsx"
      }
    },
    {
      "tool": "rename_symbol",
      "args": {
        "file_path": "src/ui/ActionButton.tsx",
        "symbol_name": "Button",
        "new_name": "ActionButton"
      }
    }
  ],
  "options": {
    "atomic": true
  }
}
```

### Example 3: Large-Scale Reorganization

**Scenario**: Reorganizing an entire feature module

```json
{
  "operations": [
    {
      "tool": "rename_file",
      "args": {
        "old_path": "src/features/auth/types.ts",
        "new_path": "src/modules/authentication/types.ts"
      }
    },
    {
      "tool": "rename_file",
      "args": {
        "old_path": "src/features/auth/constants.ts",
        "new_path": "src/modules/authentication/constants.ts"
      }
    },
    {
      "tool": "rename_file",
      "args": {
        "old_path": "src/features/auth/service.ts",
        "new_path": "src/modules/authentication/auth-service.ts"
      }
    },
    {
      "tool": "rename_file",
      "args": {
        "old_path": "src/features/auth/components/LoginForm.tsx",
        "new_path": "src/modules/authentication/components/LoginForm.tsx"
      }
    }
  ],
  "options": {
    "atomic": true,
    "parallel": false,
    "stop_on_error": true
  }
}
```

**Tip**: Use `parallel: false` for interdependent operations to ensure correct execution order.

### Example 4: Multiple Directory Moves (Recommended Approach)

**Scenario**: Restructuring project organization by moving multiple directories

```json
{
  "operations": [
    {
      "tool": "rename_directory",
      "args": {
        "old_path": "src/components",
        "new_path": "lib/ui/components"
      }
    },
    {
      "tool": "rename_directory",
      "args": {
        "old_path": "src/utils",
        "new_path": "lib/shared/utils"
      }
    },
    {
      "tool": "rename_directory",
      "args": {
        "old_path": "src/services",
        "new_path": "lib/core/services"
      }
    }
  ],
  "options": {
    "atomic": true,
    "parallel": false,
    "stop_on_error": true
  }
}
```

**Why this works**:
- Each `rename_directory` operation handles all files in the directory atomically
- Import paths are updated across the entire project scope
- If any directory move fails, all previous moves are rolled back
- Sequential execution prevents conflicts between directory moves

**❌ What NOT to do**: Never try to move directories by moving individual files - this misses cross-references and creates inconsistent states.

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
- **Prevention**: Use `preview: true` option to catch conflicts before execution

**Issue**: "Source file does not exist"
- **Solution**: Verify the source path is correct
- **Prevention**: Double-check file paths before execution

**Issue**: "Operation failed during execution"
- **Solution**: Enable `atomic: true` to ensure automatic rollback
- **Prevention**: Use `preview: true` and `dry_run: true` to test first

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

The `batch_execute` tool handles all operations in a single transaction, automatically sequencing them based on dependencies and your chosen execution mode (sequential or parallel).

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

CodeFlow Buddy's `batch_execute` tool provides enterprise-grade safety and intelligence for complex refactoring operations. With atomic operations, automatic rollback, and the ability to combine any MCP tools, you can confidently restructure large codebases without the risk of introducing breaking changes.

The key to successful refactoring is using the preview feature, enabling atomic mode for safety, and choosing the right execution strategy (sequential vs parallel) based on your operation dependencies.