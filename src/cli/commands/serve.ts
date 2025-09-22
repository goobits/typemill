import { CodeFlowWebSocketServer, type TLSOptions } from '../../server/ws-server.js';

export interface ServeOptions {
  port?: number;
  maxClients?: number;
  requireAuth?: boolean;
  jwtSecret?: string;
  tlsKey?: string;
  tlsCert?: string;
  tlsCa?: string;
  enableFuse?: boolean;
  workspaceConfig?: {
    baseWorkspaceDir?: string;
    fuseMountPrefix?: string;
    maxWorkspaces?: number;
    workspaceTimeoutMs?: number;
  };
}

export async function serveCommand(options: ServeOptions = {}): Promise<void> {
  const port = options.port || 3000;
  const maxClients = options.maxClients || 10;
  const requireAuth = options.requireAuth || false;
  const jwtSecret = options.jwtSecret;
  const enableFuse = options.enableFuse || process.env.ENABLE_FUSE === 'true';

  // Configure TLS if both key and cert are provided
  let tls: TLSOptions | undefined;
  if (options.tlsKey && options.tlsCert) {
    tls = {
      keyPath: options.tlsKey,
      certPath: options.tlsCert,
      caPath: options.tlsCa,
    };
  }

  const protocol = tls ? 'WSS (Secure WebSocket)' : 'WS (WebSocket)';
  console.log(`Starting CodeFlow ${protocol} server on port ${port}`);
  console.log(`Maximum clients: ${maxClients}`);
  console.log(`Authentication: ${requireAuth ? 'Enabled' : 'Disabled'}`);
  console.log(`TLS/SSL: ${tls ? 'Enabled' : 'Disabled'}`);
  console.log(`FUSE Isolation: ${enableFuse ? 'Enabled' : 'Disabled'}`);

  if (tls) {
    console.log(`TLS Key: ${tls.keyPath}`);
    console.log(`TLS Certificate: ${tls.certPath}`);
    if (tls.caPath) {
      console.log(`CA Certificate: ${tls.caPath} (Client cert validation enabled)`);
    }
  }

  if (requireAuth && !jwtSecret) {
    console.log(
      'Warning: Authentication enabled but no JWT secret provided. Using auto-generated secret.'
    );
  }

  const server = new CodeFlowWebSocketServer({
    port,
    maxClients,
    requireAuth,
    jwtSecret,
    tls,
    enableFuse,
    workspaceConfig: options.workspaceConfig,
  });

  // Handle graceful shutdown
  const shutdown = async () => {
    console.log('\nShutting down server...');
    await server.shutdown();
    process.exit(0);
  };

  process.on('SIGINT', shutdown);
  process.on('SIGTERM', shutdown);

  // Keep the process alive
  process.on('exit', () => {
    console.log('Server process exiting');
  });

  // Log server stats periodically
  setInterval(() => {
    const stats = server.getServerStats();
    console.log(
      `Server Stats - Clients: ${stats.clientCount}, Projects: ${stats.activeProjects.length}, LSP Servers: ${stats.activeServers.length}`
    );
  }, 30000); // Every 30 seconds
}
