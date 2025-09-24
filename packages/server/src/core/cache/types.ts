export interface CacheEntry<T> {
  data: T;
  timestamp: number;
  fileHash: string; // A hash of the file content when the data was cached
}
