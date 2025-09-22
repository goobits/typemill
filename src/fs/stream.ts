import type { ClientSession } from '../transports/websocket.js';
import type { WebSocketTransport } from '../transports/websocket.js';
import { FileCache } from '../core/cache.js';
import { logger } from '../core/logger.js';

export interface FileReadRequest {
  path: string;
}

export interface FileReadResponse {
  content: string;
  mtime: number;
}

export interface FileChangedNotification {
  path: string;
  changeType: 'created' | 'changed' | 'deleted';
}

export class StreamingFileAccess {
  private fileCache = new FileCache();

  constructor(private transport: WebSocketTransport) {}

  async readFile(session: ClientSession, path: string): Promise<string> {
    try {
      // Check cache first
      const cached = this.fileCache.getFile(session.id, path);
      if (cached) {
        logger.debug('File read cache hit', {
          component: 'StreamingFileAccess',
          sessionId: session.id,
          projectId: session.projectId,
          path,
          mtime: cached.mtime
        });
        return cached.content;
      }

      // Cache miss - fetch from client
      logger.debug('File read cache miss', {
        component: 'StreamingFileAccess',
        sessionId: session.id,
        projectId: session.projectId,
        path
      });

      const response = (await this.transport.sendRequest(session, 'client/readFile', {
        path,
      } as FileReadRequest)) as FileReadResponse;

      // Cache the result
      this.fileCache.setFile(session.id, path, response.content, response.mtime);

      logger.debug('File read completed and cached', {
        component: 'StreamingFileAccess',
        sessionId: session.id,
        projectId: session.projectId,
        path,
        contentLength: response.content.length,
        mtime: response.mtime
      });

      return response.content;
    } catch (error) {
      logger.error('File read failed', error as Error, {
        component: 'StreamingFileAccess',
        sessionId: session.id,
        projectId: session.projectId,
        path
      });

      throw new Error(
        `Failed to read file ${path}: ${error instanceof Error ? error.message : 'Unknown error'}`
      );
    }
  }

  async writeFile(session: ClientSession, path: string, content: string): Promise<void> {
    try {
      await this.transport.sendRequest(session, 'client/writeFile', { path, content });

      // Invalidate cache for this file since it was modified
      this.fileCache.invalidateFile(session.id, path);

      logger.debug('File write completed, cache invalidated', {
        component: 'StreamingFileAccess',
        sessionId: session.id,
        projectId: session.projectId,
        path,
        contentLength: content.length
      });

    } catch (error) {
      logger.error('File write failed', error as Error, {
        component: 'StreamingFileAccess',
        sessionId: session.id,
        projectId: session.projectId,
        path
      });

      throw new Error(
        `Failed to write file ${path}: ${error instanceof Error ? error.message : 'Unknown error'}`
      );
    }
  }

  async fileExists(session: ClientSession, path: string): Promise<boolean> {
    try {
      const response = await this.transport.sendRequest(session, 'client/fileExists', { path });
      return response.exists;
    } catch (error) {
      // If the request fails, assume file doesn't exist
      return false;
    }
  }

  async listDirectory(session: ClientSession, path: string): Promise<string[]> {
    try {
      const response = await this.transport.sendRequest(session, 'client/listDirectory', { path });
      return response.files || [];
    } catch (error) {
      throw new Error(
        `Failed to list directory ${path}: ${error instanceof Error ? error.message : 'Unknown error'}`
      );
    }
  }

  async getFileStats(
    session: ClientSession,
    path: string
  ): Promise<{ size: number; mtime: Date; isDirectory: boolean }> {
    try {
      const response = await this.transport.sendRequest(session, 'client/getFileStats', { path });
      return {
        size: response.size,
        mtime: new Date(response.mtime),
        isDirectory: response.isDirectory,
      };
    } catch (error) {
      throw new Error(
        `Failed to get file stats for ${path}: ${error instanceof Error ? error.message : 'Unknown error'}`
      );
    }
  }

  // Convert absolute client path to relative path within project
  toProjectPath(session: ClientSession, clientPath: string): string {
    if (clientPath.startsWith(session.projectRoot)) {
      return clientPath.slice(session.projectRoot.length).replace(/^\/+/, '');
    }
    return clientPath;
  }

  // Convert relative project path to absolute client path
  toClientPath(session: ClientSession, projectPath: string): string {
    const cleanPath = projectPath.replace(/^\/+/, '');
    return `${session.projectRoot}/${cleanPath}`.replace(/\/+/g, '/');
  }

  // Handle file change notifications from client
  handleFileChanged(session: ClientSession, notification: FileChangedNotification): void {
    // Invalidate cache for changed/deleted files
    if (notification.changeType === 'changed' || notification.changeType === 'deleted') {
      const wasInvalidated = this.fileCache.invalidateFile(session.id, notification.path);

      logger.debug('File change notification processed', {
        component: 'StreamingFileAccess',
        sessionId: session.id,
        projectId: session.projectId,
        path: notification.path,
        changeType: notification.changeType,
        cacheInvalidated: wasInvalidated
      });
    } else {
      logger.debug('File change notification processed', {
        component: 'StreamingFileAccess',
        sessionId: session.id,
        projectId: session.projectId,
        path: notification.path,
        changeType: notification.changeType
      });
    }
  }

  // Clean up cache for a disconnected session
  cleanupSession(sessionId: string): void {
    const deletedCount = this.fileCache.invalidateSession(sessionId);

    logger.debug('Session cache cleanup completed', {
      component: 'StreamingFileAccess',
      sessionId,
      deletedEntries: deletedCount
    });
  }

  // Get cache statistics for monitoring
  getCacheStats(): {
    size: number;
    hitRate?: number;
    entries: Array<{
      key: string;
      age: number;
      ttl: number;
      isExpired: boolean;
    }>;
  } {
    return this.fileCache.getStats();
  }

  // Clean up resources
  dispose(): void {
    this.fileCache.dispose();
  }
}
