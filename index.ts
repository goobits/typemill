#!/usr/bin/env node

import { resolve } from 'node:path';
import { Server } from '@modelcontextprotocol/sdk/server/index.js';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';
import { CallToolRequestSchema, ListToolsRequestSchema } from '@modelcontextprotocol/sdk/types.js';
import { LSPClient } from './src/lsp-client.js';
import { allToolDefinitions } from './src/mcp/definitions/index.js';
import {
  handleFindDefinition,
  handleFindReferences,
  handleRenameSymbol,
  handleRenameSymbolStrict,
  handleGetCodeActions,
  handleFormatDocument,
  handleSearchWorkspaceSymbols,
  handleGetDocumentSymbols,
  handleGetDiagnostics,
  handleRestartServer,
  handleRenameFile,
  handleGetHover,
  handleGetCompletions,
  handleGetInlayHints,
  handleGetSemanticTokens,
  handlePrepareCallHierarchy,
  handleGetCallHierarchyIncomingCalls,
  handleGetCallHierarchyOutgoingCalls,
  handlePrepareTypeHierarchy,
  handleGetTypeHierarchySupertypes,
  handleGetTypeHierarchySubtypes,
  handleGetSelectionRange,
} from './src/mcp/handlers/index.js';
import { createMCPError } from './src/mcp/utils.js';

// Handle subcommands
const args = process.argv.slice(2);
if (args.length > 0) {
  const subcommand = args[0];

  if (subcommand === 'setup') {
    const { main } = await import('./src/setup.js');
    await main();
    process.exit(0);
  } else {
    console.error(`Unknown subcommand: ${subcommand}`);
    console.error('Available subcommands:');
    console.error('  setup    Configure cclsp for your project');
    console.error('');
    console.error('Run without arguments to start the MCP server.');
    process.exit(1);
  }
}

const lspClient = new LSPClient();

const server = new Server(
  {
    name: 'cclsp',
    version: '0.1.0',
  },
  {
    capabilities: {
      tools: {},
    },
  }
);

server.setRequestHandler(ListToolsRequestSchema, async () => {
  return {
    tools: allToolDefinitions,
  };
});

server.setRequestHandler(CallToolRequestSchema, async (request) => {
  const { name, arguments: args } = request.params;

  try {
    switch (name) {
      case 'find_definition':
        return await handleFindDefinition(lspClient, args as any);
      case 'find_references':
        return await handleFindReferences(lspClient, args as any);
      case 'rename_symbol':
        return await handleRenameSymbol(lspClient, args as any);
      case 'rename_symbol_strict':
        return await handleRenameSymbolStrict(lspClient, args as any);
      case 'get_code_actions':
        return await handleGetCodeActions(lspClient, args as any);
      case 'format_document':
        return await handleFormatDocument(lspClient, args as any);
      case 'search_workspace_symbols':
        return await handleSearchWorkspaceSymbols(lspClient, args as any);
      case 'get_document_symbols':
        return await handleGetDocumentSymbols(lspClient, args as any);
      case 'get_diagnostics':
        return await handleGetDiagnostics(lspClient, args as any);
      case 'restart_server':
        return await handleRestartServer(lspClient, args as any);
      case 'rename_file':
        return await handleRenameFile(lspClient, args as any);
      // Intelligence tools
      case 'get_hover':
        return await handleGetHover(lspClient, args as any);
      case 'get_completions':
        return await handleGetCompletions(lspClient, args as any);
      case 'get_inlay_hints':
        return await handleGetInlayHints(lspClient, args as any);
      case 'get_semantic_tokens':
        return await handleGetSemanticTokens(lspClient, args as any);
      // Hierarchy tools
      case 'prepare_call_hierarchy':
        return await handlePrepareCallHierarchy(lspClient, args as any);
      case 'get_call_hierarchy_incoming_calls':
        return await handleGetCallHierarchyIncomingCalls(lspClient, args as any);
      case 'get_call_hierarchy_outgoing_calls':
        return await handleGetCallHierarchyOutgoingCalls(lspClient, args as any);
      case 'prepare_type_hierarchy':
        return await handlePrepareTypeHierarchy(lspClient, args as any);
      case 'get_type_hierarchy_supertypes':
        return await handleGetTypeHierarchySupertypes(lspClient, args as any);
      case 'get_type_hierarchy_subtypes':
        return await handleGetTypeHierarchySubtypes(lspClient, args as any);
      case 'get_selection_range':
        return await handleGetSelectionRange(lspClient, args as any);
      default:
        throw new Error(`Unknown tool: ${name}`);
    }
  } catch (error) {
    return createMCPError(error);
  }
});

process.on('SIGINT', () => {
  lspClient.dispose();
  process.exit(0);
});

process.on('SIGTERM', () => {
  lspClient.dispose();
  process.exit(0);
});

async function main() {
  const transport = new StdioServerTransport();
  await server.connect(transport);
  process.stderr.write('CCLSP Server running on stdio\n');

  // Preload LSP servers for file types found in the project
  try {
    await lspClient.preloadServers();
  } catch (error) {
    process.stderr.write(`Failed to preload LSP servers: ${error}\n`);
  }
}

main().catch((error) => {
  process.stderr.write(`Server error: ${error}\n`);
  lspClient.dispose();
  process.exit(1);
});