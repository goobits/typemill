import { processManager } from './process-manager.js';

/**
 * Check if a process is running by PID
 * @param pid Process ID to check
 * @returns true if process is running, false otherwise
 */
export function isProcessRunning(pid: number): boolean {
  return processManager.isRunning(pid);
}

/**
 * Terminate a process by PID using cross-platform process manager
 * @param pid Process ID to terminate
 * @returns Promise that resolves when process is terminated
 */
export function terminateProcess(pid: number): Promise<void> {
  return processManager.terminate(pid, {
    force: false,
    timeout: 5000,
  });
}
