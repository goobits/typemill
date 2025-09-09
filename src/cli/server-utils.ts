#!/usr/bin/env node

import { type ChildProcess, spawn } from 'node:child_process';

const TIMEOUT_MS = 2000;

/**
 * Test if a command is available and working
 */
export async function testCommand(command: string[]): Promise<boolean> {
  if (!command.length) return false;

  const [cmd, ...args] = command;

  if (!cmd) return false;

  // Special handling for npx commands
  if (cmd === 'npx') {
    // First check if npm is available
    if (!(await testCommand(['npm', '--version']))) {
      return false;
    }

    // For npx commands, we assume they work if npm is available
    // since typescript-language-server is bundled
    return true;
  }

  return new Promise((resolve) => {
    const testArgs = getTestArgs(cmd);
    const proc = spawn(cmd, testArgs, {
      stdio: 'ignore',
      shell: false,
    }) as ChildProcess;

    let resolved = false;

    proc.on('error', () => {
      if (!resolved) {
        resolved = true;
        resolve(false);
      }
    });

    proc.on('exit', (code: number | null) => {
      if (!resolved) {
        resolved = true;
        resolve(code === 0);
      }
    });

    // Timeout
    setTimeout(() => {
      if (!resolved) {
        resolved = true;
        proc.kill();
        resolve(false);
      }
    }, TIMEOUT_MS);
  });
}

/**
 * Get appropriate test arguments for a command
 */
function getTestArgs(command: string): string[] {
  const versionCommands = new Set([
    'pylsp',
    'gopls',
    'rust-analyzer',
    'clangd',
    'jdtls',
    'solargraph',
    'intelephense',
    'npm',
  ]);

  const helpCommands = new Set(['docker-langserver']);

  if (versionCommands.has(command)) {
    return ['--version'];
  }
  if (helpCommands.has(command)) {
    return ['--help'];
  }
  return ['--version']; // Default
}

/**
 * Check if a process with given PID is running
 */
export function isProcessRunning(pid: number): boolean {
  try {
    // process.kill with signal 0 doesn't kill, just tests if process exists
    process.kill(pid, 0);
    return true;
  } catch (error) {
    return false;
  }
}

/**
 * Get install instructions for a command
 */
export function getInstallInstructions(command: string): string {
  const instructions: Record<string, string> = {
    'typescript-language-server': 'npm install -g typescript-language-server typescript',
    pylsp: 'pip install python-lsp-server',
    gopls: 'go install golang.org/x/tools/gopls@latest',
    'rust-analyzer': 'rustup component add rust-analyzer',
    clangd: 'apt install clangd OR brew install llvm',
    jdtls: 'Download from Eclipse JDT releases',
    solargraph: 'gem install solargraph',
    intelephense: 'npm install -g intelephense',
    'docker-langserver': 'npm install -g dockerfile-language-server-nodejs',
    'yaml-language-server': 'npm install -g yaml-language-server',
    'bash-language-server': 'npm install -g bash-language-server',
    'vscode-json-language-server': 'npm install -g vscode-langservers-extracted',
    'vscode-html-language-server': 'npm install -g vscode-langservers-extracted',
    'vscode-css-language-server': 'npm install -g vscode-langservers-extracted',
  };

  return instructions[command] || `Install ${command}`;
}
