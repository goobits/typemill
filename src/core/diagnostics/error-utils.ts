/**
 * Standardized error handling utilities for consistent error management
 * and improved debugging across the codebase.
 */

import { allToolDefinitions } from '../../mcp/definitions/index.js';
import { getLogger } from './structured-logger.js';

const logger = getLogger('ErrorUtils');

/**
 * Custom error types for better error categorization
 */

export class LSPError extends Error {
  constructor(
    message: string,
    public readonly method?: string,
    public readonly serverCommand?: string,
    public readonly originalError?: unknown
  ) {
    super(message);
    this.name = 'LSPError';
  }
}

export class ConfigurationError extends Error {
  constructor(
    message: string,
    public readonly configPath?: string,
    public readonly originalError?: unknown
  ) {
    super(message);
    this.name = 'ConfigurationError';
  }
}

export class FileSystemError extends Error {
  constructor(
    message: string,
    public readonly filePath: string,
    public readonly operation: string,
    public readonly originalError?: unknown
  ) {
    super(message);
    this.name = 'FileSystemError';
  }
}

export class ServerNotAvailableError extends Error {
  constructor(
    message: string,
    public readonly extensions: string[],
    public readonly command: string[],
    public readonly originalError?: unknown
  ) {
    super(message);
    this.name = 'ServerNotAvailableError';
  }
}

/**
 * Utility functions for consistent error handling
 */

/**
 * Safely extract error message from unknown error type
 */
export function getErrorMessage(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }
  if (typeof error === 'string') {
    return error;
  }
  if (error && typeof error === 'object' && 'message' in error) {
    return String((error as { message: unknown }).message);
  }
  return String(error);
}

/**
 * Create detailed error context for logging and debugging
 */
function createErrorContext(
  error: unknown,
  operation: string,
  additionalContext?: Record<string, unknown>
): {
  message: string;
  operation: string;
  errorType: string;
  context?: Record<string, unknown>;
  stack?: string;
} {
  const errorType = error instanceof Error ? error.constructor.name : typeof error;
  const message = getErrorMessage(error);
  const stack = error instanceof Error ? error.stack : undefined;

  return {
    message,
    operation,
    errorType,
    context: additionalContext,
    stack,
  };
}

/**
 * Log error with consistent format and context
 */
export function logError(
  component: string,
  operation: string,
  error: unknown,
  additionalContext?: Record<string, unknown>
): void {
  const errorContext = createErrorContext(error, operation, additionalContext);

  // Use structured logger with component context
  const componentLogger = getLogger(component);
  componentLogger.error(`${operation}: ${errorContext.message}`, error, {
    operation,
    error_type: errorContext.errorType,
    ...additionalContext,
  });
}

/**
 * Wrap try-catch with consistent error handling
 */
async function withErrorHandling<T>(
  operation: () => Promise<T>,
  context: {
    component: string;
    operation: string;
    fallbackValue?: T;
    additionalContext?: Record<string, unknown>;
  }
): Promise<T> {
  try {
    return await operation();
  } catch (error) {
    logError(context.component, context.operation, error, context.additionalContext);

    if (context.fallbackValue !== undefined) {
      return context.fallbackValue;
    }

    throw error;
  }
}

/**
 * Wrap sync operations with consistent error handling
 */
function withSyncErrorHandling<T>(
  operation: () => T,
  context: {
    component: string;
    operation: string;
    fallbackValue?: T;
    additionalContext?: Record<string, unknown>;
  }
): T {
  try {
    return operation();
  } catch (error) {
    logError(context.component, context.operation, error, context.additionalContext);

    if (context.fallbackValue !== undefined) {
      return context.fallbackValue;
    }

    throw error;
  }
}

/**
 * LSP-specific error handling utilities
 */

/**
 * Handle LSP request errors with appropriate error types and context
 */
export function handleLSPError(
  error: unknown,
  method: string,
  serverCommand?: string | string[]
): never {
  const message = getErrorMessage(error);

  // Check for common LSP error patterns
  if (message.includes('ENOENT') || message.includes('not found')) {
    throw new ServerNotAvailableError(
      `Language server not found: ${Array.isArray(serverCommand) ? serverCommand.join(' ') : serverCommand || 'unknown'}`,
      [],
      Array.isArray(serverCommand) ? serverCommand : [],
      error
    );
  }

  if (message.includes('timeout') || message.includes('timed out')) {
    throw new LSPError(
      `LSP request timed out: ${method}`,
      method,
      Array.isArray(serverCommand) ? serverCommand.join(' ') : serverCommand,
      error
    );
  }

  if (message.includes('process') || message.includes('killed')) {
    throw new LSPError(
      `LSP server process error: ${message}`,
      method,
      Array.isArray(serverCommand) ? serverCommand.join(' ') : serverCommand,
      error
    );
  }

  // Generic LSP error
  throw new LSPError(
    `LSP request failed: ${message}`,
    method,
    Array.isArray(serverCommand) ? serverCommand.join(' ') : serverCommand,
    error
  );
}

/**
 * Handle file system operation errors with appropriate context
 */
export function handleFileSystemError(error: unknown, filePath: string, operation: string): never {
  const message = getErrorMessage(error);

  if (message.includes('ENOENT')) {
    throw new FileSystemError(`File not found: ${filePath}`, filePath, operation, error);
  }

  if (message.includes('EACCES') || message.includes('permission')) {
    throw new FileSystemError(`Permission denied: ${filePath}`, filePath, operation, error);
  }

  if (message.includes('EISDIR')) {
    throw new FileSystemError(
      `Expected file but found directory: ${filePath}`,
      filePath,
      operation,
      error
    );
  }

  throw new FileSystemError(
    `File system error during ${operation}: ${message}`,
    filePath,
    operation,
    error
  );
}

/**
 * Handle configuration loading errors with appropriate context
 */
export function handleConfigurationError(
  error: unknown,
  configPath?: string,
  operation = 'load configuration'
): never {
  const message = getErrorMessage(error);

  if (message.includes('JSON') || message.includes('parse')) {
    throw new ConfigurationError(
      `Invalid JSON in configuration file${configPath ? `: ${configPath}` : ''}`,
      configPath,
      error
    );
  }

  if (message.includes('ENOENT')) {
    throw new ConfigurationError(
      `Configuration file not found${configPath ? `: ${configPath}` : ''}`,
      configPath,
      error
    );
  }

  throw new ConfigurationError(
    `Failed to ${operation}${configPath ? ` from ${configPath}` : ''}: ${message}`,
    configPath,
    error
  );
}

/**
 * Create user-friendly error messages for MCP responses
 */
export function createUserFriendlyErrorMessage(
  error: unknown,
  operation: string,
  suggestions?: string[],
  context?: { filePath?: string }
): string {
  if (error instanceof ServerNotAvailableError) {
    if (context?.filePath) {
      return createLSPServerUnavailableMessage(context.filePath, operation);
    }
    const installCmd = getInstallInstructions(error.command[0] || '');
    return `Language server not available. To enable support: ${installCmd}`;
  }

  if (error instanceof ConfigurationError) {
    return `Configuration error: ${error.message}. Please check your codebuddy.json file or run 'codebuddy setup'.`;
  }

  if (error instanceof FileSystemError) {
    if (error.message.includes('not found')) {
      return createFileNotFoundMessage(error.filePath, operation);
    }
    if (error.message.includes('permission')) {
      return `Permission denied accessing: ${error.filePath}. Please check file permissions.`;
    }
    return `File system error: ${error.message}`;
  }

  if (error instanceof LSPError) {
    if (error.message.includes('timeout')) {
      return 'Operation timed out. The language server may be busy or overloaded.';
    }
    return `Language server error: ${error.message}`;
  }

  const message = getErrorMessage(error);
  let response = `Error during ${operation}: ${message}`;

  if (suggestions && suggestions.length > 0) {
    response += `\n\nSuggestions:\n${suggestions.map((s) => `• ${s}`).join('\n')}`;
  }

  return response;
}

/**
 * Get install instructions for common language servers
 */
function getInstallInstructions(command: string): string {
  const instructions: Record<string, string> = {
    'typescript-language-server': 'npm install -g typescript-language-server typescript',
    pylsp: 'pip install python-lsp-server',
    gopls: 'go install golang.org/x/tools/gopls@latest',
    'rust-analyzer': 'rustup component add rust-analyzer',
    clangd: 'apt install clangd OR brew install llvm',
    jdtls: 'Download from Eclipse JDT releases',
    solargraph: 'gem install solargraph',
    intelephense: 'npm install -g intelephense',
  };

  return instructions[command] || `Install ${command} for your system`;
}

/**
 * Chain error handling for nested operations
 */
function chainError(
  error: unknown,
  parentOperation: string,
  additionalContext?: Record<string, unknown>
): Error {
  const message = getErrorMessage(error);
  const context = additionalContext ? ` (context: ${JSON.stringify(additionalContext)})` : '';

  if (error instanceof Error) {
    // Preserve original error type but add context
    const chainedError = new (error.constructor as new (message: string) => Error)(
      `${parentOperation}: ${message}${context}`
    );
    chainedError.stack = error.stack;
    return chainedError;
  }

  return new Error(`${parentOperation}: ${message}${context}`);
}

/**
 * Enhanced, user-friendly error messages with actionable guidance
 * (Merged from enhanced-error-messages.ts)
 */

/**
 * Create contextual error message for LSP server not available
 */
export function createLSPServerUnavailableMessage(filePath: string, operation: string): string {
  const extension = filePath.split('.').pop()?.toLowerCase() || 'unknown';

  // Common language mappings
  const languageInfo = getLanguageInfo(extension);

  let message = `❌ **${operation} not available** for ${languageInfo.name} files\n\n`;
  message += `**What happened:** No language server is configured for .${extension} files\n\n`;
  message += '**To fix this:**\n';

  if (languageInfo.servers.length > 0) {
    message += `1. **Install a ${languageInfo.name} language server:**\n`;
    for (let i = 0; i < languageInfo.servers.length; i++) {
      const server = languageInfo.servers[i];
      if (server) {
        message += `   ${i + 1}. ${server.install} (${server.description})\n`;
      }
    }
    message += '\n2. **Configure Codebuddy:**\n';
    message += `   Run: \`codebuddy init\` and select ${languageInfo.name} support\n\n`;
  } else {
    message += '1. Run: `codebuddy init` to set up language servers\n';
    message += `2. Check if there's a language server available for .${extension} files\n\n`;
  }

  message += '**Alternative:** Try the operation on a supported file type:\n';
  message += '• TypeScript/JavaScript (.ts, .js) - Full support\n';
  message += '• Python (.py) - Good support\n';
  message += '• Go (.go) - Good support\n';

  return message;
}


/**
 * Create helpful message for unknown tool errors with suggestions
 */
export function createUnknownToolMessage(toolName: string): string {
  const availableTools = allToolDefinitions.map((t) => t.name);
  const suggestions = findSimilarTools(toolName, availableTools);

  let message = `❌ **Unknown tool:** \`${toolName}\`\n\n`;

  if (suggestions.length > 0) {
    message += '**Did you mean:**\n';
    for (const suggestion of suggestions.slice(0, 3)) {
      message += `• \`${suggestion.name}\` - ${suggestion.description}\n`;
    }
    message += '\n';
  }

  message += '**Available tools:**\n';
  message += '• **Navigation:** find_definition, find_references, search_workspace_symbols\n';
  message += '• **Intelligence:** get_hover, get_completions, get_diagnostics\n';
  message += '• **Refactoring:** rename_symbol, format_document, get_code_actions\n';
  message += '• **Hierarchy:** prepare_call_hierarchy, prepare_type_hierarchy\n';
  message += '• **System:** health_check, restart_server\n\n';

  message += '**Full list:** Use the MCP client to list all available tools\n';

  return message;
}

/**
 * Create helpful message for file not found errors
 */
export function createFileNotFoundMessage(filePath: string, operation: string): string {
  let message = `❌ **File not found:** \`${filePath}\`\n\n`;
  message += `**What happened:** Cannot perform ${operation} - file doesn't exist\n\n`;
  message += '**Please check:**\n';
  message += '• **Path spelling:** Verify the file path is correct\n';
  message += '• **Current directory:** Are you in the right folder?\n';
  message += '• **File existence:** Does the file actually exist?\n';
  message += '• **Permissions:** Do you have read access to the file?\n\n';

  // Try to suggest similar files in the directory
  try {
    const dir = filePath.substring(0, filePath.lastIndexOf('/')) || '.';
    const fileName = filePath.substring(filePath.lastIndexOf('/') + 1);
    message += `**Tip:** Check if similar files exist in \`${dir}/\`\n`;
  } catch {
    // Ignore path parsing errors
  }

  return message;
}

/**
 * Get language-specific information for better error messages
 */
function getLanguageInfo(extension: string): {
  name: string;
  servers: Array<{ install: string; description: string }>;
} {
  const languageMap: Record<
    string,
    { name: string; servers: Array<{ install: string; description: string }> }
  > = {
    ts: {
      name: 'TypeScript',
      servers: [
        {
          install: 'npm install -g typescript-language-server typescript',
          description: 'Official TypeScript server',
        },
      ],
    },
    tsx: {
      name: 'TypeScript React',
      servers: [
        {
          install: 'npm install -g typescript-language-server typescript',
          description: 'Official TypeScript server',
        },
      ],
    },
    js: {
      name: 'JavaScript',
      servers: [
        {
          install: 'npm install -g typescript-language-server typescript',
          description: 'TypeScript server (works for JS)',
        },
      ],
    },
    jsx: {
      name: 'JavaScript React',
      servers: [
        {
          install: 'npm install -g typescript-language-server typescript',
          description: 'TypeScript server (works for JSX)',
        },
      ],
    },
    py: {
      name: 'Python',
      servers: [
        { install: 'pip install python-lsp-server', description: 'Python Language Server' },
        { install: 'pip install pylsp', description: 'Alternative Python server' },
      ],
    },
    go: {
      name: 'Go',
      servers: [
        {
          install: 'go install golang.org/x/tools/gopls@latest',
          description: 'Official Go language server',
        },
      ],
    },
    rs: {
      name: 'Rust',
      servers: [
        { install: 'rustup component add rust-analyzer', description: 'Official Rust analyzer' },
      ],
    },
    java: {
      name: 'Java',
      servers: [{ install: 'Download Eclipse JDT Language Server', description: 'Eclipse JDT LS' }],
    },
    cpp: {
      name: 'C++',
      servers: [
        {
          install: 'Install clangd via your package manager',
          description: 'Clang language server',
        },
      ],
    },
    c: {
      name: 'C',
      servers: [
        {
          install: 'Install clangd via your package manager',
          description: 'Clang language server',
        },
      ],
    },
  };

  return (
    languageMap[extension] || {
      name: extension.toUpperCase(),
      servers: [],
    }
  );
}

/**
 * Find tools with similar names using simple string similarity
 */
function findSimilarTools(
  input: string,
  availableTools: string[]
): Array<{ name: string; description: string }> {
  const similarities = availableTools
    .map((tool) => ({
      name: tool,
      description: getToolDescription(tool),
      similarity: calculateSimilarity(input.toLowerCase(), tool.toLowerCase()),
    }))
    .filter((item) => item.similarity > 0.4) // Only include reasonably similar tools
    .sort((a, b) => b.similarity - a.similarity);

  return similarities;
}

/**
 * Get description for a tool name
 */
function getToolDescription(toolName: string): string {
  const tool = allToolDefinitions.find((t) => t.name === toolName);
  return tool?.description || 'No description available';
}

/**
 * Calculate simple string similarity using Levenshtein-like algorithm
 */
function calculateSimilarity(str1: string, str2: string): number {
  const longer = str1.length > str2.length ? str1 : str2;
  const shorter = str1.length > str2.length ? str2 : str1;

  if (longer.length === 0) return 1.0;

  // Check for substring matches (higher weight)
  if (longer.includes(shorter) || shorter.includes(longer)) {
    return 0.8;
  }

  // Simple character overlap
  const overlap = [...shorter].filter((char) => longer.includes(char)).length;
  return overlap / longer.length;
}
