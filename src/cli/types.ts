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
