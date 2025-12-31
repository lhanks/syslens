import { Injectable, inject, signal, computed, OnDestroy } from '@angular/core';
import { Subject, interval, forkJoin, of } from 'rxjs';
import { takeUntil, switchMap, startWith, catchError } from 'rxjs/operators';

import { HardwareService } from './hardware.service';
import { StorageService } from './storage.service';
import { NetworkService } from './network.service';
import { TauriService } from './tauri.service';
import { NetworkAdapter, AdapterStats, MemoryInfo, GpuMetrics, SelfMetrics } from '../models';

const MAX_HISTORY_POINTS = 60; // 60 seconds of history
const NETWORK_EMA_ALPHA = 0.3; // Smoothing factor for network speeds (0-1, lower = smoother)
const MAX_DECAY_RATE = 0.95; // How fast the smoothed max decreases (per update)

export interface AdapterTrafficHistory {
  adapterId: string;
  adapterName: string;
  downloadHistory: number[];
  uploadHistory: number[];
  downloadSpeed: number;
  uploadSpeed: number;
  maxSpeed: number;
}

/**
 * Service for continuous metrics collection and history tracking.
 * This service runs from app startup and records history regardless of which tab is active.
 */
@Injectable({
  providedIn: 'root'
})
export class MetricsHistoryService implements OnDestroy {
  private hardwareService = inject(HardwareService);
  private storageService = inject(StorageService);
  private networkService = inject(NetworkService);
  private tauriService = inject(TauriService);
  private destroy$ = new Subject<void>();

  // Pre-fill arrays with zeros for constant length from start
  private initialHistory = (): number[] => new Array(MAX_HISTORY_POINTS).fill(0);

  // History signals
  private _cpuHistory = signal<number[]>(this.initialHistory());
  private _memoryHistory = signal<number[]>(this.initialHistory());
  private _diskHistory = signal<number[]>(this.initialHistory());
  private _networkDownHistory = signal<number[]>(this.initialHistory());
  private _networkUpHistory = signal<number[]>(this.initialHistory());

  // Current values
  private _cpuUsage = signal(0);
  private _memoryUsedBytes = signal(0);
  private _memoryTotalBytes = signal(0);
  private _diskActivity = signal(0);
  private _networkDownSpeed = signal(0);
  private _networkUpSpeed = signal(0);
  private _gpuUsage = signal(0);
  private _gpuMetrics = signal<GpuMetrics[]>([]);

  // Self (Syslens process) metrics
  private _selfCpuUsage = signal(0);
  private _selfMemoryBytes = signal(0);

  // Network tracking state
  private activeAdapters: NetworkAdapter[] = [];
  private previousNetworkStats = new Map<string, { bytesReceived: number; bytesSent: number; timestamp: number }>();
  private memoryInfo: MemoryInfo | null = null;
  private isStarted = false;
  private _primaryIpAddress = signal<string | null>(null);

  // Per-adapter traffic history (for Network tab graphs)
  private _adapterTrafficHistory = signal<Map<string, AdapterTrafficHistory>>(new Map());

  // Smoothed network speeds (EMA filtered)
  private smoothedDownSpeed = 0;
  private smoothedUpSpeed = 0;

  // Smoothed max value for stable graph scaling
  private _smoothedNetworkMax = signal(1024); // Start with 1KB minimum

  // Public read-only accessors
  cpuHistory = computed(() => this._cpuHistory());
  memoryHistory = computed(() => this._memoryHistory());
  diskHistory = computed(() => this._diskHistory());
  networkDownHistory = computed(() => this._networkDownHistory());
  networkUpHistory = computed(() => this._networkUpHistory());

  cpuUsage = computed(() => this._cpuUsage());
  memoryUsedBytes = computed(() => this._memoryUsedBytes());
  memoryTotalBytes = computed(() => this._memoryTotalBytes());
  memoryUsage = computed(() => {
    const total = this._memoryTotalBytes();
    if (total === 0) return 0;
    return (this._memoryUsedBytes() / total) * 100;
  });
  diskActivity = computed(() => this._diskActivity());
  networkDownSpeed = computed(() => this._networkDownSpeed());
  networkUpSpeed = computed(() => this._networkUpSpeed());
  gpuUsage = computed(() => this._gpuUsage());
  gpuMetrics = computed(() => this._gpuMetrics());

  // Self (Syslens process) metrics
  selfCpuUsage = computed(() => this._selfCpuUsage());
  selfMemoryBytes = computed(() => this._selfMemoryBytes());

  // Use smoothed max for stable graph scaling (prevents jitter from scale changes)
  networkMaxSpeed = computed(() => this._smoothedNetworkMax());

  // Primary IP address from the first active adapter with IPv4
  primaryIpAddress = computed(() => this._primaryIpAddress());

  // Per-adapter traffic history for Network tab graphs
  adapterTrafficHistory = computed(() => this._adapterTrafficHistory());

  /**
   * Start continuous metrics polling.
   * Safe to call multiple times - only starts once.
   */
  start(): void {
    if (this.isStarted) return;
    this.isStarted = true;

    this.loadInitialData();
  }

  ngOnDestroy(): void {
    this.destroy$.next();
    this.destroy$.complete();
  }

  private loadInitialData(): void {
    // Load static data first
    forkJoin({
      memoryInfo: this.hardwareService.getMemoryInfo(),
      adapters: this.networkService.getNetworkAdapters()
    }).pipe(
      takeUntil(this.destroy$)
    ).subscribe(({ memoryInfo, adapters }) => {
      this.memoryInfo = memoryInfo;
      this._memoryTotalBytes.set(memoryInfo.totalBytes);
      this.updateAdapters(adapters);

      // Start polling after initial data is loaded
      this.startPolling();

      // Subscribe to adapter changes for dynamic updates
      this.subscribeToAdapterChanges();
    });
  }

  private subscribeToAdapterChanges(): void {
    // Listen for backend events (if implemented)
    this.networkService.onAdapterChange().pipe(
      takeUntil(this.destroy$)
    ).subscribe(adapters => {
      this.updateAdapters(adapters);
    });

    // Poll for adapter changes every 10 seconds to detect enable/disable
    // Use direct Tauri call to bypass NetworkService cache
    interval(10000).pipe(
      takeUntil(this.destroy$),
      switchMap(() => this.tauriService.invoke<NetworkAdapter[]>('get_network_adapters'))
    ).subscribe(adapters => {
      this.updateAdapters(adapters);
    });
  }

  private updateAdapters(adapters: NetworkAdapter[]): void {
    this.activeAdapters = adapters.filter(a => a.status === 'Up');

    // Extract primary IP address from first active adapter with IPv4
    const adapterWithIp = this.activeAdapters.find(a => a.ipv4Config?.address);
    this._primaryIpAddress.set(adapterWithIp?.ipv4Config?.address ?? null);

    // Clean up stats for adapters that no longer exist
    const activeIds = new Set(this.activeAdapters.map(a => a.id));
    for (const id of this.previousNetworkStats.keys()) {
      if (!activeIds.has(id)) {
        this.previousNetworkStats.delete(id);
      }
    }

    // Initialize traffic history for new adapters
    const currentHistory = this._adapterTrafficHistory();
    const newHistory = new Map(currentHistory);
    let changed = false;

    for (const adapter of this.activeAdapters) {
      if (!newHistory.has(adapter.id)) {
        newHistory.set(adapter.id, {
          adapterId: adapter.id,
          adapterName: adapter.name,
          downloadHistory: new Array(MAX_HISTORY_POINTS).fill(0),
          uploadHistory: new Array(MAX_HISTORY_POINTS).fill(0),
          downloadSpeed: 0,
          uploadSpeed: 0,
          maxSpeed: 1024 // Start with 1KB minimum
        });
        changed = true;
      }
    }

    // Remove history for adapters that no longer exist
    for (const id of newHistory.keys()) {
      if (!activeIds.has(id)) {
        newHistory.delete(id);
        changed = true;
      }
    }

    if (changed) {
      this._adapterTrafficHistory.set(newHistory);
    }
  }

  private startPolling(): void {
    // CPU metrics - every second
    this.hardwareService.getCpuMetricsPolling()
      .pipe(takeUntil(this.destroy$))
      .subscribe(metrics => {
        this._cpuUsage.set(metrics.totalUsage);
        this.pushToHistory(this._cpuHistory, metrics.totalUsage);
      });

    // Memory metrics - every second (store bytes for GB display on y-axis)
    this.hardwareService.getMemoryMetricsPolling()
      .pipe(takeUntil(this.destroy$))
      .subscribe(metrics => {
        this._memoryUsedBytes.set(metrics.inUseBytes);
        this.pushToHistory(this._memoryHistory, metrics.inUseBytes);
      });

    // Disk performance - every second
    this.storageService.getDiskPerformancePolling()
      .pipe(takeUntil(this.destroy$))
      .subscribe(perf => {
        if (perf.length > 0) {
          const maxActivity = Math.max(...perf.map(d => d.activeTimePercent));
          this._diskActivity.set(maxActivity);
          this.pushToHistory(this._diskHistory, maxActivity);
        }
      });

    // GPU metrics - every 2 seconds (matches hardware service polling)
    this.hardwareService.getGpuMetricsPolling()
      .pipe(takeUntil(this.destroy$))
      .subscribe(metrics => {
        this._gpuMetrics.set(metrics);
        // Use the primary (first) GPU usage for the sidebar display
        const primaryUsage = metrics.length > 0 ? metrics[0].usagePercent : 0;
        this._gpuUsage.set(primaryUsage);
      });

    // Self (Syslens process) metrics - every second
    interval(1000).pipe(
      startWith(0),
      takeUntil(this.destroy$),
      switchMap(() => this.tauriService.invoke<SelfMetrics>('get_self_metrics').pipe(
        catchError(() => of({ pid: 0, cpuUsage: 0, memoryBytes: 0, virtualMemoryBytes: 0 }))
      ))
    ).subscribe(metrics => {
      this._selfCpuUsage.set(metrics.cpuUsage);
      this._selfMemoryBytes.set(metrics.memoryBytes);
    });

    // Network stats - every second
    interval(1000).pipe(
      startWith(0),
      takeUntil(this.destroy$),
      switchMap(() => {
        if (this.activeAdapters.length === 0) {
          return of([]);
        }
        const statsRequests = this.activeAdapters.map(adapter =>
          this.networkService.getAdapterStats(adapter.id).pipe(
            catchError(() => of(null))
          )
        );
        return forkJoin(statsRequests);
      })
    ).subscribe(allStats => {
      this.calculateNetworkSpeeds(allStats.filter((s): s is AdapterStats => s !== null));
    });
  }

  private calculateNetworkSpeeds(currentStats: AdapterStats[]): void {
    let totalDownloadSpeed = 0;
    let totalUploadSpeed = 0;
    const now = Date.now();

    // Track per-adapter speeds for updating history
    const adapterSpeeds = new Map<string, { down: number; up: number }>();

    for (const stats of currentStats) {
      const previous = this.previousNetworkStats.get(stats.adapterId);

      if (previous) {
        const timeDeltaSeconds = (now - previous.timestamp) / 1000;
        if (timeDeltaSeconds > 0) {
          const downloadDelta = stats.bytesReceived - previous.bytesReceived;
          const uploadDelta = stats.bytesSent - previous.bytesSent;

          const adapterDownSpeed = downloadDelta >= 0 ? downloadDelta / timeDeltaSeconds : 0;
          const adapterUpSpeed = uploadDelta >= 0 ? uploadDelta / timeDeltaSeconds : 0;

          adapterSpeeds.set(stats.adapterId, {
            down: Math.round(adapterDownSpeed),
            up: Math.round(adapterUpSpeed)
          });

          totalDownloadSpeed += adapterDownSpeed;
          totalUploadSpeed += adapterUpSpeed;
        }
      }

      this.previousNetworkStats.set(stats.adapterId, {
        bytesReceived: stats.bytesReceived,
        bytesSent: stats.bytesSent,
        timestamp: now
      });
    }

    // Update per-adapter traffic history
    this.updateAdapterTrafficHistory(adapterSpeeds);

    // Apply exponential moving average (EMA) smoothing to reduce jitter
    this.smoothedDownSpeed = this.smoothedDownSpeed === 0
      ? totalDownloadSpeed
      : this.smoothedDownSpeed * (1 - NETWORK_EMA_ALPHA) + totalDownloadSpeed * NETWORK_EMA_ALPHA;
    this.smoothedUpSpeed = this.smoothedUpSpeed === 0
      ? totalUploadSpeed
      : this.smoothedUpSpeed * (1 - NETWORK_EMA_ALPHA) + totalUploadSpeed * NETWORK_EMA_ALPHA;

    const downSpeed = Math.round(this.smoothedDownSpeed);
    const upSpeed = Math.round(this.smoothedUpSpeed);

    this._networkDownSpeed.set(downSpeed);
    this._networkUpSpeed.set(upSpeed);
    this.pushToHistory(this._networkDownHistory, downSpeed);
    this.pushToHistory(this._networkUpHistory, upSpeed);

    // Update smoothed max for stable graph scaling
    this.updateSmoothedMax(downSpeed, upSpeed);
  }

  private updateAdapterTrafficHistory(adapterSpeeds: Map<string, { down: number; up: number }>): void {
    const currentHistory = this._adapterTrafficHistory();
    const newHistory = new Map<string, AdapterTrafficHistory>();

    for (const [adapterId, history] of currentHistory) {
      const speeds = adapterSpeeds.get(adapterId);
      const downSpeed = speeds?.down ?? 0;
      const upSpeed = speeds?.up ?? 0;

      // Create new history arrays
      const newDownloadHistory = [...history.downloadHistory, downSpeed];
      const newUploadHistory = [...history.uploadHistory, upSpeed];

      // Trim to max points
      if (newDownloadHistory.length > MAX_HISTORY_POINTS) {
        newDownloadHistory.shift();
      }
      if (newUploadHistory.length > MAX_HISTORY_POINTS) {
        newUploadHistory.shift();
      }

      // Calculate max for graph scaling
      const maxDown = Math.max(...newDownloadHistory, 1);
      const maxUp = Math.max(...newUploadHistory, 1);

      newHistory.set(adapterId, {
        adapterId: history.adapterId,
        adapterName: history.adapterName,
        downloadHistory: newDownloadHistory,
        uploadHistory: newUploadHistory,
        downloadSpeed: downSpeed,
        uploadSpeed: upSpeed,
        maxSpeed: Math.max(maxDown, maxUp, 1024) // Minimum 1KB for scale
      });
    }

    this._adapterTrafficHistory.set(newHistory);
  }

  private updateSmoothedMax(downSpeed: number, upSpeed: number): void {
    const currentMax = this._smoothedNetworkMax();
    const actualMax = Math.max(downSpeed, upSpeed);

    // If actual value exceeds current max, increase immediately (with some headroom)
    if (actualMax > currentMax) {
      // Round up to a nice value with 20% headroom
      this._smoothedNetworkMax.set(this.roundToNiceValue(actualMax * 1.2));
    } else {
      // Decay slowly towards the actual max (prevents sudden scale drops)
      const decayedMax = currentMax * MAX_DECAY_RATE;
      const minimumMax = Math.max(actualMax * 1.2, 1024); // At least 1KB or 20% above current
      this._smoothedNetworkMax.set(Math.max(decayedMax, this.roundToNiceValue(minimumMax)));
    }
  }

  private roundToNiceValue(value: number): number {
    // Round to nice byte values for cleaner axis labels
    const KB = 1024;
    const MB = 1024 * 1024;

    if (value < KB) return KB; // Minimum 1KB
    if (value < 10 * KB) return Math.ceil(value / KB) * KB; // Round to nearest KB
    if (value < 100 * KB) return Math.ceil(value / (10 * KB)) * 10 * KB; // Round to nearest 10KB
    if (value < MB) return Math.ceil(value / (100 * KB)) * 100 * KB; // Round to nearest 100KB
    if (value < 10 * MB) return Math.ceil(value / MB) * MB; // Round to nearest MB
    if (value < 100 * MB) return Math.ceil(value / (10 * MB)) * 10 * MB; // Round to nearest 10MB
    return Math.ceil(value / (100 * MB)) * 100 * MB; // Round to nearest 100MB
  }

  private pushToHistory(historySignal: ReturnType<typeof signal<number[]>>, value: number): void {
    const current = historySignal();
    const updated = [...current, value];
    if (updated.length > MAX_HISTORY_POINTS) {
      updated.shift();
    }
    historySignal.set(updated);
  }
}
