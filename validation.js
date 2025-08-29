// src/mcp/validation.ts
function validateFilePath(args) {
  return typeof args === "object" && args !== null && "file_path" in args && typeof args.file_path === "string" && args.file_path.length > 0;
}
function validatePosition(args) {
  return typeof args === "object" && args !== null && "line" in args && "character" in args && typeof args.line === "number" && typeof args.character === "number" && Number.isInteger(args.line) && Number.isInteger(args.character) && args.line >= 0 && args.character >= 0;
}
function validateQuery(args) {
  return typeof args === "object" && args !== null && "query" in args && typeof args.query === "string" && args.query.trim().length > 0;
}
function validateSymbolName(args) {
  return typeof args === "object" && args !== null && "symbol_name" in args && typeof args.symbol_name === "string" && args.symbol_name.trim().length > 0;
}
function createValidationError(fieldName, expectedType) {
  return new Error(`Invalid ${fieldName}: expected ${expectedType}`);
}
export {
  validateSymbolName,
  validateQuery,
  validatePosition,
  validateFilePath,
  createValidationError
};
