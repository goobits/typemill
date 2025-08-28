import { existsSync, readFileSync } from 'node:fs';
import { scanDirectoryForExtensions } from '../file-scanner.js';
import type { Config } from '../types.js';
import { LSPProtocol, type ServerState } from './protocol.js';
import { ServerManager } from './server-manager.js';

/**
 * Main LSP Client facade that coordinates protocol and server management
 * Provides the primary interface for LSP operations
 */
export class LSPClient {
  private config: Config;
  private protocol: LSPProtocol;
  private serverManager: ServerManager;

  constructor(configPath?: string) {
    this.protocol = new LSPProtocol();
    this.serverManager = new ServerManager(this.protocol);
    this.config = this.loadConfig(configPath);
  }

  /**
   * Load configuration from environment or file
   */
  private loadConfig(configPath?: string): Config {
    // Try environment variable first (MCP config)
    if (process.env.CCLSP_CONFIG_PATH) {
      process.stderr.write(
        `Loading config from CCLSP_CONFIG_PATH: ${process.env.CCLSP_CONFIG_PATH}\n`
      );

      if (!existsSync(process.env.CCLSP_CONFIG_PATH)) {
        process.stderr.write(
          `Config file specified in CCLSP_CONFIG_PATH does not exist: ${process.env.CCLSP_CONFIG_PATH}\n`
        );
        process.exit(1);
      }

      try {
        const configData = readFileSync(process.env.CCLSP_CONFIG_PATH, 'utf-8');
        const config = JSON.parse(configData);
        process.stderr.write(`Loaded ${config.servers.length} server configurations from env\n`);
        return config;
      } catch (error) {
        process.stderr.write(`Failed to load config from CCLSP_CONFIG_PATH: ${error}\n`);
        process.exit(1);
      }
    }

    // configPath must be provided if CCLSP_CONFIG_PATH is not set
    if (!configPath) {
      process.stderr.write(
        'Error: configPath is required when CCLSP_CONFIG_PATH environment variable is not set\n'
      );
      process.exit(1);
    }

    // Load from config file
    try {
      process.stderr.write(`Loading config from file: ${configPath}\n`);
      const configData = readFileSync(configPath, 'utf-8');
      const config = JSON.parse(configData);
      process.stderr.write(`Loaded ${config.servers.length} server configurations\n`);
      return config;
    } catch (error) {
      process.stderr.write(`Failed to load config from ${configPath}: ${error}\n`);
      process.exit(1);
    }
  }

  /**
   * Get LSP server for a file path
   */
  async getServer(filePath: string): Promise<ServerState> {
    return await this.serverManager.getServer(filePath, this.config);
  }

  /**
   * Send request through LSP protocol
   */
  async sendRequest(
    serverState: ServerState,
    method: string,
    params: unknown,
    timeout?: number
  ): Promise<unknown> {
    return await this.protocol.sendRequest(serverState.process, method, params, timeout);
  }

  /**
   * Send notification through LSP protocol
   */
  sendNotification(serverState: ServerState, method: string, params: unknown): void {
    this.protocol.sendNotification(serverState.process, method, params);
  }

  /**
   * Restart servers for specified extensions
   */
  async restartServer(extensions?: string[]): Promise<string[]> {
    return await this.serverManager.restartServer(extensions, this.config);
  }

  /**
   * Preload servers for detected file types in project
   */
  async preloadServers(): Promise<void> {
    try {
      const extensions = await scanDirectoryForExtensions(process.cwd());
      await this.serverManager.preloadServers(this.config, Array.from(extensions));
    } catch (error) {
      process.stderr.write(`Failed to scan directory for extensions: ${error}\n`);
    }
  }

  /**
   * Clean up all resources
   */
  dispose(): void {
    this.serverManager.dispose();
  }
}
