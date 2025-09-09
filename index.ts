#!/usr/bin/env node

import { resolve } from 'node:path';
import { Server } from '@modelcontextprotocol/sdk/server/index.js';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';
import { CallToolRequestSchema, ListToolsRequestSchema } from '@modelcontextprotocol/sdk/types.js';
import { LSPClient as NewLSPClient } from './src/lsp/client.js';
import { allToolDefinitions } from './src/mcp/definitions/index.js';
import type {
  ApplyWorkspaceEditArgs,
  CreateFileArgs,
  DeleteFileArgs,
  FindDefinitionArgs,
  FindReferencesArgs,
  FormatDocumentArgs,
  GetCallHierarchyIncomingCallsArgs,
  GetCallHierarchyOutgoingCallsArgs,
  GetCodeActionsArgs,
  GetCompletionsArgs,
  GetDiagnosticsArgs,
  GetDocumentLinksArgs,
  GetDocumentSymbolsArgs,
  GetFoldingRangesArgs,
  GetHoverArgs,
  GetInlayHintsArgs,
  GetSelectionRangeArgs,
  GetSemanticTokensArgs,
  GetSignatureHelpArgs,
  GetTypeHierarchySubtypesArgs,
  GetTypeHierarchySupertypesArgs,
  HealthCheckArgs,
  PrepareCallHierarchyArgs,
  PrepareTypeHierarchyArgs,
  RenameFileArgs,
  RenameSymbolArgs,
  RenameSymbolStrictArgs,
  RestartServerArgs,
  SearchWorkspaceSymbolsArgs,
} from './src/mcp/handler-types.js';
import {
  handleApplyWorkspaceEdit,
  handleCreateFile,
  handleDeleteFile,
  handleFindDefinition,
  handleFindReferences,
  handleFormatDocument,
  handleGetCallHierarchyIncomingCalls,
  handleGetCallHierarchyOutgoingCalls,
  handleGetCodeActions,
  handleGetCompletions,
  handleGetDiagnostics,
  handleGetDocumentLinks,
  handleGetDocumentSymbols,
  handleGetFoldingRanges,
  handleGetHover,
  handleGetInlayHints,
  handleGetSelectionRange,
  handleGetSemanticTokens,
  handleGetSignatureHelp,
  handleGetTypeHierarchySubtypes,
  handleGetTypeHierarchySupertypes,
  handleHealthCheck,
  handlePrepareCallHierarchy,
  handlePrepareTypeHierarchy,
  handleRenameFile,
  handleRenameSymbol,
  handleRenameSymbolStrict,
  handleRestartServer,
  handleSearchWorkspaceSymbols,
} from './src/mcp/handlers/index.js';
import { createMCPError } from './src/mcp/utils.js';
import {
  createValidationError,
  validateFilePath,
  validatePosition,
  validateQuery,
  validateSymbolName,
} from './src/mcp/validation.js';
import { DiagnosticService } from './src/services/diagnostic-service.js';
import { FileService } from './src/services/file-service.js';
import { HierarchyService } from './src/services/hierarchy-service.js';
import { IntelligenceService } from './src/services/intelligence-service.js';
import { SymbolService } from './src/services/symbol-service.js';
import { getPackageVersion } from './src/utils/version.js';

// Handle subcommands and help flags
const args = process.argv.slice(2);
if (args.length > 0) {
  const subcommand = args[0];

  if (subcommand === 'init') {
    const { initCommand } = await import('./src/cli/commands/init.js');
    await initCommand();
    process.exit(0);
  } else if (subcommand === 'status') {
    const { statusCommand } = await import('./src/cli/commands/status.js');
    await statusCommand();
    process.exit(0);
  } else if (subcommand === 'fix') {
    const { fixCommand } = await import('./src/cli/commands/fix.js');
    const options = {
      auto: args.includes('--auto'),
      manual: args.includes('--manual'),
    };
    await fixCommand(options);
    process.exit(0);
  } else if (subcommand === 'config') {
    const { configCommand } = await import('./src/cli/commands/config.js');
    const options = {
      show: args.includes('--show'),
      edit: args.includes('--edit'),
    };
    await configCommand(options);
    process.exit(0);
  } else if (subcommand === 'logs') {
    const { logsCommand } = await import('./src/cli/commands/logs.js');
    const linesIndex = args.indexOf('--lines');
    const options = {
      tail: args.includes('--tail'),
      lines:
        linesIndex >= 0 && linesIndex + 1 < args.length
          ? Number.parseInt(args[linesIndex + 1] || '50')
          : undefined,
    };
    await logsCommand(options);
    process.exit(0);
  } else if (subcommand === '--help' || subcommand === '-h' || subcommand === 'help') {
    console.log('codebuddy - MCP server for accessing LSP functionality');
    console.log('');
    console.log('Usage: codebuddy [command] [options]');
    console.log('');
    console.log('Commands:');
    console.log('  init          Smart setup with auto-detection');
    console.log("  status        Show what's working right now");
    console.log('  fix           Actually fix problems (auto-install when possible)');
    console.log('  config        Show/edit configuration');
    console.log('  logs          Debug output when things go wrong');
    console.log('  help          Show this help message');
    console.log('');
    console.log('Fix options:');
    console.log('  --auto        Auto-install without prompting');
    console.log('  --manual      Show manual installation commands');
    console.log('');
    console.log('Config options:');
    console.log('  --show        Print configuration to stdout');
    console.log('  --edit        Open configuration in $EDITOR');
    console.log('');
    console.log('Logs options:');
    console.log('  --tail        Follow logs in real-time');
    console.log('  --lines N     Show last N lines (default: 50)');
    console.log('');
    console.log('Run without arguments to start the MCP server.');
    process.exit(0);
  } else {
    console.error(`Unknown command: ${subcommand}`);
    console.error('Available commands:');
    console.error('  init     Smart setup with auto-detection');
    console.error("  status   Show what's working right now");
    console.error('  fix      Actually fix problems');
    console.error('  config   Show/edit configuration');
    console.error('  logs     Debug output');
    console.error('  help     Show help message');
    console.error('');
    console.error('Run without arguments to start the MCP server.');
    process.exit(1);
  }
}

// Create LSP clients and services with proper error handling
let newLspClient: NewLSPClient;
let symbolService: SymbolService;
let fileService: FileService;
let diagnosticService: DiagnosticService;
let intelligenceService: IntelligenceService;
let hierarchyService: HierarchyService;

try {
  // Create new LSP client
  newLspClient = new NewLSPClient();

  // Create ServiceContext for all services
  const { ServiceContextUtils } = await import('./src/services/service-context.js');
  const serviceContext = ServiceContextUtils.createServiceContext(
    newLspClient.getServer.bind(newLspClient),
    newLspClient.protocol
  );

  // Initialize services with ServiceContext
  symbolService = new SymbolService(serviceContext);
  fileService = new FileService(serviceContext);
  diagnosticService = new DiagnosticService(serviceContext);
  intelligenceService = new IntelligenceService(serviceContext);
  hierarchyService = new HierarchyService(serviceContext);
} catch (error) {
  process.stderr.write(
    `Failed to initialize LSP clients: ${error instanceof Error ? error.message : String(error)}\n`
  );
  process.exit(1);
}

const server = new Server(
  {
    name: 'codebuddy',
    version: getPackageVersion(),
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
        if (!validateFilePath(args) || !validateSymbolName(args)) {
          throw createValidationError(
            'find_definition args',
            'object with file_path and symbol_name strings'
          );
        }
        return await handleFindDefinition(symbolService, args as unknown as FindDefinitionArgs);
      case 'find_references':
        if (!validateFilePath(args) || !validateSymbolName(args)) {
          throw createValidationError(
            'find_references args',
            'object with file_path and symbol_name strings'
          );
        }
        return await handleFindReferences(symbolService, args as unknown as FindReferencesArgs);
      case 'rename_symbol':
        return await handleRenameSymbol(symbolService, args as unknown as RenameSymbolArgs);
      case 'rename_symbol_strict':
        if (!validateFilePath(args) || !validatePosition(args)) {
          throw createValidationError(
            'rename_symbol_strict args',
            'object with file_path, line, and character'
          );
        }
        return await handleRenameSymbolStrict(
          symbolService,
          args as unknown as RenameSymbolStrictArgs
        );
      case 'get_code_actions':
        return await handleGetCodeActions(fileService, args as unknown as GetCodeActionsArgs);
      case 'format_document':
        return await handleFormatDocument(fileService, args as unknown as FormatDocumentArgs);
      case 'search_workspace_symbols':
        if (!validateQuery(args)) {
          throw createValidationError('search_workspace_symbols args', 'object with query string');
        }
        return await handleSearchWorkspaceSymbols(
          symbolService,
          args as unknown as SearchWorkspaceSymbolsArgs,
          newLspClient
        );
      case 'get_document_symbols':
        return await handleGetDocumentSymbols(
          symbolService,
          args as unknown as GetDocumentSymbolsArgs
        );
      case 'get_folding_ranges':
        return await handleGetFoldingRanges(
          fileService,
          args as unknown as GetFoldingRangesArgs,
          newLspClient
        );
      case 'get_document_links':
        return await handleGetDocumentLinks(
          fileService,
          args as unknown as GetDocumentLinksArgs,
          newLspClient
        );
      case 'get_diagnostics':
        return await handleGetDiagnostics(diagnosticService, args as unknown as GetDiagnosticsArgs);
      case 'restart_server':
        return await handleRestartServer(newLspClient, args as unknown as RestartServerArgs);
      case 'rename_file':
        return await handleRenameFile(args as unknown as RenameFileArgs);
      // Intelligence tools
      case 'get_hover':
        return await handleGetHover(intelligenceService, args as unknown as GetHoverArgs);
      case 'get_completions':
        return await handleGetCompletions(
          intelligenceService,
          args as unknown as GetCompletionsArgs
        );
      case 'get_inlay_hints':
        return await handleGetInlayHints(intelligenceService, args as unknown as GetInlayHintsArgs);
      case 'get_semantic_tokens':
        return await handleGetSemanticTokens(
          intelligenceService,
          args as unknown as GetSemanticTokensArgs
        );
      case 'get_signature_help':
        return await handleGetSignatureHelp(
          intelligenceService,
          args as unknown as GetSignatureHelpArgs
        );
      // Hierarchy tools
      case 'prepare_call_hierarchy':
        return await handlePrepareCallHierarchy(
          hierarchyService,
          args as unknown as PrepareCallHierarchyArgs
        );
      case 'get_call_hierarchy_incoming_calls':
        return await handleGetCallHierarchyIncomingCalls(
          hierarchyService,
          args as unknown as GetCallHierarchyIncomingCallsArgs
        );
      case 'get_call_hierarchy_outgoing_calls':
        return await handleGetCallHierarchyOutgoingCalls(
          hierarchyService,
          args as unknown as GetCallHierarchyOutgoingCallsArgs
        );
      case 'prepare_type_hierarchy':
        return await handlePrepareTypeHierarchy(
          hierarchyService,
          args as unknown as PrepareTypeHierarchyArgs
        );
      case 'get_type_hierarchy_supertypes':
        return await handleGetTypeHierarchySupertypes(
          hierarchyService,
          args as unknown as GetTypeHierarchySupertypesArgs
        );
      case 'get_type_hierarchy_subtypes':
        return await handleGetTypeHierarchySubtypes(
          hierarchyService,
          args as unknown as GetTypeHierarchySubtypesArgs
        );
      case 'get_selection_range':
        return await handleGetSelectionRange(
          hierarchyService,
          args as unknown as GetSelectionRangeArgs
        );
      case 'apply_workspace_edit':
        return await handleApplyWorkspaceEdit(
          fileService,
          args as unknown as ApplyWorkspaceEditArgs
        );
      case 'create_file':
        return await handleCreateFile(args as unknown as CreateFileArgs);
      case 'delete_file':
        return await handleDeleteFile(args as unknown as DeleteFileArgs);
      case 'health_check': {
        const { ServiceContextUtils } = await import('./src/services/service-context.js');
        const serviceContext = ServiceContextUtils.createServiceContext(
          newLspClient.getServer.bind(newLspClient),
          newLspClient.protocol
        );
        return await handleHealthCheck(args as unknown as HealthCheckArgs, serviceContext);
      }
      default:
        throw new Error(`Unknown tool: ${name}`);
    }
  } catch (error) {
    return createMCPError(error);
  }
});

process.on('SIGINT', () => {
  newLspClient.dispose();
  process.exit(0);
});

process.on('SIGTERM', () => {
  newLspClient.dispose();
  process.exit(0);
});

async function main() {
  // Initialize logging
  const { appendLog } = await import('./src/cli/directory-utils.js');
  appendLog('Codebuddy MCP server starting...');

  const transport = new StdioServerTransport();
  await server.connect(transport);
  process.stderr.write('Codebuddy Server running on stdio\n');
  appendLog('MCP server connected and ready');

  // Preload LSP servers for file types found in the project
  try {
    process.stderr.write('Starting LSP server preload...\n');
    appendLog('Starting LSP server preload');
    await newLspClient.preloadServers();
    process.stderr.write('LSP servers preloaded successfully\n');
    appendLog('LSP servers preloaded successfully');
  } catch (error) {
    const errorMsg = `Failed to preload LSP servers: ${error}`;
    process.stderr.write(`${errorMsg}\n`);
    appendLog(errorMsg);
  }
}

main().catch((error) => {
  process.stderr.write(`Server error: ${error}\n`);
  newLspClient.dispose();
  process.exit(1);
});
