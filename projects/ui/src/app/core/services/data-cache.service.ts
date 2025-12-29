import { Injectable } from '@angular/core';

/**
 * Service for persisting static hardware/system data to localStorage.
 * Provides instant data on startup while fresh data is fetched in background.
 */

interface CacheEntry<T> {
  data: T;
  timestamp: number;
}

interface CacheMetadata {
  version: string;
  lastUpdated: number;
}

const CACHE_VERSION = '1.0';
const CACHE_PREFIX = 'syslens_cache_';
const METADATA_KEY = 'syslens_cache_metadata';

// Cache keys for different data types
export const CacheKeys = {
  CPU_INFO: 'cpu_info',
  GPU_INFO: 'gpu_info',
  MEMORY_INFO: 'memory_info',
  MOTHERBOARD_INFO: 'motherboard_info',
  MONITORS: 'monitors',
  DEVICE_INFO: 'device_info',
  BIOS_INFO: 'bios_info',
  OS_INFO: 'os_info',
  NETWORK_ADAPTERS: 'network_adapters',
  STORAGE_DEVICES: 'storage_devices',
} as const;

export type CacheKey = (typeof CacheKeys)[keyof typeof CacheKeys];

@Injectable({
  providedIn: 'root',
})
export class DataCacheService {
  private memoryCache = new Map<string, unknown>();

  constructor() {
    this.initializeCache();
  }

  /**
   * Initialize cache and handle version migrations
   */
  private initializeCache(): void {
    const metadata = this.getMetadata();

    if (!metadata || metadata.version !== CACHE_VERSION) {
      // Clear cache on version mismatch
      this.clearAll();
      this.saveMetadata({
        version: CACHE_VERSION,
        lastUpdated: Date.now(),
      });
    }
  }

  /**
   * Save data to cache (both memory and localStorage)
   */
  save<T>(key: CacheKey, data: T): void {
    const fullKey = CACHE_PREFIX + key;
    const entry: CacheEntry<T> = {
      data,
      timestamp: Date.now(),
    };

    // Save to memory cache for fast access
    this.memoryCache.set(key, data);

    // Persist to localStorage
    try {
      localStorage.setItem(fullKey, JSON.stringify(entry));
      this.updateMetadata();
    } catch (error) {
      // localStorage might be full or unavailable
      console.warn(`Failed to persist cache for ${key}:`, error);
    }
  }

  /**
   * Load data from cache (memory first, then localStorage)
   */
  load<T>(key: CacheKey): T | null {
    // Check memory cache first
    if (this.memoryCache.has(key)) {
      return this.memoryCache.get(key) as T;
    }

    // Fall back to localStorage
    const fullKey = CACHE_PREFIX + key;
    try {
      const stored = localStorage.getItem(fullKey);
      if (stored) {
        const entry: CacheEntry<T> = JSON.parse(stored);
        // Populate memory cache
        this.memoryCache.set(key, entry.data);
        return entry.data;
      }
    } catch (error) {
      console.warn(`Failed to load cache for ${key}:`, error);
    }

    return null;
  }

  /**
   * Check if cache entry exists and is not expired
   */
  has(key: CacheKey, maxAgeMs?: number): boolean {
    const fullKey = CACHE_PREFIX + key;
    try {
      const stored = localStorage.getItem(fullKey);
      if (!stored) return false;

      if (maxAgeMs !== undefined) {
        const entry: CacheEntry<unknown> = JSON.parse(stored);
        const age = Date.now() - entry.timestamp;
        return age < maxAgeMs;
      }

      return true;
    } catch {
      return false;
    }
  }

  /**
   * Get the timestamp of a cached entry
   */
  getTimestamp(key: CacheKey): number | null {
    const fullKey = CACHE_PREFIX + key;
    try {
      const stored = localStorage.getItem(fullKey);
      if (stored) {
        const entry: CacheEntry<unknown> = JSON.parse(stored);
        return entry.timestamp;
      }
    } catch {
      // Ignore parse errors
    }
    return null;
  }

  /**
   * Clear a specific cache entry
   */
  clear(key: CacheKey): void {
    const fullKey = CACHE_PREFIX + key;
    this.memoryCache.delete(key);
    localStorage.removeItem(fullKey);
  }

  /**
   * Clear all cached data
   */
  clearAll(): void {
    this.memoryCache.clear();

    // Remove all syslens cache entries from localStorage
    const keysToRemove: string[] = [];
    for (let i = 0; i < localStorage.length; i++) {
      const key = localStorage.key(i);
      if (key?.startsWith(CACHE_PREFIX)) {
        keysToRemove.push(key);
      }
    }
    keysToRemove.forEach((key) => localStorage.removeItem(key));
  }

  /**
   * Get cache statistics
   */
  getStats(): { entries: number; totalSize: number; lastUpdated: number | null } {
    let entries = 0;
    let totalSize = 0;

    for (let i = 0; i < localStorage.length; i++) {
      const key = localStorage.key(i);
      if (key?.startsWith(CACHE_PREFIX)) {
        entries++;
        const value = localStorage.getItem(key);
        if (value) {
          totalSize += value.length * 2; // UTF-16 characters are 2 bytes
        }
      }
    }

    const metadata = this.getMetadata();

    return {
      entries,
      totalSize,
      lastUpdated: metadata?.lastUpdated ?? null,
    };
  }

  /**
   * Load all cached data at once (for startup optimization)
   */
  loadAll(): Map<CacheKey, unknown> {
    const result = new Map<CacheKey, unknown>();

    for (const key of Object.values(CacheKeys)) {
      const data = this.load(key);
      if (data !== null) {
        result.set(key, data);
      }
    }

    return result;
  }

  private getMetadata(): CacheMetadata | null {
    try {
      const stored = localStorage.getItem(METADATA_KEY);
      if (stored) {
        return JSON.parse(stored);
      }
    } catch {
      // Ignore parse errors
    }
    return null;
  }

  private saveMetadata(metadata: CacheMetadata): void {
    try {
      localStorage.setItem(METADATA_KEY, JSON.stringify(metadata));
    } catch {
      // Ignore storage errors
    }
  }

  private updateMetadata(): void {
    const metadata = this.getMetadata() || { version: CACHE_VERSION, lastUpdated: 0 };
    metadata.lastUpdated = Date.now();
    this.saveMetadata(metadata);
  }
}
