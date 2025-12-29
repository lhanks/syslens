import { Injectable, inject } from '@angular/core';
import { Observable, from, shareReplay, map } from 'rxjs';
import { TauriService } from './tauri.service';
import {
  DeviceDeepInfo,
  DeviceType,
  DatabaseStats,
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
}
