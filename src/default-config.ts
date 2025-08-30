import type { Config, LSPServerConfig } from './types.js';

/**
 * Default configurations for common language servers
 * These are carefully chosen to work on most systems
 */
export const DEFAULT_SERVERS: LSPServerConfig[] = [
  {
    // TypeScript/JavaScript - Already bundled as dependency!
    extensions: ['ts', 'tsx', 'js', 'jsx', 'mjs', 'cjs'],
    command: ['npx', '--', 'typescript-language-server', '--stdio'],
    restartInterval: 10,
  },
  {
    // Python - Try common installations
    extensions: ['py', 'pyi'],
    command: ['pylsp'],
    restartInterval: 5,
  },
  {
    // Go - Standard installation
    extensions: ['go'],
    command: ['gopls'],
    restartInterval: 15,
  },
  {
    // Rust - Via rustup
    extensions: ['rs'],
    command: ['rust-analyzer'],
    restartInterval: 15,
  },
  {
    // JSON/YAML - Via vscode-json-languageserver
    extensions: ['json', 'jsonc'],
    command: ['npx', '--', 'vscode-json-languageserver', '--stdio'],
    restartInterval: 5,
  },
  {
    // HTML/CSS - Via vscode-css-languageserver
    extensions: ['html', 'htm'],
    command: ['npx', '--', 'vscode-html-languageserver', '--stdio'],
    restartInterval: 5,
  },
  {
    // CSS/SCSS/LESS
    extensions: ['css', 'scss', 'sass', 'less'],
    command: ['npx', '--', 'vscode-css-languageserver', '--stdio'],
    restartInterval: 5,
  },
  {
    // Vue
    extensions: ['vue'],
    command: ['npx', '--', 'vue-language-server', '--stdio'],
    restartInterval: 10,
  },
  {
    // Svelte
    extensions: ['svelte'],
    command: ['npx', '--', 'svelteserver', '--stdio'],
    restartInterval: 10,
  },
  {
    // C/C++ via clangd
    extensions: ['c', 'cpp', 'cc', 'cxx', 'h', 'hpp'],
    command: ['clangd'],
    restartInterval: 15,
  },
  {
    // Java
    extensions: ['java'],
    command: ['jdtls'],
    restartInterval: 20,
  },
  {
    // Ruby
    extensions: ['rb', 'ruby'],
    command: ['solargraph', 'stdio'],
    restartInterval: 10,
  },
  {
    // PHP
    extensions: ['php'],
    command: ['intelephense', '--stdio'],
    restartInterval: 10,
  },
  {
    // Shell scripts
    extensions: ['sh', 'bash', 'zsh'],
    command: ['npx', '--', 'bash-language-server', 'start'],
    restartInterval: 5,
  },
  {
    // Dockerfile
    extensions: ['dockerfile', 'Dockerfile'],
    command: ['docker-langserver', '--stdio'],
    restartInterval: 5,
  },
  {
    // YAML
    extensions: ['yaml', 'yml'],
    command: ['npx', '--', 'yaml-language-server', '--stdio'],
    restartInterval: 5,
  },
  {
    // Markdown
    extensions: ['md', 'markdown'],
    command: ['npx', '--', 'markdownlint-language-server', '--stdio'],
    restartInterval: 5,
  },
];

/**
 * Create default configuration
 */
export function createDefaultConfig(): Config {
  return {
    servers: DEFAULT_SERVERS,
  };
}

/**
 * Check if a command is available on the system
 */
export async function isCommandAvailable(command: string[]): Promise<boolean> {
  try {
    const { spawn } = await import('node:child_process');
    const [cmd, ...args] = command;

    // Special handling for npx commands - check if npm is available
    if (cmd === 'npx') {
      return await isCommandAvailable(['npm', '--version']);
    }

    // Try to run the command with --version or --help
    return new Promise((resolve) => {
      const testArgs = cmd === 'npm' ? ['--version'] : ['--version'];
      const proc = spawn(cmd!, testArgs, {
        stdio: 'ignore',
        shell: false,
      });

      proc.on('error', () => resolve(false));
      proc.on('exit', (code) => resolve(code === 0));

      // Timeout after 2 seconds
      setTimeout(() => {
        proc.kill();
        resolve(false);
      }, 2000);
    });
  } catch {
    return false;
  }
}

/**
 * Filter default servers to only those with available commands
 */
export async function getAvailableDefaultServers(): Promise<LSPServerConfig[]> {
  const available: LSPServerConfig[] = [];

  for (const server of DEFAULT_SERVERS) {
    // Always include npx-based servers since typescript-language-server is bundled
    if (server.command[0] === 'npx') {
      available.push(server);
      continue;
    }

    // Check if the command is available
    if (await isCommandAvailable(server.command)) {
      available.push(server);
    }
  }

  return available;
}

/**
 * Merge user config with defaults
 * User config takes precedence
 */
export function mergeWithDefaults(userConfig?: Partial<Config>): Config {
  if (!userConfig?.servers) {
    return createDefaultConfig();
  }

  // Create a map of user-configured extensions
  const userExtensions = new Set<string>();
  for (const server of userConfig.servers) {
    for (const ext of server.extensions) {
      userExtensions.add(ext);
    }
  }

  // Add default servers for unconfigured extensions
  const mergedServers = [...userConfig.servers];
  for (const defaultServer of DEFAULT_SERVERS) {
    const hasUnconfiguredExtension = defaultServer.extensions.some(
      (ext) => !userExtensions.has(ext)
    );

    if (hasUnconfiguredExtension) {
      // Only add extensions that aren't already configured
      const unconfiguredExtensions = defaultServer.extensions.filter(
        (ext) => !userExtensions.has(ext)
      );

      if (unconfiguredExtensions.length > 0) {
        mergedServers.push({
          ...defaultServer,
          extensions: unconfiguredExtensions,
        });
      }
    }
  }

  return {
    servers: mergedServers,
  };
}
