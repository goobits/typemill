// Hierarchy and Navigation LSP Methods for LLM agents
import type { 
  Position, 
  CallHierarchyItem, 
  CallHierarchyIncomingCall, 
  CallHierarchyOutgoingCall,
  TypeHierarchyItem,
  SelectionRange
} from '../types.js';

export interface HierarchyMethodsContext {
  getServer: (filePath: string) => Promise<any>;
  ensureFileOpen: (serverState: any, filePath: string) => Promise<void>;
  sendRequest: (serverState: any, method: string, params: any) => Promise<any>;
}

export async function prepareCallHierarchy(
  context: HierarchyMethodsContext,
  filePath: string,
  position: Position
): Promise<CallHierarchyItem[]> {
  const serverState = await context.getServer(filePath);
  if (!serverState) {
    throw new Error('No LSP server available for this file type');
  }

  await context.ensureFileOpen(serverState, filePath);

  const response = await context.sendRequest(serverState, 'textDocument/prepareCallHierarchy', {
    textDocument: { uri: `file://${filePath}` },
    position,
  });

  return response || [];
}

export async function getCallHierarchyIncomingCalls(
  context: HierarchyMethodsContext,
  item: CallHierarchyItem
): Promise<CallHierarchyIncomingCall[]> {
  // Extract the file path from the item's URI to determine the correct server
  const filePath = item.uri.replace('file://', '');
  const serverState = await context.getServer(filePath);
  if (!serverState) {
    throw new Error('No LSP server available for this file type');
  }

  const response = await context.sendRequest(serverState, 'callHierarchy/incomingCalls', {
    item,
  });

  return response || [];
}

export async function getCallHierarchyOutgoingCalls(
  context: HierarchyMethodsContext,
  item: CallHierarchyItem
): Promise<CallHierarchyOutgoingCall[]> {
  // Extract the file path from the item's URI to determine the correct server
  const filePath = item.uri.replace('file://', '');
  const serverState = await context.getServer(filePath);
  if (!serverState) {
    throw new Error('No LSP server available for this file type');
  }

  const response = await context.sendRequest(serverState, 'callHierarchy/outgoingCalls', {
    item,
  });

  return response || [];
}

export async function prepareTypeHierarchy(
  context: HierarchyMethodsContext,
  filePath: string,
  position: Position
): Promise<TypeHierarchyItem[]> {
  const serverState = await context.getServer(filePath);
  if (!serverState) {
    throw new Error('No LSP server available for this file type');
  }

  await context.ensureFileOpen(serverState, filePath);

  const response = await context.sendRequest(serverState, 'textDocument/prepareTypeHierarchy', {
    textDocument: { uri: `file://${filePath}` },
    position,
  });

  return response || [];
}

export async function getTypeHierarchySupertypes(
  context: HierarchyMethodsContext,
  item: TypeHierarchyItem
): Promise<TypeHierarchyItem[]> {
  // Extract the file path from the item's URI to determine the correct server
  const filePath = item.uri.replace('file://', '');
  const serverState = await context.getServer(filePath);
  if (!serverState) {
    throw new Error('No LSP server available for this file type');
  }

  const response = await context.sendRequest(serverState, 'typeHierarchy/supertypes', {
    item,
  });

  return response || [];
}

export async function getTypeHierarchySubtypes(
  context: HierarchyMethodsContext,
  item: TypeHierarchyItem
): Promise<TypeHierarchyItem[]> {
  // Extract the file path from the item's URI to determine the correct server
  const filePath = item.uri.replace('file://', '');
  const serverState = await context.getServer(filePath);
  if (!serverState) {
    throw new Error('No LSP server available for this file type');
  }

  const response = await context.sendRequest(serverState, 'typeHierarchy/subtypes', {
    item,
  });

  return response || [];
}

export async function getSelectionRange(
  context: HierarchyMethodsContext,
  filePath: string,
  positions: Position[]
): Promise<SelectionRange[]> {
  const serverState = await context.getServer(filePath);
  if (!serverState) {
    throw new Error('No LSP server available for this file type');
  }

  await context.ensureFileOpen(serverState, filePath);

  const response = await context.sendRequest(serverState, 'textDocument/selectionRange', {
    textDocument: { uri: `file://${filePath}` },
    positions,
  });

  return response || [];
}