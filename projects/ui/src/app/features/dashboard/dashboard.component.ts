import { Component, OnInit, OnDestroy, inject, effect } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterLink } from '@angular/router';
import { Subject, takeUntil, interval, forkJoin, of } from 'rxjs';
import { switchMap, startWith, catchError } from 'rxjs/operators';

import { HardwareService, SystemService, NetworkService, StorageService, StatusService, NetworkHistoryService } from '@core/services';
import { CpuMetrics, MemoryMetrics, CpuInfo, MemoryInfo, NetworkAdapter, AdapterStats } from '@core/models';
import { ProgressRingComponent, LineGraphComponent } from '@shared/components';
import { BytesPipe, UptimePipe } from '@shared/pipes';

@Component({
  selector: 'app-dashboard',
  standalone: true,
  imports: [
    CommonModule,
    RouterLink,
    ProgressRingComponent,
    LineGraphComponent,
    BytesPipe,
    UptimePipe
  ],
  template: `
    <div class="p-6 space-y-6">
      <!-- Header -->
      <div class="flex items-center justify-between">
        <div>
          <h1 class="text-2xl font-bold text-syslens-text-primary">Dashboard</h1>
          <p class="text-syslens-text-secondary">System overview and health status</p>
        </div>
        <div class="text-right">
          <p class="text-sm text-syslens-text-muted">Uptime</p>
          <p class="text-lg font-mono text-syslens-text-primary">{{ uptimeSeconds | uptime }}</p>
        </div>
      </div>

      <!-- System Info Banner -->
      @if (deviceName) {
        <div class="card bg-gradient-to-r from-syslens-bg-secondary to-syslens-bg-tertiary">
          <div class="flex items-center gap-4">
            <div class="w-12 h-12 rounded-xl bg-syslens-accent-blue/20 flex items-center justify-center">
              <svg class="w-6 h-6 text-syslens-accent-blue" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                  d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
              </svg>
            </div>
            <div>
              <h2 class="text-lg font-semibold text-syslens-text-primary">{{ deviceName }}</h2>
              <p class="text-sm text-syslens-text-secondary">{{ osName }} {{ osVersion }}</p>
            </div>
          </div>
        </div>
      }

      <!-- Real-time Metrics -->
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <!-- CPU Usage -->
        <a routerLink="/hardware" class="card-hover flex flex-col items-center cursor-pointer">
          <app-progress-ring
            [value]="cpuUsage"
            label="CPU"
            [size]="100"
          />
          <p class="mt-2 text-sm text-syslens-text-secondary">{{ cpuName }}</p>
        </a>

        <!-- Memory Usage -->
        <a routerLink="/hardware" class="card-hover flex flex-col items-center cursor-pointer">
          <app-progress-ring
            [value]="memoryUsage"
            label="Memory"
            [size]="100"
            colorClass="stroke-syslens-accent-purple"
          />
          <p class="mt-2 text-sm font-mono text-syslens-text-secondary">
            <span style="min-width: 6ch; display: inline-block; text-align: right;">{{ memoryUsedBytes | bytes }}</span> / {{ memoryTotalBytes | bytes }}
          </p>
        </a>

        <!-- Primary Disk -->
        <a routerLink="/storage" class="card-hover flex flex-col items-center cursor-pointer">
          <app-progress-ring
            [value]="diskUsage"
            label="Disk"
            [size]="100"
            colorClass="stroke-syslens-accent-cyan"
          />
          <p class="mt-2 text-sm font-mono text-syslens-text-secondary">
            <span style="min-width: 6ch; display: inline-block; text-align: right;">{{ diskUsedBytes | bytes }}</span> / {{ diskTotalBytes | bytes }}
          </p>
        </a>

        <!-- Network -->
        <a routerLink="/network" class="card-hover cursor-pointer">
          <h3 class="text-sm text-syslens-text-muted mb-2">Network</h3>
          <div class="mb-2">
            <app-line-graph
              [width]="180"
              [height]="50"
              [series1]="downloadHistory"
              [series2]="uploadHistory"
              [maxValue]="networkHistoryService.maxSpeed()"
              series1Color="syslens-accent-green"
              series2Color="syslens-accent-blue"
            />
          </div>
          <div class="space-y-1">
            <div class="flex justify-between items-center">
              <span class="text-xs text-syslens-text-secondary">Download</span>
              <span class="font-mono text-xs text-syslens-accent-green" style="min-width: 8ch; text-align: right;">{{ downloadSpeed | bytes }}/s</span>
            </div>
            <div class="flex justify-between items-center">
              <span class="text-xs text-syslens-text-secondary">Upload</span>
              <span class="font-mono text-xs text-syslens-accent-blue" style="min-width: 8ch; text-align: right;">{{ uploadSpeed | bytes }}/s</span>
            </div>
          </div>
          <p class="mt-2 text-xs text-syslens-text-muted">{{ networkAdapterCount }} adapter(s)</p>
        </a>
      </div>

      <!-- Quick Navigation -->
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <a routerLink="/network" class="card-hover group">
          <div class="flex items-center gap-3">
            <div class="w-10 h-10 rounded-lg bg-syslens-accent-blue/20 flex items-center justify-center group-hover:bg-syslens-accent-blue/30 transition-colors">
              <svg class="w-5 h-5 text-syslens-accent-blue" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                  d="M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9a9 9 0 01-9-9m9 9c1.657 0 3-4.03 3-9s-1.343-9-3-9m0 18c-1.657 0-3-4.03-3-9s1.343-9 3-9m-9 9a9 9 0 019-9" />
              </svg>
            </div>
            <div>
              <h3 class="font-medium text-syslens-text-primary">Network</h3>
              <p class="text-xs text-syslens-text-muted">Adapters, DNS, Routes</p>
            </div>
          </div>
        </a>

        <a routerLink="/system" class="card-hover group">
          <div class="flex items-center gap-3">
            <div class="w-10 h-10 rounded-lg bg-syslens-accent-green/20 flex items-center justify-center group-hover:bg-syslens-accent-green/30 transition-colors">
              <svg class="w-5 h-5 text-syslens-accent-green" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                  d="M9 3v2m6-2v2M9 19v2m6-2v2M5 9H3m2 6H3m18-6h-2m2 6h-2M7 19h10a2 2 0 002-2V7a2 2 0 00-2-2H7a2 2 0 00-2 2v10a2 2 0 002 2zM9 9h6v6H9V9z" />
              </svg>
            </div>
            <div>
              <h3 class="font-medium text-syslens-text-primary">System</h3>
              <p class="text-xs text-syslens-text-muted">BIOS, OS, Boot</p>
            </div>
          </div>
        </a>

        <a routerLink="/hardware" class="card-hover group">
          <div class="flex items-center gap-3">
            <div class="w-10 h-10 rounded-lg bg-syslens-accent-purple/20 flex items-center justify-center group-hover:bg-syslens-accent-purple/30 transition-colors">
              <svg class="w-5 h-5 text-syslens-accent-purple" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                  d="M9 3v2m6-2v2M9 19v2m6-2v2M5 9H3m2 6H3m18-6h-2m2 6h-2M7 19h10a2 2 0 002-2V7a2 2 0 00-2-2H7a2 2 0 00-2 2v10a2 2 0 002 2zM9 9h6v6H9V9z" />
              </svg>
            </div>
            <div>
              <h3 class="font-medium text-syslens-text-primary">Hardware</h3>
              <p class="text-xs text-syslens-text-muted">CPU, RAM, GPU</p>
            </div>
          </div>
        </a>

        <a routerLink="/storage" class="card-hover group">
          <div class="flex items-center gap-3">
            <div class="w-10 h-10 rounded-lg bg-syslens-accent-cyan/20 flex items-center justify-center group-hover:bg-syslens-accent-cyan/30 transition-colors">
              <svg class="w-5 h-5 text-syslens-accent-cyan" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                  d="M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4m0 5c0 2.21-3.582 4-8 4s-8-1.79-8-4" />
              </svg>
            </div>
            <div>
              <h3 class="font-medium text-syslens-text-primary">Storage</h3>
              <p class="text-xs text-syslens-text-muted">Disks, Volumes, Health</p>
            </div>
          </div>
        </a>
      </div>
    </div>
  `
})
export class DashboardComponent implements OnInit, OnDestroy {
  private hardwareService = inject(HardwareService);
  private systemService = inject(SystemService);
  private networkService = inject(NetworkService);
  private storageService = inject(StorageService);
  private statusService = inject(StatusService);
  networkHistoryService = inject(NetworkHistoryService);
  private destroy$ = new Subject<void>();

  // System info
  deviceName = '';
  osName = '';
  osVersion = '';
  uptimeSeconds = 0;

  // CPU
  cpuName = '';
  cpuUsage = 0;

  // Memory
  memoryTotalBytes = 0;
  memoryUsedBytes = 0;
  memoryUsage = 0;

  // Disk
  diskTotalBytes = 0;
  diskUsedBytes = 0;
  diskUsage = 0;

  // Network
  downloadSpeed = 0;
  uploadSpeed = 0;
  networkAdapterCount = 0;
  private activeAdapters: NetworkAdapter[] = [];
  private previousNetworkStats: Map<string, { bytesReceived: number; bytesSent: number; timestamp: number }> = new Map();

  // Network history for graph - use stable arrays updated via effect
  // (getters with .map() create new arrays on every access, breaking change detection)
  downloadHistory: number[] = [];
  uploadHistory: number[] = [];

  constructor() {
    // Update history arrays only when service data actually changes
    effect(() => {
      const points = this.networkHistoryService.dataPoints();
      this.downloadHistory = points.map(p => p.downloadSpeed);
      this.uploadHistory = points.map(p => p.uploadSpeed);
    });
  }

  ngOnInit(): void {
    this.loadInitialData();
  }

  ngOnDestroy(): void {
    this.destroy$.next();
    this.destroy$.complete();
  }

  private loadInitialData(): void {
    this.statusService.startOperation('dashboard-init', 'Loading dashboard data...');

    // Parallelize ALL initial data fetches for faster startup
    forkJoin({
      deviceInfo: this.systemService.getDeviceInfo(),
      osInfo: this.systemService.getOsInfo(),
      cpuInfo: this.hardwareService.getCpuInfo(),
      memoryInfo: this.hardwareService.getMemoryInfo(),
      volumes: this.storageService.getVolumes(),
      adapters: this.networkService.getNetworkAdapters()
    }).pipe(
      takeUntil(this.destroy$)
    ).subscribe(({ deviceInfo, osInfo, cpuInfo, memoryInfo, volumes, adapters }) => {
      this.statusService.endOperation('dashboard-init');
      // System info
      this.deviceName = deviceInfo.computerName;
      this.osName = osInfo.name;
      this.osVersion = osInfo.version;

      // Hardware info
      this.cpuName = cpuInfo.name;
      this.memoryTotalBytes = memoryInfo.totalBytes;

      // Storage - get system volume
      const systemVolume = volumes.find(v => v.isSystem) || volumes[0];
      if (systemVolume) {
        this.diskTotalBytes = systemVolume.totalBytes;
        this.diskUsedBytes = systemVolume.usedBytes;
        this.diskUsage = systemVolume.percentUsed;
      }

      // Network adapters
      this.activeAdapters = adapters.filter(a => a.status === 'Up');
      this.networkAdapterCount = this.activeAdapters.length;

      // Start polling AFTER initial data is loaded (deferred startup)
      this.startRealtimeUpdates();
    });
  }

  private startRealtimeUpdates(): void {
    // CPU metrics
    this.hardwareService.getCpuMetricsPolling()
      .pipe(takeUntil(this.destroy$))
      .subscribe(metrics => {
        this.cpuUsage = Math.round(metrics.totalUsage);
      });

    // Memory metrics
    this.hardwareService.getMemoryMetricsPolling()
      .pipe(takeUntil(this.destroy$))
      .subscribe(metrics => {
        this.memoryUsedBytes = metrics.inUseBytes;
        this.memoryUsage = Math.round((metrics.inUseBytes / this.memoryTotalBytes) * 100);
      });

    // Uptime
    this.systemService.getUptime()
      .pipe(takeUntil(this.destroy$))
      .subscribe(uptime => {
        this.uptimeSeconds = uptime.uptimeSeconds;
      });

    // Network stats - poll every second
    interval(1000).pipe(
      startWith(0),
      takeUntil(this.destroy$),
      switchMap(() => {
        if (this.activeAdapters.length === 0) {
          return of([]);
        }
        // Get stats for all active adapters
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

          // Only count positive deltas (counter resets can cause negative values)
          if (downloadDelta >= 0) {
            totalDownloadSpeed += downloadDelta / timeDeltaSeconds;
          }
          if (uploadDelta >= 0) {
            totalUploadSpeed += uploadDelta / timeDeltaSeconds;
          }
        }
      }

      // Store current values for next calculation
      this.previousNetworkStats.set(stats.adapterId, {
        bytesReceived: stats.bytesReceived,
        bytesSent: stats.bytesSent,
        timestamp: now
      });
    }

    this.downloadSpeed = Math.round(totalDownloadSpeed);
    this.uploadSpeed = Math.round(totalUploadSpeed);

    // Record history for graph
    this.networkHistoryService.addDataPoint(this.downloadSpeed, this.uploadSpeed);
  }
}
