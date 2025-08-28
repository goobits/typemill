import { resolve } from 'node:path';
import { existsSync, writeFileSync, unlinkSync } from 'node:fs';
import { dirname } from 'node:path';
import { mkdirSync } from 'node:fs';
import type { LSPClient } from '../../lsp-client.js';

// Handler for get_diagnostics tool
export async function handleGetDiagnostics(
  lspClient: LSPClient,
  args: { file_path: string }
) {
  const { file_path } = args;
  const absolutePath = resolve(file_path);

  try {
    const diagnostics = await lspClient.getDiagnostics(absolutePath);

    if (diagnostics.length === 0) {
      return {
        content: [
          {
            type: 'text',
            text: `No diagnostics found for ${file_path}. The file has no errors, warnings, or hints.`,
          },
        ],
      };
    }

    const severityMap = {
      1: 'Error',
      2: 'Warning',
      3: 'Information',
      4: 'Hint',
    };

    const diagnosticMessages = diagnostics.map((diag) => {
      const severity = diag.severity ? severityMap[diag.severity] || 'Unknown' : 'Unknown';
      const code = diag.code ? ` [${diag.code}]` : '';
      const source = diag.source ? ` (${diag.source})` : '';
      const { start, end } = diag.range;

      return `• ${severity}${code}${source}: ${diag.message}\n  Location: Line ${start.line + 1}, Column ${start.character + 1} to Line ${end.line + 1}, Column ${end.character + 1}`;
    });

    return {
      content: [
        {
          type: 'text',
          text: `Found ${diagnostics.length} diagnostic${diagnostics.length === 1 ? '' : 's'} in ${file_path}:\n\n${diagnosticMessages.join('\n\n')}`,
        },
      ],
    };
  } catch (error) {
    return {
      content: [
        {
          type: 'text',
          text: `Error getting diagnostics: ${error instanceof Error ? error.message : String(error)}`,
        },
      ],
    };
  }
}

// Handler for restart_server tool
export async function handleRestartServer(
  lspClient: LSPClient,
  args: { extensions?: string[] }
) {
  const { extensions } = args;

  try {
    const result = await lspClient.restartServers(extensions);

    let response = result.message;

    if (result.restarted.length > 0) {
      response += `\n\nRestarted servers:\n${result.restarted.map((s) => `• ${s}`).join('\n')}`;
    }

    if (result.failed.length > 0) {
      response += `\n\nFailed to restart:\n${result.failed.map((s) => `• ${s}`).join('\n')}`;
    }

    return {
      content: [
        {
          type: 'text',
          text: response,
        },
      ],
    };
  } catch (error) {
    return {
      content: [
        {
          type: 'text',
          text: `Error restarting servers: ${error instanceof Error ? error.message : String(error)}`,
        },
      ],
    };
  }
}

// Handler for rename_file tool
export async function handleRenameFile(
  lspClient: LSPClient,
  args: {
    old_path: string;
    new_path: string;
    dry_run?: boolean;
  }
) {
  const { old_path, new_path, dry_run = false } = args;
  
  try {
    const { renameFile } = await import('../../file-editor.js');
    const result = await renameFile(
      old_path,
      new_path,
      lspClient,
      { dry_run }
    );
    
    if (!result.success) {
      return {
        content: [
          {
            type: 'text',
            text: `Failed to rename file: ${result.error}`,
          },
        ],
      };
    }
    
    if (dry_run) {
      // In dry-run mode, show what would be changed
      const message = result.error || '[DRY RUN] No changes would be made';
      return {
        content: [
          {
            type: 'text',
            text: message,
          },
        ],
      };
    }
    
    // Success message
    const importCount = result.importUpdates 
      ? Object.keys(result.importUpdates.changes || {}).length 
      : 0;
    
    return {
      content: [
        {
          type: 'text',
          text: `✅ Successfully renamed ${old_path} to ${new_path}\n\n` +
                `Files modified: ${result.filesModified.length}\n` +
                (importCount > 0 ? `Files with updated imports: ${importCount}` : 'No import updates needed'),
        },
      ],
    };
  } catch (error) {
    return {
      content: [
        {
          type: 'text',
          text: `Error renaming file: ${error instanceof Error ? error.message : String(error)}`,
        },
      ],
    };
  }
}

// Handler for create_file tool
export async function handleCreateFile(
  lspClient: LSPClient,
  args: {
    file_path: string;
    content?: string;
    overwrite?: boolean;
  }
) {
  const { file_path, content = '', overwrite = false } = args;
  const absolutePath = resolve(file_path);

  try {
    // Check if file already exists
    if (existsSync(absolutePath) && !overwrite) {
      return {
        content: [
          {
            type: 'text',
            text: `File ${file_path} already exists. Use overwrite: true to replace it.`,
          },
        ],
      };
    }

    // Ensure parent directory exists
    const parentDir = dirname(absolutePath);
    if (!existsSync(parentDir)) {
      mkdirSync(parentDir, { recursive: true });
    }

    // Write file content
    writeFileSync(absolutePath, content, 'utf8');

    // Notify LSP servers about the new file
    try {
      const serverState = await lspClient.getServer(absolutePath);
      if (serverState.initialized) {
        // Send workspace/didCreateFiles notification if supported
        const capabilities = serverState.capabilities;
        const fileOperations = capabilities?.workspace?.fileOperations;
        
        if (fileOperations?.didCreate) {
          await lspClient.sendNotification(serverState.process, 'workspace/didCreateFiles', {
            files: [{
              uri: `file://${absolutePath}`
            }]
          });
        }
      }
    } catch (lspError) {
      // LSP notification failure is not critical for file creation
      process.stderr.write(`[WARNING] Could not notify LSP server about file creation: ${lspError}\n`);
    }

    return {
      content: [
        {
          type: 'text',
          text: `✅ Successfully created ${file_path}${content ? ` with ${content.length} characters` : ' (empty file)'}`,
        },
      ],
    };
  } catch (error) {
    return {
      content: [
        {
          type: 'text',
          text: `Error creating file: ${error instanceof Error ? error.message : String(error)}`,
        },
      ],
    };
  }
}

// Handler for delete_file tool
export async function handleDeleteFile(
  lspClient: LSPClient,
  args: {
    file_path: string;
    force?: boolean;
  }
) {
  const { file_path, force = false } = args;
  const absolutePath = resolve(file_path);

  try {
    // Check if file exists
    if (!existsSync(absolutePath)) {
      return {
        content: [
          {
            type: 'text',
            text: `File ${file_path} does not exist.`,
          },
        ],
      };
    }

    // Notify LSP servers before deletion
    try {
      const serverState = await lspClient.getServer(absolutePath);
      if (serverState.initialized) {
        // Close the file if it's open
        await lspClient.sendNotification(serverState.process, 'textDocument/didClose', {
          textDocument: {
            uri: `file://${absolutePath}`
          }
        });

        // Send workspace/willDeleteFiles notification if supported
        const capabilities = serverState.capabilities;
        const fileOperations = capabilities?.workspace?.fileOperations;
        
        if (fileOperations?.willDelete) {
          await lspClient.sendRequest(serverState.process, 'workspace/willDeleteFiles', {
            files: [{
              uri: `file://${absolutePath}`
            }]
          });
        }
      }
    } catch (lspError) {
      if (!force) {
        return {
          content: [
            {
              type: 'text',
              text: `LSP server error during file deletion: ${lspError}. Use force: true to delete anyway.`,
            },
          ],
        };
      }
      // With force=true, continue despite LSP errors
      process.stderr.write(`[WARNING] LSP notification failed, but continuing with forced deletion: ${lspError}\n`);
    }

    // Delete the file
    unlinkSync(absolutePath);

    // Send post-deletion notification
    try {
      const serverState = await lspClient.getServer(absolutePath);
      if (serverState.initialized) {
        const capabilities = serverState.capabilities;
        const fileOperations = capabilities?.workspace?.fileOperations;
        
        if (fileOperations?.didDelete) {
          await lspClient.sendNotification(serverState.process, 'workspace/didDeleteFiles', {
            files: [{
              uri: `file://${absolutePath}`
            }]
          });
        }
      }
    } catch (lspError) {
      // Post-deletion notification failure is not critical
      process.stderr.write(`[WARNING] Could not notify LSP server about file deletion: ${lspError}\n`);
    }

    return {
      content: [
        {
          type: 'text',
          text: `✅ Successfully deleted ${file_path}`,
        },
      ],
    };
  } catch (error) {
    return {
      content: [
        {
          type: 'text',
          text: `Error deleting file: ${error instanceof Error ? error.message : String(error)}`,
        },
      ],
    };
  }
}