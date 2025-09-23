#!/usr/bin/env node

/**
 * CLI-specific types for better type inference
 */

export interface CLIServerConfig {
  extensions: string[];
  command: string[];
  rootDir?: string;
  restartInterval?: number;
  initializationOptions?: unknown;
}

export interface CLIConfig {
  servers: CLIServerConfig[];
}

export interface ServerState {
  pid: number;
}

export interface StateFile {
  [serverKey: string]: ServerState;
}

export interface AssistantInfo {
  name: string;
  displayName: string;
  configPath: string;
  installed: boolean;
  linked: boolean;
}

export interface LinkOptions {
  assistants?: string[];
  all?: boolean;
}

export interface UnlinkOptions {
  assistants?: string[];
  all?: boolean;
}

export interface StatusOutput {
  lsps: Array<{ name: string; status: 'ok' | 'error' }>;
  assistants: Array<{ name: string; linked: boolean }>;
  server: { status: string; uptime_sec?: number };
}
