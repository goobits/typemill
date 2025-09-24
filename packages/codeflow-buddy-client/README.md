# Codeflow Buddy Client

A comprehensive WebSocket client library and CLI for interacting with the Codeflow Buddy MCP server.

## Features

- **WebSocket Client**: Robust WebSocket client with automatic reconnection, authentication, and request/response handling
- **MCP Proxy**: High-level abstraction for programmatic server interaction
- **CLI Tool**: Interactive and non-interactive command-line interface
- **HTTP Proxy**: Bridge HTTP requests to WebSocket server
- **Profile Management**: Save and manage multiple server configurations
- **TypeScript Support**: Full TypeScript definitions included

## Installation

```bash
# Install globally for CLI usage
npm install -g @goobits/codeflow-buddy-client

# Or install as a library
npm install @goobits/codeflow-buddy-client
```

## CLI Usage

### Configure Connection

```bash
# Interactive configuration
codeflow-client configure

# Create a named profile
codeflow-client configure --profile production
```

### Send Tool Requests

```bash
# Non-interactive with JSON parameters
codeflow-client send find_definition '{"file_path": "src/index.ts", "symbol_name": "main"}'

# Interactive mode (prompts for parameters)
codeflow-client send find_definition -i

# Use specific profile
codeflow-client send --profile production tools/list
```

### Start HTTP Proxy

```bash
# Start proxy server on port 3001
codeflow-client proxy

# Custom port
codeflow-client proxy --port 8080

# Then use with curl or any HTTP client
curl -X POST http://localhost:3001/rpc \
  -H "Content-Type: application/json" \
  -d '{"method": "find_definition", "params": {...}}'
```

### Profile Management

```bash
# List all profiles
codeflow-client profile list

# Switch active profile
codeflow-client profile use production

# Delete profile
codeflow-client profile delete staging
```

### Test Connection

```bash
codeflow-client test
```

## Library Usage

### Basic Example

```typescript
import { MCPProxy } from '@goobits/codeflow-buddy-client';

// Create proxy instance
const proxy = new MCPProxy('ws://localhost:3000', {
  token: 'your-jwt-token',
});

// Send a tool request
const result = await proxy.send({
  method: 'find_definition',
  params: {
    file_path: 'src/index.ts',
    symbol_name: 'main',
  },
});

// Send multiple requests in batch
const results = await proxy.sendBatch([
  { method: 'find_definition', params: {...} },
  { method: 'find_references', params: {...} },
]);

// Disconnect when done
await proxy.disconnect();
```

### Advanced WebSocket Client

```typescript
import { WebSocketClient } from '@goobits/codeflow-buddy-client';

const client = new WebSocketClient('ws://localhost:3000', {
  token: 'your-jwt-token',
  reconnect: true,
  reconnectMaxRetries: 10,
  requestTimeout: 30000,
});

// Listen to events
client.on('connected', () => console.log('Connected'));
client.on('disconnected', (info) => console.log('Disconnected', info));
client.on('reconnecting', ({ attempt, delay }) => 
  console.log(`Reconnecting attempt ${attempt} in ${delay}ms`)
);
client.on('notification', (notif) => 
  console.log('Server notification:', notif)
);

// Connect and send requests
await client.connect();
const result = await client.send('find_definition', { ... });
await client.disconnect();
```

### HTTP Proxy Server

```typescript
import { MCPProxy, createProxyServer } from '@goobits/codeflow-buddy-client';

const proxy = new MCPProxy('ws://localhost:3000');
const server = createProxyServer(proxy, 3001);

server.listen(3001, () => {
  console.log('HTTP proxy running on http://localhost:3001');
});
```

### Configuration Management

```typescript
import { 
  loadConfig, 
  saveConfig, 
  saveProfile,
  setCurrentProfile 
} from '@goobits/codeflow-buddy-client';

// Save configuration
await saveConfig({
  url: 'ws://localhost:3000',
  token: 'your-token',
});

// Save named profile
await saveProfile('production', {
  url: 'wss://api.example.com',
  token: 'prod-token',
  description: 'Production server',
});

// Set active profile
await setCurrentProfile('production');
```

## Configuration File

Configuration is stored in `~/.codeflow-buddy/config.json`:

```json
{
  "url": "ws://localhost:3000",
  "token": "default-token",
  "profiles": {
    "local": {
      "url": "ws://localhost:3000",
      "description": "Local development server"
    },
    "production": {
      "url": "wss://api.example.com",
      "token": "prod-token",
      "description": "Production server"
    }
  },
  "currentProfile": "local"
}
```

## API Reference

### MCPProxy

- `constructor(url: string, options?: ProxyOptions)`
- `send<T>(call: MCPToolCall): Promise<T>`
- `sendBatch<T>(calls: MCPToolCall[]): Promise<MCPToolResponse<T>[]>`
- `listTools(): Promise<any>`
- `connect(): Promise<void>`
- `disconnect(): Promise<void>`
- `isConnected(): boolean`
- `on(event: string, handler: Function): this`
- `off(event: string, handler: Function): this`

### WebSocketClient

- `constructor(url: string, options?: WebSocketClientOptions)`
- `connect(): Promise<void>`
- `disconnect(): Promise<void>`
- `send<T>(method: string, params?: unknown): Promise<T>`
- `isConnected(): boolean`
- Events: `connected`, `disconnected`, `reconnecting`, `error`, `notification`, `status`

## Environment Variables

The client respects these environment variables:

- `CODEFLOW_URL`: Default server URL
- `CODEFLOW_TOKEN`: Default authentication token
- `CODEFLOW_PROFILE`: Default profile to use

## License

MIT