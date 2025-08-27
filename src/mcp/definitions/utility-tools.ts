// Utility MCP tool definitions for diagnostics and server management

export const utilityToolDefinitions = [
  {
    name: 'get_diagnostics',
    description:
      'Get language diagnostics (errors, warnings, hints) for a file. Uses LSP textDocument/diagnostic to pull current diagnostics.',
    inputSchema: {
      type: 'object',
      properties: {
        file_path: {
          type: 'string',
          description: 'The path to the file to get diagnostics for',
        },
      },
      required: ['file_path'],
    },
  },
  {
    name: 'restart_server',
    description:
      'Manually restart LSP servers. Can restart servers for specific file extensions or all running servers.',
    inputSchema: {
      type: 'object',
      properties: {
        extensions: {
          type: 'array',
          items: { type: 'string' },
          description:
            'Array of file extensions to restart servers for (e.g., ["ts", "tsx"]). If not provided, all servers will be restarted.',
        },
      },
    },
  },
  {
    name: 'rename_file',
    description:
      'Rename or move a file and automatically update all import statements that reference it. Works with TypeScript, JavaScript, JSX, and TSX files.',
    inputSchema: {
      type: 'object',
      properties: {
        old_path: {
          type: 'string',
          description: 'Current path to the file',
        },
        new_path: {
          type: 'string',
          description: 'New path for the file (can be in a different directory)',
        },
        dry_run: {
          type: 'boolean',
          description: 'Preview changes without applying them (default: false)',
          default: false,
        },
      },
      required: ['old_path', 'new_path'],
    },
  },
] as const;