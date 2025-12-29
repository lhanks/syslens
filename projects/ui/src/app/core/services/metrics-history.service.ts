import { Injectable, inject, signal, computed, OnDestroy } from '@angular/core';
import { Subject, interval, forkJoin, of } from 'rxjs';
import { takeUntil, switchMap, startWith, catchError } from 'rxjs/operators';

import { HardwareService } from './hardware.service';
import { StorageService } from './storage.service';
import { NetworkService } from './network.service';
import { NetworkAdapter, AdapterStats, MemoryInfo } from '../models';

const MAX_HISTORY_POINTS = 60; // 60 seconds of history

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

  // Network tracking state
  private activeAdapters: NetworkAdapter[] = [];
  private previousNetworkStats = new Map<string, { bytesReceived: number; bytesSent: number; timestamp: number }>();
  private memoryInfo: MemoryInfo | null = null;
  private isStarted = false;

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

  networkMaxSpeed = computed(() => {
    const down = this._networkDownHistory();
    const up = this._networkUpHistory();
    const maxDown = Math.max(...down, 1);
    const maxUp = Math.max(...up, 1);
    return Math.max(maxDown, maxUp);
  });

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
      this.activeAdapters = adapters.filter(a => a.status === 'Up');

      // Start polling after initial data is loaded
      this.startPolling();
    });
  }

  private startPolling(): void {
    // CPU metrics - every second
    this.hardwareService.getCpuMetricsPolling()
      .pipe(takeUntil(this.destroy$))
      .subscribe(metrics => {
        this._cpuUsage.set(metrics.totalUsage);
        this.pushToHistory(this._cpuHistory, metrics.totalUsage);
      });

    // Memory metrics - every second
    this.hardwareService.getMemoryMetricsPolling()
      .pipe(takeUntil(this.destroy$))
      .subscribe(metrics => {
        this._memoryUsedBytes.set(metrics.inUseBytes);
        if (this.memoryInfo) {
          const usage = (metrics.inUseBytes / this.memoryInfo.totalBytes) * 100;
          this.pushToHistory(this._memoryHistory, usage);
        }
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

    for (const stats of currentStats) {
      const previous = this.previousNetworkStats.get(stats.adapterId);

      if (previous) {
        const timeDeltaSeconds = (now - previous.timestamp) / 1000;
        if (timeDeltaSeconds > 0) {
          const downloadDelta = stats.bytesReceived - previous.bytesReceived;
          const uploadDelta = stats.bytesSent - previous.bytesSent;

          if (downloadDelta >= 0) {
            totalDownloadSpeed += downloadDelta / timeDeltaSeconds;
          }
          if (uploadDelta >= 0) {
            totalUploadSpeed += uploadDelta / timeDeltaSeconds;
          }
        }
      }

      this.previousNetworkStats.set(stats.adapterId, {
        bytesReceived: stats.bytesReceived,
        bytesSent: stats.bytesSent,
        timestamp: now
      });
    }

    const downSpeed = Math.round(totalDownloadSpeed);
    const upSpeed = Math.round(totalUploadSpeed);

    this._networkDownSpeed.set(downSpeed);
    this._networkUpSpeed.set(upSpeed);
    this.pushToHistory(this._networkDownHistory, downSpeed);
    this.pushToHistory(this._networkUpHistory, upSpeed);
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
