/**
 * Analysis tool definitions for MCP
 * Phase 3: Advanced code analysis features
 */

import type { Tool } from '@modelcontextprotocol/sdk/types.js';

export const ANALYSIS_TOOLS: Tool[] = [
  {
    name: 'find_dead_code',
    description: 'Find potentially dead (unused) code in the codebase using MCP tools',
    inputSchema: {
      type: 'object',
      properties: {
        files: {
          type: 'array',
          items: { type: 'string' },
          description: 'Specific files to analyze (optional, defaults to common source files)',
        },
        exclude_tests: {
          type: 'boolean',
          description: 'Whether to exclude test files from analysis (default: true)',
          default: true,
        },
        min_references: {
          type: 'number',
          description:
            'Minimum number of references required to not be considered dead (default: 1)',
          default: 1,
        },
      },
      additionalProperties: false,
    },
  },
];
