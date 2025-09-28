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
  {
    name: 'create_file',
    description:
      'Create a new file with optional content and notify relevant LSP servers. Ensures proper LSP workspace synchronization for newly created files.',
    inputSchema: {
      type: 'object',
      properties: {
        file_path: {
          type: 'string',
          description: 'The path where the new file should be created',
        },
        content: {
          type: 'string',
          description: 'Initial content for the file (default: empty string)',
          default: '',
        },
        overwrite: {
          type: 'boolean',
          description: 'Whether to overwrite existing file if it exists (default: false)',
          default: false,
        },
      },
      required: ['file_path'],
    },
  },
  {
    name: 'delete_file',
    description:
      'Delete a file and notify relevant LSP servers. Ensures proper LSP workspace synchronization and cleanup for deleted files.',
    inputSchema: {
      type: 'object',
      properties: {
        file_path: {
          type: 'string',
          description: 'The path to the file to delete',
        },
        force: {
          type: 'boolean',
          description: 'Force deletion even if file has uncommitted changes (default: false)',
          default: false,
        },
      },
      required: ['file_path'],
    },
  },
  {
    name: 'health_check',
    description:
      'Get health status of the LSP servers and system resources. Returns information about active servers, resource usage, and system health.',
    inputSchema: {
      type: 'object',
      properties: {
        include_details: {
          type: 'boolean',
          description: 'Include detailed server information (default: false)',
          default: false,
        },
      },
    },
  },
  {
    name: 'update_dependencies',
    description:
      'Universal dependency management tool that works across multiple languages (Node.js, Python, Rust, Go). Auto-detects file type and applies appropriate dependency updates. More comprehensive than update_package_json.',
    inputSchema: {
      type: 'object',
      properties: {
        file_path: {
          type: 'string',
          description: 'Path to dependency file (package.json, requirements.txt, Cargo.toml, go.mod, pyproject.toml)',
        },
        add_dependencies: {
          type: 'object',
          description: 'Dependencies to add to the main dependencies section',
          additionalProperties: { type: 'string' },
        },
        add_dev_dependencies: {
          type: 'object',
          description: 'Dependencies to add to the development dependencies section',
          additionalProperties: { type: 'string' },
        },
        remove_dependencies: {
          type: 'array',
          items: { type: 'string' },
          description: 'Dependency names to remove from all dependency sections',
        },
        add_scripts: {
          type: 'object',
          description: 'Scripts to add (Node.js package.json only)',
          additionalProperties: { type: 'string' },
        },
        remove_scripts: {
          type: 'array',
          items: { type: 'string' },
          description: 'Script names to remove (Node.js package.json only)',
        },
        update_version: {
          type: 'string',
          description: 'Update the version field (works across all supported languages)',
        },
        workspace_config: {
          type: 'object',
          description: 'Workspace configuration (Node.js package.json only)',
          properties: {
            workspaces: {
              type: 'array',
              items: { type: 'string' },
              description: 'Array of workspace glob patterns',
            },
          },
        },
        dry_run: {
          type: 'boolean',
          description: 'Preview changes without applying them (default: false)',
          default: false,
        },
      },
      required: ['file_path'],
    },
  },
] as const;
