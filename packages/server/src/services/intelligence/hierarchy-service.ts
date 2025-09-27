import { logDebugMessage } from '../../core/diagnostics/debug-logger.js';
import { pathToUri, uriToPath } from '../../core/file-operations/path-utils.js';
import type {
  CallHierarchyIncomingCall,
  CallHierarchyItem,
  CallHierarchyOutgoingCall,
  Position,
} from '../../types.js';
import type { ServiceContext } from '../service-context.js';

// Hierarchy service constants
const _RELATED_FILES_LIMIT = 30; // Maximum related files to open for context

/**
 * Service for hierarchy and navigation-related LSP operations
 * Handles call hierarchy
 */
export class HierarchyService {
  constructor(private context: ServiceContext) {}

  /**
   * Prepare call hierarchy at position
   */
  async prepareCallHierarchy(filePath: string, position: Position): Promise<CallHierarchyItem[]> {
    const serverState = await this.context.prepareFile(filePath);
    if (!serverState) {
      throw new Error('No LSP server available for this file type');
    }

    // ProjectScanner integration disabled - module not available
    logDebugMessage('HierarchyService', 'Project scanner not available for related files');

    const response = await this.context.protocol.sendRequest(
      serverState.process,
      'textDocument/prepareCallHierarchy',
      {
        textDocument: { uri: pathToUri(filePath) },
        position,
      }
    );

    return Array.isArray(response) ? response : [];
  }

  /**
   * Get incoming calls for call hierarchy item
   */
  async getCallHierarchyIncomingCalls(
    item: CallHierarchyItem
  ): Promise<CallHierarchyIncomingCall[]> {
    // Extract the file path from the item's URI to determine the correct server
    const filePath = uriToPath(item.uri);
    const serverState = await this.context.prepareFile(filePath);
    if (!serverState) {
      throw new Error('No LSP server available for this file type');
    }

    const response = await this.context.protocol.sendRequest(
      serverState.process,
      'callHierarchy/incomingCalls',
      {
        item,
      }
    );

    return Array.isArray(response) ? response : [];
  }

  /**
   * Get outgoing calls for call hierarchy item
   */
  async getCallHierarchyOutgoingCalls(
    item: CallHierarchyItem
  ): Promise<CallHierarchyOutgoingCall[]> {
    // Extract the file path from the item's URI to determine the correct server
    const filePath = uriToPath(item.uri);
    const serverState = await this.context.prepareFile(filePath);
    if (!serverState) {
      throw new Error('No LSP server available for this file type');
    }

    const response = await this.context.protocol.sendRequest(
      serverState.process,
      'callHierarchy/outgoingCalls',
      {
        item,
      }
    );

    return Array.isArray(response) ? response : [];
  }

  // ensureFileOpen() and getLanguageId() methods removed - provided by ServiceContext
  // This eliminates ~45 lines of duplicated code from this service
}
