// Orchestration MCP tool definitions for complex multi-file operations
// These tools coordinate existing LSP capabilities for batch operations

export const orchestrationToolDefinitions = [
  {
    name: 'analyze_refactor_impact',
    description:
      'Analyze the impact of moving or renaming multiple files. Returns dependency analysis, risk assessment, and recommendations for safe refactoring. Uses existing LSP intelligence and project scanning.',
    inputSchema: {
      type: 'object',
      properties: {
        operations: {
          type: 'array',
          items: {
            type: 'object',
            properties: {
              type: {
                type: 'string',
                enum: ['move_file', 'rename_symbol'],
                description: 'Type of operation to analyze',
              },
              old_path: {
                type: 'string',
                description: 'Current file path (for move_file operations)',
              },
              new_path: {
                type: 'string',
                description: 'New file path (for move_file operations)',
              },
              file_path: {
                type: 'string',
                description: 'File containing symbol (for rename_symbol operations)',
              },
              symbol_name: {
                type: 'string',
                description: 'Symbol to rename (for rename_symbol operations)',
              },
              symbol_kind: {
                type: 'string',
                description: 'Optional symbol kind (for rename_symbol operations)',
              },
              new_name: {
                type: 'string',
                description: 'New symbol name (for rename_symbol operations)',
              },
            },
            required: ['type'],
          },
          description: 'List of refactoring operations to analyze',
        },
        include_recommendations: {
          type: 'boolean',
          description: 'Include safety recommendations and risk assessment (default: true)',
          default: true,
        },
      },
      required: ['operations'],
    },
  },
  {
    name: 'batch_move_files',
    description:
      'Move multiple files atomically with automatic import updates. Coordinates existing rename_file operations with rollback on failure. Safer than individual moves for complex refactoring.',
    inputSchema: {
      type: 'object',
      properties: {
        moves: {
          type: 'array',
          items: {
            type: 'object',
            properties: {
              old_path: {
                type: 'string',
                description: 'Current path to the file',
              },
              new_path: {
                type: 'string',
                description: 'New path for the file',
              },
            },
            required: ['old_path', 'new_path'],
          },
          description: 'List of file moves to execute atomically',
        },
        dry_run: {
          type: 'boolean',
          description: 'Preview all changes without applying them (default: false)',
          default: false,
        },
        strategy: {
          type: 'string',
          enum: ['safe', 'force'],
          description:
            'Execution strategy: safe (abort on any failure) or force (continue despite non-critical errors)',
          default: 'safe',
        },
      },
      required: ['moves'],
    },
  },
  {
    name: 'preview_batch_operation',
    description:
      'Preview the complete impact of a complex refactoring operation without applying any changes. Shows what files would be modified, what imports would be updated, and potential risks.',
    inputSchema: {
      type: 'object',
      properties: {
        operations: {
          type: 'array',
          items: {
            type: 'object',
            properties: {
              type: {
                type: 'string',
                enum: ['move_file', 'rename_symbol', 'rename_file'],
                description: 'Type of operation',
              },
              old_path: {
                type: 'string',
                description: 'Current file path (for file operations)',
              },
              new_path: {
                type: 'string',
                description: 'New file path (for file operations)',
              },
              file_path: {
                type: 'string',
                description: 'File containing symbol (for symbol operations)',
              },
              symbol_name: {
                type: 'string',
                description: 'Symbol name (for symbol operations)',
              },
              symbol_kind: {
                type: 'string',
                description: 'Optional symbol kind (for symbol operations)',
              },
              new_name: {
                type: 'string',
                description: 'New name (for rename operations)',
              },
            },
            required: ['type'],
          },
          description: 'List of operations to preview',
        },
        detailed: {
          type: 'boolean',
          description: 'Include detailed file-by-file change preview (default: false)',
          default: false,
        },
      },
      required: ['operations'],
    },
  },
] as const;
