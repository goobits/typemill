import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { relative, resolve } from 'node:path';
import {
  type ApplyEditResult,
  type WorkspaceEdit,
  applyWorkspaceEdit,
  mergeWorkspaceEdits,
  previewWorkspaceEdit,
} from '../../core/file-operations/editor.js';
import { uriToPath } from '../../core/file-operations/path-utils.js';
import type { LSPClient } from '../../lsp/client.js';
import type { SymbolService } from '../../services/symbol-service.js';
import { createListResponse, createMCPResponse, createSuccessResponse } from '../utils.js';
import { getRenameSymbolWorkspaceEdit, handleRenameSymbol } from './core-handlers.js';
import { getRenameFileWorkspaceEdit, handleRenameFile } from './utility-handlers.js';

interface FileMove {
  old_path: string;
  new_path: string;
}

interface Operation {
  type: 'move_file' | 'rename_symbol' | 'rename_file';
  old_path?: string;
  new_path?: string;
  file_path?: string;
  symbol_name?: string;
  symbol_kind?: string;
  new_name?: string;
}

interface ImpactAnalysis {
  operation: Operation;
  risk_level: 'low' | 'medium' | 'high';
  dependent_files: string[];
  recommendations: string[];
  estimated_changes: number;
}

/**
 * Analyze the impact of multiple refactoring operations
 * Uses existing projectScanner to understand dependencies
 */
export async function handleAnalyzeRefactorImpact(
  symbolService: SymbolService,
  args: {
    operations: Operation[];
    include_recommendations?: boolean;
  }
) {
  const { operations, include_recommendations = true } = args;

  try {
    const { projectScanner } = await import('../../services/project-analyzer.js');
    const analyses: ImpactAnalysis[] = [];

    for (const operation of operations) {
      const analysis: ImpactAnalysis = {
        operation,
        risk_level: 'low',
        dependent_files: [],
        recommendations: [],
        estimated_changes: 0,
      };

      if (operation.type === 'move_file' && operation.old_path) {
        const absolutePath = resolve(operation.old_path);

        // Check if file exists
        if (!existsSync(absolutePath)) {
          analysis.risk_level = 'high';
          analysis.recommendations.push(`File ${operation.old_path} does not exist`);
        } else {
          // Find files that import this file
          const importers = await projectScanner.findImporters(absolutePath);
          analysis.dependent_files = importers.map((f) => relative(process.cwd(), f));
          analysis.estimated_changes = importers.length;

          // Assess risk based on number of dependencies
          if (importers.length === 0) {
            analysis.risk_level = 'low';
            analysis.recommendations.push('Safe to move - no import dependencies found');
          } else if (importers.length <= 3) {
            analysis.risk_level = 'medium';
            analysis.recommendations.push(`Will update imports in ${importers.length} file(s)`);
          } else {
            analysis.risk_level = 'high';
            analysis.recommendations.push(
              `High impact - will update imports in ${importers.length} files`
            );
            analysis.recommendations.push('Consider moving in smaller batches');
          }
        }
      } else if (operation.type === 'rename_symbol') {
        // For symbol renames, get actual impact from LSP
        if (operation.file_path && operation.symbol_name && operation.new_name) {
          try {
            const renameResult = await getRenameSymbolWorkspaceEdit(symbolService, {
              file_path: operation.file_path,
              symbol_name: operation.symbol_name,
              symbol_kind: operation.symbol_kind,
              new_name: operation.new_name,
            });

            if (renameResult.success && renameResult.workspaceEdit?.changes) {
              const affectedFiles = Object.keys(renameResult.workspaceEdit.changes);
              analysis.dependent_files = affectedFiles.map((uri) => {
                try {
                  return relative(process.cwd(), uriToPath(uri));
                } catch {
                  return uri; // Fallback to URI if conversion fails
                }
              });
              analysis.estimated_changes = Object.values(renameResult.workspaceEdit.changes).reduce(
                (sum, edits) => sum + edits.length,
                0
              );

              // Assess risk based on number of affected files
              if (affectedFiles.length <= 1) {
                analysis.risk_level = 'low';
                analysis.recommendations.push('Single file impact - safe to execute');
              } else if (affectedFiles.length <= 5) {
                analysis.risk_level = 'medium';
                analysis.recommendations.push(`Will update ${affectedFiles.length} files`);
              } else {
                analysis.risk_level = 'high';
                analysis.recommendations.push(
                  `High impact - will update ${affectedFiles.length} files`
                );
                analysis.recommendations.push('Consider executing separately from file moves');
              }
            } else {
              analysis.risk_level = 'high';
              analysis.estimated_changes = 0;
              analysis.recommendations.push(
                renameResult.error || 'Failed to analyze symbol rename impact'
              );
            }
          } catch (error) {
            analysis.risk_level = 'high';
            analysis.estimated_changes = 0;
            analysis.recommendations.push(
              `Error analyzing symbol: ${error instanceof Error ? error.message : String(error)}`
            );
          }
        } else {
          analysis.risk_level = 'high';
          analysis.estimated_changes = 0;
          analysis.recommendations.push('Missing required fields for symbol rename analysis');
        }
      }

      analyses.push(analysis);
    }

    // Generate summary
    const totalFiles = new Set(analyses.flatMap((a) => a.dependent_files)).size;
    const totalChanges = analyses.reduce((sum, a) => sum + a.estimated_changes, 0);
    const highRiskOps = analyses.filter((a) => a.risk_level === 'high').length;
    const mediumRiskOps = analyses.filter((a) => a.risk_level === 'medium').length;

    let summary = '## Refactoring Impact Analysis\\n\\n';
    summary += `**Operations**: ${operations.length}\\n`;
    summary += `**Estimated file changes**: ${totalChanges}\\n`;
    summary += `**Unique files affected**: ${totalFiles}\\n`;
    summary += `**Risk assessment**: ${highRiskOps} high, ${mediumRiskOps} medium, ${analyses.length - highRiskOps - mediumRiskOps} low\\n\\n`;

    if (include_recommendations) {
      summary += '### Recommendations\\n\\n';

      if (highRiskOps > 0) {
        summary += `⚠️ **${highRiskOps} high-risk operation(s)** - consider breaking into smaller batches\\n`;
      }

      if (totalChanges > 10) {
        summary += `⚠️ **High impact** - ${totalChanges} estimated changes across ${totalFiles} files\\n`;
      } else if (totalChanges > 0) {
        summary += `✅ **Moderate impact** - ${totalChanges} estimated changes\\n`;
      } else {
        summary += '✅ **Low impact** - minimal changes expected\\n';
      }

      summary += '\\n### Execution Strategy\\n';
      if (highRiskOps > 0) {
        summary += '- Use `dry_run: true` first to preview changes\\n';
        summary += '- Consider executing high-risk operations individually\\n';
        summary += '- Ensure you have backup/version control\\n';
      } else {
        summary += '- Safe to execute as batch operation\\n';
        summary += '- Still recommended to use `dry_run: true` first\\n';
      }
    }

    // Add detailed analysis for each operation
    if (analyses.length > 0) {
      summary += '\\n### Operation Details\\n\\n';

      for (let i = 0; i < analyses.length; i++) {
        const analysis = analyses[i];
        if (!analysis) continue;
        const op = analysis.operation;

        summary += `**${i + 1}. ${op.type}** `;
        if (op.type === 'move_file') {
          summary += `${op.old_path} → ${op.new_path}`;
        } else if (op.type === 'rename_symbol') {
          summary += `${op.symbol_name} → ${op.new_name} in ${op.file_path}`;
        }
        summary += '\\n';

        summary += `- **Risk**: ${analysis.risk_level}\\n`;
        summary += `- **Estimated changes**: ${analysis.estimated_changes}\\n`;

        if (analysis.dependent_files.length > 0) {
          summary += `- **Dependent files**: ${analysis.dependent_files.slice(0, 5).join(', ')}`;
          if (analysis.dependent_files.length > 5) {
            summary += ` and ${analysis.dependent_files.length - 5} more`;
          }
          summary += '\\n';
        }

        if (analysis.recommendations.length > 0) {
          summary += `- **Notes**: ${analysis.recommendations.join('; ')}\\n`;
        }

        summary += '\\n';
      }
    }

    return createMCPResponse(summary);
  } catch (error) {
    return createMCPResponse(
      `Error analyzing refactor impact: ${error instanceof Error ? error.message : String(error)}`
    );
  }
}

/**
 * Execute multiple file moves atomically with rollback capability
 * Uses WorkspaceEdit for true atomic operations
 */
export async function handleBatchMoveFiles(args: {
  moves: FileMove[];
  dry_run?: boolean;
  strategy?: 'safe' | 'force';
}) {
  const { moves, dry_run = false, strategy = 'safe' } = args;

  if (moves.length === 0) {
    return createMCPResponse('No file moves specified');
  }

  try {
    // Step 1: Collect all WorkspaceEdit objects from dry runs
    const workspaceEdits: WorkspaceEdit[] = [];
    const validationErrors: Array<{ move: FileMove; error: string }> = [];

    for (const move of moves) {
      // Validate that source file exists
      if (!existsSync(resolve(move.old_path))) {
        validationErrors.push({
          move,
          error: `Source file does not exist: ${move.old_path}`,
        });
        continue;
      }

      // Validate that target doesn't exist
      if (existsSync(resolve(move.new_path))) {
        validationErrors.push({
          move,
          error: `Target file already exists: ${move.new_path}`,
        });
        continue;
      }

      // Get WorkspaceEdit for this move
      const editResult = await getRenameFileWorkspaceEdit({
        old_path: move.old_path,
        new_path: move.new_path,
      });

      if (!editResult.success) {
        validationErrors.push({
          move,
          error: editResult.error || 'Unknown error getting workspace edit',
        });
        continue;
      }

      if (editResult.workspaceEdit) {
        workspaceEdits.push(editResult.workspaceEdit);
      }
    }

    // Check for validation errors
    if (validationErrors.length > 0) {
      if (strategy === 'safe' || validationErrors.length === moves.length) {
        let summary = '## Batch Move Validation Failed\\n\\n';
        summary += `**Total operations**: ${moves.length}\\n`;
        summary += `**Validation errors**: ${validationErrors.length}\\n\\n`;
        summary += '### ❌ Validation Errors\\n\\n';

        validationErrors.forEach((error, i) => {
          summary += `${i + 1}. **${error.move.old_path} → ${error.move.new_path}**\\n`;
          summary += `   Error: ${error.error}\\n\\n`;
        });

        if (strategy === 'safe') {
          summary += '**Strategy**: safe - aborting due to validation errors\\n';
        }

        return createMCPResponse(summary);
      }
    }

    // Step 2: Merge all WorkspaceEdit objects into a single atomic operation
    const mergedEdit = mergeWorkspaceEdits(workspaceEdits);

    if (dry_run) {
      // Step 3a: Generate preview for dry run
      const preview = previewWorkspaceEdit(mergedEdit);

      let summary = '## Batch Move Preview (DRY RUN)\\n\\n';
      summary += `**Operations**: ${moves.length}\\n`;
      summary += `**Validation errors**: ${validationErrors.length}\\n`;
      summary += `**Import updates**: ${preview.summary}\\n\\n`;

      if (validationErrors.length > 0) {
        summary += '### ⚠️ Validation Issues\\n\\n';
        validationErrors.forEach((error, i) => {
          summary += `${i + 1}. **${error.move.old_path} → ${error.move.new_path}**\\n`;
          summary += `   Error: ${error.error}\\n\\n`;
        });
      }

      summary += '### File Operations\\n\\n';
      const validMoves = moves.filter(
        (move) => !validationErrors.some((error) => error.move === move)
      );

      validMoves.forEach((move, i) => {
        summary += `${i + 1}. **${move.old_path} → ${move.new_path}**\\n`;
        summary += '   Status: ✅ Ready to move\\n\\n';
      });

      if (preview.filesAffected > 0) {
        summary += '### Import Updates Preview\\n\\n';
        preview.details.forEach((detail, i) => {
          summary += `${i + 1}. **${detail.filePath}**\\n`;
          summary += `   ${detail.preview}\\n\\n`;
        });
      }

      return createMCPResponse(summary);
    }

    // Step 3b: Execute atomically
    const validMoves = moves.filter(
      (move) => !validationErrors.some((error) => error.move === move)
    );

    // Create in-memory backup of all files that will be modified
    const backups = new Map<string, string>();

    if (mergedEdit.changes) {
      for (const uri of Object.keys(mergedEdit.changes)) {
        const filePath = uriToPath(uri);
        try {
          const content = readFileSync(filePath, 'utf-8');
          backups.set(filePath, content);
        } catch (error) {
          throw new Error(`Failed to backup file ${filePath}: ${error}`);
        }
      }
    }

    // Step 4: Apply import updates atomically
    let editResult: ApplyEditResult = { success: true, filesModified: [], backupFiles: [] };

    if (mergedEdit.changes && Object.keys(mergedEdit.changes).length > 0) {
      editResult = await applyWorkspaceEdit(mergedEdit);

      if (!editResult.success) {
        // Rollback is handled automatically by applyWorkspaceEdit on failure
        throw new Error(`Failed to apply import updates: ${editResult.error}`);
      }
    }

    // Step 5: Move files atomically (with rollback on failure)
    const movedFiles: Array<{ old: string; new: string }> = [];

    try {
      for (const move of validMoves) {
        const { mkdirSync } = await import('node:fs');
        const { dirname } = await import('node:path');
        const { renameSync } = await import('node:fs');

        // Create parent directory if needed
        const newDir = dirname(resolve(move.new_path));
        if (!existsSync(newDir)) {
          mkdirSync(newDir, { recursive: true });
        }

        // Perform the file move
        renameSync(resolve(move.old_path), resolve(move.new_path));
        movedFiles.push({ old: move.old_path, new: move.new_path });
      }
    } catch (error) {
      // Critical failure during file moves - rollback everything

      // Rollback file moves
      for (const moved of movedFiles) {
        try {
          const { renameSync } = await import('node:fs');
          renameSync(resolve(moved.new), resolve(moved.old));
        } catch (rollbackError) {
          console.error(`Failed to rollback file move ${moved.new} → ${moved.old}:`, rollbackError);
        }
      }

      // Rollback import changes
      for (const [filePath, originalContent] of backups.entries()) {
        try {
          writeFileSync(filePath, originalContent, 'utf-8');
        } catch (rollbackError) {
          console.error(`Failed to rollback file content ${filePath}:`, rollbackError);
        }
      }

      throw new Error(`Atomic operation failed and rolled back: ${error}`);
    }

    // Step 6: Generate success summary
    let summary = '## Batch Move Results\\n\\n';
    summary += `**Total operations**: ${moves.length}\\n`;
    summary += `**Successful moves**: ${validMoves.length}\\n`;
    summary += `**Validation errors**: ${validationErrors.length}\\n`;
    summary += `**Import updates**: ${editResult.filesModified.length} files\\n`;
    summary += `**Strategy**: ${strategy}\\n\\n`;

    if (validMoves.length > 0) {
      summary += `### ✅ Successful Moves (${validMoves.length})\\n\\n`;
      validMoves.forEach((move, i) => {
        summary += `${i + 1}. ${move.old_path} → ${move.new_path}\\n`;
      });
      summary += '\\n';
    }

    if (validationErrors.length > 0) {
      summary += `### ❌ Validation Errors (${validationErrors.length})\\n\\n`;
      validationErrors.forEach((error, i) => {
        summary += `${i + 1}. ${error.move.old_path} → ${error.move.new_path}\\n`;
        summary += `   Error: ${error.error}\\n\\n`;
      });
    }

    if (editResult.filesModified.length > 0) {
      summary += '### 📝 Import Updates\\n\\n';
      summary += `Updated imports in ${editResult.filesModified.length} file${editResult.filesModified.length === 1 ? '' : 's'}\\n\\n`;
    }

    if (validMoves.length === moves.length) {
      summary += '🎉 **All operations completed successfully!**';
    } else if (validMoves.length > 0) {
      summary += `⚠️ **Partial success** - ${validationErrors.length} validation error(s)`;
    } else {
      summary += '💥 **All operations failed validation**';
    }

    return createMCPResponse(summary);
  } catch (error) {
    return createMCPResponse(
      `Error in batch move operation: ${error instanceof Error ? error.message : String(error)}`
    );
  }
}

/**
 * Preview a complex batch operation showing all potential changes
 */
export async function handlePreviewBatchOperation(
  symbolService: SymbolService,
  args: {
    operations: Operation[];
    detailed?: boolean;
  }
) {
  const { operations, detailed = false } = args;

  try {
    let summary = '## Batch Operation Preview\\n\\n';
    summary += `**Operations**: ${operations.length}\\n`;
    summary += `**Preview mode**: ${detailed ? 'Detailed' : 'Summary'}\\n\\n`;

    const results: Array<{ operation: Operation; preview: string; success: boolean }> = [];

    for (let i = 0; i < operations.length; i++) {
      const op = operations[i];
      if (!op) continue;
      let preview = '';
      let success = true;

      try {
        if (op.type === 'move_file' || op.type === 'rename_file') {
          if (!op.old_path || !op.new_path) {
            throw new Error('Missing old_path or new_path for file operation');
          }

          // Use the new centralized preview approach
          const editResult = await getRenameFileWorkspaceEdit({
            old_path: op.old_path,
            new_path: op.new_path,
          });

          if (!editResult.success) {
            throw new Error(editResult.error || 'Failed to get workspace edit');
          }

          if (editResult.workspaceEdit) {
            const previewData = previewWorkspaceEdit(editResult.workspaceEdit);
            preview = `Would move file and ${previewData.summary}`;
          } else {
            preview = 'Would move file with no import updates needed';
          }
        } else if (op.type === 'rename_symbol') {
          if (!op.file_path || !op.symbol_name || !op.new_name) {
            throw new Error('Missing required fields for symbol rename');
          }

          // Use the new centralized preview approach for symbol renames
          const renameResult = await getRenameSymbolWorkspaceEdit(symbolService, {
            file_path: op.file_path,
            symbol_name: op.symbol_name,
            symbol_kind: op.symbol_kind,
            new_name: op.new_name,
          });

          if (!renameResult.success) {
            throw new Error(renameResult.error || 'Failed to get symbol rename workspace edit');
          }

          if (renameResult.workspaceEdit) {
            const previewData = previewWorkspaceEdit(renameResult.workspaceEdit);
            preview = `Would rename symbol "${op.symbol_name}" to "${op.new_name}" and ${previewData.summary}`;
          } else {
            preview = `Would rename symbol "${op.symbol_name}" to "${op.new_name}" with no additional changes`;
          }
        }
      } catch (error) {
        success = false;
        preview = `Error: ${error instanceof Error ? error.message : String(error)}`;
      }

      results.push({ operation: op, preview, success });
    }

    // Add operation summaries
    summary += '### Operations Preview\\n\\n';

    for (let i = 0; i < results.length; i++) {
      const result = results[i];
      if (!result) continue;
      const op = result.operation;

      summary += `**${i + 1}. ${op.type}** `;
      if (op.type === 'move_file' || op.type === 'rename_file') {
        summary += `${op.old_path} → ${op.new_path}`;
      } else if (op.type === 'rename_symbol') {
        summary += `${op.symbol_name} → ${op.new_name} in ${op.file_path}`;
      }
      summary += '\\n';

      summary += `Status: ${result.success ? '✅ Ready' : '❌ Error'}\\n`;

      if (detailed) {
        summary += `Preview: ${result.preview}\\n`;
      } else if (!result.success) {
        summary += `Error: ${result.preview}\\n`;
      }

      summary += '\\n';
    }

    const successCount = results.filter((r) => r.success).length;
    const readyForExecution = successCount === operations.length;

    summary += '### Summary\\n\\n';
    summary += `**Ready for execution**: ${successCount}/${operations.length}\\n`;

    if (readyForExecution) {
      summary += '✅ **All operations validated** - safe to execute\\n\\n';
      summary += 'To execute these operations:\\n';
      summary += '- Use `batch_move_files` for file moves\\n';
      summary += '- Use individual `rename_symbol` calls for symbol renames\\n';
    } else {
      summary += `❌ **${operations.length - successCount} operation(s) have errors** - fix before execution\\n`;
    }

    return createMCPResponse(summary);
  } catch (error) {
    return createMCPResponse(
      `Error previewing batch operation: ${error instanceof Error ? error.message : String(error)}`
    );
  }
}
