#!/usr/bin/env node

import { scanDirectoryForExtensions } from '../../file-scanner.js';
import * as DirectoryUtils from '../directory-utils.js';
import * as ServerUtils from '../server-utils.js';

// Default server configurations - simplified from existing presets
const DEFAULT_SERVER_CONFIGS = [
  {
    extensions: ['ts', 'tsx', 'js', 'jsx', 'mjs', 'cjs'],
    command: ['npx', '--', 'typescript-language-server', '--stdio'],
    name: 'TypeScript',
  },
  {
    extensions: ['py', 'pyi'],
    command: ['pylsp'],
    name: 'Python',
  },
  {
    extensions: ['go'],
    command: ['gopls'],
    name: 'Go',
  },
  {
    extensions: ['rs'],
    command: ['rust-analyzer'],
    name: 'Rust',
  },
  {
    extensions: ['json', 'jsonc'],
    command: [
      'npx',
      '--',
      'vscode-langservers-extracted',
      '--',
      'vscode-json-language-server',
      '--stdio',
    ],
    name: 'JSON',
  },
  {
    extensions: ['html', 'htm'],
    command: [
      'npx',
      '--',
      'vscode-langservers-extracted',
      '--',
      'vscode-html-language-server',
      '--stdio',
    ],
    name: 'HTML',
  },
  {
    extensions: ['css', 'scss', 'sass', 'less'],
    command: [
      'npx',
      '--',
      'vscode-langservers-extracted',
      '--',
      'vscode-css-language-server',
      '--stdio',
    ],
    name: 'CSS',
  },
  {
    extensions: ['yaml', 'yml'],
    command: ['npx', '--', 'yaml-language-server', '--stdio'],
    name: 'YAML',
  },
  {
    extensions: ['sh', 'bash', 'zsh'],
    command: ['npx', '--', 'bash-language-server', 'start'],
    name: 'Shell',
  },
];

interface InitOptions {
  auto?: boolean;
}

/**
 * Init command - smart setup with auto-detection
 * Exactly Bob's vision
 */
export async function initCommand(options: InitOptions = {}): Promise<void> {
  console.clear();
  console.log('🚀 codebuddy initialization\n');

  // Check if config already exists
  const existingConfig = DirectoryUtils.readConfigSilent();
  if (existingConfig) {
    console.log('Configuration already exists');
    console.log(`Location: ${DirectoryUtils.getConfigPath()}`);
    console.log('\nUse: codebuddy config --show');
    return;
  }

  // Auto-migrate old config
  if (DirectoryUtils.migrateOldConfig()) {
    console.log('✅ Migrated existing configuration');
    return;
  }

  console.log('Detecting project languages...');

  // Scan for file extensions
  let detectedExtensions: Set<string>;
  try {
    detectedExtensions = await scanDirectoryForExtensions(process.cwd());

    if (detectedExtensions.size === 0) {
      console.log('📝 No source files detected in project');
    } else {
      const langList = Array.from(detectedExtensions).slice(0, 8).join(', ');
      const more = detectedExtensions.size > 8 ? '...' : '';
      console.log(`📝 Found: ${langList}${more}`);
    }
  } catch (error) {
    console.log('⚠️  Could not scan project files');
    detectedExtensions = new Set();
  }

  console.log('\nChecking language servers...');

  // Find relevant servers and test availability
  const relevantServers = [];
  const availableServers = [];
  const missingServers = [];

  for (const serverConfig of DEFAULT_SERVER_CONFIGS) {
    const isRelevant =
      detectedExtensions.size === 0 ||
      serverConfig.extensions.some((ext) => detectedExtensions.has(ext));

    if (isRelevant) {
      relevantServers.push(serverConfig);

      const available = await ServerUtils.testCommand(serverConfig.command);
      if (available) {
        console.log(`✓ ${serverConfig.name}: ${serverConfig.command[0]}`);
        availableServers.push(serverConfig);
      } else {
        console.log(`✗ ${serverConfig.name}: ${serverConfig.command[0]} not found`);
        missingServers.push(serverConfig);
      }
    }
  }

  // Create configuration with available servers
  const config = {
    servers: availableServers.map((server) => ({
      extensions: server.extensions,
      command: server.command,
    })),
  };

  // Save configuration
  DirectoryUtils.writeConfig(config);
  console.log(`\n✅ Configuration created: ${DirectoryUtils.getConfigPath()}`);
  console.log(`🔧 ${availableServers.length} language servers ready`);

  // Show missing servers and suggest fix
  if (missingServers.length > 0) {
    console.log(`⚠️  ${missingServers.length} servers missing`);

    // Auto-install option
    if (options.auto) {
      console.log('\n🔄 Auto-installing missing servers...');
      // Would call fix command here
      const { fixCommand } = await import('./fix.js');
      await fixCommand({ auto: true });
    } else {
      console.log('\nTo install missing servers:');
      console.log('  codebuddy fix');

      console.log('\nOr install manually:');
      for (const server of missingServers.slice(0, 3)) {
        const cmd = server.command[0] || '';
        console.log(`  ${server.name}: ${ServerUtils.getInstallInstructions(cmd)}`);
      }
    }
  }

  console.log('\n✨ Ready to use with Claude Code!');
}
