#!/usr/bin/env node

import * as DirectoryUtils from '../directory-utils.js';
import * as ServerUtils from '../server-utils.js';

interface ServerInfo {
  name: string;
  extensions: string[];
  command: string[];
  available: boolean;
  running: boolean;
  pid?: number;
}

/**
 * Status command - shows what's working right now
 * Exactly what Bob specified
 */
export async function statusCommand(): Promise<void> {
  // Auto-migrate if needed
  DirectoryUtils.migrateOldConfig();

  console.log('Language Servers:');

  const config = DirectoryUtils.readConfig();

  if (!config || !config.servers?.length) {
    console.log('  No configuration found');
    console.log('  Run: codebuddy init');
    return;
  }

  const state = DirectoryUtils.readState();
  const servers: ServerInfo[] = [];
  let activeCount = 0;
  let issueCount = 0;

  // Test each server
  for (const server of config.servers) {
    const serverKey = getServerKey(server);
    const serverState = state[serverKey];
    const running = serverState ? ServerUtils.isProcessRunning(serverState.pid) : false;

    // Test if server command is available
    const available = await ServerUtils.testCommand(server.command);

    const serverInfo: ServerInfo = {
      name: getServerName(server.command[0] || 'unknown'),
      extensions: server.extensions,
      command: server.command,
      available,
      running,
      pid: running ? serverState?.pid : undefined,
    };

    servers.push(serverInfo);

    if (available) {
      activeCount++;
    } else {
      issueCount++;
    }
  }

  // Display results
  for (const server of servers) {
    const status = server.available ? '✓' : '✗';
    const extList = `(${server.extensions.map((ext) => `.${ext}`).join(' ')})`;
    const runningInfo = server.running ? ` [PID: ${server.pid}]` : '';
    const fixHint = server.available ? '' : " - run 'codebuddy fix'";

    console.log(`  ${status} ${server.name}  ${extList}${runningInfo}${fixHint}`);
  }

  console.log('');
  console.log(`Active: ${activeCount} servers`);
  if (issueCount > 0) {
    console.log(`Issues: ${issueCount} (fixable)`);
  }
  console.log(`Config: ${DirectoryUtils.getConfigPath()}`);
}

function getServerKey(server: { command: string[] }): string {
  return JSON.stringify(server.command);
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
