#!/usr/bin/env node

import { type ChildProcess, spawn } from 'node:child_process';
import * as DirectoryUtils from '../directory-utils.js';
import * as ServerUtils from '../server-utils.js';

// Auto-install commands - taken from existing setup.ts but simplified
const AUTO_INSTALL_COMMANDS: Record<string, string[]> = {
  'typescript-language-server': [
    'npm',
    'install',
    '-g',
    'typescript-language-server',
    'typescript',
  ],
  pylsp: ['pip', 'install', 'python-lsp-server[all]'],
  gopls: ['go', 'install', 'golang.org/x/tools/gopls@latest'],
  'rust-analyzer': ['rustup', 'component', 'add', 'rust-analyzer'],
  clangd: ['apt', 'install', 'clangd'], // Basic - user can adjust
  jdtls: [], // No auto-install - too complex
  solargraph: ['gem', 'install', 'solargraph'],
  intelephense: ['npm', 'install', '-g', 'intelephense'],
  'docker-langserver': ['npm', 'install', '-g', 'dockerfile-language-server-nodejs'],
  'yaml-language-server': ['npm', 'install', '-g', 'yaml-language-server'],
  'bash-language-server': ['npm', 'install', '-g', 'bash-language-server'],
  'vscode-json-language-server': ['npm', 'install', '-g', 'vscode-langservers-extracted'],
  'vscode-html-language-server': ['npm', 'install', '-g', 'vscode-langservers-extracted'],
  'vscode-css-language-server': ['npm', 'install', '-g', 'vscode-langservers-extracted'],
};

interface FixOptions {
  auto?: boolean;
  manual?: boolean;
}

/**
 * Fix command - actually fix problems
 * Bob's "fix should actually fix" philosophy
 */
export async function fixCommand(options: FixOptions = {}): Promise<void> {
  // Auto-migrate if needed
  DirectoryUtils.migrateOldConfig();

  // Manual mode - just show install commands
  if (options.manual) {
    console.log('Manual installation commands:\n');

    const config = DirectoryUtils.readConfig();
    if (!config?.servers?.length) {
      console.log('No configuration found. Run: codebuddy init');
      return;
    }

    for (const server of config.servers) {
      const command = server.command[0] || '';
      const available = await ServerUtils.testCommand(server.command);

      if (!available) {
        const installCmd = AUTO_INSTALL_COMMANDS[command];
        if (installCmd?.length) {
          console.log(`${getServerName(command)}: ${installCmd.join(' ')}`);
        } else {
          console.log(`${getServerName(command)}: ${ServerUtils.getInstallInstructions(command)}`);
        }
      }
    }
    return;
  }

  console.log('Detecting issues...\n');

  const config = DirectoryUtils.readConfig();
  if (!config?.servers?.length) {
    console.log('No configuration found');
    console.log('Run: codebuddy init');
    return;
  }

  const issues = [];
  const fixable = [];

  // Find broken servers
  for (const server of config.servers) {
    const available = await ServerUtils.testCommand(server.command);
    const command = server.command[0] || '';

    if (!available) {
      const serverName = getServerName(command);
      const installCmd = AUTO_INSTALL_COMMANDS[command];

      issues.push({
        name: serverName,
        command,
        extensions: server.extensions,
        installCommand: installCmd,
        canAutoFix: installCmd && installCmd.length > 0,
      });

      if (installCmd && installCmd.length > 0) {
        fixable.push(issues[issues.length - 1]);
      }
    }
  }

  if (issues.length === 0) {
    console.log('✅ All language servers are working correctly');
    return;
  }

  // Show issues and attempt fixes
  let fixedCount = 0;
  let failedCount = 0;

  for (const issue of issues) {
    console.log(`✗ ${issue.name} files detected but ${issue.command} not working`);

    if (issue.canAutoFix) {
      console.log('  Attempting automatic fix...');
      console.log(`  → Trying: ${issue.installCommand?.join(' ')}`);

      // Prompt for permission unless --auto
      let shouldInstall = options.auto || false;

      if (!options.auto) {
        shouldInstall = await promptYesNo(`  Install ${issue.name} server? [y/N]: `);
      }

      if (shouldInstall) {
        const success = await runInstallCommand(issue.installCommand as string[], issue.name);
        if (success) {
          console.log(`  ✓ Fixed: ${issue.name} support enabled\n`);
          fixedCount++;
        } else {
          console.log(`  ✗ Failed to install ${issue.name}\n`);
          failedCount++;
        }
      } else {
        console.log(`  ⏭ Skipped: ${issue.name}\n`);
      }
    } else {
      console.log('  Manual fix required:');
      console.log(`    ${ServerUtils.getInstallInstructions(issue.command)}\n`);
      failedCount++;
    }
  }

  // Summary
  console.log('📊 Fix Summary:');
  if (fixedCount > 0) {
    console.log(`   ✅ Fixed: ${fixedCount} issues`);
  }
  if (failedCount > 0) {
    console.log(`   ❌ Remaining: ${failedCount} require manual intervention`);
  }
}

async function runInstallCommand(command: string[], serverName: string): Promise<boolean> {
  return new Promise((resolve) => {
    const [cmd, ...args] = command;

    if (!cmd) {
      resolve(false);
      return;
    }

    const proc = spawn(cmd, args, {
      stdio: 'pipe',
    }) as ChildProcess;

    let output = '';
    let error = '';

    proc.stdout?.on('data', (data: Buffer) => {
      output += data.toString();
    });

    proc.stderr?.on('data', (data: Buffer) => {
      error += data.toString();
    });

    proc.on('error', (err: Error) => {
      console.log(`    Error: ${err.message}`);
      resolve(false);
    });

    proc.on('close', (code: number | null) => {
      if (code === 0) {
        resolve(true);
      } else {
        if (error.trim()) {
          console.log(`    Error: ${error.trim()}`);
        }
        resolve(false);
      }
    });
  });
}

async function promptYesNo(question: string): Promise<boolean> {
  process.stdout.write(question);

  return new Promise((resolve) => {
    process.stdin.once('data', (data) => {
      const answer = data.toString().trim().toLowerCase();
      resolve(answer === 'y' || answer === 'yes');
    });

    // Handle Ctrl+C gracefully
    process.once('SIGINT', () => {
      console.log('\n\nOperation cancelled.');
      process.exit(0);
    });
  });
}

function getServerName(command: string): string {
  const nameMap: Record<string, string> = {
    npx: 'TypeScript',
    'typescript-language-server': 'TypeScript',
    pylsp: 'Python',
    gopls: 'Go',
    'rust-analyzer': 'Rust',
    clangd: 'C/C++',
    jdtls: 'Java',
    solargraph: 'Ruby',
    intelephense: 'PHP',
    'docker-langserver': 'Docker',
    'yaml-language-server': 'YAML',
    'bash-language-server': 'Shell',
    'vscode-json-language-server': 'JSON',
    'vscode-html-language-server': 'HTML',
    'vscode-css-language-server': 'CSS',
  };

  return nameMap[command] || command;
}
