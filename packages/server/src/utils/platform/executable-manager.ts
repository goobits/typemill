/**
 * Cross-platform executable finding and management
 * Phase 1: Foundation layer - interface and stub implementations only
 */

import { getPlatformInfo } from './platform-detector.js';

/**
 * Information about a found executable
 */
export interface ExecutableInfo {
  path: string;
  exists: boolean;
  version?: string;
}

/**
 * Executable manager interface
 */
export interface ExecutableManager {
  /**
   * Find an executable in the system PATH
   */
  find(executable: string): Promise<ExecutableInfo>;

  /**
   * Check if an executable exists and is accessible
   */
  exists(executable: string): Promise<boolean>;

  /**
   * Get version information for an executable
   */
  getVersion(executable: string): Promise<string | null>;

  /**
   * Get installation suggestions for a missing executable
   */
  getInstallationSuggestions(executable: string): string[];
}

/**
 * Stub implementation for Phase 1
 * Will be replaced with real implementation in Phase 4
 */
class ExecutableManagerStub implements ExecutableManager {
  async find(executable: string): Promise<ExecutableInfo> {
    // Stub implementation - always returns not found
    return {
      path: '',
      exists: false,
    };
  }

  async exists(executable: string): Promise<boolean> {
    // Stub implementation - always returns false
    return false;
  }

  async getVersion(executable: string): Promise<string | null> {
    // Stub implementation - always returns null
    return null;
  }

  getInstallationSuggestions(executable: string): string[] {
    const platform = getPlatformInfo();

    // Basic platform-specific suggestions (to be enhanced later)
    if (platform.isWindows) {
      return [`choco install ${executable}`, `winget install ${executable}`];
    } else if (platform.isMacOS) {
      return [`brew install ${executable}`];
    } else if (platform.isLinux) {
      return [`sudo apt-get install ${executable}`, `sudo yum install ${executable}`];
    }

    return [`Install ${executable} using your system's package manager`];
  }
}

// Export singleton instance
export const executableManager: ExecutableManager = new ExecutableManagerStub();

// Convenience exports
export const { find, exists, getVersion, getInstallationSuggestions } = executableManager;