/**
 * Simple in-memory cache with TTL for file operations
 * Helps reduce redundant file read requests to clients
 */

export interface CacheEntry<T> {
  value: T;
  timestamp: Date;
  ttl: number; // Time to live in milliseconds
}

export class SimpleCache<T> {
  private cache = new Map<string, CacheEntry<T>>();
  private cleanupInterval: NodeJS.Timeout;

  constructor(private defaultTTL: number = 5000) {
    // Clean up expired entries every 30 seconds
    this.cleanupInterval = setInterval(() => {
      this.cleanup();
    }, 30000);
  }

  set(key: string, value: T, ttl?: number): void {
    const entry: CacheEntry<T> = {
      value,
      timestamp: new Date(),
      ttl: ttl || this.defaultTTL
    };

    this.cache.set(key, entry);
  }

  get(key: string): T | null {
    const entry = this.cache.get(key);

    if (!entry) {
      return null;
    }

    // Check if entry has expired
    const now = Date.now();
    const entryTime = entry.timestamp.getTime();

    if (now - entryTime > entry.ttl) {
      this.cache.delete(key);
      return null;
    }

    return entry.value;
  }

  has(key: string): boolean {
    return this.get(key) !== null;
  }

  delete(key: string): boolean {
    return this.cache.delete(key);
  }

  clear(): void {
    this.cache.clear();
  }

  size(): number {
    return this.cache.size;
  }

  /**
   * Remove expired entries from cache
   */
  private cleanup(): void {
    const now = Date.now();
    const keysToDelete: string[] = [];

    for (const [key, entry] of this.cache.entries()) {
      const entryTime = entry.timestamp.getTime();

      if (now - entryTime > entry.ttl) {
        keysToDelete.push(key);
      }
    }

    for (const key of keysToDelete) {
      this.cache.delete(key);
    }

    if (keysToDelete.length > 0) {
      console.debug(`Cache cleanup: removed ${keysToDelete.length} expired entries`);
    }
  }

  /**
   * Get cache statistics
   */
  getStats(): {
    size: number;
    entries: Array<{
      key: string;
      age: number;
      ttl: number;
      isExpired: boolean;
    }>;
  } {
    const now = Date.now();
    const entries = Array.from(this.cache.entries()).map(([key, entry]) => {
      const age = now - entry.timestamp.getTime();
      return {
        key,
        age,
        ttl: entry.ttl,
        isExpired: age > entry.ttl
      };
    });

    return {
      size: this.cache.size,
      entries
    };
  }

  /**
   * Clean up resources
   */
  dispose(): void {
    if (this.cleanupInterval) {
      clearInterval(this.cleanupInterval);
    }
    this.clear();
  }
}

/**
 * File-specific cache implementation
 */
export interface FileContent {
  content: string;
  mtime: number;
}

export class FileCache extends SimpleCache<FileContent> {
  constructor() {
    super(5000); // 5 second TTL for file reads
  }

  /**
   * Generate cache key for file read operations
   */
  static getFileKey(sessionId: string, filePath: string): string {
    return `file:${sessionId}:${filePath}`;
  }

  /**
   * Cache file content with validation
   */
  setFile(sessionId: string, filePath: string, content: string, mtime: number): void {
    const key = FileCache.getFileKey(sessionId, filePath);
    this.set(key, { content, mtime });
  }

  /**
   * Get cached file content if not expired and mtime matches
   */
  getFile(sessionId: string, filePath: string, currentMtime?: number): FileContent | null {
    const key = FileCache.getFileKey(sessionId, filePath);
    const cached = this.get(key);

    if (!cached) {
      return null;
    }

    // If mtime is provided, check if file was modified
    if (currentMtime && cached.mtime !== currentMtime) {
      this.delete(key); // Remove stale cache entry
      return null;
    }

    return cached;
  }

  /**
   * Invalidate cache for a specific file
   */
  invalidateFile(sessionId: string, filePath: string): boolean {
    const key = FileCache.getFileKey(sessionId, filePath);
    return this.delete(key);
  }

  /**
   * Invalidate all cache entries for a session
   */
  invalidateSession(sessionId: string): number {
    let deletedCount = 0;
    const keysToDelete: string[] = [];

    // Use the parent class's getStats method to get all entries
    const stats = this.getStats();
    for (const entry of stats.entries) {
      if (entry.key.startsWith(`file:${sessionId}:`)) {
        keysToDelete.push(entry.key);
      }
    }

    for (const key of keysToDelete) {
      if (this.delete(key)) {
        deletedCount++;
      }
    }

    return deletedCount;
  }
}