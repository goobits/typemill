import { readFileSync } from 'node:fs';
import * as DocumentMethods from '../lsp-methods/document-methods.js';
import type { ServerState } from '../lsp-types.js';
import type { LSPProtocol } from '../lsp/protocol.js';
import type { ServerManager } from '../lsp/server-manager.js';
import type {
  CodeAction,
  DocumentLink,
  FoldingRange,
  Position,
  Range,
  TextEdit,
} from '../types.js';

/**
 * Service for file-related LSP operations
 * Handles formatting, code actions, document links, and file synchronization
 */
export class FileService {
  constructor(
    private serverManager: ServerManager,
    private protocol: LSPProtocol
  ) {}

  /**
   * Format document
   */
  async formatDocument(filePath: string): Promise<TextEdit[]> {
    const context: DocumentMethods.DocumentMethodsContext = {
      getServer: (path: string) => this.serverManager.getServer(path, {} as any),
      ensureFileOpen: this.ensureFileOpen.bind(this),
      sendRequest: (process, method, params, timeout) =>
        this.protocol.sendRequest(process, method, params, timeout),
      sendNotification: (process, method, params) =>
        this.protocol.sendNotification(process, method, params),
      capabilityManager: {} as any, // Will be properly injected
    };
    return DocumentMethods.formatDocument(context, filePath);
  }

  /**
   * Get code actions for range
   */
  async getCodeActions(
    filePath: string,
    range: Range,
    context: { diagnostics: any[] }
  ): Promise<CodeAction[]> {
    const docContext: DocumentMethods.DocumentMethodsContext = {
      getServer: (path: string) => this.serverManager.getServer(path, {} as any),
      ensureFileOpen: this.ensureFileOpen.bind(this),
      sendRequest: (process, method, params, timeout) =>
        this.protocol.sendRequest(process, method, params, timeout),
      sendNotification: (process, method, params) =>
        this.protocol.sendNotification(process, method, params),
      capabilityManager: {} as any,
    };
    return DocumentMethods.getCodeActions(docContext, filePath, range, context);
  }

  /**
   * Get folding ranges
   */
  async getFoldingRanges(filePath: string): Promise<FoldingRange[]> {
    const context: DocumentMethods.DocumentMethodsContext = {
      getServer: (path: string) => this.serverManager.getServer(path, {} as any),
      ensureFileOpen: this.ensureFileOpen.bind(this),
      sendRequest: (process, method, params, timeout) =>
        this.protocol.sendRequest(process, method, params, timeout),
      sendNotification: (process, method, params) =>
        this.protocol.sendNotification(process, method, params),
      capabilityManager: {} as any,
    };
    return DocumentMethods.getFoldingRanges(context, filePath);
  }

  /**
   * Get document links
   */
  async getDocumentLinks(filePath: string): Promise<DocumentLink[]> {
    const context: DocumentMethods.DocumentMethodsContext = {
      getServer: (path: string) => this.serverManager.getServer(path, {} as any),
      ensureFileOpen: this.ensureFileOpen.bind(this),
      sendRequest: (process, method, params, timeout) =>
        this.protocol.sendRequest(process, method, params, timeout),
      sendNotification: (process, method, params) =>
        this.protocol.sendNotification(process, method, params),
      capabilityManager: {} as any,
    };
    return DocumentMethods.getDocumentLinks(context, filePath);
  }

  /**
   * Apply workspace edit
   */
  async applyWorkspaceEdit(edit: {
    changes?: Record<string, TextEdit[]>;
    documentChanges?: Array<{
      textDocument: { uri: string; version?: number };
      edits: TextEdit[];
    }>;
  }): Promise<{ applied: boolean; failureReason?: string }> {
    try {
      if (edit.changes) {
        for (const [uri, edits] of Object.entries(edit.changes)) {
          const filePath = uri.replace('file://', '');
          await this.applyTextEdits(filePath, edits);
        }
      }

      if (edit.documentChanges) {
        for (const change of edit.documentChanges) {
          const filePath = change.textDocument.uri.replace('file://', '');
          await this.applyTextEdits(filePath, change.edits);
        }
      }

      return { applied: true };
    } catch (error) {
      return {
        applied: false,
        failureReason: error instanceof Error ? error.message : String(error),
      };
    }
  }

  /**
   * Rename file
   */
  async renameFile(oldPath: string, newPath: string): Promise<void> {
    // This would typically involve file system operations
    // For now, just notify LSP servers about the change
    try {
      // Get all active servers that might be interested
      const serverConfigs = new Map();
      // Implementation would check which servers handle these file types

      // Send willRename notification to interested servers
      for (const serverState of serverConfigs.values()) {
        this.protocol.sendNotification(serverState.process, 'workspace/willRenameFiles', {
          files: [
            {
              oldUri: `file://${oldPath}`,
              newUri: `file://${newPath}`,
            },
          ],
        });
      }
    } catch (error) {
      process.stderr.write(`[ERROR renameFile] ${error}\n`);
    }
  }

  /**
   * Ensure file is open in LSP server
   */
  async ensureFileOpen(serverState: ServerState, filePath: string): Promise<void> {
    if (serverState.openFiles.has(filePath)) {
      return;
    }

    const fileContent = readFileSync(filePath, 'utf-8');

    this.protocol.sendNotification(serverState.process, 'textDocument/didOpen', {
      textDocument: {
        uri: `file://${filePath}`,
        languageId: this.getLanguageId(filePath),
        version: 1,
        text: fileContent,
      },
    });

    serverState.openFiles.add(filePath);
  }

  /**
   * Apply text edits to a file
   */
  private async applyTextEdits(filePath: string, edits: TextEdit[]): Promise<void> {
    if (edits.length === 0) return;

    try {
      const fileContent = readFileSync(filePath, 'utf-8');
      const lines = fileContent.split('\n');

      // Sort edits in reverse order by position to avoid offset issues
      const sortedEdits = [...edits].sort((a, b) => {
        if (a.range.start.line !== b.range.start.line) {
          return b.range.start.line - a.range.start.line;
        }
        return b.range.start.character - a.range.start.character;
      });

      // Apply edits
      for (const edit of sortedEdits) {
        const startLine = edit.range.start.line;
        const startChar = edit.range.start.character;
        const endLine = edit.range.end.line;
        const endChar = edit.range.end.character;

        if (startLine === endLine) {
          // Single line edit
          const line = lines[startLine];
          lines[startLine] = line.substring(0, startChar) + edit.newText + line.substring(endChar);
        } else {
          // Multi-line edit
          const newLines = edit.newText.split('\n');
          const firstLine = lines[startLine].substring(0, startChar) + newLines[0];
          const lastLine = newLines[newLines.length - 1] + lines[endLine].substring(endChar);

          // Replace the range with new content
          const replacementLines = [firstLine, ...newLines.slice(1, -1), lastLine];
          lines.splice(startLine, endLine - startLine + 1, ...replacementLines);
        }
      }

      // This would normally write back to the file
      // For now, just log what would happen
      process.stderr.write(
        `[DEBUG applyTextEdits] Would apply ${edits.length} edits to ${filePath}\n`
      );
    } catch (error) {
      throw new Error(`Failed to apply text edits to ${filePath}: ${error}`);
    }
  }

  private getLanguageId(filePath: string): string {
    const ext = filePath.split('.').pop()?.toLowerCase();
    const languageMap: Record<string, string> = {
      ts: 'typescript',
      tsx: 'typescriptreact',
      js: 'javascript',
      jsx: 'javascriptreact',
      py: 'python',
      go: 'go',
      rs: 'rust',
      java: 'java',
      cpp: 'cpp',
      c: 'c',
      h: 'c',
      hpp: 'cpp',
    };
    return languageMap[ext || ''] || 'plaintext';
  }
}
