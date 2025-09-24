// Main exports for library usage
export { MCPProxy, type ProxyOptions, type MCPToolCall, type MCPToolResponse } from './mcp-proxy.js';
export { WebSocketClient, type WebSocketClientOptions, type ConnectionStatus } from './websocket.js';
export { 
  loadConfig, 
  saveConfig, 
  getConfig,
  saveProfile,
  setCurrentProfile,
  listProfiles,
  deleteProfile,
  type ClientConfig,
  type ProfileConfig 
} from './config.js';
export { createProxyServer } from './http-proxy.js';