import { execSync } from 'node:child_process';

/**
 * Helper utilities for managing LSP server processes during tests
 * Used for error recovery and crash simulation testing
 */

export interface ProcessInfo {
  pid: number;
  command: string;
  args: string[];
}

/**
 * Find running LSP server processes by name
 */
export function findLSPServers(serverName: string): ProcessInfo[] {
  try {
    // Use ps to find processes (works on Linux/Mac)
    const output = execSync(`ps aux | grep -i "${serverName}" | grep -v grep`, {
      encoding: 'utf-8',
    });

    const processes: ProcessInfo[] = [];
    const lines = output.trim().split('\n');

    for (const line of lines) {
      const parts = line.split(/\s+/);
      if (parts.length > 10) {
        const pid = Number.parseInt(parts[1], 10);
        if (!Number.isNaN(pid)) {
          // Extract command and args from the ps output
          const commandParts = parts.slice(10);
          processes.push({
            pid,
            command: commandParts[0] || '',
            args: commandParts.slice(1),
          });
        }
      }
    }

    return processes;
  } catch {
    // If grep finds nothing, it returns non-zero exit code
    return [];
  }
}

/**
 * Kill a process by PID with optional signal
 */
export function killProcess(pid: number, signal = 'TERM'): boolean {
  try {
    execSync(`kill -${signal} ${pid}`, { encoding: 'utf-8' });
    return true;
  } catch {
    return false;
  }
}

/**
 * Kill all LSP servers matching a name pattern
 */
export function killLSPServers(serverName: string): number {
  const servers = findLSPServers(serverName);
  let killed = 0;

  for (const server of servers) {
    if (killProcess(server.pid)) {
      killed++;
    }
  }

  return killed;
}

/**
 * Force kill all TypeScript language servers
 */
export function killTypeScriptServers(): number {
  return killLSPServers('typescript-language-server');
}

/**
 * Force kill all Python language servers
 */
export function killPythonServers(): number {
  return killLSPServers('pylsp');
}

/**
 * Check if a process is still running
 */
export function isProcessRunning(pid: number): boolean {
  try {
    // kill -0 just checks if process exists without sending signal
    execSync(`kill -0 ${pid}`, { encoding: 'utf-8' });
    return true;
  } catch {
    return false;
  }
}

/**
 * Wait for a process to terminate with timeout
 */
export async function waitForProcessDeath(pid: number, timeoutMs = 5000): Promise<boolean> {
  const startTime = Date.now();

  while (Date.now() - startTime < timeoutMs) {
    if (!isProcessRunning(pid)) {
      return true;
    }
    await new Promise((resolve) => setTimeout(resolve, 100));
  }

  return false;
}

/**
 * Simulate a server crash by killing it abruptly
 */
export async function simulateServerCrash(serverName: string): Promise<boolean> {
  const servers = findLSPServers(serverName);
  if (servers.length === 0) {
    return false;
  }

  // Kill with SIGKILL for immediate termination
  for (const server of servers) {
    killProcess(server.pid, 'KILL');
  }

  // Wait a bit for processes to die
  await new Promise((resolve) => setTimeout(resolve, 500));

  // Verify they're dead
  const remaining = findLSPServers(serverName);
  return remaining.length === 0;
}

/**
 * Get memory usage of a process in MB
 */
export function getProcessMemory(pid: number): number | null {
  try {
    const output = execSync(`ps -o rss= -p ${pid}`, { encoding: 'utf-8' });
    const kb = Number.parseInt(output.trim(), 10);
    if (!Number.isNaN(kb)) {
      return kb / 1024; // Convert to MB
    }
  } catch {
    // Process not found or ps command failed
  }
  return null;
}

/**
 * Monitor memory usage of LSP servers
 */
export function getLSPServerMemory(serverName: string): Map<number, number> {
  const servers = findLSPServers(serverName);
  const memoryMap = new Map<number, number>();

  for (const server of servers) {
    const memory = getProcessMemory(server.pid);
    if (memory !== null) {
      memoryMap.set(server.pid, memory);
    }
  }

  return memoryMap;
}

/**
 * Clean up all known LSP servers (useful for test cleanup)
 */
export function cleanupAllLSPServers(): void {
  killTypeScriptServers();
  killPythonServers();
  // Add more as needed
}

/**
 * Wait for LSP server to start up
 */
export async function waitForLSPServer(
  serverName: string,
  timeoutMs = 10000
): Promise<ProcessInfo | null> {
  const startTime = Date.now();

  while (Date.now() - startTime < timeoutMs) {
    const servers = findLSPServers(serverName);
    if (servers.length > 0) {
      return servers[0];
    }
    await new Promise((resolve) => setTimeout(resolve, 500));
  }

  return null;
}
