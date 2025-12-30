import { Component, inject, signal, OnInit, OnDestroy } from '@angular/core';
import { CommonModule } from '@angular/common';
import { Subject, forkJoin } from 'rxjs';
import { takeUntil } from 'rxjs/operators';

import { HardwareService, StorageService, NetworkService } from '@core/services';
import { SystemService } from '@core/services/system.service';
import { BytesPipe } from '@shared/pipes';
import { DeviceDetailModalComponent } from '@shared/components/device-detail-modal/device-detail-modal.component';
import {
  OsInfo,
  CpuInfo,
  MemoryInfo,
  GpuInfo,
  GpuMetrics,
  PhysicalDisk,
  NetworkAdapter,
  UsbDevice,
  Monitor,
  DeviceType
} from '@core/models';

@Component({
  selector: 'app-right-sidebar',
  standalone: true,
  imports: [CommonModule, BytesPipe, DeviceDetailModalComponent],
  template: `
    <aside class="w-72 h-full bg-syslens-bg-secondary border-l border-syslens-border-primary flex flex-col overflow-y-auto">
      <!-- Header -->
      <div class="p-3 border-b border-syslens-border-primary">
        <h2 class="text-sm font-semibold text-syslens-text-primary">System Info</h2>
      </div>

      <div class="p-3 space-y-4 text-xs">
        <!-- Windows Version -->
        @if (osInfo()) {
          <div class="space-y-1.5">
            <div class="flex items-center gap-2">
              <div class="w-5 h-5 rounded bg-syslens-accent-blue/20 flex items-center justify-center flex-shrink-0">
                <svg class="w-3 h-3 text-syslens-accent-blue" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                    d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
                </svg>
              </div>
              <span class="text-syslens-text-muted font-medium">Windows</span>
            </div>
            <div class="pl-7 space-y-0.5">
              <p class="text-syslens-text-secondary">{{ osInfo()!.name }}</p>
              <p class="text-syslens-text-muted">Build {{ osInfo()!.build }}</p>
              <p class="text-syslens-text-muted">{{ osInfo()!.architecture }}</p>
            </div>
          </div>
        }

        <!-- CPU -->
        @if (cpuInfo()) {
          <div class="space-y-1.5 cursor-pointer hover:bg-syslens-bg-tertiary/50 -mx-3 px-3 py-1.5 rounded transition-colors"
               (click)="openDeviceDetail(cpuInfo()!.name, 'Cpu')">
            <div class="flex items-center gap-2">
              <div class="w-5 h-5 rounded bg-syslens-accent-purple/20 flex items-center justify-center flex-shrink-0">
                <svg class="w-3 h-3 text-syslens-accent-purple" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                    d="M9 3v2m6-2v2M9 19v2m6-2v2M5 9H3m2 6H3m18-6h-2m2 6h-2M7 19h10a2 2 0 002-2V7a2 2 0 00-2-2H7a2 2 0 00-2 2v10a2 2 0 002 2zM9 9h6v6H9V9z" />
                </svg>
              </div>
              <span class="text-syslens-text-muted font-medium">Processor</span>
            </div>
            <div class="pl-7 space-y-0.5">
              <p class="text-syslens-text-secondary">{{ cpuInfo()!.name }}</p>
              <p class="text-syslens-text-muted">{{ cpuInfo()!.physicalCores }} cores, {{ cpuInfo()!.logicalProcessors }} threads</p>
              <p class="text-syslens-text-muted">{{ formatMhz(cpuInfo()!.baseClockMhz) }} base</p>
            </div>
          </div>
        }

        <!-- RAM -->
        @if (memoryInfo()) {
          <div class="space-y-1.5 cursor-pointer hover:bg-syslens-bg-tertiary/50 -mx-3 px-3 py-1.5 rounded transition-colors"
               (click)="openDeviceDetail(getMemoryIdentifier(), 'Memory')">
            <div class="flex items-center gap-2">
              <div class="w-5 h-5 rounded bg-syslens-accent-cyan/20 flex items-center justify-center flex-shrink-0">
                <svg class="w-3 h-3 text-syslens-accent-cyan" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                    d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
                </svg>
              </div>
              <span class="text-syslens-text-muted font-medium">Memory</span>
            </div>
            <div class="pl-7 space-y-0.5">
              <p class="text-syslens-text-secondary">{{ memoryInfo()!.totalBytes | bytes }} {{ memoryInfo()!.memoryType }}</p>
              @if (getMemoryVendor()) {
                <p class="text-syslens-text-muted">{{ getMemoryVendor() }}</p>
              }
              <p class="text-syslens-text-muted">{{ memoryInfo()!.speedMhz }} MHz</p>
              <p class="text-syslens-text-muted">{{ memoryInfo()!.slotsUsed }}/{{ memoryInfo()!.slotsTotal }} slots</p>
            </div>
          </div>
        }

        <!-- Storage -->
        @if (disks().length > 0) {
          <div class="space-y-1.5">
            <div class="flex items-center gap-2">
              <div class="w-5 h-5 rounded bg-syslens-accent-yellow/20 flex items-center justify-center flex-shrink-0">
                <svg class="w-3 h-3 text-syslens-accent-yellow" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                    d="M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4" />
                </svg>
              </div>
              <span class="text-syslens-text-muted font-medium">Storage</span>
            </div>
            <div class="pl-7 space-y-1.5">
              @for (disk of disks(); track disk.deviceId) {
                <div class="space-y-0.5 cursor-pointer hover:bg-syslens-bg-tertiary/50 -mx-3 px-3 py-1 rounded transition-colors"
                     (click)="openDeviceDetail(disk.model, 'Storage')">
                  <p class="text-syslens-text-secondary truncate" [title]="disk.model">{{ disk.model }}</p>
                  <p class="text-syslens-text-muted">{{ disk.sizeBytes | bytes }} ({{ disk.mediaType }})</p>
                </div>
              }
            </div>
          </div>
        }

        <!-- Ports Summary -->
        @if (hasPortData()) {
          <div class="space-y-1.5">
            <div class="flex items-center gap-2">
              <div class="w-5 h-5 rounded bg-syslens-accent-orange/20 flex items-center justify-center flex-shrink-0">
                <svg class="w-3 h-3 text-syslens-accent-orange" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                    d="M12 11c0 3.517-1.009 6.799-2.753 9.571m-3.44-2.04l.054-.09A13.916 13.916 0 008 11a4 4 0 118 0c0 1.017-.07 2.019-.203 3m-2.118 6.844A21.88 21.88 0 0015.171 17m3.839 1.132c.645-2.266.99-4.659.99-7.132A8 8 0 008 4.07M3 15.364c.64-1.319 1-2.8 1-4.364 0-1.457.39-2.823 1.07-4" />
                </svg>
              </div>
              <span class="text-syslens-text-muted font-medium">Ports</span>
            </div>
            <div class="pl-7">
              <div class="flex flex-wrap gap-x-3 gap-y-1">
                @for (port of getPortCounts(); track port.type) {
                  <span class="text-syslens-text-secondary">{{ port.type }} <span class="text-syslens-text-muted">×{{ port.count }}</span></span>
                }
              </div>
            </div>
          </div>
        }

        <!-- GPU -->
        @if (gpus().length > 0) {
          <div class="space-y-1.5">
            <div class="flex items-center gap-2">
              <div class="w-5 h-5 rounded bg-syslens-accent-green/20 flex items-center justify-center flex-shrink-0">
                <svg class="w-3 h-3 text-syslens-accent-green" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                    d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
                </svg>
              </div>
              <span class="text-syslens-text-muted font-medium">Graphics</span>
            </div>
            <div class="pl-7 space-y-1.5">
              @for (gpu of gpus(); track gpu.id) {
                <div class="space-y-0.5 cursor-pointer hover:bg-syslens-bg-tertiary/50 -mx-3 px-3 py-1 rounded transition-colors"
                     (click)="openDeviceDetail(gpu.name, 'Gpu')">
                  <div class="flex items-center justify-between">
                    <p class="text-syslens-text-secondary truncate flex-1" [title]="gpu.name">{{ gpu.name }}</p>
                    @if (getGpuUsage(gpu.id) !== null) {
                      <span class="font-mono text-syslens-text-muted ml-2">{{ getGpuUsage(gpu.id)?.toFixed(0) }}%</span>
                    }
                  </div>
                  @if (getGpuUsage(gpu.id) !== null) {
                    <div class="h-1 bg-syslens-bg-tertiary rounded-full overflow-hidden">
                      <div class="h-full rounded-full transition-all duration-300"
                           [style.width.%]="getGpuUsage(gpu.id)"
                           [class.bg-syslens-accent-green]="getGpuUsage(gpu.id)! < 50"
                           [class.bg-syslens-accent-yellow]="getGpuUsage(gpu.id)! >= 50 && getGpuUsage(gpu.id)! < 80"
                           [class.bg-syslens-accent-red]="getGpuUsage(gpu.id)! >= 80">
                      </div>
                    </div>
                  }
                  <p class="text-syslens-text-muted">
                    @if (getGpuVramUsed(gpu.id) !== null) {
                      {{ getGpuVramUsed(gpu.id) | bytes }} / {{ gpu.vramBytes | bytes }} VRAM
                    } @else {
                      {{ gpu.vramBytes | bytes }} VRAM
                    }
                  </p>
                </div>
              }
            </div>
          </div>
        }

        <!-- Network / IP Addresses -->
        @if (adapters().length > 0) {
          <div class="space-y-1.5">
            <div class="flex items-center gap-2">
              <div class="w-5 h-5 rounded bg-syslens-accent-blue/20 flex items-center justify-center flex-shrink-0">
                <svg class="w-3 h-3 text-syslens-accent-blue" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                    d="M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9a9 9 0 01-9-9m9 9c1.657 0 3-4.03 3-9s-1.343-9-3-9m0 18c-1.657 0-3-4.03-3-9s1.343-9 3-9m-9 9a9 9 0 019-9" />
                </svg>
              </div>
              <span class="text-syslens-text-muted font-medium">Network</span>
            </div>
            <div class="pl-7 space-y-1.5">
              @for (adapter of adapters(); track adapter.id) {
                <div class="space-y-0.5">
                  <p class="text-syslens-text-secondary truncate" [title]="adapter.name">{{ adapter.name }}</p>
                  <p class="text-syslens-text-muted font-mono">{{ adapter.ipv4Config?.address }}</p>
                </div>
              }
            </div>
          </div>
        }
      </div>

      <!-- Loading state -->
      @if (isLoading()) {
        <div class="flex-1 flex items-center justify-center">
          <div class="animate-spin w-6 h-6 border-2 border-syslens-accent-blue border-t-transparent rounded-full"></div>
        </div>
      }
    </aside>

    <!-- Device Detail Modal -->
    <app-device-detail-modal
      [isOpen]="isModalOpen()"
      [deviceId]="selectedDeviceId()"
      [deviceType]="selectedDeviceType()"
      (closed)="closeDeviceDetail()"
    />
  `,
  styles: [`
    :host {
      display: block;
      height: 100%;
    }
  `]
})
export class RightSidebarComponent implements OnInit, OnDestroy {
  private systemService = inject(SystemService);
  private hardwareService = inject(HardwareService);
  private storageService = inject(StorageService);
  private networkService = inject(NetworkService);
  private destroy$ = new Subject<void>();

  // Data signals
  osInfo = signal<OsInfo | null>(null);
  cpuInfo = signal<CpuInfo | null>(null);
  memoryInfo = signal<MemoryInfo | null>(null);
  gpus = signal<GpuInfo[]>([]);
  gpuMetrics = signal<Map<string, GpuMetrics>>(new Map());
  disks = signal<PhysicalDisk[]>([]);
  adapters = signal<NetworkAdapter[]>([]);
  usbDevices = signal<UsbDevice[]>([]);
  monitors = signal<Monitor[]>([]);
  isLoading = signal(true);

  // Modal state
  isModalOpen = signal(false);
  selectedDeviceId = signal('');
  selectedDeviceType = signal<DeviceType>('Cpu');

  ngOnInit(): void {
    this.loadSystemInfo();
    this.subscribeToAdapterChanges();
    this.subscribeToGpuMetrics();
  }

  ngOnDestroy(): void {
    this.destroy$.next();
    this.destroy$.complete();
  }

  private loadSystemInfo(): void {
    forkJoin({
      osInfo: this.systemService.getOsInfo(),
      cpuInfo: this.hardwareService.getCpuInfo(),
      memoryInfo: this.hardwareService.getMemoryInfo(),
      gpus: this.hardwareService.getGpuInfo(),
      disks: this.storageService.getPhysicalDisks(),
      adapters: this.networkService.getNetworkAdapters(),
      usbDevices: this.hardwareService.getUsbDevices(),
      monitors: this.hardwareService.getMonitors()
    }).pipe(
      takeUntil(this.destroy$)
    ).subscribe({
      next: (data) => {
        this.osInfo.set(data.osInfo);
        this.cpuInfo.set(data.cpuInfo);
        this.memoryInfo.set(data.memoryInfo);
        this.gpus.set(data.gpus);
        this.disks.set(data.disks);
        // Filter to only show adapters that are up and have an IP
        this.adapters.set(data.adapters.filter(a => a.status === 'Up' && a.ipv4Config?.address));
        this.usbDevices.set(data.usbDevices);
        this.monitors.set(data.monitors);
        this.isLoading.set(false);
      },
      error: () => {
        this.isLoading.set(false);
      }
    });
  }

  private subscribeToAdapterChanges(): void {
    this.networkService.onAdapterChange().pipe(
      takeUntil(this.destroy$)
    ).subscribe(adapters => {
      // Update adapters when network changes occur
      this.adapters.set(adapters.filter(a => a.status === 'Up' && a.ipv4Config?.address));
    });
  }

  private subscribeToGpuMetrics(): void {
    this.hardwareService.getGpuMetricsPolling().pipe(
      takeUntil(this.destroy$)
    ).subscribe(metrics => {
      const metricsMap = new Map<string, GpuMetrics>();
      for (const metric of metrics) {
        metricsMap.set(metric.gpuId, metric);
      }
      this.gpuMetrics.set(metricsMap);
    });
  }

  getGpuUsage(gpuId: string): number | null {
    const metric = this.gpuMetrics().get(gpuId);
    return metric?.usagePercent ?? null;
  }

  getGpuVramUsed(gpuId: string): number | null {
    const metric = this.gpuMetrics().get(gpuId);
    return metric?.vramUsedBytes ?? null;
  }

  getMemoryVendor(): string | null {
    const info = this.memoryInfo();
    if (!info?.modules?.length) return null;

    // Get unique manufacturers from all modules
    const manufacturers = [...new Set(
      info.modules
        .map(m => m.manufacturer)
        .filter(m => m && m !== 'Unknown' && m.trim() !== '')
    )];

    if (manufacturers.length === 0) return null;
    if (manufacturers.length === 1) return manufacturers[0];
    return manufacturers.join(' / ');
  }

  formatMhz(mhz: number): string {
    if (mhz >= 1000) {
      return `${(mhz / 1000).toFixed(1)} GHz`;
    }
    return `${mhz} MHz`;
  }

  /**
   * Get port type counts from USB devices and monitors.
   * Returns an array of { type, count } sorted by count descending.
   */
  getPortCounts(): { type: string; count: number }[] {
    const counts = new Map<string, number>();

    // Count USB ports by speed
    for (const usb of this.usbDevices()) {
      let portType: string;
      switch (usb.speed) {
        case 'Super':
          portType = 'USB 3.0';
          break;
        case 'SuperPlus':
          portType = 'USB 3.1+';
          break;
        case 'High':
          portType = 'USB 2.0';
          break;
        case 'Full':
        case 'Low':
          portType = 'USB 1.x';
          break;
        default:
          portType = 'USB';
      }
      counts.set(portType, (counts.get(portType) || 0) + 1);
    }

    // Count display connections from monitors
    for (const monitor of this.monitors()) {
      const conn = monitor.connection || 'Unknown';
      counts.set(conn, (counts.get(conn) || 0) + 1);
    }

    // Convert to array and sort by count
    return Array.from(counts.entries())
      .map(([type, count]) => ({ type, count }))
      .sort((a, b) => b.count - a.count);
  }

  hasPortData(): boolean {
    return this.usbDevices().length > 0 || this.monitors().length > 0;
  }

  /**
   * Open the device detail modal for a specific device
   */
  openDeviceDetail(deviceId: string, type: DeviceType): void {
    this.selectedDeviceId.set(deviceId);
    this.selectedDeviceType.set(type);
    this.isModalOpen.set(true);
  }

  /**
   * Close the device detail modal
   */
  closeDeviceDetail(): void {
    this.isModalOpen.set(false);
  }

  /**
   * Get a unique identifier for memory to use as deviceId
   */
  getMemoryIdentifier(): string {
    const info = this.memoryInfo();
    if (!info?.modules?.length) return 'Unknown Memory';
    const module = info.modules[0];
    // Use part number if available, otherwise use manufacturer + type
    if (module.partNumber && module.partNumber !== 'Unknown') {
      return module.partNumber;
    }
    const vendor = this.getMemoryVendor();
    return vendor ? `${vendor} ${info.memoryType}` : `${info.memoryType} Memory`;
  }
}
