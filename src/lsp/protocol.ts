import type { ChildProcess } from 'node:child_process';
import type { LSPError } from '../types.js';
import { debugLog } from '../utils/debug-logger.js';
import { getErrorMessage, handleLSPError, logError } from '../utils/error-utils.js';

// Protocol constants
const DEFAULT_REQUEST_TIMEOUT_MS = 30000; // Default timeout for LSP requests
const LSP_METHOD_NOT_FOUND_ERROR = -32601; // LSP error code for method not found

interface LSPMessage {
  jsonrpc: string;
  id?: number;
  method?: string;
  params?: unknown;
  result?: unknown;
  error?: LSPError;
}

// Re-export ServerState from lsp-types to maintain compatibility;

/**
 * Handles LSP JSON-RPC protocol communication
 * Manages message framing, correlation, and timeouts
 */
export class LSPProtocol {
  private nextId = 1;
  private pendingRequests: Map<
    number,
    { resolve: (value: unknown) => void; reject: (reason?: unknown) => void }
  > = new Map();

  /**
   * Send LSP request and wait for response
   */
  async sendRequest(
    process: ChildProcess,
    method: string,
    params: unknown,
    timeout = DEFAULT_REQUEST_TIMEOUT_MS
  ): Promise<unknown> {
    return new Promise((resolve, reject) => {
      const id = this.nextId++;
      const message: LSPMessage = {
        jsonrpc: '2.0',
        id,
        method,
        params,
      };

      this.pendingRequests.set(id, { resolve, reject });

      // Set up timeout
      const timeoutId = setTimeout(() => {
        this.pendingRequests.delete(id);
        const timeoutError = new Error(`Request timed out after ${timeout}ms: ${method}`);
        logError('LSPProtocol', `Request timeout for ${method}`, timeoutError, { timeout, method });
        reject(timeoutError);
      }, timeout);

      // Clear timeout on response
      const originalResolve = resolve;
      const originalReject = reject;
      this.pendingRequests.set(id, {
        resolve: (value) => {
          clearTimeout(timeoutId);
          originalResolve(value);
        },
        reject: (reason) => {
          clearTimeout(timeoutId);
          originalReject(reason);
        },
      });

      this.sendMessage(process, message);
    });
  }

  /**
   * Send LSP notification (no response expected)
   */
  sendNotification(process: ChildProcess, method: string, params: unknown): void {
    const message: LSPMessage = {
      jsonrpc: '2.0',
      method,
      params,
    };
    this.sendMessage(process, message);
  }

  /**
   * Handle incoming LSP message
   */
  handleMessage(message: LSPMessage, serverState?: import('../lsp-types.js').ServerState): void {
    if (message.id && this.pendingRequests.has(message.id)) {
      const request = this.pendingRequests.get(message.id);
      if (!request) return;
      const { resolve, reject } = request;
      this.pendingRequests.delete(message.id);

      if (message.error) {
        // Check if this is a "method not found" error (LSP error code -32601)
        // or if the error message indicates an unhandled/unsupported method
        if (
          message.error.code === LSP_METHOD_NOT_FOUND_ERROR ||
          message.error.message?.toLowerCase().includes('unhandled method') ||
          message.error.message?.toLowerCase().includes('method not found')
        ) {
          // For unsupported methods, resolve with null instead of rejecting
          resolve(null);
        } else {
          // For actual LSP errors, create detailed error with context
          const lspError = new Error(message.error.message || 'LSP Error');
          logError('LSPProtocol', 'LSP server error', lspError, {
            code: message.error.code,
            data: message.error.data,
            method: message.method,
          });
          reject(lspError);
        }
      } else {
        resolve(message.result);
      }
    }

    // Handle server-initiated notifications
    if (message.method && serverState) {
      this.handleServerNotification(message, serverState);
    }
  }

  /**
   * Parse LSP messages from buffer (handles Content-Length framing)
   */
  parseMessages(buffer: string): { messages: LSPMessage[]; remainingBuffer: string } {
    const messages: LSPMessage[] = [];
    let remaining = buffer;

    while (true) {
      const headerEndIndex = remaining.indexOf('\r\n\r\n');
      if (headerEndIndex === -1) break;

      const headers = remaining.substring(0, headerEndIndex);
      const contentLengthMatch = headers.match(/Content-Length: (\d+)/);

      if (!contentLengthMatch || !contentLengthMatch[1]) {
        remaining = remaining.substring(headerEndIndex + 4);
        continue;
      }

      const contentLength = Number.parseInt(contentLengthMatch[1], 10);
      const messageStart = headerEndIndex + 4;

      if (remaining.length < messageStart + contentLength) break;

      const messageContent = remaining.substring(messageStart, messageStart + contentLength);

      try {
        const message = JSON.parse(messageContent) as LSPMessage;
        messages.push(message);
      } catch (error) {
        logError('LSPProtocol', 'Failed to parse LSP message', error, {
          messageContent: `${messageContent.substring(0, 100)}...`,
        });
      }

      remaining = remaining.substring(messageStart + contentLength);
    }

    return { messages, remainingBuffer: remaining };
  }

  /**
   * Send message with proper Content-Length framing
   */
  private sendMessage(process: ChildProcess, message: LSPMessage): void {
    try {
      if (!process.stdin || process.stdin.destroyed) {
        throw new Error('LSP process stdin is not available or destroyed');
      }

      const content = JSON.stringify(message);
      const header = `Content-Length: ${Buffer.byteLength(content)}\r\n\r\n`;

      // Check if we can write before attempting to write
      if (process.stdin.writable) {
        process.stdin.write(header + content);
      } else {
        throw new Error('LSP process stdin is not writable');
      }
    } catch (error) {
      const errorMessage = getErrorMessage(error);
      logError('LSPProtocol', 'Failed to send LSP message', error, {
        method: message.method,
        processAlive: !process.killed,
      });
      throw new Error(`Failed to send LSP message: ${errorMessage}`);
    }
  }

  /**
   * Handle server-initiated notifications
   */
  private handleServerNotification(
    message: LSPMessage,
    serverState: import('../lsp-types.js').ServerState
  ): void {
    if (message.method === 'initialized') {
      debugLog('LSPProtocol', 'Received initialized notification from server');
      serverState.initialized = true;
      if (serverState.initializationResolve) {
        serverState.initializationResolve();
        serverState.initializationResolve = undefined;
      }
    } else if (message.method === 'textDocument/publishDiagnostics') {
      const params = message.params as {
        uri: string;
        diagnostics: import('../types.js').Diagnostic[];
        version?: number;
      };
      if (params?.uri) {
        debugLog(
          'LSPProtocol',
          `Received publishDiagnostics for ${params.uri} with ${params.diagnostics?.length || 0} diagnostics${params.version !== undefined ? ` (version: ${params.version})` : ''}`
        );
        serverState.diagnostics.set(params.uri, params.diagnostics || []);
        serverState.lastDiagnosticUpdate.set(params.uri, Date.now());
        if (params.version !== undefined) {
          serverState.diagnosticVersions.set(params.uri, params.version);
        }
      }
    }
  }

  /**
   * Clean up pending requests on disposal
   */
  dispose(): void {
    const pendingCount = this.pendingRequests.size;
    if (pendingCount > 0) {
      logError('LSPProtocol', 'Disposing with pending requests', new Error('LSP client disposed'), {
        pendingRequestCount: pendingCount,
      });
    }

    for (const [id, request] of this.pendingRequests) {
      request.reject(new Error('LSP client disposed'));
    }
    this.pendingRequests.clear();
  }
}
