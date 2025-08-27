import { resolve } from 'node:path';
import type { LSPClient } from '../../lsp-client.js';
import { applyWorkspaceEdit } from '../../file-editor.js';
import { pathToUri, uriToPath } from '../../utils.js';

// Handler for get_code_actions tool
export async function handleGetCodeActions(
  lspClient: LSPClient,
  args: {
    file_path: string;
    range?: { start: { line: number; character: number }; end: { line: number; character: number } };
  }
) {
  const { file_path, range } = args;
  const absolutePath = resolve(file_path);

  try {
    const codeActions = await lspClient.getCodeActions(absolutePath, range);

    if (codeActions.length === 0) {
      return {
        content: [
          {
            type: 'text',
            text: `No code actions available for ${file_path}${range ? ` at lines ${range.start.line + 1}-${range.end.line + 1}` : ''}.`,
          },
        ],
      };
    }

    const actionDescriptions = codeActions
      .filter((action) => action && (action.title || action.kind))
      .map((action, index) => {
        if (action.title) {
          return `${index + 1}. ${action.title}${action.kind ? ` (${action.kind})` : ''}`;
        }
        return `${index + 1}. Code action (${action.kind || 'unknown'})`;
      });

    return {
      content: [
        {
          type: 'text',
          text: `Found ${codeActions.length} code action${codeActions.length === 1 ? '' : 's'} for ${file_path}:\n\n${actionDescriptions.join('\n')}\n\nNote: These actions show what's available but cannot be applied directly through this tool. Use your editor's code action functionality to apply them.`,
        },
      ],
    };
  } catch (error) {
    return {
      content: [
        {
          type: 'text',
          text: `Error getting code actions: ${error instanceof Error ? error.message : String(error)}`,
        },
      ],
    };
  }
}

// Handler for format_document tool
export async function handleFormatDocument(
  lspClient: LSPClient,
  args: {
    file_path: string;
    options?: {
      tab_size?: number;
      insert_spaces?: boolean;
      trim_trailing_whitespace?: boolean;
      insert_final_newline?: boolean;
      trim_final_newlines?: boolean;
    };
  }
) {
  const { file_path, options } = args;
  const absolutePath = resolve(file_path);

  try {
    // Convert snake_case to camelCase for LSP client
    const lspOptions = options ? {
      tabSize: options.tab_size,
      insertSpaces: options.insert_spaces,
      trimTrailingWhitespace: options.trim_trailing_whitespace,
      insertFinalNewline: options.insert_final_newline,
      trimFinalNewlines: options.trim_final_newlines,
    } : undefined;

    const formatEdits = await lspClient.formatDocument(absolutePath, lspOptions);

    if (formatEdits.length === 0) {
      return {
        content: [
          {
            type: 'text',
            text: `No formatting changes needed for ${file_path}. The file is already properly formatted.`,
          },
        ],
      };
    }

    // Apply the formatting edits using the existing infrastructure
    const workspaceEdit = {
      changes: {
        [pathToUri(absolutePath)]: formatEdits,
      },
    };

    const editResult = await applyWorkspaceEdit(workspaceEdit, { lspClient });

    if (!editResult.success) {
      return {
        content: [
          {
            type: 'text',
            text: `Failed to apply formatting: ${editResult.error}`,
          },
        ],
      };
    }

    return {
      content: [
        {
          type: 'text',
          text: `✅ Successfully formatted ${file_path} with ${formatEdits.length} change${formatEdits.length === 1 ? '' : 's'}.`,
        },
      ],
    };
  } catch (error) {
    return {
      content: [
        {
          type: 'text',
          text: `Error formatting document: ${error instanceof Error ? error.message : String(error)}`,
        },
      ],
    };
  }
}

// Handler for search_workspace_symbols tool
export async function handleSearchWorkspaceSymbols(
  lspClient: LSPClient,
  args: { query: string }
) {
  const { query } = args;

  try {
    const symbols = await lspClient.searchWorkspaceSymbols(query);

    if (symbols.length === 0) {
      return {
        content: [
          {
            type: 'text',
            text: `No symbols found matching "${query}". Try a different search term or ensure the language server is properly configured.`,
          },
        ],
      };
    }

    const symbolDescriptions = symbols
      .slice(0, 50) // Limit to first 50 results
      .map((symbol, index) => {
        const location = symbol.location;
        const filePath = uriToPath(location.uri);
        const line = location.range.start.line + 1;
        const character = location.range.start.character + 1;
        const symbolKind = symbol.kind ? lspClient.symbolKindToString(symbol.kind) : 'unknown';
        
        return `${index + 1}. ${symbol.name} (${symbolKind}) - ${filePath}:${line}:${character}`;
      });

    const resultText = symbols.length > 50
      ? `Found ${symbols.length} symbols matching "${query}" (showing first 50):\n\n${symbolDescriptions.join('\n')}`
      : `Found ${symbols.length} symbol${symbols.length === 1 ? '' : 's'} matching "${query}":\n\n${symbolDescriptions.join('\n')}`;

    return {
      content: [
        {
          type: 'text',
          text: resultText,
        },
      ],
    };
  } catch (error) {
    return {
      content: [
        {
          type: 'text',
          text: `Error searching workspace symbols: ${error instanceof Error ? error.message : String(error)}`,
        },
      ],
    };
  }
}

// Handler for get_document_symbols tool
export async function handleGetDocumentSymbols(
  lspClient: LSPClient,
  args: { file_path: string }
) {
  const { file_path } = args;
  const absolutePath = resolve(file_path);

  try {
    const symbols = await lspClient.getDocumentSymbols(absolutePath);

    if (symbols.length === 0) {
      return {
        content: [
          {
            type: 'text',
            text: `No symbols found in ${file_path}. The file may be empty or the language server may not support this file type.`,
          },
        ],
      };
    }

    // Check if we have DocumentSymbols (hierarchical) or SymbolInformation (flat)
    const isHierarchical = lspClient.isDocumentSymbolArray(symbols);
    
    let symbolDescriptions: string[];
    
    if (isHierarchical) {
      // Handle hierarchical DocumentSymbol[]
      const formatDocumentSymbol = (symbol: any, indent = 0): string[] => {
        const prefix = '  '.repeat(indent);
        const line = symbol.range.start.line + 1;
        const character = symbol.range.start.character + 1;
        const symbolKind = lspClient.symbolKindToString(symbol.kind);
        
        const result = [`${prefix}${symbol.name} (${symbolKind}) - Line ${line}:${character}`];
        
        if (symbol.children && symbol.children.length > 0) {
          for (const child of symbol.children) {
            result.push(...formatDocumentSymbol(child, indent + 1));
          }
        }
        
        return result;
      };
      
      symbolDescriptions = [];
      for (const symbol of symbols) {
        symbolDescriptions.push(...formatDocumentSymbol(symbol));
      }
    } else {
      // Handle flat SymbolInformation[]
      symbolDescriptions = symbols.map((symbol: any, index: number) => {
        const line = symbol.location.range.start.line + 1;
        const character = symbol.location.range.start.character + 1;
        const symbolKind = symbol.kind ? lspClient.symbolKindToString(symbol.kind) : 'unknown';
        
        return `${index + 1}. ${symbol.name} (${symbolKind}) - Line ${line}:${character}`;
      });
    }

    return {
      content: [
        {
          type: 'text',
          text: `Document outline for ${file_path}:\n\n${symbolDescriptions.join('\n')}`,
        },
      ],
    };
  } catch (error) {
    return {
      content: [
        {
          type: 'text',
          text: `Error getting document symbols: ${error instanceof Error ? error.message : String(error)}`,
        },
      ],
    };
  }
}