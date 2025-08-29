// Basic type guards for MCP request validation
// Provides lightweight runtime validation for critical properties

/**
 * Validate that args has a file_path property that's a non-empty string
 */
export function validateFilePath(args: unknown): args is { file_path: string } {
  return (
    typeof args === 'object' &&
    args !== null &&
    'file_path' in args &&
    typeof (args as any).file_path === 'string' &&
    (args as any).file_path.length > 0
  );
}

/**
 * Validate that args has position properties (line and character as numbers)
 */
export function validatePosition(args: unknown): args is { line: number; character: number } {
  return (
    typeof args === 'object' &&
    args !== null &&
    'line' in args &&
    'character' in args &&
    typeof (args as any).line === 'number' &&
    typeof (args as any).character === 'number' &&
    Number.isInteger((args as any).line) &&
    Number.isInteger((args as any).character) &&
    (args as any).line >= 0 &&
    (args as any).character >= 0
  );
}

/**
 * Validate that args has a query property that's a non-empty string
 */
export function validateQuery(args: unknown): args is { query: string } {
  return (
    typeof args === 'object' &&
    args !== null &&
    'query' in args &&
    typeof (args as any).query === 'string' &&
    (args as any).query.trim().length > 0
  );
}

/**
 * Validate that args has symbol_name property that's a non-empty string
 */
export function validateSymbolName(args: unknown): args is { symbol_name: string } {
  return (
    typeof args === 'object' &&
    args !== null &&
    'symbol_name' in args &&
    typeof (args as any).symbol_name === 'string' &&
    (args as any).symbol_name.trim().length > 0
  );
}

/**
 * Generic validation error creator
 */
export function createValidationError(fieldName: string, expectedType: string): Error {
  return new Error(`Invalid ${fieldName}: expected ${expectedType}`);
}
