/**
 * FUSE filesystem mount for exposing client filesystem to LSP servers
 * Provides native filesystem access through FUSE with WebSocket backend
 */

import Fuse from 'fuse-native';
import type { EnhancedClientSession } from '../types/enhanced-session.js';
import type { WebSocketTransport } from '../transports/websocket.js';
import { FuseOperations } from './fuse-operations.js';
import { logger } from '../core/logger.js';

export interface FuseMountConfig {
  mountOptions?: string[];
  debugFuse?: boolean;
  allowOther?: boolean;
  allowRoot?: boolean;
  defaultPermissions?: boolean;
}

export class FuseMount {
  private session: EnhancedClientSession;
  private transport: WebSocketTransport;
  private operations: FuseOperations;
  private fuse?: Fuse;
  private mountPath: string;
  private mounted = false;
  private config: FuseMountConfig;

  constructor(
    session: EnhancedClientSession,
    transport: WebSocketTransport,
    mountPath: string,
    config: FuseMountConfig = {}
  ) {
    this.session = session;
    this.transport = transport;
    this.mountPath = mountPath;
    this.config = config;
    this.operations = new FuseOperations(session, transport);
  }

  /**
   * Mount the FUSE filesystem
   */
  async mount(): Promise<void> {
    if (this.mounted) {
      throw new Error(`FUSE filesystem already mounted at ${this.mountPath}`);
    }

    try {
      logger.info('Mounting FUSE filesystem', {
        component: 'FuseMount',
        sessionId: this.session.id,
        mountPath: this.mountPath,
        config: this.config
      });

      // Build mount options
      const options = this.buildMountOptions();

      // Create FUSE instance with operations
      this.fuse = new Fuse(this.mountPath, {
        readdir: this.operations.readdir.bind(this.operations),
        getattr: this.operations.getattr.bind(this.operations),
        open: this.operations.open.bind(this.operations),
        read: this.operations.read.bind(this.operations),
        write: this.operations.write.bind(this.operations),
        release: this.operations.release.bind(this.operations),
        truncate: this.operations.truncate.bind(this.operations),
        mkdir: this.operations.mkdir.bind(this.operations),
        rmdir: this.operations.rmdir.bind(this.operations),
        unlink: this.operations.unlink.bind(this.operations),
        rename: this.operations.rename.bind(this.operations)
      }, options);

      // Mount the filesystem
      await new Promise<void>((resolve, reject) => {
        this.fuse!.mount((error) => {
          if (error) {
            reject(new Error(`Failed to mount FUSE filesystem: ${error.message}`));
          } else {
            this.mounted = true;
            resolve();
          }
        });
      });

      logger.info('FUSE filesystem mounted successfully', {
        component: 'FuseMount',
        sessionId: this.session.id,
        mountPath: this.mountPath
      });
    } catch (error) {
      logger.error('Failed to mount FUSE filesystem', error as Error, {
        component: 'FuseMount',
        sessionId: this.session.id,
        mountPath: this.mountPath
      });
      throw error;
    }
  }

  /**
   * Unmount the FUSE filesystem
   */
  async unmount(): Promise<void> {
    if (!this.mounted || !this.fuse) {
      logger.warn('FUSE filesystem not mounted, skipping unmount', {
        component: 'FuseMount',
        sessionId: this.session.id,
        mountPath: this.mountPath
      });
      return;
    }

    try {
      logger.info('Unmounting FUSE filesystem', {
        component: 'FuseMount',
        sessionId: this.session.id,
        mountPath: this.mountPath
      });

      // Cleanup pending operations first
      this.operations.cleanup();

      // Unmount the filesystem
      await new Promise<void>((resolve, reject) => {
        this.fuse!.unmount((error) => {
          if (error) {
            logger.error('Error during FUSE unmount', error, {
              component: 'FuseMount',
              sessionId: this.session.id,
              mountPath: this.mountPath
            });
            // Don't reject - we want to continue cleanup
          }
          resolve();
        });
      });

      this.mounted = false;
      this.fuse = undefined;

      logger.info('FUSE filesystem unmounted successfully', {
        component: 'FuseMount',
        sessionId: this.session.id,
        mountPath: this.mountPath
      });
    } catch (error) {
      logger.error('Failed to unmount FUSE filesystem', error as Error, {
        component: 'FuseMount',
        sessionId: this.session.id,
        mountPath: this.mountPath
      });
      throw error;
    }
  }

  /**
   * Check if filesystem is mounted
   */
  isMounted(): boolean {
    return this.mounted;
  }

  /**
   * Get mount path
   */
  getMountPath(): string {
    return this.mountPath;
  }

  /**
   * Handle FUSE response from client
   */
  handleFuseResponse(response: any): void {
    this.operations.handleFuseResponse(response);
  }

  /**
   * Build mount options from config
   */
  private buildMountOptions(): object {
    const options: any = {
      debug: this.config.debugFuse || false
    };

    // Add FUSE-specific options
    if (this.config.allowOther) {
      options.allow_other = true;
    }

    if (this.config.allowRoot) {
      options.allow_root = true;
    }

    if (this.config.defaultPermissions) {
      options.default_permissions = true;
    }

    // Add custom mount options
    if (this.config.mountOptions) {
      for (const option of this.config.mountOptions) {
        const [key, value] = option.split('=');
        if (value !== undefined) {
          options[key] = value;
        } else {
          options[key] = true;
        }
      }
    }

    return options;
  }

  /**
   * Get filesystem statistics for monitoring
   */
  getStats(): {
    mounted: boolean;
    mountPath: string;
    sessionId: string;
    pendingOperations: number;
    openFiles: number;
  } {
    return {
      mounted: this.mounted,
      mountPath: this.mountPath,
      sessionId: this.session.id,
      pendingOperations: (this.operations as any).pendingOperations?.size || 0,
      openFiles: (this.operations as any).fileDescriptors?.size || 0
    };
  }

  /**
   * Force cleanup - used during emergency shutdown
   */
  async forceCleanup(): Promise<void> {
    try {
      logger.warn('Force cleaning up FUSE mount', {
        component: 'FuseMount',
        sessionId: this.session.id,
        mountPath: this.mountPath
      });

      // Cleanup operations first
      this.operations.cleanup();

      // Force unmount if still mounted
      if (this.mounted && this.fuse) {
        // Use system unmount as fallback
        const { execSync } = await import('node:child_process');
        try {
          execSync(`fusermount -u "${this.mountPath}"`, { timeout: 5000 });
        } catch (error) {
          // Try lazy unmount
          try {
            execSync(`fusermount -uz "${this.mountPath}"`, { timeout: 5000 });
          } catch (lazyError) {
            logger.error('Failed to force unmount FUSE filesystem', lazyError as Error, {
              component: 'FuseMount',
              sessionId: this.session.id,
              mountPath: this.mountPath
            });
          }
        }
      }

      this.mounted = false;
      this.fuse = undefined;
    } catch (error) {
      logger.error('Error during force cleanup', error as Error, {
        component: 'FuseMount',
        sessionId: this.session.id,
        mountPath: this.mountPath
      });
    }
  }
}