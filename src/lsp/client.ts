import { existsSync, readFileSync } from 'node:fs';
import {
  createDefaultConfig,
  getAvailableDefaultServers,
  mergeWithDefaults,
} from '../default-config.js';
import { scanDirectoryForExtensions } from '../file-scanner.js';
import type { ServerState } from '../lsp-types.js';
import type { Config } from '../types.js';
import { LSPProtocol } from './protocol.js';
import { ServerManager } from './server-manager.js';

/**
 * Main LSP Client facade that coordinates protocol and server management
 * Provides the primary interface for LSP operations
 */
export class LSPClient {
  private config: Config;
  private _protocol: LSPProtocol;
  private _serverManager: ServerManager;

  // Public getters for facade access
  public get protocol(): LSPProtocol {
    return this._protocol;
  }
  public get serverManager(): ServerManager {
    return this._serverManager;
  }

  constructor(configPath?: string) {
    this._protocol = new LSPProtocol();
    this._serverManager = new ServerManager(this._protocol);
    this.config = this.loadConfig(configPath);
  }

  /**
   * Load configuration from environment, file, or use defaults
   */
  private loadConfig(configPath?: string): Config {
    // Try environment variable first (MCP config)
    if (process.env.CCLSP_CONFIG_PATH) {
      process.stderr.write(
        `Loading config from CCLSP_CONFIG_PATH: ${process.env.CCLSP_CONFIG_PATH}\n`
      );

      if (!existsSync(process.env.CCLSP_CONFIG_PATH)) {
        process.stderr.write(
          `Warning: Config file specified in CCLSP_CONFIG_PATH does not exist: ${process.env.CCLSP_CONFIG_PATH}\n`
        );
        process.stderr.write('Falling back to default configuration...\n');
        return this.loadDefaultConfig();
      }

      try {
        const configData = readFileSync(process.env.CCLSP_CONFIG_PATH, 'utf-8');
        const config = JSON.parse(configData);
        process.stderr.write(`Loaded ${config.servers.length} server configurations from env\n`);
        return mergeWithDefaults(config);
      } catch (error) {
        process.stderr.write(
          `Warning: Failed to load config from CCLSP_CONFIG_PATH: ${error instanceof Error ? error.message : String(error)}\n`
        );
        process.stderr.write('Falling back to default configuration...\n');
        return this.loadDefaultConfig();
      }
    }

    // Try loading from provided path
    if (configPath) {
      try {
        process.stderr.write(`Loading config from file: ${configPath}\n`);
        const configData = readFileSync(configPath, 'utf-8');
        const config = JSON.parse(configData);
        process.stderr.write(`Loaded ${config.servers.length} server configurations\n`);
        return mergeWithDefaults(config);
      } catch (error) {
        process.stderr.write(
          `Warning: Failed to load config from ${configPath}: ${error instanceof Error ? error.message : String(error)}\n`
        );
        process.stderr.write('Falling back to default configuration...\n');
        return this.loadDefaultConfig();
      }
    }

    // Try to find cclsp.json in current directory
    const defaultConfigPath = 'cclsp.json';
    if (existsSync(defaultConfigPath)) {
      try {
        process.stderr.write('Found cclsp.json in current directory, loading...\n');
        const configData = readFileSync(defaultConfigPath, 'utf-8');
        const config = JSON.parse(configData);
        process.stderr.write(`Loaded ${config.servers.length} server configurations\n`);
        return mergeWithDefaults(config);
      } catch (error) {
        process.stderr.write(
          `Warning: Failed to load cclsp.json: ${error instanceof Error ? error.message : String(error)}\n`
        );
      }
    }

    // Use default configuration
    process.stderr.write('No configuration found, using smart defaults...\n');
    return this.loadDefaultConfig();
  }

  /**
   * Load default configuration with all potential language servers
   * Actual availability will be checked when servers are started
   */
  private loadDefaultConfig(): Config {
    const defaultConfig = createDefaultConfig();
    process.stderr.write(
      `Using default configuration with support for ${defaultConfig.servers.length} languages\n`
    );
    process.stderr.write('TypeScript/JavaScript works out of the box (bundled dependency)\n');
    process.stderr.write('Other languages work if their servers are installed\n');
    process.stderr.write('To customize, create a cclsp.json file or run: cclsp setup\n');
    return defaultConfig;
  }

  private getLanguageName(extension: string): string | null {
    const languageMap: Record<string, string> = {
      ts: 'TypeScript',
      tsx: 'TypeScript',
      js: 'JavaScript',
      jsx: 'JavaScript',
      py: 'Python',
      go: 'Go',
      rs: 'Rust',
      java: 'Java',
      rb: 'Ruby',
      php: 'PHP',
      c: 'C',
      cpp: 'C++',
      css: 'CSS',
      html: 'HTML',
      json: 'JSON',
      yaml: 'YAML',
      vue: 'Vue',
      svelte: 'Svelte',
    };
    return languageMap[extension] || null;
  }

  /**
   * Get LSP server for a file path
   */
  async getServer(filePath: string): Promise<ServerState> {
    return await this._serverManager.getServer(filePath, this.config);
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
    return await this._protocol.sendRequest(serverState.process, method, params, timeout);
  }

  /**
   * Send notification through LSP protocol
   */
  sendNotification(serverState: ServerState, method: string, params: unknown): void {
    this._protocol.sendNotification(serverState.process, method, params);
  }

  /**
   * Restart servers for specified extensions
   */
  async restartServer(extensions?: string[]): Promise<string[]> {
    return await this._serverManager.restartServer(extensions, this.config);
  }

  /**
   * Preload servers for detected file types in project
   */
  async preloadServers(): Promise<void> {
    try {
      const extensions = await scanDirectoryForExtensions(process.cwd());
      await this._serverManager.preloadServers(this.config, Array.from(extensions));
    } catch (error) {
      process.stderr.write(`Failed to scan directory for extensions: ${error}\n`);
    }
  }

  /**
   * Clean up all resources
   */
  dispose(): void {
    this._serverManager.dispose();
  }
}
