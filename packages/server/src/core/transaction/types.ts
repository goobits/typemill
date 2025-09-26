export interface FileSystemSnapshot {
  files: Map<string, string | null>; // File path -> File content (null if file doesn't exist)
  operations: FileOperation[]; // Track file operations for rollback
}

export interface FileOperation {
  type: 'CREATE' | 'DELETE' | 'MOVE' | 'MODIFY';
  path: string;
  originalPath?: string; // For MOVE operations
  originalContent?: string | null; // For MODIFY operations
  timestamp: number;
}

export interface Transaction {
  id: string;
  checkpoints: Map<string, FileSystemSnapshot>;
  operations: FileOperation[]; // Track all operations in this transaction
}
