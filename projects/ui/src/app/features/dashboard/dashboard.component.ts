import { Component, OnInit, OnDestroy, inject, computed } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterLink } from '@angular/router';
import { Subject, takeUntil } from 'rxjs';

import { HardwareService, SystemService, NetworkService, StorageService, StatusService, MetricsHistoryService } from '@core/services';
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
      <div class="card bg-gradient-to-r from-syslens-bg-secondary to-syslens-bg-tertiary">
        <div class="flex items-center gap-4">
          <div class="w-12 h-12 rounded-xl bg-syslens-accent-blue/20 flex items-center justify-center">
            <svg class="w-6 h-6 text-syslens-accent-blue" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
            </svg>
          </div>
          @if (deviceName) {
            <div>
              <h2 class="text-lg font-semibold text-syslens-text-primary">{{ deviceName }}</h2>
              <p class="text-sm text-syslens-text-secondary">{{ osName }} {{ osVersion }}</p>
            </div>
          } @else {
            <div class="space-y-2">
              <div class="h-5 w-40 bg-syslens-bg-tertiary rounded animate-pulse"></div>
              <div class="h-4 w-56 bg-syslens-bg-tertiary rounded animate-pulse"></div>
            </div>
          }
        </div>
      </div>

      <!-- Real-time Metrics -->
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <!-- CPU Usage -->
        <a routerLink="/hardware" class="card-hover flex flex-col items-center cursor-pointer">
          <app-progress-ring
            [value]="cpuUsage"
            label="CPU"
            [size]="100"
          />
          @if (cpuName) {
            <p class="mt-2 text-sm text-syslens-text-secondary">{{ cpuName }}</p>
          } @else {
            <div class="mt-2 h-4 w-32 bg-syslens-bg-tertiary rounded animate-pulse"></div>
          }
        </a>

        <!-- Memory Usage -->
        <a routerLink="/hardware" class="card-hover flex flex-col items-center cursor-pointer">
          <app-progress-ring
            [value]="memoryUsage"
            label="Memory"
            [size]="100"
            colorClass="stroke-syslens-accent-purple"
          />
          @if (memoryTotalBytes) {
            <p class="mt-1 text-sm text-syslens-text-secondary">{{ memoryType }}</p>
            <p class="text-sm font-mono text-syslens-text-muted">
              <span style="min-width: 6ch; display: inline-block; text-align: right;">{{ memoryUsedBytes | bytes }}</span> / {{ memoryTotalBytes | bytes }}
            </p>
          } @else {
            <div class="mt-2 h-4 w-28 bg-syslens-bg-tertiary rounded animate-pulse"></div>
          }
        </a>

        <!-- Primary Disk -->
        <a routerLink="/storage" class="card-hover flex flex-col items-center cursor-pointer">
          <app-progress-ring
            [value]="diskUsage"
            label="Disk"
            [size]="100"
            colorClass="stroke-syslens-accent-cyan"
          />
          @if (diskTotalBytes) {
            <p class="mt-1 text-sm text-syslens-text-secondary truncate max-w-full" [title]="diskName">{{ diskName }}</p>
            <p class="text-sm font-mono text-syslens-text-muted">
              <span style="min-width: 6ch; display: inline-block; text-align: right;">{{ diskUsedBytes | bytes }}</span> / {{ diskTotalBytes | bytes }}
            </p>
          } @else {
            <div class="mt-2 h-4 w-28 bg-syslens-bg-tertiary rounded animate-pulse"></div>
          }
        </a>

        <!-- Network - Per Adapter -->
        <a routerLink="/network" class="card-hover cursor-pointer">
          <h3 class="text-sm text-syslens-text-muted mb-2">Network</h3>
          @if (adapterHistoryArray().length > 0) {
            <div class="space-y-3 max-h-[200px] overflow-y-auto">
              @for (adapter of adapterHistoryArray(); track adapter.adapterId) {
                <div class="space-y-1">
                  <div class="flex items-center justify-between">
                    <span class="text-xs text-syslens-text-secondary truncate max-w-[100px]" [title]="adapter.adapterName">{{ adapter.adapterName }}</span>
                    <div class="flex gap-2 text-xs font-mono">
                      <span class="text-syslens-accent-green">↓{{ adapter.downloadSpeed | bytes }}/s</span>
                      <span class="text-syslens-accent-blue">↑{{ adapter.uploadSpeed | bytes }}/s</span>
                    </div>
                  </div>
                  <app-line-graph
                    [height]="35"
                    [series1]="adapter.downloadHistory"
                    [series2]="adapter.uploadHistory"
                    [maxValue]="adapter.maxSpeed"
                    series1Color="syslens-accent-green"
                    series2Color="syslens-accent-blue"
                    [showYAxis]="false"
                  />
                </div>
              }
            </div>
          } @else {
            <div class="h-[50px] flex items-center justify-center text-syslens-text-muted text-xs">
              No active adapters
            </div>
          }
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
  metricsService = inject(MetricsHistoryService);
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
  memoryType = '';

  // Disk
  diskTotalBytes = 0;
  diskUsedBytes = 0;
  diskUsage = 0;
  diskName = '';

  // Network
  networkAdapterCount = 0;
  networkAdapterName = '';
  networkLoaded = false;

  // Per-adapter traffic history from metrics service
  adapterHistoryArray = computed(() => {
    const historyMap = this.metricsService.adapterTrafficHistory();
    return Array.from(historyMap.values());
  });

  ngOnInit(): void {
    this.loadInitialData();
    // Start real-time updates immediately for responsive UI
    this.startRealtimeUpdates();
  }

  ngOnDestroy(): void {
    this.destroy$.next();
    this.destroy$.complete();
  }

  private loadInitialData(): void {
    this.statusService.startOperation('dashboard-init', 'Loading dashboard data...');

    // Progressive loading: fire all requests in parallel, update UI as each completes
    // This gives instant perceived responsiveness vs waiting for all to complete

    // System info (fast, prioritized)
    this.systemService.getDeviceInfo()
      .pipe(takeUntil(this.destroy$))
      .subscribe(deviceInfo => {
        this.deviceName = deviceInfo.computerName;
      });

    this.systemService.getOsInfo()
      .pipe(takeUntil(this.destroy$))
      .subscribe(osInfo => {
        this.osName = osInfo.name;
        this.osVersion = osInfo.version;
      });

    // Hardware info (medium priority)
    this.hardwareService.getCpuInfo()
      .pipe(takeUntil(this.destroy$))
      .subscribe(cpuInfo => {
        this.cpuName = cpuInfo.name;
      });

    this.hardwareService.getMemoryInfo()
      .pipe(takeUntil(this.destroy$))
      .subscribe(memoryInfo => {
        this.memoryTotalBytes = memoryInfo.totalBytes;
        // Build memory type string (e.g., "64 GB DDR5")
        const sizeGB = Math.round(memoryInfo.totalBytes / (1024 * 1024 * 1024));
        this.memoryType = `${sizeGB} GB ${memoryInfo.memoryType}`;
      });

    // Storage (lower priority - visible but not critical)
    this.storageService.getVolumes()
      .pipe(takeUntil(this.destroy$))
      .subscribe(volumes => {
        const systemVolume = volumes.find(v => v.isSystem) || volumes[0];
        if (systemVolume) {
          this.diskTotalBytes = systemVolume.totalBytes;
          this.diskUsedBytes = systemVolume.usedBytes;
          this.diskUsage = systemVolume.percentUsed;
          // Build disk name string (e.g., "Local Disk (C:)" or just "C:")
          const label = systemVolume.label || 'Local Disk';
          this.diskName = `${label} (${systemVolume.driveLetter}:)`;
        }
      });

    // Network adapters (lower priority)
    this.networkService.getNetworkAdapters()
      .pipe(takeUntil(this.destroy$))
      .subscribe(adapters => {
        const activeAdapters = adapters.filter(a => a.status === 'Up');
        this.networkAdapterCount = activeAdapters.length;
        // Get primary adapter name (first active adapter)
        if (activeAdapters.length > 0) {
          this.networkAdapterName = activeAdapters[0].name;
        } else {
          this.networkAdapterName = 'No active adapter';
        }
        this.networkLoaded = true;
        this.statusService.endOperation('dashboard-init');
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

    // Network stats are handled by MetricsHistoryService
  }
}
