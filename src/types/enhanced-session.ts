/**
 * Enhanced session types for FUSE integration and project isolation
 */

import type { ClientSession } from '../transports/websocket.js';

export interface EnhancedClientSession extends ClientSession {
  globalProjectId: string; // Server-generated unique ID for true isolation
  workspaceId: string; // Isolated workspace identifier
  fuseMount?: string; // FUSE mount point path
  workspaceDir?: string; // Isolated working directory for LSP servers
}

export interface WorkspaceInfo {
  workspaceId: string;
  workspaceDir: string;
  fuseMount: string;
  sessionId: string;
  globalProjectId: string;
  createdAt: Date;
  lastAccessed: Date;
}

export interface FuseOperationRequest {
  method: 'fuse/read' | 'fuse/write' | 'fuse/stat' | 'fuse/readdir' | 'fuse/open' | 'fuse/release';
  path: string;
  flags?: number;
  data?: Buffer;
  correlationId: string;
}

export interface FuseOperationResponse {
  success: boolean;
  data?: Buffer | any;
  error?: string;
  errno?: number;
  correlationId: string;
}
