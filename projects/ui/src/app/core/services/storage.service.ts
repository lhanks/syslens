import { Injectable, inject } from '@angular/core';
import { Observable, interval, switchMap, startWith, shareReplay, of, tap, concat } from 'rxjs';
import { TauriService } from './tauri.service';
import { DataCacheService, CacheKeys } from './data-cache.service';
import {
  PhysicalDisk,
  Partition,
  Volume,
  DiskHealth,
  DiskPerformance,
  NetworkDrive,
} from '../models/storage.model';

/**
 * Service for retrieving storage configuration and metrics.
 * Uses cache-first pattern: returns cached data immediately, then refreshes in background.
 */
@Injectable({
  providedIn: 'root',
})
export class StorageService {
  private tauri = inject(TauriService);
  private cache = inject(DataCacheService);

  // Cached observable for static data
  private disksCache$: Observable<PhysicalDisk[]> | null = null;

  // --- Physical Disks ---

  /**
   * Get all physical disks.
   * Returns cached data immediately if available, then fetches fresh data.
   */
  getPhysicalDisks(): Observable<PhysicalDisk[]> {
    if (!this.disksCache$) {
      const cached = this.cache.load<PhysicalDisk[]>(CacheKeys.STORAGE_DEVICES);
      const fetch$ = this.tauri.invoke<PhysicalDisk[]>('get_physical_disks').pipe(
        tap((data) => this.cache.save(CacheKeys.STORAGE_DEVICES, data))
      );

      this.disksCache$ = cached
        ? concat(of(cached), fetch$).pipe(shareReplay(1))
        : fetch$.pipe(shareReplay(1));
    }
    return this.disksCache$;
  }

  /**
   * Get partitions for a specific disk.
   * @param diskId - The disk device ID
   */
  getPartitions(diskId: number): Observable<Partition[]> {
    return this.tauri.invoke<Partition[]>('get_partitions', { diskId });
  }

  // --- Volumes ---

  /**
   * Get all volumes.
   */
  getVolumes(): Observable<Volume[]> {
    return this.tauri.invoke<Volume[]>('get_volumes');
  }

  /**
   * Get volumes with polling (every 30 seconds).
   */
  getVolumesPolling(): Observable<Volume[]> {
    return interval(30000).pipe(
      startWith(0),
      switchMap(() => this.getVolumes()),
      shareReplay(1)
    );
  }

  // --- Health & Performance ---

  /**
   * Get disk health information (S.M.A.R.T. data).
   * @param diskId - The disk device ID
   */
  getDiskHealth(diskId: number): Observable<DiskHealth> {
    return this.tauri.invoke<DiskHealth>('get_disk_health', { diskId });
  }

  /**
   * Get real-time disk performance metrics.
   */
  getDiskPerformance(): Observable<DiskPerformance[]> {
    return this.tauri.invoke<DiskPerformance[]>('get_disk_performance');
  }

  /**
   * Get disk performance with polling (every second).
   */
  getDiskPerformancePolling(): Observable<DiskPerformance[]> {
    return interval(1000).pipe(
      startWith(0),
      switchMap(() => this.getDiskPerformance())
    );
  }

  // --- Network Drives ---

  /**
   * Get mapped network drives.
   */
  getNetworkDrives(): Observable<NetworkDrive[]> {
    return this.tauri.invoke<NetworkDrive[]>('get_network_drives');
  }

  // --- Events ---

  /**
   * Listen for storage device changes.
   */
  onStorageChange(): Observable<void> {
    return this.tauri.listen<void>('storage-changed');
  }

  /**
   * Clear cached storage data (both in-memory and persistent).
   */
  clearCache(): void {
    // Clear observable cache
    this.disksCache$ = null;

    // Clear persistent cache
    this.cache.clear(CacheKeys.STORAGE_DEVICES);
  }

  /**
   * Refresh storage data (clear cache and fetch new data).
   */
  refresh(): Observable<PhysicalDisk[]> {
    this.clearCache();
    return this.getPhysicalDisks();
  }
}
