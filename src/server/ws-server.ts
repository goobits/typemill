import type WebSocket from 'ws';
import { WebSocketServer } from 'ws';
import { createServer, type IncomingMessage, type ServerResponse } from 'node:http';
import { StreamingFileAccess } from '../fs/stream.js';
import { LSPClient } from '../lsp/client.js';
import { toolRegistry } from '../mcp/tool-registry.js';
import { WebSocketTransport } from '../transports/websocket.js';
import type { ClientSession, MCPMessage } from '../transports/websocket.js';
import { LSPServerPool } from './lsp-pool.js';
import { SessionManager } from './session.js';
import { logger } from '../core/logger.js';
import { getPackageVersion } from '../utils/version.js';

// Import handlers to ensure they register themselves
import '../mcp/handlers/core-handlers.js';
import '../mcp/handlers/batch-handlers.js';
import '../mcp/handlers/advanced-handlers.js';
import '../mcp/handlers/hierarchy-handlers.js';
import '../mcp/handlers/intelligence-handlers.js';
import '../mcp/handlers/utility-handlers.js';

export interface WebSocketServerOptions {
  port: number;
  maxClients?: number;
}

export class CodeFlowWebSocketServer {
  private wss: WebSocketServer;
  private httpServer: ReturnType<typeof createServer>;
  private transport: WebSocketTransport;
  private sessionManager: SessionManager;
  private lspServerPool: LSPServerPool;
  private streamingFS: StreamingFileAccess;
  private lspClient: LSPClient;
  private clientCount = 0;
  private startTime = Date.now();

  constructor(private options: WebSocketServerOptions) {
    this.sessionManager = new SessionManager();

    // Initialize LSP client (will load configuration automatically)
    this.lspClient = new LSPClient();

    this.lspServerPool = new LSPServerPool(this.lspClient);

    this.transport = new WebSocketTransport(
      this.handleMCPMessage.bind(this),
      this.handleSessionReconnect.bind(this),
      this.handleSessionDisconnect.bind(this)
    );
    this.streamingFS = new StreamingFileAccess(this.transport);

    // Create HTTP server for health checks and WebSocket upgrade
    this.httpServer = createServer(this.handleHttpRequest.bind(this));

    this.wss = new WebSocketServer({
      server: this.httpServer,
      clientTracking: true,
    });

    this.setupServer();
  }

  private setupServer(): void {
    this.wss.on('connection', (ws: WebSocket, req) => {
      this.clientCount++;

      logger.logConnection('connect', {
        component: 'WebSocketServer',
        clientCount: this.clientCount,
        remoteAddress: req.socket.remoteAddress,
        userAgent: req.headers['user-agent']
      });

      // Check client limit
      if (this.options.maxClients && this.clientCount > this.options.maxClients) {
        logger.warn('Server at capacity, rejecting connection', {
          component: 'WebSocketServer',
          clientCount: this.clientCount,
          maxClients: this.options.maxClients
        });
        ws.close(1008, 'Server at capacity');
        this.clientCount--;
        return;
      }

      // Set up the WebSocket transport for this connection
      this.transport.handleConnection(ws);

      ws.on('close', () => {
        this.clientCount--;
        logger.logConnection('disconnect', {
          component: 'WebSocketServer',
          clientCount: this.clientCount
        });
      });
    });

    this.wss.on('error', (error) => {
      logger.error('WebSocket server error', error, {
        component: 'WebSocketServer'
      });
    });

    // Start HTTP server
    this.httpServer.listen(this.options.port, () => {
      logger.info('WebSocket server started', {
        component: 'WebSocketServer',
        port: this.options.port,
        maxClients: this.options.maxClients
      });
    });

    this.httpServer.on('error', (error) => {
      logger.error('HTTP server error', error, {
        component: 'HTTPServer'
      });
    });
  }

  private handleHttpRequest(req: IncomingMessage, res: ServerResponse): void {
    // Set CORS headers
    res.setHeader('Access-Control-Allow-Origin', '*');
    res.setHeader('Access-Control-Allow-Methods', 'GET, OPTIONS');
    res.setHeader('Access-Control-Allow-Headers', 'Content-Type');

    // Handle preflight requests
    if (req.method === 'OPTIONS') {
      res.writeHead(200);
      res.end();
      return;
    }

    // Health check endpoint
    if (req.url === '/healthz') {
      this.handleHealthCheck(req, res);
      return;
    }

    // Metrics endpoint
    if (req.url === '/metrics') {
      this.handleMetrics(req, res);
      return;
    }

    // 404 for all other routes
    res.writeHead(404, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify({ error: 'Not found' }));
  }

  private handleHealthCheck(req: IncomingMessage, res: ServerResponse): void {
    try {
      const sessionStats = this.sessionManager.getStats();
      const lspServers = this.lspServerPool.getActiveServers();

      const healthData = {
        status: 'healthy',
        timestamp: new Date().toISOString(),
        version: getPackageVersion(),
        uptime: Math.floor((Date.now() - this.startTime) / 1000),
        connections: {
          active: sessionStats.activeSessions,
          disconnected: sessionStats.disconnectedSessions,
          total: this.clientCount
        },
        sessions: {
          active: sessionStats.activeSessions,
          disconnected: sessionStats.disconnectedSessions,
          projects: sessionStats.activeProjects,
          ...(sessionStats.oldestDisconnection && {
            oldestDisconnection: sessionStats.oldestDisconnection.toISOString()
          })
        },
        lspServers: {
          active: lspServers.length,
          crashed: lspServers.filter(s => s.refCount === 0).length,
          projects: [...new Set(lspServers.map(s => s.projectId))].length,
          languages: [...new Set(lspServers.map(s => s.language))].length
        },
        cache: {
          ...this.streamingFS.getCacheStats()
        }
      };

      res.writeHead(200, { 'Content-Type': 'application/json' });
      res.end(JSON.stringify(healthData, null, 2));

      logger.debug('Health check requested', {
        component: 'HTTPServer',
        userAgent: req.headers['user-agent'],
        remoteAddress: req.socket.remoteAddress
      });

    } catch (error) {
      logger.error('Health check failed', error as Error, {
        component: 'HTTPServer'
      });

      res.writeHead(503, { 'Content-Type': 'application/json' });
      res.end(JSON.stringify({
        status: 'unhealthy',
        timestamp: new Date().toISOString(),
        error: 'Internal server error'
      }));
    }
  }

  private handleMetrics(req: IncomingMessage, res: ServerResponse): void {
    try {
      const sessionStats = this.sessionManager.getStats();
      const lspServers = this.lspServerPool.getActiveServers();

      // Prometheus-style metrics
      const metrics = [
        `# HELP codeflow_connections_active Number of active WebSocket connections`,
        `# TYPE codeflow_connections_active gauge`,
        `codeflow_connections_active ${sessionStats.activeSessions}`,
        ``,
        `# HELP codeflow_connections_disconnected Number of disconnected sessions waiting for reconnection`,
        `# TYPE codeflow_connections_disconnected gauge`,
        `codeflow_connections_disconnected ${sessionStats.disconnectedSessions}`,
        ``,
        `# HELP codeflow_projects_active Number of active projects`,
        `# TYPE codeflow_projects_active gauge`,
        `codeflow_projects_active ${sessionStats.activeProjects}`,
        ``,
        `# HELP codeflow_lsp_servers_active Number of active LSP servers`,
        `# TYPE codeflow_lsp_servers_active gauge`,
        `codeflow_lsp_servers_active ${lspServers.length}`,
        ``,
        `# HELP codeflow_uptime_seconds Server uptime in seconds`,
        `# TYPE codeflow_uptime_seconds counter`,
        `codeflow_uptime_seconds ${Math.floor((Date.now() - this.startTime) / 1000)}`,
        ``
      ].join('\n');

      res.writeHead(200, { 'Content-Type': 'text/plain' });
      res.end(metrics);

    } catch (error) {
      logger.error('Metrics request failed', error as Error, {
        component: 'HTTPServer'
      });

      res.writeHead(500, { 'Content-Type': 'text/plain' });
      res.end('# Error generating metrics\n');
    }
  }

  private handleSessionReconnect(sessionId: string, socket: any): ClientSession | null {
    return this.sessionManager.reconnectSession(sessionId, socket);
  }

  private handleSessionDisconnect(sessionId: string): void {
    this.sessionManager.handleDisconnection(sessionId, (expiredSession) => {
      // Clean up cache for expired session
      this.streamingFS.cleanupSession(sessionId);

      logger.info('Session expired and cleaned up', {
        component: 'WebSocketServer',
        sessionId,
        projectId: expiredSession.projectId
      });
    });
  }

  private async handleMCPMessage(session: ClientSession, message: MCPMessage): Promise<any> {
    const startTime = Date.now();

    return logger.withTiming(
      `MCP ${message.method}`,
      async () => {
        // Add session to manager if not already tracked
        if (!this.sessionManager.getSession(session.id)) {
          this.sessionManager.addSession(session);
        }

        // Handle file change notifications
        if (message.method === 'server/fileChanged') {
          // Convert client absolute path to project-relative path
          const notification = {
            ...message.params,
            path: this.streamingFS.toProjectPath(session, message.params.path)
          };
          this.streamingFS.handleFileChanged(session, notification);
          return {};
        }

        // Look up the tool handler
        const toolInfo = toolRegistry.get(message.method);
        if (!toolInfo) {
          throw new Error(`Unknown tool: ${message.method}`);
        }

      // Prepare services for the handler
      const services: any = {};

      if (toolInfo.requiresService === 'symbol' || toolInfo.requiresService === 'file') {
        // For LSP-based tools, we need the appropriate LSP server
        const clientFilePath = message.params.file_path;
        if (clientFilePath && typeof clientFilePath === 'string') {
          // Convert client absolute path to project-relative path
          const projectPath = this.streamingFS.toProjectPath(session, clientFilePath);
          const extension = this.getFileExtension(projectPath);
          const server = await this.lspServerPool.getServer(session.projectId, extension);
          services.lspClient = this.lspClient;
          services.server = server;

          // Update the message params to use project-relative path for LSP operations
          message.params = {
            ...message.params,
            file_path: projectPath
          };
        }
      }

      if (toolInfo.requiresService === 'file') {
        // For file-based tools, provide streaming file access
        services.fileAccess = this.streamingFS;
        services.session = session;
      }

      if (toolInfo.requiresService === 'batch') {
        // For batch tools, provide the tool registry and other services
        services.toolRegistry = toolRegistry;
        services.lspClient = this.lspClient;
        services.lspServerPool = this.lspServerPool;
        services.fileAccess = this.streamingFS;
        services.session = session;
      }

      // Execute the tool handler
      const result = await toolInfo.handler(message.params, services);

      // Release server reference if we used one
      if (services.server) {
        const filePath = message.params.file_path;
        if (filePath && typeof filePath === 'string') {
          const extension = this.getFileExtension(filePath);
          this.lspServerPool.releaseServer(session.projectId, extension);
        }
      }

      return result;
      },
      {
        component: 'WebSocketServer',
        sessionId: session.id,
        projectId: session.projectId,
        method: message.method
      }
    );
  }

  private getFileExtension(filePath: string): string {
    const parts = filePath.split('.');
    if (parts.length > 1) {
      return parts[parts.length - 1]!;
    }
    return '';
  }

  getServerStats(): {
    clientCount: number;
    activeProjects: string[];
    activeServers: Array<{ projectId: string; language: string; refCount: number; lastUsed: Date }>;
  } {
    return {
      clientCount: this.clientCount,
      activeProjects: this.sessionManager.getActiveProjects(),
      activeServers: this.lspServerPool.getActiveServers(),
    };
  }

  async shutdown(): Promise<void> {
    logger.info('Starting server shutdown', { component: 'WebSocketServer' });

    // Close all client connections
    this.wss.clients.forEach((ws) => {
      ws.close(1001, 'Server shutdown');
    });

    // Stop the WebSocket server
    await new Promise<void>((resolve) => {
      this.wss.close(() => resolve());
    });

    // Stop the HTTP server
    await new Promise<void>((resolve) => {
      this.httpServer.close(() => resolve());
    });

    // Shutdown LSP server pool
    await this.lspServerPool.shutdown();

    // Shutdown session manager
    this.sessionManager.shutdown();

    logger.info('Server shutdown complete', { component: 'WebSocketServer' });
  }
}
