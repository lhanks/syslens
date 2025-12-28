import { Injectable, inject } from '@angular/core';
import { Observable, interval, switchMap, startWith, shareReplay, combineLatest, map } from 'rxjs';
import { TauriService } from './tauri.service';
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
  Monitor
} from '../models/hardware.model';

/**
 * Service for retrieving hardware configuration and real-time metrics.
 */
@Injectable({
  providedIn: 'root'
})
export class HardwareService {
  private tauri = inject(TauriService);

  // Cached observables for static hardware data
  private cpuInfoCache$: Observable<CpuInfo> | null = null;
  private memoryInfoCache$: Observable<MemoryInfo> | null = null;
  private gpuInfoCache$: Observable<GpuInfo[]> | null = null;
  private motherboardCache$: Observable<MotherboardInfo> | null = null;

  // --- CPU ---

  /**
   * Get CPU static information.
   */
  getCpuInfo(): Observable<CpuInfo> {
    if (!this.cpuInfoCache$) {
      this.cpuInfoCache$ = this.tauri.invoke<CpuInfo>('get_cpu_info').pipe(
        shareReplay(1)
      );
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
   */
  getMemoryInfo(): Observable<MemoryInfo> {
    if (!this.memoryInfoCache$) {
      this.memoryInfoCache$ = this.tauri.invoke<MemoryInfo>('get_memory_info').pipe(
        shareReplay(1)
      );
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
   */
  getGpuInfo(): Observable<GpuInfo[]> {
    if (!this.gpuInfoCache$) {
      this.gpuInfoCache$ = this.tauri.invoke<GpuInfo[]>('get_gpu_info').pipe(
        shareReplay(1)
      );
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
   */
  getMotherboardInfo(): Observable<MotherboardInfo> {
    if (!this.motherboardCache$) {
      this.motherboardCache$ = this.tauri.invoke<MotherboardInfo>('get_motherboard_info').pipe(
        shareReplay(1)
      );
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
   */
  getMonitors(): Observable<Monitor[]> {
    return this.tauri.invoke<Monitor[]>('get_monitors');
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
   * Clear cached hardware data.
   */
  clearCache(): void {
    this.cpuInfoCache$ = null;
    this.memoryInfoCache$ = null;
    this.gpuInfoCache$ = null;
    this.motherboardCache$ = null;
  }
}
