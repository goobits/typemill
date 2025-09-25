/**
 * Tool definitions for atomic refactoring operations
 * These are compatibility wrappers for legacy test support
 */

export const atomicRefactoringTools = [
  {
    name: 'analyze_refactor_impact',
    description: 'Analyze the impact of refactoring operations before execution',
    inputSchema: {
      type: 'object',
      properties: {
        operations: {
          type: 'array',
          description: 'List of refactoring operations to analyze',
          items: {
            type: 'object',
            properties: {
              type: {
                type: 'string',
                description: 'Type of operation (move_file, rename_symbol, etc)',
              },
              old_path: {
                type: 'string',
                description: 'Original file path',
              },
              new_path: {
                type: 'string',
                description: 'New file path',
              },
            },
            required: ['type', 'old_path', 'new_path'],
          },
        },
        include_recommendations: {
          type: 'boolean',
          description: 'Include recommendations in the analysis',
          default: false,
        },
      },
      required: ['operations'],
    },
  },
  {
    name: 'batch_move_files',
    description: 'Move multiple files atomically with automatic import updates',
    inputSchema: {
      type: 'object',
      properties: {
        moves: {
          type: 'array',
          description: 'List of file moves to perform',
          items: {
            type: 'object',
            properties: {
              old_path: {
                type: 'string',
                description: 'Current file path',
              },
              new_path: {
                type: 'string',
                description: 'Destination file path',
              },
            },
            required: ['old_path', 'new_path'],
          },
        },
        dry_run: {
          type: 'boolean',
          description: 'Preview changes without applying them',
          default: false,
        },
        strategy: {
          type: 'string',
          enum: ['safe', 'force'],
          description: 'Execution strategy (safe: atomic with rollback, force: best effort)',
          default: 'safe',
        },
      },
      required: ['moves'],
    },
  },
  {
    name: 'preview_batch_operation',
    description: 'Preview the effects of batch operations without executing them',
    inputSchema: {
      type: 'object',
      properties: {
        operations: {
          type: 'array',
          description: 'List of operations to preview',
          items: {
            type: 'object',
            properties: {
              type: {
                type: 'string',
                description: 'Type of operation',
              },
              old_path: {
                type: 'string',
                description: 'Original file path',
              },
              new_path: {
                type: 'string',
                description: 'New file path',
              },
            },
            required: ['type', 'old_path', 'new_path'],
          },
        },
      },
      required: ['operations'],
    },
  },
];