import { Injectable, inject } from '@angular/core';
import { Observable, interval, switchMap, startWith, shareReplay } from 'rxjs';
import { TauriService } from './tauri.service';
import {
  DeviceInfo,
  BiosInfo,
  BootConfig,
  OsInfo,
  SystemUptime,
  DomainInfo,
  UserInfo
} from '../models/system.model';

/**
 * Service for retrieving system configuration information.
 */
@Injectable({
  providedIn: 'root'
})
export class SystemService {
  private tauri = inject(TauriService);

  // Cached observables for static data
  private deviceInfoCache$: Observable<DeviceInfo> | null = null;
  private biosInfoCache$: Observable<BiosInfo> | null = null;
  private osInfoCache$: Observable<OsInfo> | null = null;

  /**
   * Get device identification information.
   */
  getDeviceInfo(): Observable<DeviceInfo> {
    if (!this.deviceInfoCache$) {
      this.deviceInfoCache$ = this.tauri.invoke<DeviceInfo>('get_device_info').pipe(
        shareReplay(1)
      );
    }
    return this.deviceInfoCache$;
  }

  /**
   * Get BIOS/UEFI information.
   */
  getBiosInfo(): Observable<BiosInfo> {
    if (!this.biosInfoCache$) {
      this.biosInfoCache$ = this.tauri.invoke<BiosInfo>('get_bios_info').pipe(
        shareReplay(1)
      );
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
   */
  getOsInfo(): Observable<OsInfo> {
    if (!this.osInfoCache$) {
      this.osInfoCache$ = this.tauri.invoke<OsInfo>('get_os_info').pipe(
        shareReplay(1)
      );
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
   * Clear cached data (useful when system config changes).
   */
  clearCache(): void {
    this.deviceInfoCache$ = null;
    this.biosInfoCache$ = null;
    this.osInfoCache$ = null;
  }
}
