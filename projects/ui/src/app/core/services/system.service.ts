import { Injectable, inject } from '@angular/core';
import {
  Observable,
  interval,
  switchMap,
  startWith,
  shareReplay,
  BehaviorSubject,
  of,
  tap,
  catchError,
} from 'rxjs';
import { TauriService } from './tauri.service';
import { DataCacheService, CacheKeys } from './data-cache.service';
import {
  DeviceInfo,
  BiosInfo,
  BootConfig,
  OsInfo,
  SystemUptime,
  DomainInfo,
  UserInfo,
} from '../models/system.model';

/**
 * Service for retrieving system configuration information.
 * Uses cache-first pattern: returns cached data immediately, then refreshes in background.
 */
@Injectable({
  providedIn: 'root',
})
export class SystemService {
  private tauri = inject(TauriService);
  private cache = inject(DataCacheService);

  // BehaviorSubjects for cache-first pattern
  private deviceInfo$ = new BehaviorSubject<DeviceInfo | null>(null);
  private biosInfo$ = new BehaviorSubject<BiosInfo | null>(null);
  private osInfo$ = new BehaviorSubject<OsInfo | null>(null);

  // Track if fresh data has been fetched
  private deviceInfoFetched = false;
  private biosInfoFetched = false;
  private osInfoFetched = false;

  constructor() {
    this.loadCachedData();
  }

  /**
   * Load all cached system data on startup
   */
  private loadCachedData(): void {
    const deviceInfo = this.cache.load<DeviceInfo>(CacheKeys.DEVICE_INFO);
    if (deviceInfo) this.deviceInfo$.next(deviceInfo);

    const biosInfo = this.cache.load<BiosInfo>(CacheKeys.BIOS_INFO);
    if (biosInfo) this.biosInfo$.next(biosInfo);

    const osInfo = this.cache.load<OsInfo>(CacheKeys.OS_INFO);
    if (osInfo) this.osInfo$.next(osInfo);
  }

  /**
   * Get device identification information.
   * Returns cached data immediately, then fetches fresh data in background.
   */
  getDeviceInfo(): Observable<DeviceInfo> {
    if (!this.deviceInfoFetched) {
      this.deviceInfoFetched = true;
      this.tauri
        .invoke<DeviceInfo>('get_device_info')
        .pipe(
          tap((data) => {
            this.cache.save(CacheKeys.DEVICE_INFO, data);
            this.deviceInfo$.next(data);
          }),
          catchError((err) => {
            console.error('Failed to fetch device info:', err);
            return of(null);
          })
        )
        .subscribe();
    }

    return this.deviceInfo$.asObservable().pipe(
      switchMap((data) => (data ? of(data) : this.tauri.invoke<DeviceInfo>('get_device_info'))),
      shareReplay(1)
    );
  }

  /**
   * Get BIOS/UEFI information.
   * Returns cached data immediately, then fetches fresh data in background.
   */
  getBiosInfo(): Observable<BiosInfo> {
    if (!this.biosInfoFetched) {
      this.biosInfoFetched = true;
      this.tauri
        .invoke<BiosInfo>('get_bios_info')
        .pipe(
          tap((data) => {
            this.cache.save(CacheKeys.BIOS_INFO, data);
            this.biosInfo$.next(data);
          }),
          catchError((err) => {
            console.error('Failed to fetch BIOS info:', err);
            return of(null);
          })
        )
        .subscribe();
    }

    return this.biosInfo$.asObservable().pipe(
      switchMap((data) => (data ? of(data) : this.tauri.invoke<BiosInfo>('get_bios_info'))),
      shareReplay(1)
    );
  }

  /**
   * Get boot configuration.
   */
  getBootConfig(): Observable<BootConfig> {
    return this.tauri.invoke<BootConfig>('get_boot_config');
  }

  /**
   * Get operating system information.
   * Returns cached data immediately, then fetches fresh data in background.
   */
  getOsInfo(): Observable<OsInfo> {
    if (!this.osInfoFetched) {
      this.osInfoFetched = true;
      this.tauri
        .invoke<OsInfo>('get_os_info')
        .pipe(
          tap((data) => {
            this.cache.save(CacheKeys.OS_INFO, data);
            this.osInfo$.next(data);
          }),
          catchError((err) => {
            console.error('Failed to fetch OS info:', err);
            return of(null);
          })
        )
        .subscribe();
    }

    return this.osInfo$.asObservable().pipe(
      switchMap((data) => (data ? of(data) : this.tauri.invoke<OsInfo>('get_os_info'))),
      shareReplay(1)
    );
  }

  /**
   * Get system uptime with real-time updates (every second).
   */
  getUptime(): Observable<SystemUptime> {
    return interval(1000).pipe(
      startWith(0),
      switchMap(() => this.tauri.invoke<SystemUptime>('get_uptime'))
    );
  }

  /**
   * Get uptime once (no polling).
   */
  getUptimeOnce(): Observable<SystemUptime> {
    return this.tauri.invoke<SystemUptime>('get_uptime');
  }

  /**
   * Get domain/workgroup information.
   */
  getDomainInfo(): Observable<DomainInfo> {
    return this.tauri.invoke<DomainInfo>('get_domain_info');
  }

  /**
   * Get current user information.
   */
  getUserInfo(): Observable<UserInfo> {
    return this.tauri.invoke<UserInfo>('get_user_info');
  }

  /**
   * Clear cached data (both in-memory and persistent).
   */
  clearCache(): void {
    // Reset fetch flags
    this.deviceInfoFetched = false;
    this.biosInfoFetched = false;
    this.osInfoFetched = false;

    // Clear in-memory cache
    this.deviceInfo$.next(null);
    this.biosInfo$.next(null);
    this.osInfo$.next(null);

    // Clear persistent cache
    this.cache.clear(CacheKeys.DEVICE_INFO);
    this.cache.clear(CacheKeys.BIOS_INFO);
    this.cache.clear(CacheKeys.OS_INFO);
  }
}
