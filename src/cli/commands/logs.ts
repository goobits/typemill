#!/usr/bin/env node

import { spawn } from 'node:child_process';
import { existsSync, readFileSync } from 'node:fs';
import * as DirectoryUtils from '../directory-utils.js';

interface LogsOptions {
  tail?: boolean;
  lines?: number;
}

/**
 * Logs command - debug output when things go wrong
 * Simple file-based approach
 */
export async function logsCommand(options: LogsOptions = {}): Promise<void> {
  const logPath = DirectoryUtils.getLogPath();

  // Check if log file exists
  if (!existsSync(logPath)) {
    console.log('No log file found');
    console.log('Start the MCP server to begin logging:');
    console.log('  codebuddy');
    return;
  }

  // Default behavior: show recent logs
  if (!options.tail) {
    try {
      const logContent = readFileSync(logPath, 'utf-8');
      const lines = logContent.trim().split('\n');

      // Show last 50 lines by default, or specified amount
      const maxLines = options.lines || 50;
      const recentLines = lines.slice(-maxLines);

      if (recentLines.length < lines.length) {
        console.log(`... (showing last ${recentLines.length} of ${lines.length} lines)`);
      }

      for (const line of recentLines) {
        console.log(line);
      }
    } catch (error) {
      console.error(`Error reading log file: ${error}`);
      process.exit(1);
    }
    return;
  }

  // --tail: follow logs in real-time
  try {
    const tailProc = spawn('tail', ['-f', logPath], {
      stdio: 'inherit',
    });

    // Handle Ctrl+C gracefully
    process.on('SIGINT', () => {
      tailProc.kill();
      console.log('\\nStopped following logs');
      process.exit(0);
    });

    tailProc.on('error', (error) => {
      // Fallback: basic polling if tail command not available
      if (error.message.includes('ENOENT')) {
        console.log('Tail command not available, using basic mode...');
        basicTailFollow(logPath);
      } else {
        console.error(`Error following logs: ${error.message}`);
        process.exit(1);
      }
    });
  } catch (error) {
    console.error(`Error following logs: ${error}`);
    process.exit(1);
  }
}

/**
 * Basic implementation of tail -f when tail command is not available
 */
function basicTailFollow(logPath: string): void {
  let lastSize = 0;

  // Get initial file size
  try {
    const stats = require('node:fs').statSync(logPath);
    lastSize = stats.size;
  } catch (error) {
    console.error(`Error accessing log file: ${error}`);
    process.exit(1);
  }

  console.log('Following log file (basic mode)...');
  console.log('Press Ctrl+C to stop');

  const pollInterval = setInterval(() => {
    try {
      const stats = require('node:fs').statSync(logPath);

      if (stats.size > lastSize) {
        // Read new content
        const fd = require('node:fs').openSync(logPath, 'r');
        const buffer = Buffer.alloc(stats.size - lastSize);
        require('node:fs').readSync(fd, buffer, 0, buffer.length, lastSize);
        require('node:fs').closeSync(fd);

        const newContent = buffer.toString('utf-8');
        process.stdout.write(newContent);

        lastSize = stats.size;
      }
    } catch (error) {
      console.error(`Error reading log updates: ${error}`);
      clearInterval(pollInterval);
      process.exit(1);
    }
  }, 1000); // Poll every second

  // Handle Ctrl+C
  process.on('SIGINT', () => {
    clearInterval(pollInterval);
    console.log('\\nStopped following logs');
    process.exit(0);
  });
}
