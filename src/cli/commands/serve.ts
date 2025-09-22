import { CodeFlowWebSocketServer } from '../../server/ws-server.js';

export interface ServeOptions {
  port?: number;
  maxClients?: number;
}

export async function serveCommand(options: ServeOptions = {}): Promise<void> {
  const port = options.port || 3000;
  const maxClients = options.maxClients || 10;

  console.log(`Starting CodeFlow WebSocket server on port ${port}`);
  console.log(`Maximum clients: ${maxClients}`);

  const server = new CodeFlowWebSocketServer({
    port,
    maxClients,
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
