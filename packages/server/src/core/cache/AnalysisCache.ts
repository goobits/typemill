import { createHash } from 'node:crypto';
import type { CacheEntry } from './types';

export class AnalysisCache<T> {
  private cache = new Map<string, CacheEntry<T>>();
  private ttl: number; // Time-to-live in milliseconds

  constructor(ttl = 3_600_000) {
    // Default TTL of 1 hour
    this.ttl = ttl;
  }

  get(key: string, fileContent: string): T | null {
    const entry = this.cache.get(key);
    if (!entry) return null;

    // Check TTL
    if (Date.now() - entry.timestamp > this.ttl) {
      this.cache.delete(key);
      return null;
    }

    // Check content hash
    const currentHash = this.hash(fileContent);
    if (entry.fileHash !== currentHash) {
      this.cache.delete(key);
      return null;
    }

    return entry.data;
  }

  set(key: string, data: T, fileContent: string): void {
    const entry: CacheEntry<T> = {
      data,
      timestamp: Date.now(),
      fileHash: this.hash(fileContent),
    };
    this.cache.set(key, entry);
  }

  private hash(content: string): string {
    return createHash('sha256').update(content).digest('hex');
  }
}
