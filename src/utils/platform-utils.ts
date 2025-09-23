/**
 * Cross-platform utility functions for system operations
 */

import { spawn } from 'node:child_process';
import { homedir, platform, tmpdir } from 'node:os';
import { join } from 'node:path';

/**
 * Cross-platform process termination
 * Handles Windows (taskkill) and Unix (signals) differently
 */
export function terminateProcess(pid: number, force = false): Promise<void> {
  return new Promise((resolve, reject) => {
    if (platform() === 'win32') {
      // Windows: use taskkill command
      const args = force ? ['/F', '/PID', pid.toString()] : ['/PID', pid.toString()];
      const proc = spawn('taskkill', args, {
        detached: true,
        stdio: 'ignore',
      });

      proc.on('error', (err) => {
        // If taskkill fails, try process.kill as fallback
        try {
          process.kill(pid, 0); // Check if process exists
          process.kill(pid, 9); // Force kill with signal 9
          resolve();
        } catch (fallbackErr) {
          reject(err);
        }
      });

      proc.on('exit', (code) => {
        if (code === 0 || code === 128) {
          // 128 = process not found, which is OK
          resolve();
        } else {
          reject(new Error(`taskkill exited with code ${code}`));
        }
      });
    } else {
      // Unix/Linux/macOS: use signals
      try {
        process.kill(pid, force ? 'SIGKILL' : 'SIGTERM');
        resolve();
      } catch (err) {
        if (
          err instanceof Error &&
          'code' in err &&
          (err as NodeJS.ErrnoException).code === 'ESRCH'
        ) {
          // Process doesn't exist, consider it success
          resolve();
        } else {
          reject(err);
        }
      }
    }
  });
}

/**
 * Check if a process is running (cross-platform)
 */
export function isProcessRunning(pid: number): boolean {
  try {
    // This works on all platforms
    process.kill(pid, 0);
    return true;
  } catch (error) {
    return false;
  }
}

/**
 * Get platform-specific paths for LSP servers
 */
export function getLSPServerPaths(serverName: string): string[] {
  const paths: string[] = [];
  const home = homedir();
  const plat = platform();

  // Add common npm global paths
  if (plat === 'win32') {
    paths.push(
      join(home, 'AppData', 'Roaming', 'npm', serverName),
      join(home, 'AppData', 'Roaming', 'npm', `${serverName}.cmd`),
      join('C:', 'Program Files', 'nodejs', serverName),
      join('C:', 'tools', serverName)
    );
  } else if (plat === 'darwin') {
    paths.push(
      join(home, '.npm-global', 'bin', serverName),
      join('/usr', 'local', 'bin', serverName),
      join('/opt', 'homebrew', 'bin', serverName),
      join(home, 'Library', 'Application Support', serverName)
    );
  } else {
    // Linux and other Unix-like
    paths.push(
      join(home, '.local', 'bin', serverName),
      join('/usr', 'local', 'bin', serverName),
      join('/usr', 'bin', serverName),
      join('/opt', serverName, 'bin', serverName)
    );
  }

  // Language-specific paths
  switch (serverName) {
    case 'gopls':
      if (plat === 'win32') {
        paths.push(join(home, 'go', 'bin', 'gopls.exe'), join('C:', 'Go', 'bin', 'gopls.exe'));
      } else {
        paths.push(join(home, 'go', 'bin', 'gopls'), join('/usr', 'local', 'go', 'bin', 'gopls'));
      }
      break;

    case 'rust-analyzer':
      if (plat === 'win32') {
        paths.push(
          join(home, '.cargo', 'bin', 'rust-analyzer.exe'),
          join('C:', 'Users', process.env.USERNAME || '', '.cargo', 'bin', 'rust-analyzer.exe')
        );
      } else {
        paths.push(
          join(home, '.cargo', 'bin', 'rust-analyzer'),
          join(home, '.rustup', 'toolchains', 'stable-*', 'bin', 'rust-analyzer')
        );
      }
      break;
  }

  return paths;
}
