// LLM Agent Intelligence LSP Methods
import type { Hover, Position, CompletionItem, InlayHint, InlayHintParams, SemanticTokens, SemanticTokensParams } from '../types.js';

export interface IntelligenceMethodsContext {
  getServer: (filePath: string) => Promise<any>;
  ensureFileOpen: (serverState: any, filePath: string) => Promise<void>;
  sendRequest: (serverState: any, method: string, params: any) => Promise<any>;
}

export async function getHover(
  context: IntelligenceMethodsContext,
  filePath: string,
  position: Position
): Promise<Hover | null> {
  const serverState = await context.getServer(filePath);
  if (!serverState) {
    throw new Error('No LSP server available for this file type');
  }

  await context.ensureFileOpen(serverState, filePath);

  const response = await context.sendRequest(serverState, 'textDocument/hover', {
    textDocument: { uri: `file://${filePath}` },
    position,
  });

  return response || null;
}

export async function getCompletions(
  context: IntelligenceMethodsContext,
  filePath: string,
  position: Position,
  triggerCharacter?: string
): Promise<CompletionItem[]> {
  const serverState = await context.getServer(filePath);
  if (!serverState) {
    throw new Error('No LSP server available for this file type');
  }

  await context.ensureFileOpen(serverState, filePath);

  const completionParams = {
    textDocument: { uri: `file://${filePath}` },
    position,
    context: triggerCharacter
      ? {
          triggerKind: 2, // TriggerCharacter
          triggerCharacter,
        }
      : {
          triggerKind: 1, // Invoked
        },
  };

  const response = await context.sendRequest(serverState, 'textDocument/completion', completionParams);

  // Handle both CompletionList and CompletionItem[] responses
  if (response && typeof response === 'object') {
    if (Array.isArray(response)) {
      return response;
    }
    if (response.items && Array.isArray(response.items)) {
      return response.items;
    }
  }

  return [];
}

export async function getInlayHints(
  context: IntelligenceMethodsContext,
  filePath: string,
  range: { start: Position; end: Position }
): Promise<InlayHint[]> {
  const serverState = await context.getServer(filePath);
  if (!serverState) {
    throw new Error('No LSP server available for this file type');
  }

  await context.ensureFileOpen(serverState, filePath);

  const inlayHintParams: InlayHintParams = {
    textDocument: { uri: `file://${filePath}` },
    range,
  };

  const response = await context.sendRequest(serverState, 'textDocument/inlayHint', inlayHintParams);

  return response || [];
}

export async function getSemanticTokens(
  context: IntelligenceMethodsContext,
  filePath: string
): Promise<SemanticTokens | null> {
  const serverState = await context.getServer(filePath);
  if (!serverState) {
    throw new Error('No LSP server available for this file type');
  }

  await context.ensureFileOpen(serverState, filePath);

  const semanticTokensParams: SemanticTokensParams = {
    textDocument: { uri: `file://${filePath}` },
  };

  const response = await context.sendRequest(serverState, 'textDocument/semanticTokens/full', semanticTokensParams);

  return response || null;
}