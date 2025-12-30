import { Component, inject, computed } from '@angular/core';
import { Router, RouterLink, RouterLinkActive } from '@angular/router';
import { PreloadService, MetricsHistoryService } from '@core/services';
import { BytesPipe } from '@shared/pipes';

interface NavItem {
  label: string;
  route: string;
  icon: string;
}

@Component({
  selector: 'app-sidebar',
  standalone: true,
  imports: [RouterLink, RouterLinkActive, BytesPipe],
  template: `
    <aside class="w-64 h-screen bg-syslens-bg-secondary border-r border-syslens-border-primary flex flex-col">
      <!-- Logo / Header -->
      <div class="p-4 border-b border-syslens-border-primary">
        <div class="flex items-center gap-3">
          <img src="assets/logo-icon.svg" alt="Syslens" class="w-10 h-10">
          <div>
            <h1 class="text-lg font-semibold text-syslens-text-primary">Syslens</h1>
            <p class="text-xs text-syslens-text-muted">System Monitor</p>
          </div>
        </div>
      </div>

      <!-- System Status Summary -->
      <div class="p-3 border-b border-syslens-border-primary space-y-2">
        <!-- CPU -->
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
              <span class="text-syslens-text-muted">CPU</span>
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

        <!-- Memory -->
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
              <span class="text-syslens-text-muted">RAM</span>
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

        <!-- Disk -->
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
              <span class="text-syslens-text-muted">Disk</span>
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

        <!-- GPU -->
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
              <span class="text-syslens-text-muted">GPU</span>
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

        <!-- Network - Per Adapter -->
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
              <div class="font-mono text-xs text-syslens-text-secondary flex gap-1.5">
                <span class="text-syslens-accent-green">↓{{ adapter.downloadSpeed | bytes }}/s</span>
                <span class="text-syslens-accent-blue">↑{{ adapter.uploadSpeed | bytes }}/s</span>
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
      </div>

      <!-- Navigation -->
      <nav class="flex-1 p-3 space-y-1 overflow-y-auto">
        @for (item of navItems; track item.route) {
          <a
            [routerLink]="item.route"
            routerLinkActive="bg-syslens-bg-hover border-syslens-accent-blue text-syslens-text-primary"
            [routerLinkActiveOptions]="{ exact: item.route === '/dashboard' }"
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
  `]
})
export class SidebarComponent {
  private router = inject(Router);
  private preloadService = inject(PreloadService);
  private metricsService = inject(MetricsHistoryService);

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
      label: 'Dashboard',
      route: '/dashboard',
      icon: `<svg fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
          d="M4 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2V6zM14 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2V6zM4 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2v-2zM14 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2v-2z" />
      </svg>`
    },
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
      label: 'Restore Points',
      route: '/restore-points',
      icon: `<svg fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
          d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
      </svg>`
    }
  ];

  /**
   * Navigate to the specified route.
   */
  navigateTo(route: string): void {
    this.router.navigate([route]);
  }

  /**
   * Trigger priority preload when user hovers over a navigation link.
   * This provides instant loading when the user clicks.
   */
  onNavHover(route: string): void {
    // Extract view name from route (e.g., '/hardware' -> 'hardware')
    const view = route.replace('/', '') || 'dashboard';
    this.preloadService.priorityPreload(view);
  }
}
