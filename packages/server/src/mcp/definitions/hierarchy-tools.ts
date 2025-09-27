// MCP Tool Definitions for Hierarchy and Navigation Features

export const hierarchyToolDefinitions = [
  {
    name: 'prepare_call_hierarchy',
    description:
      'Prepare call hierarchy for a symbol. Gets the call hierarchy item that can be used to explore incoming and outgoing calls.',
    inputSchema: {
      type: 'object',
      properties: {
        file_path: {
          type: 'string',
          description: 'The path to the file',
        },
        line: {
          type: 'number',
          description: 'The line number (1-indexed)',
        },
        character: {
          type: 'number',
          description: 'The character position in the line (0-indexed)',
        },
      },
      required: ['file_path', 'line', 'character'],
    },
  },
  {
    name: 'get_call_hierarchy_incoming_calls',
    description:
      'Get all incoming calls to a function/method. Shows where this function is called from throughout the codebase. Provide EITHER: 1) an "item" from prepare_call_hierarchy, OR 2) "file_path", "line", and "character".',
    inputSchema: {
      type: 'object',
      properties: {
        item: {
          type: 'object',
          description:
            'The call hierarchy item (from prepare_call_hierarchy) - use this OR file_path/line/character',
          properties: {
            name: { type: 'string' },
            kind: { type: 'number' },
            uri: { type: 'string' },
            range: {
              type: 'object',
              properties: {
                start: {
                  type: 'object',
                  properties: {
                    line: { type: 'number' },
                    character: { type: 'number' },
                  },
                  required: ['line', 'character'],
                },
                end: {
                  type: 'object',
                  properties: {
                    line: { type: 'number' },
                    character: { type: 'number' },
                  },
                  required: ['line', 'character'],
                },
              },
              required: ['start', 'end'],
            },
            selectionRange: {
              type: 'object',
              properties: {
                start: {
                  type: 'object',
                  properties: {
                    line: { type: 'number' },
                    character: { type: 'number' },
                  },
                  required: ['line', 'character'],
                },
                end: {
                  type: 'object',
                  properties: {
                    line: { type: 'number' },
                    character: { type: 'number' },
                  },
                  required: ['line', 'character'],
                },
              },
              required: ['start', 'end'],
            },
          },
          required: ['name', 'kind', 'uri', 'range', 'selectionRange'],
        },
        file_path: {
          type: 'string',
          description: 'The path to the file - use with line and character',
        },
        line: {
          type: 'number',
          description: 'The line number (1-indexed) - use with file_path and character',
        },
        character: {
          type: 'number',
          description: 'The character position (0-indexed) - use with file_path and line',
        },
      },
      required: [],
    },
  },
  {
    name: 'get_call_hierarchy_outgoing_calls',
    description:
      'Get all outgoing calls from a function/method. Shows what functions this function calls. Provide EITHER: 1) an "item" from prepare_call_hierarchy, OR 2) "file_path", "line", and "character".',
    inputSchema: {
      type: 'object',
      properties: {
        item: {
          type: 'object',
          description:
            'The call hierarchy item (from prepare_call_hierarchy) - use this OR file_path/line/character',
          properties: {
            name: { type: 'string' },
            kind: { type: 'number' },
            uri: { type: 'string' },
            range: {
              type: 'object',
              properties: {
                start: {
                  type: 'object',
                  properties: {
                    line: { type: 'number' },
                    character: { type: 'number' },
                  },
                  required: ['line', 'character'],
                },
                end: {
                  type: 'object',
                  properties: {
                    line: { type: 'number' },
                    character: { type: 'number' },
                  },
                  required: ['line', 'character'],
                },
              },
              required: ['start', 'end'],
            },
            selectionRange: {
              type: 'object',
              properties: {
                start: {
                  type: 'object',
                  properties: {
                    line: { type: 'number' },
                    character: { type: 'number' },
                  },
                  required: ['line', 'character'],
                },
                end: {
                  type: 'object',
                  properties: {
                    line: { type: 'number' },
                    character: { type: 'number' },
                  },
                  required: ['line', 'character'],
                },
              },
              required: ['start', 'end'],
            },
          },
          required: ['name', 'kind', 'uri', 'range', 'selectionRange'],
        },
        file_path: {
          type: 'string',
          description: 'The path to the file - use with line and character',
        },
        line: {
          type: 'number',
          description: 'The line number (1-indexed) - use with file_path and character',
        },
        character: {
          type: 'number',
          description: 'The character position (0-indexed) - use with file_path and line',
        },
      },
      required: [],
    },
  },
];
