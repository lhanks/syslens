import { Injectable, inject } from '@angular/core';
import { Observable, from, shareReplay, map } from 'rxjs';
import { TauriService } from './tauri.service';
import {
  DeviceDeepInfo,
  DeviceType,
  DatabaseStats,
  EnrichedDeviceInfo,
  EnrichmentSource,
  ImageCacheResponse,
  ImageCacheStats,
  CleanupResponse,
} from '../models/device-info.model';

/**
 * Service for retrieving deep device information with internet lookup.
 */
@Injectable({
  providedIn: 'root',
})
export class DeviceInfoService {
  private tauri = inject(TauriService);

  // Cache for device info requests
  private deviceCache = new Map<string, Observable<DeviceDeepInfo>>();

  /**
   * Get deep device information by device ID and type.
   * @param deviceId The device identifier
   * @param deviceType The type of device (Cpu, Gpu, Motherboard, Memory, Storage)
   * @param forceRefresh Force refresh from source (bypass cache)
   */
  getDeviceDeepInfo(
    deviceId: string,
    deviceType: DeviceType,
    forceRefresh = false
  ): Observable<DeviceDeepInfo> {
    const cacheKey = `${deviceType}-${deviceId}`;

    // Return cached observable if available and not forcing refresh
    if (!forceRefresh && this.deviceCache.has(cacheKey)) {
      return this.deviceCache.get(cacheKey)!;
    }

    const request$ = this.tauri
      .invoke<DeviceDeepInfo>('get_device_deep_info', {
        deviceId,
        deviceType,
        forceRefresh,
      })
      .pipe(shareReplay(1));

    // Cache the observable
    this.deviceCache.set(cacheKey, request$);

    return request$;
  }

  /**
   * Search for device information by manufacturer and model.
   * @param manufacturer Device manufacturer
   * @param model Device model
   * @param deviceType The type of device
   */
  searchDeviceInfo(
    manufacturer: string,
    model: string,
    deviceType: DeviceType
  ): Observable<DeviceDeepInfo | null> {
    return this.tauri.invoke<DeviceDeepInfo | null>('search_device_info', {
      manufacturer,
      model,
      deviceType,
    });
  }

  /**
   * Get all cached device information.
   */
  getCachedDevices(): Observable<DeviceDeepInfo[]> {
    return this.tauri.invoke<DeviceDeepInfo[]>('get_cached_devices');
  }

  /**
   * Clear cache for a specific device or all devices.
   * @param deviceId Optional device ID to clear
   * @param deviceType Optional device type to clear
   */
  clearCache(deviceId?: string, deviceType?: DeviceType): Observable<void> {
    // Clear local cache
    if (deviceId && deviceType) {
      this.deviceCache.delete(`${deviceType}-${deviceId}`);
    } else {
      this.deviceCache.clear();
    }

    return this.tauri.invoke<void>('clear_device_cache', {
      deviceId,
      deviceType,
    });
  }

  /**
   * Cleanup expired cache entries.
   * @returns Number of expired entries removed
   */
  cleanupExpiredCache(): Observable<number> {
    return this.tauri.invoke<number>('cleanup_device_cache');
  }

  /**
   * Get database statistics.
   */
  getDatabaseStats(): Observable<DatabaseStats> {
    return this.tauri.invoke<DatabaseStats>('get_device_database_stats');
  }

  /**
   * Check if device information is available.
   * @param deviceId The device identifier
   * @param deviceType The type of device
   */
  hasDeviceInfo(deviceId: string, deviceType: DeviceType): Observable<boolean> {
    return this.searchDeviceInfo('', deviceId, deviceType).pipe(
      map((result) => result !== null)
    );
  }

  /**
   * Get a human-readable label for the data source.
   */
  getSourceLabel(source: string): string {
    switch (source) {
      case 'LocalDatabase':
        return 'Local Database';
      case 'ManufacturerWebsite':
        return 'Manufacturer';
      case 'ThirdPartyDatabase':
        return 'Third-Party';
      case 'AiAgent':
        return 'AI Agent';
      case 'Cache':
        return 'Cached';
      default:
        return source;
    }
  }

  /**
   * Get a CSS class for the data source badge.
   */
  getSourceBadgeClass(source: string): string {
    switch (source) {
      case 'ManufacturerWebsite':
        return 'bg-green-500';
      case 'ThirdPartyDatabase':
        return 'bg-blue-500';
      case 'AiAgent':
        return 'bg-purple-500';
      case 'LocalDatabase':
        return 'bg-gray-500';
      case 'Cache':
        return 'bg-yellow-500';
      default:
        return 'bg-gray-500';
    }
  }

  /**
   * Format time since last update.
   */
  getTimeSinceUpdate(lastUpdated: string): string {
    const updated = new Date(lastUpdated);
    const now = new Date();
    const diffMs = now.getTime() - updated.getTime();
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

    if (diffDays === 0) return 'Today';
    if (diffDays === 1) return 'Yesterday';
    if (diffDays < 7) return `${diffDays} days ago`;
    if (diffDays < 30) return `${Math.floor(diffDays / 7)} weeks ago`;
    return `${Math.floor(diffDays / 30)} months ago`;
  }

  // ===========================================================================
  // Device Enrichment API (Phase 4)
  // ===========================================================================

  // Cache for enriched device requests
  private enrichmentCache = new Map<string, Observable<EnrichedDeviceInfo>>();

  /**
   * Enrich a device with comprehensive information from multiple sources.
   * This fetches product images, specifications, documentation, and driver info.
   * @param deviceType The type of device
   * @param manufacturer Device manufacturer
   * @param model Device model
   * @param forceRefresh Force refresh from sources (bypass cache)
   */
  enrichDevice(
    deviceType: DeviceType,
    manufacturer: string,
    model: string,
    forceRefresh = false
  ): Observable<EnrichedDeviceInfo> {
    const cacheKey = `enriched-${deviceType}-${manufacturer}-${model}`;

    // Return cached observable if available and not forcing refresh
    if (!forceRefresh && this.enrichmentCache.has(cacheKey)) {
      return this.enrichmentCache.get(cacheKey)!;
    }

    const request$ = this.tauri
      .invoke<EnrichedDeviceInfo>('enrich_device', {
        deviceType,
        manufacturer,
        model,
        forceRefresh,
      })
      .pipe(shareReplay(1));

    // Cache the observable
    this.enrichmentCache.set(cacheKey, request$);

    return request$;
  }

  /**
   * List available enrichment sources with their priorities.
   */
  listEnrichmentSources(): Observable<EnrichmentSource[]> {
    return this.tauri.invoke<EnrichmentSource[]>('list_enrichment_sources');
  }

  /**
   * Cleanup enrichment cache (images and metadata).
   * @param maxAgeDays Remove items older than this many days
   */
  cleanupEnrichmentCache(maxAgeDays: number): Observable<CleanupResponse> {
    return this.tauri.invoke<CleanupResponse>('cleanup_enrichment_cache', {
      maxAgeDays,
    });
  }

  // ===========================================================================
  // Image Cache API
  // ===========================================================================

  /**
   * Fetch and cache a device image from URL.
   * @param url The image URL to fetch
   */
  fetchDeviceImage(url: string): Observable<ImageCacheResponse> {
    return this.tauri.invoke<ImageCacheResponse>('fetch_device_image', { url });
  }

  /**
   * Fetch and cache a device image with a custom cache key.
   * @param url The image URL to fetch
   * @param cacheKey Custom cache key for the image
   */
  fetchDeviceImageWithKey(
    url: string,
    cacheKey: string
  ): Observable<ImageCacheResponse> {
    return this.tauri.invoke<ImageCacheResponse>('fetch_device_image_with_key', {
      url,
      cacheKey,
    });
  }

  /**
   * Get the cached path for an image by cache key.
   * @param cacheKey The cache key to look up
   */
  getCachedImagePath(cacheKey: string): Observable<string | null> {
    return this.tauri.invoke<string | null>('get_cached_image_path', {
      cacheKey,
    });
  }

  /**
   * Check if an image is cached.
   * @param cacheKey The cache key to check
   */
  isImageCached(cacheKey: string): Observable<boolean> {
    return this.tauri.invoke<boolean>('is_image_cached', { cacheKey });
  }

  /**
   * Generate a cache key for a device image.
   * @param deviceType Device type
   * @param manufacturer Manufacturer name
   * @param model Model name
   */
  generateDeviceImageCacheKey(
    deviceType: string,
    manufacturer: string,
    model: string
  ): Observable<string> {
    return this.tauri.invoke<string>('generate_device_image_cache_key', {
      deviceType,
      manufacturer,
      model,
    });
  }

  /**
   * Get image cache statistics.
   */
  getImageCacheStats(): Observable<ImageCacheStats> {
    return this.tauri.invoke<ImageCacheStats>('get_image_cache_stats');
  }

  /**
   * Cleanup old cached images.
   * @param maxAgeDays Remove images older than this many days
   */
  cleanupImageCache(maxAgeDays: number): Observable<number> {
    return this.tauri.invoke<number>('cleanup_image_cache', { maxAgeDays });
  }

  /**
   * Clear the local enrichment cache.
   */
  clearEnrichmentCache(): void {
    this.enrichmentCache.clear();
  }
}
