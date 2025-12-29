import { Injectable, inject } from '@angular/core';
import {
  Observable,
  interval,
  switchMap,
  startWith,
  shareReplay,
  combineLatest,
  map,
  BehaviorSubject,
  of,
  tap,
  catchError,
} from 'rxjs';
import { TauriService } from './tauri.service';
import { DataCacheService, CacheKeys } from './data-cache.service';
import {
  CpuInfo,
  CpuMetrics,
  MemoryInfo,
  MemoryMetrics,
  GpuInfo,
  GpuMetrics,
  MotherboardInfo,
  UsbDevice,
  AudioDevice,
  Monitor,
} from '../models/hardware.model';

/**
 * Service for retrieving hardware configuration and real-time metrics.
 * Uses cache-first pattern: returns cached data immediately, then refreshes in background.
 */
@Injectable({
  providedIn: 'root',
})
export class HardwareService {
  private tauri = inject(TauriService);
  private cache = inject(DataCacheService);

  // BehaviorSubjects for cache-first pattern
  private cpuInfo$ = new BehaviorSubject<CpuInfo | null>(null);
  private memoryInfo$ = new BehaviorSubject<MemoryInfo | null>(null);
  private gpuInfo$ = new BehaviorSubject<GpuInfo[] | null>(null);
  private motherboardInfo$ = new BehaviorSubject<MotherboardInfo | null>(null);
  private monitors$ = new BehaviorSubject<Monitor[] | null>(null);

  // Track if fresh data has been fetched
  private cpuInfoFetched = false;
  private memoryInfoFetched = false;
  private gpuInfoFetched = false;
  private motherboardInfoFetched = false;
  private monitorsFetched = false;

  constructor() {
    this.loadCachedData();
  }

  /**
   * Load all cached hardware data on startup
   */
  private loadCachedData(): void {
    const cpuInfo = this.cache.load<CpuInfo>(CacheKeys.CPU_INFO);
    if (cpuInfo) this.cpuInfo$.next(cpuInfo);

    const memoryInfo = this.cache.load<MemoryInfo>(CacheKeys.MEMORY_INFO);
    if (memoryInfo) this.memoryInfo$.next(memoryInfo);

    const gpuInfo = this.cache.load<GpuInfo[]>(CacheKeys.GPU_INFO);
    if (gpuInfo) this.gpuInfo$.next(gpuInfo);

    const motherboardInfo = this.cache.load<MotherboardInfo>(CacheKeys.MOTHERBOARD_INFO);
    if (motherboardInfo) this.motherboardInfo$.next(motherboardInfo);

    const monitors = this.cache.load<Monitor[]>(CacheKeys.MONITORS);
    if (monitors) this.monitors$.next(monitors);
  }

  // --- CPU ---

  /**
   * Get CPU static information.
   * Returns cached data immediately, then fetches fresh data in background.
   */
  getCpuInfo(): Observable<CpuInfo> {
    // Trigger background fetch if not already done
    if (!this.cpuInfoFetched) {
      this.cpuInfoFetched = true;
      this.tauri
        .invoke<CpuInfo>('get_cpu_info')
        .pipe(
          tap((data) => {
            this.cache.save(CacheKeys.CPU_INFO, data);
            this.cpuInfo$.next(data);
          }),
          catchError((err) => {
            console.error('Failed to fetch CPU info:', err);
            return of(null);
          })
        )
        .subscribe();
    }

    // Return observable that emits cached data and updates
    return this.cpuInfo$.asObservable().pipe(
      // Filter out null values (no cache and not yet fetched)
      switchMap((data) => (data ? of(data) : this.tauri.invoke<CpuInfo>('get_cpu_info'))),
      shareReplay(1)
    );
  }

  /**
   * Get real-time CPU metrics.
   */
  getCpuMetrics(): Observable<CpuMetrics> {
    return this.tauri.invoke<CpuMetrics>('get_cpu_metrics');
  }

  /**
   * Get CPU metrics with polling (every second).
   */
  getCpuMetricsPolling(): Observable<CpuMetrics> {
    return interval(1000).pipe(
      startWith(0),
      switchMap(() => this.getCpuMetrics())
    );
  }

  // --- Memory ---

  /**
   * Get memory static information.
   * Returns cached data immediately, then fetches fresh data in background.
   */
  getMemoryInfo(): Observable<MemoryInfo> {
    if (!this.memoryInfoFetched) {
      this.memoryInfoFetched = true;
      this.tauri
        .invoke<MemoryInfo>('get_memory_info')
        .pipe(
          tap((data) => {
            this.cache.save(CacheKeys.MEMORY_INFO, data);
            this.memoryInfo$.next(data);
          }),
          catchError((err) => {
            console.error('Failed to fetch memory info:', err);
            return of(null);
          })
        )
        .subscribe();
    }

    return this.memoryInfo$.asObservable().pipe(
      switchMap((data) => (data ? of(data) : this.tauri.invoke<MemoryInfo>('get_memory_info'))),
      shareReplay(1)
    );
  }

  /**
   * Get real-time memory metrics.
   */
  getMemoryMetrics(): Observable<MemoryMetrics> {
    return this.tauri.invoke<MemoryMetrics>('get_memory_metrics');
  }

  /**
   * Get memory metrics with polling (every second).
   */
  getMemoryMetricsPolling(): Observable<MemoryMetrics> {
    return interval(1000).pipe(
      startWith(0),
      switchMap(() => this.getMemoryMetrics())
    );
  }

  // --- GPU ---

  /**
   * Get GPU static information for all GPUs.
   * Returns cached data immediately, then fetches fresh data in background.
   */
  getGpuInfo(): Observable<GpuInfo[]> {
    if (!this.gpuInfoFetched) {
      this.gpuInfoFetched = true;
      this.tauri
        .invoke<GpuInfo[]>('get_gpu_info')
        .pipe(
          tap((data) => {
            this.cache.save(CacheKeys.GPU_INFO, data);
            this.gpuInfo$.next(data);
          }),
          catchError((err) => {
            console.error('Failed to fetch GPU info:', err);
            return of(null);
          })
        )
        .subscribe();
    }

    return this.gpuInfo$.asObservable().pipe(
      switchMap((data) => (data ? of(data) : this.tauri.invoke<GpuInfo[]>('get_gpu_info'))),
      shareReplay(1)
    );
  }

  /**
   * Get real-time GPU metrics for all GPUs.
   */
  getGpuMetrics(): Observable<GpuMetrics[]> {
    return this.tauri.invoke<GpuMetrics[]>('get_gpu_metrics');
  }

  /**
   * Get GPU metrics with polling (every 2 seconds).
   */
  getGpuMetricsPolling(): Observable<GpuMetrics[]> {
    return interval(2000).pipe(
      startWith(0),
      switchMap(() => this.getGpuMetrics())
    );
  }

  // --- Motherboard ---

  /**
   * Get motherboard information.
   * Returns cached data immediately, then fetches fresh data in background.
   */
  getMotherboardInfo(): Observable<MotherboardInfo> {
    if (!this.motherboardInfoFetched) {
      this.motherboardInfoFetched = true;
      this.tauri
        .invoke<MotherboardInfo>('get_motherboard_info')
        .pipe(
          tap((data) => {
            this.cache.save(CacheKeys.MOTHERBOARD_INFO, data);
            this.motherboardInfo$.next(data);
          }),
          catchError((err) => {
            console.error('Failed to fetch motherboard info:', err);
            return of(null);
          })
        )
        .subscribe();
    }

    return this.motherboardInfo$.asObservable().pipe(
      switchMap((data) =>
        data ? of(data) : this.tauri.invoke<MotherboardInfo>('get_motherboard_info')
      ),
      shareReplay(1)
    );
  }

  // --- Peripherals ---

  /**
   * Get connected USB devices.
   */
  getUsbDevices(): Observable<UsbDevice[]> {
    return this.tauri.invoke<UsbDevice[]>('get_usb_devices');
  }

  /**
   * Get audio devices.
   */
  getAudioDevices(): Observable<AudioDevice[]> {
    return this.tauri.invoke<AudioDevice[]>('get_audio_devices');
  }

  /**
   * Get connected monitors.
   * Returns cached data immediately, then fetches fresh data in background.
   */
  getMonitors(): Observable<Monitor[]> {
    if (!this.monitorsFetched) {
      this.monitorsFetched = true;
      this.tauri
        .invoke<Monitor[]>('get_monitors')
        .pipe(
          tap((data) => {
            this.cache.save(CacheKeys.MONITORS, data);
            this.monitors$.next(data);
          }),
          catchError((err) => {
            console.error('Failed to fetch monitors:', err);
            return of(null);
          })
        )
        .subscribe();
    }

    return this.monitors$.asObservable().pipe(
      switchMap((data) => (data ? of(data) : this.tauri.invoke<Monitor[]>('get_monitors'))),
      shareReplay(1)
    );
  }

  /**
   * Listen for USB device changes.
   */
  onUsbDeviceChange(): Observable<UsbDevice[]> {
    return this.tauri.listen<UsbDevice[]>('usb-device-changed');
  }

  /**
   * Get combined hardware summary for dashboard.
   */
  getHardwareSummary(): Observable<{
    cpu: CpuInfo;
    cpuMetrics: CpuMetrics;
    memory: MemoryInfo;
    memoryMetrics: MemoryMetrics;
    gpus: GpuInfo[];
  }> {
    return combineLatest([
      this.getCpuInfo(),
      this.getCpuMetrics(),
      this.getMemoryInfo(),
      this.getMemoryMetrics(),
      this.getGpuInfo()
    ]).pipe(
      map(([cpu, cpuMetrics, memory, memoryMetrics, gpus]) => ({
        cpu,
        cpuMetrics,
        memory,
        memoryMetrics,
        gpus
      }))
    );
  }

  /**
   * Clear cached hardware data (both in-memory and persistent).
   */
  clearCache(): void {
    // Reset fetch flags
    this.cpuInfoFetched = false;
    this.memoryInfoFetched = false;
    this.gpuInfoFetched = false;
    this.motherboardInfoFetched = false;
    this.monitorsFetched = false;

    // Clear in-memory cache
    this.cpuInfo$.next(null);
    this.memoryInfo$.next(null);
    this.gpuInfo$.next(null);
    this.motherboardInfo$.next(null);
    this.monitors$.next(null);

    // Clear persistent cache
    this.cache.clear(CacheKeys.CPU_INFO);
    this.cache.clear(CacheKeys.MEMORY_INFO);
    this.cache.clear(CacheKeys.GPU_INFO);
    this.cache.clear(CacheKeys.MOTHERBOARD_INFO);
    this.cache.clear(CacheKeys.MONITORS);
  }
}
