/**
 * Centralized debug logging utility for CCLSP
 *
 * Provides component-based debug logging that respects DEBUG environment variables.
 * Only logs when DEBUG or CCLSP_DEBUG environment variables are set.
 */

/**
 * Debug logger instance that writes to stderr in the same format as existing debug statements
 */
export class DebugLogger {
  private static instance: DebugLogger;
  private debugEnabled: boolean;

  private constructor() {
    // Enable debug if DEBUG or CCLSP_DEBUG environment variables are set
    this.debugEnabled = !!(process.env.DEBUG || process.env.CCLSP_DEBUG);
  }

  public static getInstance(): DebugLogger {
    if (!DebugLogger.instance) {
      DebugLogger.instance = new DebugLogger();
    }
    return DebugLogger.instance;
  }

  /**
   * Log a debug message with component context
   * @param component - Component name (e.g., 'LSP', 'MCP', 'SymbolService')
   * @param message - Debug message
   * @param data - Optional data to stringify and include
   */
  public log(component: string, message: string, data?: unknown): void {
    if (!this.debugEnabled) {
      return;
    }

    let logMessage = `[DEBUG ${component}] ${message}`;

    if (data !== undefined) {
      if (typeof data === 'object' && data !== null) {
        logMessage += ` ${JSON.stringify(data)}`;
      } else {
        logMessage += ` ${data}`;
      }
    }

    process.stderr.write(`${logMessage}\n`);
  }

  /**
   * Log a debug message without component prefix (for backward compatibility)
   * @param message - Debug message
   * @param data - Optional data to stringify and include
   */
  public logPlain(message: string, data?: unknown): void {
    if (!this.debugEnabled) {
      return;
    }

    let logMessage = message;

    if (data !== undefined) {
      if (typeof data === 'object' && data !== null) {
        logMessage += ` ${JSON.stringify(data)}`;
      } else {
        logMessage += ` ${data}`;
      }
    }

    process.stderr.write(`${logMessage}\n`);
  }

  /**
   * Check if debug logging is enabled
   */
  public isEnabled(): boolean {
    return this.debugEnabled;
  }

  /**
   * Enable/disable debug logging programmatically
   */
  public setEnabled(enabled: boolean): void {
    this.debugEnabled = enabled;
  }
}

// Export convenience function
export const debugLog = (component: string, message: string, data?: unknown) => {
  DebugLogger.getInstance().log(component, message, data);
};
