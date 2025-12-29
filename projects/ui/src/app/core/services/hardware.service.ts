import { Injectable, inject } from '@angular/core';
import {
  Observable,
  interval,
  switchMap,
  startWith,
  shareReplay,
  combineLatest,
  map,
  of,
  tap,
  concat,
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

  // Cached observables for static hardware data
  private cpuInfoCache$: Observable<CpuInfo> | null = null;
  private memoryInfoCache$: Observable<MemoryInfo> | null = null;
  private gpuInfoCache$: Observable<GpuInfo[]> | null = null;
  private motherboardCache$: Observable<MotherboardInfo> | null = null;
  private monitorsCache$: Observable<Monitor[]> | null = null;

  // --- CPU ---

  /**
   * Get CPU static information.
   * Returns cached data immediately if available, then fetches fresh data.
   */
  getCpuInfo(): Observable<CpuInfo> {
    if (!this.cpuInfoCache$) {
      const cached = this.cache.load<CpuInfo>(CacheKeys.CPU_INFO);
      const fetch$ = this.tauri.invoke<CpuInfo>('get_cpu_info').pipe(
        tap((data) => this.cache.save(CacheKeys.CPU_INFO, data))
      );

      this.cpuInfoCache$ = cached ? concat(of(cached), fetch$).pipe(shareReplay(1)) : fetch$.pipe(shareReplay(1));
    }
    return this.cpuInfoCache$;
  }

  /**
   * Get real-time CPU metrics.
   */
  getCpuMetrics(): Observable<CpuMetrics> {
    return this.tauri.invoke<CpuMetrics>('get_cpu_metrics');
  }

  /**
   * Get CPU metrics with polling (every 2 seconds for performance).
   */
  getCpuMetricsPolling(): Observable<CpuMetrics> {
    return interval(2000).pipe(
      startWith(0),
      switchMap(() => this.getCpuMetrics())
    );
  }

  // --- Memory ---

  /**
   * Get memory static information.
   * Returns cached data immediately if available, then fetches fresh data.
   */
  getMemoryInfo(): Observable<MemoryInfo> {
    if (!this.memoryInfoCache$) {
      const cached = this.cache.load<MemoryInfo>(CacheKeys.MEMORY_INFO);
      const fetch$ = this.tauri.invoke<MemoryInfo>('get_memory_info').pipe(
        tap((data) => this.cache.save(CacheKeys.MEMORY_INFO, data))
      );

      this.memoryInfoCache$ = cached
        ? concat(of(cached), fetch$).pipe(shareReplay(1))
        : fetch$.pipe(shareReplay(1));
    }
    return this.memoryInfoCache$;
  }

  /**
   * Get real-time memory metrics.
   */
  getMemoryMetrics(): Observable<MemoryMetrics> {
    return this.tauri.invoke<MemoryMetrics>('get_memory_metrics');
  }

  /**
   * Get memory metrics with polling (every 2 seconds for performance).
   */
  getMemoryMetricsPolling(): Observable<MemoryMetrics> {
    return interval(2000).pipe(
      startWith(0),
      switchMap(() => this.getMemoryMetrics())
    );
  }

  // --- GPU ---

  /**
   * Get GPU static information for all GPUs.
   * Returns cached data immediately if available, then fetches fresh data.
   */
  getGpuInfo(): Observable<GpuInfo[]> {
    if (!this.gpuInfoCache$) {
      const cached = this.cache.load<GpuInfo[]>(CacheKeys.GPU_INFO);
      const fetch$ = this.tauri.invoke<GpuInfo[]>('get_gpu_info').pipe(
        tap((data) => this.cache.save(CacheKeys.GPU_INFO, data))
      );

      this.gpuInfoCache$ = cached
        ? concat(of(cached), fetch$).pipe(shareReplay(1))
        : fetch$.pipe(shareReplay(1));
    }
    return this.gpuInfoCache$;
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
   * Returns cached data immediately if available, then fetches fresh data.
   */
  getMotherboardInfo(): Observable<MotherboardInfo> {
    if (!this.motherboardCache$) {
      const cached = this.cache.load<MotherboardInfo>(CacheKeys.MOTHERBOARD_INFO);
      const fetch$ = this.tauri.invoke<MotherboardInfo>('get_motherboard_info').pipe(
        tap((data) => this.cache.save(CacheKeys.MOTHERBOARD_INFO, data))
      );

      this.motherboardCache$ = cached
        ? concat(of(cached), fetch$).pipe(shareReplay(1))
        : fetch$.pipe(shareReplay(1));
    }
    return this.motherboardCache$;
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
   * Returns cached data immediately if available, then fetches fresh data.
   */
  getMonitors(): Observable<Monitor[]> {
    if (!this.monitorsCache$) {
      const cached = this.cache.load<Monitor[]>(CacheKeys.MONITORS);
      const fetch$ = this.tauri.invoke<Monitor[]>('get_monitors').pipe(
        tap((data) => this.cache.save(CacheKeys.MONITORS, data))
      );

      this.monitorsCache$ = cached
        ? concat(of(cached), fetch$).pipe(shareReplay(1))
        : fetch$.pipe(shareReplay(1));
    }
    return this.monitorsCache$;
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
      this.getGpuInfo(),
    ]).pipe(
      map(([cpu, cpuMetrics, memory, memoryMetrics, gpus]) => ({
        cpu,
        cpuMetrics,
        memory,
        memoryMetrics,
        gpus,
      }))
    );
  }

  /**
   * Clear cached hardware data (both in-memory and persistent).
   */
  clearCache(): void {
    // Clear observable cache
    this.cpuInfoCache$ = null;
    this.memoryInfoCache$ = null;
    this.gpuInfoCache$ = null;
    this.motherboardCache$ = null;
    this.monitorsCache$ = null;

    // Clear persistent cache
    this.cache.clear(CacheKeys.CPU_INFO);
    this.cache.clear(CacheKeys.MEMORY_INFO);
    this.cache.clear(CacheKeys.GPU_INFO);
    this.cache.clear(CacheKeys.MOTHERBOARD_INFO);
    this.cache.clear(CacheKeys.MONITORS);
  }
}
