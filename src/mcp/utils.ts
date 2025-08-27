// Shared MCP utilities

export interface MCPResponse {
  content: Array<{
    type: 'text';
    text: string;
  }>;
}

export function createMCPResponse(text: string): MCPResponse {
  return {
    content: [
      {
        type: 'text',
        text,
      },
    ],
  };
}

export function createMCPError(error: unknown): MCPResponse {
  const message = error instanceof Error ? error.message : String(error);
  return createMCPResponse(`Error: ${message}`);
}