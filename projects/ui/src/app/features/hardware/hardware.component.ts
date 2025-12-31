import { Component, OnInit, OnDestroy, inject } from '@angular/core';
import { CommonModule } from '@angular/common';
import { Subject, takeUntil } from 'rxjs';

import { HardwareService, StatusService, DeviceInfoService } from '@core/services';
import {
  CpuInfo, CpuMetrics,
  MemoryInfo, MemoryMetrics, MemoryModule,
  GpuInfo, GpuMetrics,
  MotherboardInfo, Monitor
} from '@core/models';
import { DeviceType, ProductImages } from '@core/models/device-info.model';
import { ProgressRingComponent, DeviceDetailModalComponent, DeviceImageComponent } from '@shared/components';
import { BytesPipe, DecimalPipe } from '@shared/pipes';

@Component({
  selector: 'app-hardware',
  standalone: true,
  imports: [CommonModule, ProgressRingComponent, DeviceDetailModalComponent, DeviceImageComponent, BytesPipe, DecimalPipe],
  template: `
    <div class="p-6 space-y-6">
      <!-- Header -->
      <div>
        <h1 class="text-2xl font-bold text-syslens-text-primary">Hardware</h1>
        <p class="text-syslens-text-secondary">CPU, Memory, GPU, and peripherals</p>
      </div>

      <!-- CPU Section -->
      <section class="card">
        <div class="flex items-center justify-between mb-4">
          <h2 class="section-title mb-0">Processor (CPU)</h2>
          @if (cpuInfo) {
            <button
              (click)="openDeviceDetails(getCpuDeviceId(), 'Cpu')"
              class="px-3 py-1 text-sm bg-syslens-bg-tertiary hover:bg-syslens-bg-primary text-syslens-text-secondary hover:text-syslens-text-primary rounded transition-colors"
            >
              Details
            </button>
          }
        </div>
        @if (cpuInfo) {
          <div class="flex flex-col lg:flex-row gap-6">
            <!-- CPU Info -->
            <div class="flex-1">
              <div class="flex items-center gap-3 mb-4">
                @if (cpuImages?.primaryImage || cpuImages?.primaryImageCached) {
                  <app-device-image
                    [src]="cpuImages?.primaryImage"
                    [cachedPath]="cpuImages?.primaryImageCached"
                    alt="CPU"
                    width="48px"
                    height="48px"
                    containerClass="rounded-lg bg-syslens-bg-tertiary flex-shrink-0"
                  />
                }
                <h3 class="text-lg font-medium text-syslens-text-primary">{{ cpuInfo.name }}</h3>
              </div>
              <div class="grid grid-cols-2 gap-4">
                <div>
                  <p class="text-xs text-syslens-text-muted">Manufacturer</p>
                  <p class="text-sm text-syslens-text-primary">{{ cpuInfo.manufacturer }}</p>
                </div>
                <div>
                  <p class="text-xs text-syslens-text-muted">Architecture</p>
                  <p class="text-sm text-syslens-text-primary">{{ cpuInfo.architecture }}</p>
                </div>
                <div>
                  <p class="text-xs text-syslens-text-muted">Cores / Threads</p>
                  <p class="text-sm text-syslens-text-primary">{{ cpuInfo.physicalCores }} / {{ cpuInfo.logicalProcessors }}</p>
                </div>
                <div>
                  <p class="text-xs text-syslens-text-muted">Base Clock</p>
                  <p class="text-sm text-syslens-text-primary">{{ cpuInfo.baseClockMhz / 1000 | decimal:1 }} GHz</p>
                </div>
                <div>
                  <p class="text-xs text-syslens-text-muted">Max Clock</p>
                  <p class="text-sm text-syslens-text-primary">{{ cpuInfo.maxClockMhz / 1000 | decimal:1 }} GHz</p>
                </div>
                <div>
                  <p class="text-xs text-syslens-text-muted">Socket</p>
                  <p class="text-sm text-syslens-text-primary">{{ cpuInfo.socket }}</p>
                </div>
              </div>
              <!-- Cache Info -->
              <div class="mt-4 pt-4 border-t border-syslens-border-primary">
                <p class="text-xs text-syslens-text-muted mb-2">Cache</p>
                <div class="flex gap-4 text-sm">
                  <span class="text-syslens-text-secondary">L1: {{ cpuInfo.cache.l1DataKb + cpuInfo.cache.l1InstructionKb }} KB</span>
                  <span class="text-syslens-text-secondary">L2: {{ cpuInfo.cache.l2Kb }} KB</span>
                  <span class="text-syslens-text-secondary">L3: {{ cpuInfo.cache.l3Kb / 1024 | decimal:0 }} MB</span>
                </div>
              </div>
            </div>

            <!-- CPU Metrics -->
            <div class="flex flex-col items-center gap-4">
              <app-progress-ring
                [value]="cpuMetrics?.totalUsage ?? 0"
                label="Usage"
                [size]="140"
              />
              @if (cpuMetrics?.currentClockMhz) {
                <p class="text-sm text-syslens-text-secondary">
                  {{ cpuMetrics!.currentClockMhz / 1000 | decimal:1 }} GHz
                </p>
              }
              @if (cpuMetrics?.temperature) {
                <p class="text-sm" [class.text-syslens-accent-red]="cpuMetrics!.temperature! > 80"
                   [class.text-syslens-text-secondary]="cpuMetrics!.temperature! <= 80">
                  {{ cpuMetrics!.temperature }}C
                </p>
              }
            </div>
          </div>

          <!-- Per-Core Usage -->
          @if (cpuMetrics?.perCoreUsage?.length) {
            <div class="mt-6 pt-4 border-t border-syslens-border-primary">
              <p class="text-xs text-syslens-text-muted mb-3">Per-Core Usage</p>
              <div class="grid grid-cols-4 md:grid-cols-8 lg:grid-cols-12 gap-2">
                @for (usage of cpuMetrics!.perCoreUsage; track $index) {
                  <div class="text-center">
                    <div class="h-12 w-full bg-syslens-bg-tertiary rounded overflow-hidden">
                      <div class="w-full transition-all duration-300"
                           [style.height.%]="usage"
                           [class.bg-syslens-accent-blue]="usage < 75"
                           [class.bg-syslens-accent-yellow]="usage >= 75 && usage < 90"
                           [class.bg-syslens-accent-red]="usage >= 90">
                      </div>
                    </div>
                    <p class="text-xs text-syslens-text-muted mt-1">{{ $index }}</p>
                  </div>
                }
              </div>
            </div>
          }
        }
      </section>

      <!-- Memory Section -->
      <section class="card">
        <h2 class="section-title">Memory (RAM)</h2>
        @if (memoryInfo) {
          <div class="flex flex-col lg:flex-row gap-6">
            <!-- Memory Info -->
            <div class="flex-1">
              <div class="grid grid-cols-2 gap-4">
                <div>
                  <p class="text-xs text-syslens-text-muted">Total Installed</p>
                  <p class="text-lg font-semibold text-syslens-text-primary">{{ memoryInfo.totalBytes | bytes }}</p>
                </div>
                <div>
                  <p class="text-xs text-syslens-text-muted">Usable</p>
                  <p class="text-lg font-semibold text-syslens-text-primary">{{ memoryInfo.usableBytes | bytes }}</p>
                </div>
                <div>
                  <p class="text-xs text-syslens-text-muted">Type</p>
                  <p class="text-sm text-syslens-text-primary">{{ memoryInfo.memoryType }}</p>
                </div>
                <div>
                  <p class="text-xs text-syslens-text-muted">Speed</p>
                  <p class="text-sm text-syslens-text-primary">{{ memoryInfo.speedMhz }} MHz</p>
                </div>
                <div>
                  <p class="text-xs text-syslens-text-muted">Slots Used</p>
                  <p class="text-sm text-syslens-text-primary">{{ memoryInfo.slotsUsed }} / {{ memoryInfo.slotsTotal }}</p>
                </div>
              </div>

              <!-- Memory Modules -->
              @if (memoryInfo.modules.length > 0) {
                <div class="mt-4 pt-4 border-t border-syslens-border-primary">
                  <p class="text-xs text-syslens-text-muted mb-2">Installed Modules</p>
                  <div class="space-y-3">
                    @for (module of memoryInfo.modules; track module.slot) {
                      <div
                        class="p-2 bg-syslens-bg-primary rounded text-sm cursor-pointer hover:bg-syslens-bg-tertiary transition-colors"
                        (click)="openDeviceDetails(getMemoryModuleDeviceId(module), 'Memory')"
                      >
                        <div class="flex items-center justify-between mb-1">
                          <span class="font-medium text-syslens-text-primary">{{ module.slot }}</span>
                          <span class="text-syslens-text-primary">{{ module.capacityBytes | bytes }}</span>
                        </div>
                        <div class="grid grid-cols-2 gap-2 text-xs text-syslens-text-muted">
                          <div>Manufacturer: <span class="text-syslens-text-secondary">{{ module.manufacturer }}</span></div>
                          <div>Speed: <span class="text-syslens-text-secondary">{{ module.speedMhz }} MHz</span></div>
                          @if (module.partNumber && module.partNumber !== 'Unknown') {
                            <div class="col-span-2">Part: <span class="text-syslens-text-secondary">{{ module.partNumber }}</span></div>
                          }
                        </div>
                      </div>
                    }
                  </div>
                </div>
              }
            </div>

            <!-- Memory Metrics -->
            <div class="flex flex-col items-center gap-4">
              @if (memoryMetrics) {
                <app-progress-ring
                  [value]="(memoryMetrics.inUseBytes / memoryInfo.totalBytes) * 100"
                  label="In Use"
                  [size]="140"
                  colorClass="stroke-syslens-accent-purple"
                />
                <div class="text-center">
                  <p class="text-sm text-syslens-text-primary">{{ memoryMetrics.inUseBytes | bytes }} used</p>
                  <p class="text-xs text-syslens-text-muted">{{ memoryMetrics.availableBytes | bytes }} available</p>
                </div>
              }
            </div>
          </div>
        }
      </section>

      <!-- GPU Section -->
      <section class="card">
        <h2 class="section-title">Graphics (GPU)</h2>
        <div class="space-y-4">
          @for (gpu of gpuInfoList; track gpu.id) {
            <div class="p-4 bg-syslens-bg-tertiary rounded-lg">
              <div class="flex flex-col lg:flex-row gap-4">
                <div class="flex-1">
                  <div class="flex items-start justify-between">
                    <div class="flex items-center gap-3">
                      @if (gpuImagesMap[gpu.id]; as images) {
                        @if (images.primaryImage || images.primaryImageCached) {
                          <app-device-image
                            [src]="images.primaryImage"
                            [cachedPath]="images.primaryImageCached"
                            alt="GPU"
                            width="48px"
                            height="48px"
                            containerClass="rounded-lg bg-syslens-bg-secondary flex-shrink-0"
                          />
                        }
                      }
                      <div>
                        <h3 class="font-medium text-syslens-text-primary">{{ gpu.name }}</h3>
                        <p class="text-sm text-syslens-text-muted">{{ gpu.manufacturer }}</p>
                      </div>
                    </div>
                    <button
                      (click)="openDeviceDetails(getGpuDeviceId(gpu), 'Gpu')"
                      class="px-3 py-1 text-sm bg-syslens-bg-secondary hover:bg-syslens-bg-primary text-syslens-text-secondary hover:text-syslens-text-primary rounded transition-colors"
                    >
                      Details
                    </button>
                  </div>
                  <div class="mt-3 grid grid-cols-2 gap-3 text-sm">
                    <div>
                      <p class="text-xs text-syslens-text-muted">VRAM</p>
                      <p class="text-syslens-text-primary">{{ gpu.vramBytes | bytes }}</p>
                    </div>
                    <div>
                      <p class="text-xs text-syslens-text-muted">Driver Version</p>
                      <p class="text-syslens-text-primary">
                        {{ gpu.driverVersion }}
                        @if (gpu.driverLink) {
                          <a [href]="gpu.driverLink" target="_blank" class="ml-1 text-syslens-accent-blue hover:underline" title="Download drivers">
                            ↗
                          </a>
                        }
                      </p>
                    </div>
                    <div>
                      <p class="text-xs text-syslens-text-muted">Driver Date</p>
                      <p class="text-syslens-text-primary">{{ gpu.driverDate }}</p>
                    </div>
                    <div>
                      <p class="text-xs text-syslens-text-muted">Type</p>
                      <p class="text-syslens-text-primary">{{ gpu.adapterType }}</p>
                    </div>
                    <div>
                      <p class="text-xs text-syslens-text-muted">Resolution</p>
                      <p class="text-syslens-text-primary">{{ gpu.currentResolution }}</p>
                    </div>
                    <div>
                      <p class="text-xs text-syslens-text-muted">Refresh Rate</p>
                      <p class="text-syslens-text-primary">{{ gpu.refreshRateHz }} Hz</p>
                    </div>
                  </div>
                </div>

                <!-- GPU Metrics (if available) -->
                @if (gpuMetricsMap[gpu.id]; as metrics) {
                  <div class="flex items-center gap-4">
                    <app-progress-ring
                      [value]="metrics.usagePercent"
                      label="GPU"
                      [size]="80"
                      colorClass="stroke-syslens-accent-cyan"
                    />
                    <div class="text-sm">
                      @if (metrics.temperature) {
                        <p class="text-syslens-text-secondary">{{ metrics.temperature }}C</p>
                      }
                      @if (metrics.vramUsedBytes) {
                        <p class="text-syslens-text-muted">{{ metrics.vramUsedBytes | bytes }} VRAM</p>
                      }
                    </div>
                  </div>
                }
              </div>
            </div>
          } @empty {
            <p class="text-syslens-text-muted text-center py-4">No GPU information available</p>
          }
        </div>
      </section>

      <!-- Monitors Section -->
      <section class="card">
        <h2 class="section-title">Displays</h2>
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          @for (monitor of monitors; track monitor.id) {
            <div
              class="p-4 bg-syslens-bg-tertiary rounded-lg cursor-pointer hover:bg-syslens-bg-secondary transition-colors"
              (click)="openDeviceDetails(getMonitorDeviceId(monitor), 'Monitor')"
            >
              <h3 class="font-medium text-syslens-text-primary">{{ monitor.name }}</h3>
              @if (monitor.manufacturer) {
                <p class="text-sm text-syslens-text-muted">{{ monitor.manufacturer }}</p>
              }
              <div class="mt-2 grid grid-cols-2 gap-2 text-sm">
                <div>
                  <p class="text-xs text-syslens-text-muted">Resolution</p>
                  <p class="text-syslens-text-primary">{{ monitor.resolution }}</p>
                </div>
                <div>
                  <p class="text-xs text-syslens-text-muted">Refresh Rate</p>
                  <p class="text-syslens-text-primary">{{ monitor.refreshRateHz }} Hz</p>
                </div>
                <div>
                  <p class="text-xs text-syslens-text-muted">Connection</p>
                  <p class="text-syslens-text-primary">{{ monitor.connection }}</p>
                </div>
                @if (monitor.sizeInches) {
                  <div>
                    <p class="text-xs text-syslens-text-muted">Size</p>
                    <p class="text-syslens-text-primary">{{ monitor.sizeInches }}"</p>
                  </div>
                }
              </div>
              @if (monitor.hdrSupport) {
                <span class="mt-2 inline-block px-2 py-0.5 text-xs bg-syslens-accent-yellow text-black rounded">HDR</span>
              }
            </div>
          } @empty {
            <p class="text-syslens-text-muted text-center py-4 col-span-full">No monitor information available</p>
          }
        </div>
      </section>

      <!-- Motherboard Section -->
      @if (motherboardInfo) {
        <section class="card">
          <div class="flex items-center justify-between mb-4">
            <h2 class="section-title mb-0">Motherboard</h2>
            <button
              (click)="openDeviceDetails(getMotherboardDeviceId(), 'Motherboard')"
              class="px-3 py-1 text-sm bg-syslens-bg-tertiary hover:bg-syslens-bg-primary text-syslens-text-secondary hover:text-syslens-text-primary rounded transition-colors"
            >
              Details
            </button>
          </div>
          <div class="flex flex-col lg:flex-row gap-6">
            <div class="flex-1">
              <div class="grid grid-cols-2 md:grid-cols-3 gap-4">
                <div>
                  <p class="text-xs text-syslens-text-muted">Manufacturer</p>
                  <p class="text-sm text-syslens-text-primary">
                    {{ motherboardInfo.manufacturer }}
                    @if (motherboardInfo.supportUrl) {
                      <a [href]="motherboardInfo.supportUrl" target="_blank" class="ml-1 text-syslens-accent-blue hover:underline" title="Manufacturer support">
                        ↗
                      </a>
                    }
                  </p>
                </div>
                <div>
                  <p class="text-xs text-syslens-text-muted">Product</p>
                  <p class="text-sm text-syslens-text-primary">{{ motherboardInfo.product }}</p>
                </div>
                @if (motherboardInfo.version) {
                  <div>
                    <p class="text-xs text-syslens-text-muted">Version</p>
                    <p class="text-sm text-syslens-text-primary">{{ motherboardInfo.version }}</p>
                  </div>
                }
                @if (motherboardInfo.chipset) {
                  <div>
                    <p class="text-xs text-syslens-text-muted">Chipset</p>
                    <p class="text-sm text-syslens-text-primary">{{ motherboardInfo.chipset }}</p>
                  </div>
                }
                @if (motherboardInfo.serialNumber) {
                  <div>
                    <p class="text-xs text-syslens-text-muted">Serial Number</p>
                    <p class="text-sm text-syslens-text-primary">{{ motherboardInfo.serialNumber }}</p>
                  </div>
                }
                @if (motherboardInfo.formFactor) {
                  <div>
                    <p class="text-xs text-syslens-text-muted">Form Factor</p>
                    <p class="text-sm text-syslens-text-primary">{{ motherboardInfo.formFactor }}</p>
                  </div>
                }
              </div>

              <!-- BIOS Information -->
              @if (motherboardInfo.biosVendor || motherboardInfo.biosVersion) {
                <div class="mt-4 pt-4 border-t border-syslens-border-primary">
                  <p class="text-xs text-syslens-text-muted mb-2">BIOS Information</p>
                  <div class="grid grid-cols-2 md:grid-cols-3 gap-4 text-sm">
                    @if (motherboardInfo.biosVendor) {
                      <div>
                        <p class="text-xs text-syslens-text-muted">Vendor</p>
                        <p class="text-syslens-text-primary">{{ motherboardInfo.biosVendor }}</p>
                      </div>
                    }
                    @if (motherboardInfo.biosVersion) {
                      <div>
                        <p class="text-xs text-syslens-text-muted">Version</p>
                        <p class="text-syslens-text-primary">{{ motherboardInfo.biosVersion }}</p>
                      </div>
                    }
                    @if (motherboardInfo.biosReleaseDate) {
                      <div>
                        <p class="text-xs text-syslens-text-muted">Release Date</p>
                        <p class="text-syslens-text-primary">{{ motherboardInfo.biosReleaseDate }}</p>
                      </div>
                    }
                  </div>
                </div>
              }

              <!-- Security & Boot Information -->
              @if (motherboardInfo.bootMode || motherboardInfo.secureBoot !== null || motherboardInfo.tpmVersion) {
                <div class="mt-4 pt-4 border-t border-syslens-border-primary">
                  <p class="text-xs text-syslens-text-muted mb-2">Security & Boot</p>
                  <div class="grid grid-cols-2 md:grid-cols-3 gap-4 text-sm">
                    @if (motherboardInfo.bootMode) {
                      <div>
                        <p class="text-xs text-syslens-text-muted">Boot Mode</p>
                        <p class="text-syslens-text-primary">{{ motherboardInfo.bootMode }}</p>
                      </div>
                    }
                    @if (motherboardInfo.secureBoot !== null) {
                      <div>
                        <p class="text-xs text-syslens-text-muted">Secure Boot</p>
                        <p [class.text-syslens-accent-green]="motherboardInfo.secureBoot"
                           [class.text-syslens-text-secondary]="!motherboardInfo.secureBoot">
                          {{ motherboardInfo.secureBoot ? 'Enabled' : 'Disabled' }}
                        </p>
                      </div>
                    }
                    @if (motherboardInfo.tpmVersion) {
                      <div>
                        <p class="text-xs text-syslens-text-muted">TPM</p>
                        <p class="text-syslens-text-primary">{{ motherboardInfo.tpmVersion }}</p>
                      </div>
                    }
                  </div>
                </div>
              }
            </div>

            <!-- Motherboard Image (if available) -->
            @if (motherboardInfo.imageUrl) {
              <div class="flex items-center justify-center lg:w-64">
                <img [src]="motherboardInfo.imageUrl" [alt]="motherboardInfo.product" class="max-w-full h-auto rounded-lg">
              </div>
            }
          </div>
        </section>
      }

      <!-- Device Detail Modal -->
      <app-device-detail-modal
        [isOpen]="modalOpen"
        [deviceId]="modalDeviceId"
        [deviceType]="modalDeviceType"
        (closed)="closeModal()"
      />
    </div>
  `
})
export class HardwareComponent implements OnInit, OnDestroy {
  private hardwareService = inject(HardwareService);
  private statusService = inject(StatusService);
  private deviceInfoService = inject(DeviceInfoService);
  private destroy$ = new Subject<void>();

  cpuInfo: CpuInfo | null = null;
  cpuMetrics: CpuMetrics | null = null;
  memoryInfo: MemoryInfo | null = null;
  memoryMetrics: MemoryMetrics | null = null;
  gpuInfoList: GpuInfo[] = [];
  gpuMetricsMap: Record<string, GpuMetrics> = {};
  motherboardInfo: MotherboardInfo | null = null;
  monitors: Monitor[] = [];

  // Device images
  cpuImages: ProductImages | null = null;
  gpuImagesMap: Record<string, ProductImages> = {};

  // Modal state
  modalOpen = false;
  modalDeviceId = '';
  modalDeviceType: DeviceType = 'Cpu';

  ngOnInit(): void {
    this.loadHardwareInfo();
    this.startRealtimeUpdates();
  }

  ngOnDestroy(): void {
    this.destroy$.next();
    this.destroy$.complete();
  }

  private loadHardwareInfo(): void {
    this.statusService.startOperation('hardware-init', 'Loading hardware information...');

    this.hardwareService.getCpuInfo()
      .pipe(takeUntil(this.destroy$))
      .subscribe(info => {
        this.cpuInfo = info;
        this.statusService.endOperation('hardware-init');
        // Fetch CPU images
        this.fetchDeviceImages('Cpu', info.manufacturer, info.name);
      });

    this.hardwareService.getMemoryInfo()
      .pipe(takeUntil(this.destroy$))
      .subscribe(info => this.memoryInfo = info);

    this.hardwareService.getGpuInfo()
      .pipe(takeUntil(this.destroy$))
      .subscribe(gpus => {
        this.gpuInfoList = gpus;
        // Fetch GPU images for each GPU
        gpus.forEach(gpu => this.fetchGpuImages(gpu));
      });

    this.hardwareService.getMotherboardInfo()
      .pipe(takeUntil(this.destroy$))
      .subscribe(info => this.motherboardInfo = info);

    this.hardwareService.getMonitors()
      .pipe(takeUntil(this.destroy$))
      .subscribe(monitors => this.monitors = monitors);
  }

  private fetchDeviceImages(deviceType: DeviceType, manufacturer: string, model: string): void {
    this.deviceInfoService.enrichDevice(deviceType, manufacturer, model)
      .pipe(takeUntil(this.destroy$))
      .subscribe({
        next: (enriched) => {
          if (enriched.images) {
            this.cpuImages = enriched.images;
          }
        },
        error: () => {
          // Silently fail - images are optional
        }
      });
  }

  private fetchGpuImages(gpu: GpuInfo): void {
    this.deviceInfoService.enrichDevice('Gpu', gpu.manufacturer, gpu.name)
      .pipe(takeUntil(this.destroy$))
      .subscribe({
        next: (enriched) => {
          if (enriched.images) {
            this.gpuImagesMap = { ...this.gpuImagesMap, [gpu.id]: enriched.images };
          }
        },
        error: () => {
          // Silently fail - images are optional
        }
      });
  }

  private startRealtimeUpdates(): void {
    this.hardwareService.getCpuMetricsPolling()
      .pipe(takeUntil(this.destroy$))
      .subscribe(metrics => this.cpuMetrics = metrics);

    this.hardwareService.getMemoryMetricsPolling()
      .pipe(takeUntil(this.destroy$))
      .subscribe(metrics => this.memoryMetrics = metrics);

    this.hardwareService.getGpuMetricsPolling()
      .pipe(takeUntil(this.destroy$))
      .subscribe(metrics => {
        this.gpuMetricsMap = {};
        metrics.forEach(m => this.gpuMetricsMap[m.gpuId] = m);
      });
  }

  // Modal methods
  openDeviceDetails(deviceId: string, deviceType: DeviceType): void {
    this.modalDeviceId = deviceId;
    this.modalDeviceType = deviceType;
    this.modalOpen = true;
  }

  closeModal(): void {
    this.modalOpen = false;
  }

  getCpuDeviceId(): string {
    if (!this.cpuInfo) return '';
    const manufacturer = this.normalizeManufacturer(this.cpuInfo.manufacturer);
    const model = this.cleanModelName(this.cpuInfo.name, manufacturer);
    return `cpu-${manufacturer}-${model}`;
  }

  getGpuDeviceId(gpu: GpuInfo): string {
    const manufacturer = this.normalizeManufacturer(gpu.manufacturer);
    const model = this.cleanModelName(gpu.name, manufacturer);
    return `gpu-${manufacturer}-${model}`;
  }

  getMotherboardDeviceId(): string {
    if (!this.motherboardInfo) return '';
    const manufacturer = this.normalizeManufacturer(this.motherboardInfo.manufacturer);
    const model = this.cleanModelName(this.motherboardInfo.product, manufacturer);
    return `mb-${manufacturer}-${model}`;
  }

  getMonitorDeviceId(monitor: Monitor): string {
    const manufacturer = monitor.manufacturer
      ? this.normalizeManufacturer(monitor.manufacturer)
      : 'unknown';
    const model = this.cleanModelName(monitor.name, manufacturer);
    return `monitor-${manufacturer}-${model}`;
  }

  getMemoryModuleDeviceId(module: MemoryModule): string {
    const manufacturer = module.manufacturer && module.manufacturer !== 'Unknown'
      ? this.normalizeManufacturer(module.manufacturer)
      : 'unknown';
    // Use part number if available, otherwise use slot + capacity
    const model = module.partNumber && module.partNumber !== 'Unknown'
      ? module.partNumber.toLowerCase().replace(/[^a-z0-9]+/g, '-')
      : `${module.slot}-${module.capacityBytes}`.toLowerCase().replace(/[^a-z0-9]+/g, '-');
    return `memory-${manufacturer}-${model}`;
  }

  /**
   * Normalize manufacturer name to a clean identifier.
   * Handles vendor IDs and long company names.
   */
  private normalizeManufacturer(raw: string): string {
    const lower = raw.toLowerCase();

    // Map common vendor IDs and variations
    if (lower.includes('authenticamd') || lower.includes('advanced micro')) return 'amd';
    if (lower.includes('genuineintel') || lower.includes('intel')) return 'intel';
    if (lower.includes('nvidia')) return 'nvidia';
    if (lower.includes('micro-star') || lower.includes('msi')) return 'msi';
    if (lower.includes('asustek') || lower.includes('asus')) return 'asus';
    if (lower.includes('gigabyte')) return 'gigabyte';
    if (lower.includes('asrock')) return 'asrock';

    // Default: clean and return
    return lower.replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '');
  }

  /**
   * Clean model name by removing manufacturer prefix if present.
   * Prevents duplication like "gpu-nvidia-nvidia-geforce-rtx-5070".
   */
  private cleanModelName(model: string, manufacturer: string): string {
    let cleaned = model.toLowerCase();

    // Remove manufacturer prefix if present at the start
    const prefixes = [
      manufacturer,
      `${manufacturer} `,
      `${manufacturer}-`,
    ];

    for (const prefix of prefixes) {
      if (cleaned.startsWith(prefix)) {
        cleaned = cleaned.slice(prefix.length);
        break;
      }
    }

    // Clean and normalize
    return cleaned.replace(/\s+/g, '-').replace(/[^a-z0-9-]/g, '').replace(/^-|-$/g, '');
  }
}
