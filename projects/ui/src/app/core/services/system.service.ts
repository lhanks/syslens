import { Injectable, inject } from '@angular/core';
import { Observable, interval, switchMap, startWith, shareReplay, of, tap, concat, from } from 'rxjs';
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
  RestorePoint,
  SystemReport,
} from '../models/system.model';
import { save } from '@tauri-apps/plugin-dialog';
import { writeTextFile } from '@tauri-apps/plugin-fs';

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

  // Cached observables for static data
  private deviceInfoCache$: Observable<DeviceInfo> | null = null;
  private biosInfoCache$: Observable<BiosInfo> | null = null;
  private osInfoCache$: Observable<OsInfo> | null = null;

  /**
   * Get device identification information.
   * Returns cached data immediately if available, then fetches fresh data.
   */
  getDeviceInfo(): Observable<DeviceInfo> {
    if (!this.deviceInfoCache$) {
      const cached = this.cache.load<DeviceInfo>(CacheKeys.DEVICE_INFO);
      const fetch$ = this.tauri.invoke<DeviceInfo>('get_device_info').pipe(
        tap((data) => this.cache.save(CacheKeys.DEVICE_INFO, data))
      );

      this.deviceInfoCache$ = cached
        ? concat(of(cached), fetch$).pipe(shareReplay(1))
        : fetch$.pipe(shareReplay(1));
    }
    return this.deviceInfoCache$;
  }

  /**
   * Get BIOS/UEFI information.
   * Returns cached data immediately if available, then fetches fresh data.
   */
  getBiosInfo(): Observable<BiosInfo> {
    if (!this.biosInfoCache$) {
      const cached = this.cache.load<BiosInfo>(CacheKeys.BIOS_INFO);
      const fetch$ = this.tauri.invoke<BiosInfo>('get_bios_info').pipe(
        tap((data) => this.cache.save(CacheKeys.BIOS_INFO, data))
      );

      this.biosInfoCache$ = cached
        ? concat(of(cached), fetch$).pipe(shareReplay(1))
        : fetch$.pipe(shareReplay(1));
    }
    return this.biosInfoCache$;
  }

  /**
   * Get boot configuration.
   */
  getBootConfig(): Observable<BootConfig> {
    return this.tauri.invoke<BootConfig>('get_boot_config');
  }

  /**
   * Get operating system information.
   * Returns cached data immediately if available, then fetches fresh data.
   */
  getOsInfo(): Observable<OsInfo> {
    if (!this.osInfoCache$) {
      const cached = this.cache.load<OsInfo>(CacheKeys.OS_INFO);
      const fetch$ = this.tauri.invoke<OsInfo>('get_os_info').pipe(
        tap((data) => this.cache.save(CacheKeys.OS_INFO, data))
      );

      this.osInfoCache$ = cached
        ? concat(of(cached), fetch$).pipe(shareReplay(1))
        : fetch$.pipe(shareReplay(1));
    }
    return this.osInfoCache$;
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
   * Get system restore points.
   */
  getRestorePoints(): Observable<RestorePoint[]> {
    return this.tauri.invoke<RestorePoint[]>('get_restore_points');
  }

  /**
   * Clear cached data (both in-memory and persistent).
   */
  clearCache(): void {
    // Clear observable cache
    this.deviceInfoCache$ = null;
    this.biosInfoCache$ = null;
    this.osInfoCache$ = null;

    // Clear persistent cache
    this.cache.clear(CacheKeys.DEVICE_INFO);
    this.cache.clear(CacheKeys.BIOS_INFO);
    this.cache.clear(CacheKeys.OS_INFO);
  }

  /**
   * Generate a complete system report.
   */
  generateSystemReport(): Observable<SystemReport> {
    return this.tauri.invoke<SystemReport>('generate_system_report');
  }

  /**
   * Export system report to a JSON file.
   * Opens a save dialog and writes the report to the selected location.
   * @returns Observable that completes when export is done, or errors if cancelled/failed
   */
  exportSystemReport(): Observable<string | null> {
    return from(this.performExport());
  }

  private async performExport(): Promise<string | null> {
    // Generate the report
    const report = await this.tauri.invoke<SystemReport>('generate_system_report').toPromise();

    if (!report) {
      throw new Error('Failed to generate system report');
    }

    // Generate default filename with computer name and timestamp
    const timestamp = new Date().toISOString().replace(/[:.]/g, '-').slice(0, 19);
    const defaultName = `syslens-report-${report.deviceInfo.computerName}-${timestamp}.json`;

    // Show save dialog
    const filePath = await save({
      defaultPath: defaultName,
      filters: [
        { name: 'JSON', extensions: ['json'] },
        { name: 'All Files', extensions: ['*'] },
      ],
      title: 'Export System Report',
    });

    if (!filePath) {
      // User cancelled
      return null;
    }

    // Write the file
    const content = JSON.stringify(report, null, 2);
    await writeTextFile(filePath, content);

    return filePath;
  }
}
