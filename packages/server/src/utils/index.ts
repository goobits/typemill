/**
 * Main utilities barrel export
 */

// File utilities
export * from './file/index.js';
export {
  normalizePath,
  pathToUrl,
  readFileContent,
  resolvePath,
  urlToPath,
  writeFileContent,
} from './file/index.js';
// Performance utilities
export * from './performance.js';
export {
  globalPerformanceTracker,
  measurePerformance,
  withPerformanceMeasurement,
} from './performance.js';
// Platform utilities
export * from './platform/index.js';

// Re-export commonly used utilities for convenience
export {
  getLSPServerPaths,
  isProcessRunning,
  terminateProcess,
} from './platform/index.js';
// Position utilities
export * from './position.js';

export {
  formatFileLocation,
  formatHumanPosition,
  parsePositionString,
  toHumanPosition,
  toLSPPosition,
} from './position.js';
// Validation utilities
export * from './validation.js';
export {
  assertFileExists,
  assertNonEmptyString,
  assertValidFilePath,
  assertValidLSPPosition,
  assertValidSymbolName,
  ValidationError,
} from './validation.js';
