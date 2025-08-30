// Basic type guards for MCP request validation
// Provides lightweight runtime validation for critical properties

/**
 * Validate that args has a file_path property that's a non-empty string
 */
export function validateFilePath(args: unknown): args is { file_path: string } {
  if (typeof args !== 'object' || args === null) {
    return false;
  }
  const obj = args as Record<string, unknown>;
  return 'file_path' in obj && typeof obj.file_path === 'string' && obj.file_path.length > 0;
}

/**
 * Validate that args has position properties (line and character as numbers)
 */
export function validatePosition(args: unknown): args is { line: number; character: number } {
  if (typeof args !== 'object' || args === null) {
    return false;
  }
  const obj = args as Record<string, unknown>;
  return (
    'line' in obj &&
    'character' in obj &&
    typeof obj.line === 'number' &&
    typeof obj.character === 'number' &&
    Number.isInteger(obj.line) &&
    Number.isInteger(obj.character) &&
    obj.line >= 0 &&
    obj.character >= 0
  );
}

/**
 * Validate that args has a query property that's a non-empty string
 */
export function validateQuery(args: unknown): args is { query: string } {
  if (typeof args !== 'object' || args === null) {
    return false;
  }
  const obj = args as Record<string, unknown>;
  return 'query' in obj && typeof obj.query === 'string' && obj.query.trim().length > 0;
}

/**
 * Validate that args has symbol_name property that's a non-empty string
 */
export function validateSymbolName(args: unknown): args is { symbol_name: string } {
  if (typeof args !== 'object' || args === null) {
    return false;
  }
  const obj = args as Record<string, unknown>;
  return (
    'symbol_name' in obj && typeof obj.symbol_name === 'string' && obj.symbol_name.trim().length > 0
  );
}

/**
 * Generic validation error creator
 */
export function createValidationError(fieldName: string, expectedType: string): Error {
  return new Error(`Invalid ${fieldName}: expected ${expectedType}`);
}
