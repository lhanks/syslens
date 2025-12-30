import { Component, inject, computed, OnInit, OnDestroy } from '@angular/core';
import { Router, RouterLink, RouterLinkActive } from '@angular/router';
import { Subject, takeUntil } from 'rxjs';
import { PreloadService, MetricsHistoryService, SystemService, ViewSettingsService, HardwareService, StorageService } from '@core/services';
import { BytesPipe, UptimePipe } from '@shared/pipes';

interface NavItem {
  label: string;
  route: string;
  icon: string;
}

@Component({
  selector: 'app-sidebar',
  standalone: true,
  imports: [RouterLink, RouterLinkActive, BytesPipe, UptimePipe],
  template: `
    <aside class="w-64 h-screen bg-syslens-bg-secondary border-r border-syslens-border-primary flex flex-col">
      <!-- Header -->
      <div class="p-4 border-b border-syslens-border-primary">
        @if (deviceName) {
          <h1 class="text-base font-semibold text-syslens-text-primary truncate" [title]="deviceName">{{ deviceName }}</h1>
          <p class="text-xs text-syslens-text-muted">Up {{ uptimeSeconds | uptime }}</p>
        } @else {
          <div class="h-5 w-32 bg-syslens-bg-tertiary rounded animate-pulse mb-1"></div>
          <div class="h-3 w-20 bg-syslens-bg-tertiary rounded animate-pulse"></div>
        }
      </div>

      <!-- System Status Summary -->
      <div class="p-3 border-b border-syslens-border-primary space-y-2">
        <!-- CPU -->
        @if (showCpu()) {
          <div class="flex items-center gap-2 cursor-pointer rounded-md px-1 py-0.5 -mx-1 hover:bg-syslens-bg-hover transition-colors"
               (click)="navigateTo('/system')">
            <div class="w-6 h-6 rounded bg-syslens-accent-blue/20 flex items-center justify-center flex-shrink-0">
              <svg class="w-3.5 h-3.5 text-syslens-accent-blue" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                  d="M9 3v2m6-2v2M9 19v2m6-2v2M5 9H3m2 6H3m18-6h-2m2 6h-2M7 19h10a2 2 0 002-2V7a2 2 0 00-2-2H7a2 2 0 00-2 2v10a2 2 0 002 2zM9 9h6v6H9V9z" />
              </svg>
            </div>
            <div class="flex-1 min-w-0">
              <div class="flex items-center justify-between text-xs mb-0.5">
                <span class="flex items-center gap-1">
                  <span class="text-syslens-text-muted">CPU</span>
                  @if (cpuVendor) {<span class="vendor-badge" [class]="getVendorBadgeClass(cpuVendor)">{{ cpuVendor }}</span>}
                </span>
                <span class="font-mono text-syslens-text-secondary">{{ cpuUsage().toFixed(0) }}%</span>
              </div>
              <div class="h-1 bg-syslens-bg-tertiary rounded-full overflow-hidden">
                <div class="h-full rounded-full transition-all duration-300"
                     [style.width.%]="cpuUsage()"
                     [class.bg-syslens-accent-green]="cpuUsage() < 50"
                     [class.bg-syslens-accent-yellow]="cpuUsage() >= 50 && cpuUsage() < 80"
                     [class.bg-syslens-accent-red]="cpuUsage() >= 80">
                </div>
              </div>
            </div>
          </div>
        }

        <!-- Memory -->
        @if (showMemory()) {
          <div class="flex items-center gap-2 cursor-pointer rounded-md px-1 py-0.5 -mx-1 hover:bg-syslens-bg-hover transition-colors"
               (click)="navigateTo('/system')">
            <div class="w-6 h-6 rounded bg-syslens-accent-purple/20 flex items-center justify-center flex-shrink-0">
              <svg class="w-3.5 h-3.5 text-syslens-accent-purple" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                  d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
              </svg>
            </div>
            <div class="flex-1 min-w-0">
              <div class="flex items-center justify-between text-xs mb-0.5">
                <span class="flex items-center gap-1">
                  <span class="text-syslens-text-muted">RAM</span>
                  @if (memoryType) {<span class="vendor-badge" [class]="getVendorBadgeClass(memoryType)">{{ memoryType }}</span>}
                </span>
                <span class="font-mono text-syslens-text-secondary">{{ memoryUsage().toFixed(0) }}%</span>
              </div>
              <div class="h-1 bg-syslens-bg-tertiary rounded-full overflow-hidden">
                <div class="h-full rounded-full transition-all duration-300"
                     [style.width.%]="memoryUsage()"
                     [class.bg-syslens-accent-green]="memoryUsage() < 60"
                     [class.bg-syslens-accent-yellow]="memoryUsage() >= 60 && memoryUsage() < 85"
                     [class.bg-syslens-accent-red]="memoryUsage() >= 85">
                </div>
              </div>
            </div>
          </div>
        }

        <!-- Disk -->
        @if (showDisk()) {
          <div class="flex items-center gap-2 cursor-pointer rounded-md px-1 py-0.5 -mx-1 hover:bg-syslens-bg-hover transition-colors"
               (click)="navigateTo('/storage')">
            <div class="w-6 h-6 rounded bg-syslens-accent-cyan/20 flex items-center justify-center flex-shrink-0">
              <svg class="w-3.5 h-3.5 text-syslens-accent-cyan" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                  d="M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4" />
              </svg>
            </div>
            <div class="flex-1 min-w-0">
              <div class="flex items-center justify-between text-xs mb-0.5">
                <span class="flex items-center gap-1">
                  <span class="text-syslens-text-muted">Disk</span>
                  @if (diskVendor) {<span class="vendor-badge" [class]="getVendorBadgeClass(diskVendor)">{{ diskVendor }}</span>}
                </span>
                <span class="font-mono text-syslens-text-secondary">{{ diskActivity().toFixed(0) }}%</span>
              </div>
              <div class="h-1 bg-syslens-bg-tertiary rounded-full overflow-hidden">
                <div class="h-full rounded-full transition-all duration-300"
                     [style.width.%]="diskActivity()"
                     [class.bg-syslens-accent-green]="diskActivity() < 50"
                     [class.bg-syslens-accent-yellow]="diskActivity() >= 50 && diskActivity() < 80"
                     [class.bg-syslens-accent-red]="diskActivity() >= 80">
                </div>
              </div>
            </div>
          </div>
        }

        <!-- GPU -->
        @if (showGpu()) {
          <div class="flex items-center gap-2 cursor-pointer rounded-md px-1 py-0.5 -mx-1 hover:bg-syslens-bg-hover transition-colors"
               (click)="navigateTo('/hardware')">
            <div class="w-6 h-6 rounded bg-syslens-accent-orange/20 flex items-center justify-center flex-shrink-0">
              <svg class="w-3.5 h-3.5 text-syslens-accent-orange" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                  d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
              </svg>
            </div>
            <div class="flex-1 min-w-0">
              <div class="flex items-center justify-between text-xs mb-0.5">
                <span class="flex items-center gap-1">
                  <span class="text-syslens-text-muted">GPU</span>
                  @if (gpuVendor) {<span class="vendor-badge" [class]="getVendorBadgeClass(gpuVendor)">{{ gpuVendor }}</span>}
                </span>
                <span class="font-mono text-syslens-text-secondary">{{ gpuUsage().toFixed(0) }}%</span>
              </div>
              <div class="h-1 bg-syslens-bg-tertiary rounded-full overflow-hidden">
                <div class="h-full rounded-full transition-all duration-300"
                     [style.width.%]="gpuUsage()"
                     [class.bg-syslens-accent-green]="gpuUsage() < 50"
                     [class.bg-syslens-accent-yellow]="gpuUsage() >= 50 && gpuUsage() < 80"
                     [class.bg-syslens-accent-red]="gpuUsage() >= 80">
                </div>
              </div>
            </div>
          </div>
        }

        <!-- Network - Per Adapter -->
        @if (showNetwork()) {
          @for (adapter of adapterHistoryArray(); track adapter.adapterId) {
            <div class="flex items-center gap-2 cursor-pointer rounded-md px-1 py-0.5 -mx-1 hover:bg-syslens-bg-hover transition-colors"
                 (click)="navigateTo('/network')">
              <div class="w-6 h-6 rounded bg-syslens-accent-green/20 flex items-center justify-center flex-shrink-0">
                <svg class="w-3.5 h-3.5 text-syslens-accent-green" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                    d="M8 16l2.879-2.879m0 0a3 3 0 104.243-4.242 3 3 0 00-4.243 4.242zM21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
              </div>
              <div class="flex-1 min-w-0">
                <div class="flex items-center justify-between text-xs mb-0.5">
                  <span class="text-syslens-text-muted truncate max-w-[60px]" [title]="adapter.adapterName">{{ adapter.adapterName }}</span>
                </div>
                <div class="font-mono text-[11px] text-syslens-text-secondary flex gap-1.5">
                  <span class="text-syslens-accent-green w-[63px] text-right">↓{{ adapter.downloadSpeed | bytes }}/s</span>
                  <span class="text-syslens-accent-blue w-[63px] text-right">↑{{ adapter.uploadSpeed | bytes }}/s</span>
                </div>
              </div>
            </div>
          } @empty {
            <div class="flex items-center gap-2 cursor-pointer rounded-md px-1 py-0.5 -mx-1 hover:bg-syslens-bg-hover transition-colors"
                 (click)="navigateTo('/network')">
              <div class="w-6 h-6 rounded bg-syslens-accent-green/20 flex items-center justify-center flex-shrink-0">
                <svg class="w-3.5 h-3.5 text-syslens-accent-green" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                    d="M8 16l2.879-2.879m0 0a3 3 0 104.243-4.242 3 3 0 00-4.243 4.242zM21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
              </div>
              <div class="flex-1 min-w-0">
                <div class="text-xs text-syslens-text-muted">No active adapters</div>
              </div>
            </div>
          }
        }
      </div>

      <!-- Navigation -->
      <nav class="flex-1 p-3 space-y-1 overflow-y-auto">
        @for (item of navItems; track item.route) {
          <a
            [routerLink]="item.route"
            routerLinkActive="bg-syslens-bg-hover border-syslens-accent-blue text-syslens-text-primary"
            [routerLinkActiveOptions]="{ exact: item.route === '/system' }"
            (mouseenter)="onNavHover(item.route)"
            class="flex items-center gap-3 px-3 py-2.5 rounded-lg text-syslens-text-secondary
                   hover:bg-syslens-bg-hover hover:text-syslens-text-primary transition-colors
                   border border-transparent"
          >
            <span class="w-5 h-5" [innerHTML]="item.icon"></span>
            <span class="font-medium">{{ item.label }}</span>
          </a>
        }
      </nav>

      <!-- Footer -->
      <div class="p-4 border-t border-syslens-border-primary">
        <div class="text-xs text-syslens-text-muted">
          <p>Syslens v0.1.0</p>
        </div>
      </div>
    </aside>
  `,
  styles: [`
    :host {
      display: block;
    }

    .vendor-badge {
      font-size: 9px;
      font-weight: 600;
      padding: 1px 4px;
      border-radius: 3px;
      letter-spacing: 0.02em;
    }

    /* Intel - Blue */
    .vendor-intel {
      background: rgba(0, 113, 197, 0.2);
      color: #0071c5;
    }

    /* AMD - Red */
    .vendor-amd {
      background: rgba(237, 28, 36, 0.2);
      color: #ed1c24;
    }

    /* NVIDIA - Green */
    .vendor-nvidia {
      background: rgba(118, 185, 0, 0.2);
      color: #76b900;
    }

    /* Samsung - Blue */
    .vendor-samsung {
      background: rgba(20, 40, 160, 0.2);
      color: #6b99d5;
    }

    /* Western Digital - Gray */
    .vendor-wd {
      background: rgba(100, 100, 100, 0.2);
      color: #888;
    }

    /* Seagate - Teal */
    .vendor-seagate {
      background: rgba(0, 150, 136, 0.2);
      color: #00bfa5;
    }

    /* Crucial - Orange */
    .vendor-crucial {
      background: rgba(255, 87, 34, 0.2);
      color: #ff7043;
    }

    /* Kingston - Red-Orange */
    .vendor-kingston {
      background: rgba(255, 69, 0, 0.2);
      color: #ff6347;
    }

    /* Corsair - Yellow */
    .vendor-corsair {
      background: rgba(255, 193, 7, 0.2);
      color: #ffc107;
    }

    /* Memory Types (DDR4, DDR5) - Purple */
    .vendor-memory {
      background: rgba(156, 39, 176, 0.2);
      color: #ab47bc;
    }

    /* Default - Muted */
    .vendor-default {
      background: rgba(255, 255, 255, 0.1);
      color: rgba(255, 255, 255, 0.5);
    }
  `]
})
export class SidebarComponent implements OnInit, OnDestroy {
  private router = inject(Router);
  private preloadService = inject(PreloadService);
  private metricsService = inject(MetricsHistoryService);
  private systemService = inject(SystemService);
  private viewSettings = inject(ViewSettingsService);
  private hardwareService = inject(HardwareService);
  private storageService = inject(StorageService);
  private destroy$ = new Subject<void>();

  // Machine info
  deviceName = '';
  uptimeSeconds = 0;

  // Vendor info for mini graphs
  cpuVendor = '';
  gpuVendor = '';
  memoryType = '';
  diskVendor = '';

  // Mini graph visibility from settings
  showCpu = this.viewSettings.showCpuMiniGraph;
  showMemory = this.viewSettings.showMemoryMiniGraph;
  showDisk = this.viewSettings.showDiskMiniGraph;
  showGpu = this.viewSettings.showGpuMiniGraph;
  showNetwork = this.viewSettings.showNetworkMiniGraph;

  // System status from metrics service
  cpuUsage = computed(() => this.metricsService.cpuUsage());
  memoryUsage = computed(() => this.metricsService.memoryUsage());
  diskActivity = computed(() => this.metricsService.diskActivity());
  networkDown = computed(() => this.metricsService.networkDownSpeed());
  networkUp = computed(() => this.metricsService.networkUpSpeed());
  gpuUsage = computed(() => this.metricsService.gpuUsage());

  // Per-adapter traffic history
  adapterHistoryArray = computed(() => {
    const historyMap = this.metricsService.adapterTrafficHistory();
    return Array.from(historyMap.values());
  });

  navItems: NavItem[] = [
    {
      label: 'Network',
      route: '/network',
      icon: `<svg fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
          d="M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9a9 9 0 01-9-9m9 9c1.657 0 3-4.03 3-9s-1.343-9-3-9m0 18c-1.657 0-3-4.03-3-9s1.343-9 3-9m-9 9a9 9 0 019-9" />
      </svg>`
    },
    {
      label: 'System',
      route: '/system',
      icon: `<svg fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
          d="M9 3v2m6-2v2M9 19v2m6-2v2M5 9H3m2 6H3m18-6h-2m2 6h-2M7 19h10a2 2 0 002-2V7a2 2 0 00-2-2H7a2 2 0 00-2 2v10a2 2 0 002 2zM9 9h6v6H9V9z" />
      </svg>`
    },
    {
      label: 'Hardware',
      route: '/hardware',
      icon: `<svg fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
          d="M9 3v2m6-2v2M9 19v2m6-2v2M5 9H3m2 6H3m18-6h-2m2 6h-2M7 19h10a2 2 0 002-2V7a2 2 0 00-2-2H7a2 2 0 00-2 2v10a2 2 0 002 2zM9 9h6v6H9V9z" />
      </svg>`
    },
    {
      label: 'Storage',
      route: '/storage',
      icon: `<svg fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
          d="M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4m0 5c0 2.21-3.582 4-8 4s-8-1.79-8-4" />
      </svg>`
    },
    {
      label: 'Processes',
      route: '/processes',
      icon: `<svg fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
          d="M4 6h16M4 10h16M4 14h16M4 18h16" />
      </svg>`
    },
    {
      label: 'Services',
      route: '/services',
      icon: `<svg fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
          d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
          d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
      </svg>`
    },
    {
      label: 'Restore Points',
      route: '/restore-points',
      icon: `<svg fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
          d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
      </svg>`
    }
  ];

  ngOnInit(): void {
    // Load device name
    this.systemService.getDeviceInfo()
      .pipe(takeUntil(this.destroy$))
      .subscribe(deviceInfo => {
        this.deviceName = deviceInfo.computerName;
      });

    // Start uptime polling
    this.systemService.getUptime()
      .pipe(takeUntil(this.destroy$))
      .subscribe(uptime => {
        this.uptimeSeconds = uptime.uptimeSeconds;
      });

    // Load vendor info for mini graphs
    this.hardwareService.getCpuInfo()
      .pipe(takeUntil(this.destroy$))
      .subscribe(cpu => {
        this.cpuVendor = this.shortenVendorName(cpu.manufacturer);
      });

    this.hardwareService.getGpuInfo()
      .pipe(takeUntil(this.destroy$))
      .subscribe(gpus => {
        if (gpus.length > 0) {
          this.gpuVendor = this.shortenVendorName(gpus[0].manufacturer);
        }
      });

    this.hardwareService.getMemoryInfo()
      .pipe(takeUntil(this.destroy$))
      .subscribe(memory => {
        this.memoryType = memory.memoryType || '';
      });

    this.storageService.getPhysicalDisks()
      .pipe(takeUntil(this.destroy$))
      .subscribe(disks => {
        if (disks.length > 0) {
          this.diskVendor = this.shortenVendorName(disks[0].manufacturer);
        }
      });
  }

  ngOnDestroy(): void {
    this.destroy$.next();
    this.destroy$.complete();
  }

  navigateTo(route: string): void {
    this.router.navigate([route]);
  }

  onNavHover(route: string): void {
    const view = route.replace('/', '') || 'system';
    this.preloadService.priorityPreload(view);
  }

  /**
   * Shorten vendor names to a clean display format.
   * E.g., "Intel(R) Corporation" -> "Intel", "NVIDIA Corporation" -> "NVIDIA"
   */
  private shortenVendorName(name: string): string {
    if (!name) return '';
    // Remove common suffixes and clean up
    let clean = name
      .replace(/\(R\)|\(TM\)|\(C\)/gi, '')  // Remove trademark symbols
      .replace(/Corporation|Corp\.?|Inc\.?|Ltd\.?|LLC/gi, '')  // Remove company suffixes
      .replace(/Advanced Micro Devices/gi, 'AMD')  // Standardize AMD
      .replace(/Western Digital/gi, 'WD')  // Standardize WD
      .replace(/Samsung Electronics/gi, 'Samsung')  // Standardize Samsung
      .trim();
    // Take first word if still too long
    const words = clean.split(/\s+/);
    return words[0] || '';
  }

  /**
   * Get vendor logo badge styling.
   * Returns a CSS class based on vendor name for distinctive styling.
   */
  getVendorBadgeClass(vendor: string): string {
    const v = vendor.toLowerCase();
    if (v.includes('intel')) return 'vendor-intel';
    if (v.includes('amd')) return 'vendor-amd';
    if (v.includes('nvidia')) return 'vendor-nvidia';
    if (v.includes('samsung')) return 'vendor-samsung';
    if (v.includes('wd') || v.includes('western')) return 'vendor-wd';
    if (v.includes('seagate')) return 'vendor-seagate';
    if (v.includes('crucial') || v.includes('micron')) return 'vendor-crucial';
    if (v.includes('kingston')) return 'vendor-kingston';
    if (v.includes('corsair')) return 'vendor-corsair';
    if (v.includes('ddr')) return 'vendor-memory';
    return 'vendor-default';
  }
}
