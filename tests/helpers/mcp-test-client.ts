import { type ChildProcess, spawn } from 'node:child_process';
import { cpus } from 'node:os';

// Shared server instance for test suite
let sharedServerInstance: MCPTestClient | null = null;

// System capability detection
function getSystemCapabilities() {
  const cpuCount = cpus().length;
  const totalMemory = require('node:os').totalmem();
  const isSlowSystem = cpuCount <= 4 || totalMemory < 8 * 1024 * 1024 * 1024;
  // More generous timeouts for all systems during testing
  return {
    isSlowSystem,
    timeoutMultiplier: isSlowSystem ? 6 : 3, // 120s for slow, 60s for fast
    baseTimeout: 20000,
  };
}

// A robust message parser that handles Content-Length headers, adapted from the project's own LSP protocol parser.
function createMessageParser() {
  let buffer = '';
  const subscribers = new Set<(message: any) => void>();

  function parse() {
    while (true) {
      const headerEndIndex = buffer.indexOf('\r\n\r\n');
      if (headerEndIndex === -1) break;

      const headers = buffer.substring(0, headerEndIndex);
      const contentLengthMatch = headers.match(/Content-Length: (\d+)/);

      if (!contentLengthMatch) {
        // Malformed header, discard and continue
        buffer = buffer.substring(headerEndIndex + 4);
        continue;
      }

      const contentLength = Number.parseInt(contentLengthMatch[1], 10);
      const messageStartIndex = headerEndIndex + 4;

      if (buffer.length < messageStartIndex + contentLength) break;

      const messageContent = buffer.substring(messageStartIndex, messageStartIndex + contentLength);
      buffer = buffer.substring(messageStartIndex + contentLength);

      try {
        const message = JSON.parse(messageContent);
        subscribers.forEach((sub) => sub(message));
      } catch (e) {
        console.error('Failed to parse message JSON:', e);
      }
    }
  }

  return {
    append: (data: string) => {
      buffer += data;
      parse();
    },
    subscribe: (callback: (message: any) => void) => {
      subscribers.add(callback);
      return () => subscribers.delete(callback);
    },
  };
}

export class MCPTestClient {
  private process!: ChildProcess;
  private parser = createMessageParser();
  private responseEmitter = new (require('node:events').EventEmitter)();
  private static sharedMode = process.env.TEST_SHARED_SERVER === 'true';
  private isShared = false;
  private initPromise: Promise<void> | null = null;

  constructor() {
    this.parser.subscribe((message) => {
      this.responseEmitter.emit(message.id, message.result);
    });
  }

  static getShared(): MCPTestClient {
    if (!sharedServerInstance) {
      sharedServerInstance = new MCPTestClient();
      sharedServerInstance.isShared = true;
    }
    return sharedServerInstance;
  }

  async start(): Promise<void> {
    // If already started (shared mode), return existing promise
    if (this.initPromise) {
      return this.initPromise;
    }

    this.initPromise = this._doStart();
    return this.initPromise;
  }

  private async _doStart(): Promise<void> {
    this.process = spawn(process.execPath, ['dist/index.js'], {
      cwd: process.cwd(),
      stdio: ['pipe', 'pipe', 'pipe'],
      env: {
        ...process.env,
        TEST_MODE: getSystemCapabilities().isSlowSystem ? 'slow' : 'fast',
      },
    });

    this.process.on('error', (err) => console.error('MCP process error:', err));
    this.process.stderr?.on('data', (data) => process.stderr.write(data));
    this.process.stdout?.on('data', (data) => this.parser.append(data.toString()));

    return new Promise<void>((resolve, reject) => {
      const capabilities = getSystemCapabilities();
      const startupTimeout = capabilities.baseTimeout * capabilities.timeoutMultiplier;

      const timeout = setTimeout(() => {
        reject(new Error(`Test client startup timed out after ${startupTimeout / 1000} seconds.`));
      }, startupTimeout);

      this.process.stderr?.on('data', (data) => {
        if (data.toString().includes('Codebuddy Server running on stdio')) {
          clearTimeout(timeout);
          resolve();
        }
      });
    });
  }

  async stop(): Promise<void> {
    // Don't stop shared server instances
    if (this.isShared) {
      console.log('⚠️ Keeping shared server alive for other tests');
      return;
    }
    this.process?.kill('SIGTERM');
  }

  static async cleanup(): Promise<void> {
    if (sharedServerInstance) {
      sharedServerInstance.process?.kill('SIGTERM');
      sharedServerInstance = null;
    }
  }

  async callTool(name: string, args: Record<string, unknown>): Promise<any> {
    const id = Math.floor(Math.random() * 100000);
    const request = {
      jsonrpc: '2.0',
      id,
      method: 'call_tool',
      params: { name, arguments: args },
    };

    const requestString = `Content-Length: ${Buffer.byteLength(JSON.stringify(request))}\r\n\r\n${JSON.stringify(request)}`;

    return new Promise((resolve, reject) => {
      const capabilities = getSystemCapabilities();
      const requestTimeout = capabilities.baseTimeout * capabilities.timeoutMultiplier;

      const timeout = setTimeout(() => {
        reject(new Error(`Request ${name} (${id}) timed out after ${requestTimeout / 1000}s.`));
      }, requestTimeout);

      this.responseEmitter.once(id, (result) => {
        clearTimeout(timeout);
        resolve(result);
      });

      if (!this.process || !this.process.stdin) {
        clearTimeout(timeout);
        reject(new Error('Process not started or stdin not available'));
        return;
      }

      this.process.stdin.write(requestString, (err) => {
        if (err) {
          clearTimeout(timeout);
          reject(err);
        }
      });
    });
  }
}

export function assertToolResult(
  result: any
): asserts result is { content: Array<{ type: 'text'; text: string }> } {
  if (!result || !result.content || !Array.isArray(result.content)) {
    console.error('Invalid tool result:', result);
    throw new Error('Invalid tool result format');
  }
}
