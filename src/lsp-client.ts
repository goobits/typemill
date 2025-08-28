import { readFileSync } from 'node:fs';
import { capabilityManager } from './capability-manager.js';
import { scanDirectoryForExtensions } from './file-scanner.js';
import * as HierarchyMethods from './lsp-methods/hierarchy-methods.js';
import * as IntelligenceMethods from './lsp-methods/intelligence-methods.js';
import type { ServerState } from './lsp-types.js';
import { LSPClient as NewLSPClient } from './lsp/client.js';
import type { LSPProtocol } from './lsp/protocol.js';
import type { ServerManager } from './lsp/server-manager.js';
import { DiagnosticService } from './services/diagnostic-service.js';
import { FileService } from './services/file-service.js';
import { SymbolService } from './services/symbol-service.js';
import type {
  CodeAction,
  Config,
  Diagnostic,
  DocumentSymbol,
  FoldingRange,
  Location,
  Position,
  Range,
  SymbolInformation,
  SymbolMatch,
  TextEdit,
} from './types.js';

/**
 * LSP Client facade that maintains backward compatibility
 * while using the new service-oriented architecture
 */
export class LSPClient {
  private newClient: NewLSPClient;
  private protocol: LSPProtocol;
  private serverManager: ServerManager;
  private symbolService: SymbolService;
  private fileService: FileService;
  private diagnosticService: DiagnosticService;

  constructor(configPath?: string) {
    this.newClient = new NewLSPClient(configPath);

    // Access internal components (would be properly injected in real refactor)
    this.protocol = (this.newClient as any).protocol;
    this.serverManager = (this.newClient as any).serverManager;

    // Initialize services
    this.symbolService = new SymbolService(this.serverManager, this.protocol);
    this.fileService = new FileService(this.serverManager, this.protocol);
    this.diagnosticService = new DiagnosticService(this.serverManager, this.protocol);
  }

  // Delegate core methods to services
  async findDefinition(filePath: string, position: Position): Promise<Location[]> {
    return this.symbolService.findDefinition(filePath, position);
  }

  async findReferences(
    filePath: string,
    position: Position,
    includeDeclaration = false
  ): Promise<Location[]> {
    return this.symbolService.findReferences(filePath, position, includeDeclaration);
  }

  async renameSymbol(
    filePath: string,
    position: Position,
    newName: string
  ): Promise<{
    changes?: Record<string, Array<{ range: { start: Position; end: Position }; newText: string }>>;
  }> {
    return this.symbolService.renameSymbol(filePath, position, newName);
  }

  async getDocumentSymbols(filePath: string): Promise<DocumentSymbol[] | SymbolInformation[]> {
    return this.symbolService.getDocumentSymbols(filePath);
  }

  async searchWorkspaceSymbols(query: string): Promise<SymbolInformation[]> {
    return this.symbolService.searchWorkspaceSymbols(query);
  }

  async findSymbolMatches(
    filePath: string,
    symbolName: string,
    symbolKind?: string
  ): Promise<SymbolMatch[]> {
    return this.symbolService.findSymbolMatches(filePath, symbolName, symbolKind);
  }

  async formatDocument(filePath: string): Promise<TextEdit[]> {
    return this.fileService.formatDocument(filePath);
  }

  async getCodeActions(
    filePath: string,
    range: Range,
    context: { diagnostics: any[] }
  ): Promise<CodeAction[]> {
    return this.fileService.getCodeActions(filePath, range, context);
  }

  async getFoldingRanges(filePath: string): Promise<FoldingRange[]> {
    return this.fileService.getFoldingRanges(filePath);
  }

  async getDocumentLinks(filePath: string): Promise<import('./types.js').DocumentLink[]> {
    return this.fileService.getDocumentLinks(filePath);
  }

  async getDiagnostics(filePath: string): Promise<Diagnostic[]> {
    return this.diagnosticService.getDiagnostics(filePath);
  }

  // Intelligence methods - still delegated to lsp-methods for now
  async getHover(filePath: string, position: Position): Promise<import('./types.js').Hover | null> {
    const context = {
      getServer: this.getServer.bind(this),
      ensureFileOpen: this.ensureFileOpen.bind(this),
      sendRequest: (serverState: ServerState, method: string, params: unknown, timeout?: number) =>
        this.protocol.sendRequest(serverState.process, method, params, timeout),
    };
    return IntelligenceMethods.getHover(context, filePath, position);
  }

  async getCompletions(
    filePath: string,
    position: Position
  ): Promise<import('./types.js').CompletionList | import('./types.js').CompletionItem[]> {
    const context = {
      getServer: this.getServer.bind(this),
      ensureFileOpen: this.ensureFileOpen.bind(this),
      sendRequest: (serverState: ServerState, method: string, params: unknown, timeout?: number) =>
        this.protocol.sendRequest(serverState.process, method, params, timeout),
    };
    return IntelligenceMethods.getCompletions(context, filePath, position);
  }

  async getSignatureHelp(
    filePath: string,
    position: Position
  ): Promise<import('./types.js').SignatureHelp | null> {
    const context = {
      getServer: this.getServer.bind(this),
      ensureFileOpen: this.ensureFileOpen.bind(this),
      sendRequest: (serverState: ServerState, method: string, params: unknown, timeout?: number) =>
        this.protocol.sendRequest(serverState.process, method, params, timeout),
    };
    return IntelligenceMethods.getSignatureHelp(context, filePath, position);
  }

  async getInlayHints(filePath: string, range: Range): Promise<import('./types.js').InlayHint[]> {
    const context = {
      getServer: this.getServer.bind(this),
      ensureFileOpen: this.ensureFileOpen.bind(this),
      sendRequest: (serverState: ServerState, method: string, params: unknown, timeout?: number) =>
        this.protocol.sendRequest(serverState.process, method, params, timeout),
    };
    return IntelligenceMethods.getInlayHints(context, filePath, range);
  }

  async getSemanticTokens(filePath: string): Promise<import('./types.js').SemanticTokens | null> {
    const context = {
      getServer: this.getServer.bind(this),
      ensureFileOpen: this.ensureFileOpen.bind(this),
      sendRequest: (serverState: ServerState, method: string, params: unknown, timeout?: number) =>
        this.protocol.sendRequest(serverState.process, method, params, timeout),
    };
    return IntelligenceMethods.getSemanticTokens(context, filePath);
  }

  // Hierarchy methods - still delegated to lsp-methods for now
  async prepareCallHierarchy(
    filePath: string,
    position: Position
  ): Promise<import('./types.js').CallHierarchyItem[]> {
    const context = {
      getServer: this.getServer.bind(this),
      ensureFileOpen: this.ensureFileOpen.bind(this),
      sendRequest: (serverState: ServerState, method: string, params: unknown, timeout?: number) =>
        this.protocol.sendRequest(serverState.process, method, params, timeout),
    };
    return HierarchyMethods.prepareCallHierarchy(context, filePath, position);
  }

  async getCallHierarchyIncomingCalls(
    item: import('./types.js').CallHierarchyItem
  ): Promise<import('./types.js').CallHierarchyIncomingCall[]> {
    const context = {
      getServer: this.getServer.bind(this),
      ensureFileOpen: this.ensureFileOpen.bind(this),
      sendRequest: (serverState: ServerState, method: string, params: unknown, timeout?: number) =>
        this.protocol.sendRequest(serverState.process, method, params, timeout),
    };
    return HierarchyMethods.getCallHierarchyIncomingCalls(context, item);
  }

  async getCallHierarchyOutgoingCalls(
    item: import('./types.js').CallHierarchyItem
  ): Promise<import('./types.js').CallHierarchyOutgoingCall[]> {
    const context = {
      getServer: this.getServer.bind(this),
      ensureFileOpen: this.ensureFileOpen.bind(this),
      sendRequest: (serverState: ServerState, method: string, params: unknown, timeout?: number) =>
        this.protocol.sendRequest(serverState.process, method, params, timeout),
    };
    return HierarchyMethods.getCallHierarchyOutgoingCalls(context, item);
  }

  // Direct delegation to new client
  async getServer(filePath: string): Promise<ServerState> {
    return this.newClient.getServer(filePath);
  }

  async sendRequest(
    process: import('node:child_process').ChildProcess,
    method: string,
    params: unknown,
    timeout?: number
  ): Promise<unknown> {
    return this.protocol.sendRequest(process, method, params, timeout);
  }

  sendNotification(
    process: import('node:child_process').ChildProcess,
    method: string,
    params: unknown
  ): void {
    this.protocol.sendNotification(process, method, params);
  }

  async restartServer(extensions?: string[]): Promise<string[]> {
    return this.newClient.restartServer(extensions);
  }

  async preloadServers(): Promise<void> {
    return this.newClient.preloadServers();
  }

  // Utility methods from services
  flattenDocumentSymbols = this.symbolService.flattenDocumentSymbols;
  isDocumentSymbolArray = this.symbolService.isDocumentSymbolArray;
  symbolKindToString = this.symbolService.symbolKindToString;
  getValidSymbolKinds = this.symbolService.getValidSymbolKinds;

  // Capability methods
  hasCapability(filePath: string, capabilityPath: string): Promise<boolean> {
    return this.getServer(filePath)
      .then((serverState) => {
        return capabilityManager.hasCapability(serverState, capabilityPath);
      })
      .catch(() => false);
  }

  async getCapabilityInfo(filePath: string): Promise<string> {
    try {
      const serverState = await this.getServer(filePath);
      return capabilityManager.getCapabilityInfo(serverState);
    } catch (error) {
      return `Error getting server: ${error instanceof Error ? error.message : String(error)}`;
    }
  }

  async validateCapabilities(
    filePath: string,
    requiredCapabilities: string[]
  ): Promise<{
    supported: boolean;
    missing: string[];
    serverDescription: string;
  }> {
    try {
      const serverState = await this.getServer(filePath);
      const validation = capabilityManager.validateRequiredCapabilities(
        serverState,
        requiredCapabilities
      );
      return {
        ...validation,
        serverDescription: capabilityManager.getServerDescription(serverState),
      };
    } catch (error) {
      return {
        supported: false,
        missing: requiredCapabilities,
        serverDescription: 'Unknown Server',
      };
    }
  }

  // File synchronization
  private async ensureFileOpen(serverState: ServerState, filePath: string): Promise<void> {
    return this.fileService.ensureFileOpen(serverState, filePath);
  }

  dispose(): void {
    this.newClient.dispose();
  }
}
