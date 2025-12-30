import { Component, OnInit, OnDestroy, inject, signal, computed } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { Subject, takeUntil } from 'rxjs';

import { ServiceService, StatusService } from '@core/services';
import { ServiceInfo, ServiceSummary } from '@core/models';

type StatusFilter = 'all' | 'running' | 'stopped';
type StartupFilter = 'all' | 'Automatic' | 'Manual' | 'Disabled';

@Component({
  selector: 'app-services',
  standalone: true,
  imports: [CommonModule, FormsModule],
  template: `
    <div class="p-6 space-y-6">
      <!-- Header -->
      <div class="flex items-center justify-between">
        <div>
          <h1 class="text-2xl font-bold text-syslens-text-primary">Services</h1>
          <p class="text-syslens-text-secondary">Windows services and their status</p>
        </div>
        <button
          (click)="refreshServices()"
          class="px-3 py-1.5 text-sm bg-syslens-accent-blue text-white rounded hover:bg-syslens-accent-blue/80 transition-colors"
          [disabled]="loading()">
          {{ loading() ? 'Loading...' : 'Refresh' }}
        </button>
      </div>

      <!-- Summary Cards -->
      @if (summary()) {
        <div class="grid grid-cols-2 md:grid-cols-5 gap-4">
          <div class="card text-center">
            <p class="text-2xl font-bold text-syslens-text-primary">{{ summary()!.total }}</p>
            <p class="text-xs text-syslens-text-muted">Total</p>
          </div>
          <div class="card text-center cursor-pointer hover:bg-syslens-bg-hover transition-colors"
               (click)="setStatusFilter('running')"
               [class.ring-2]="statusFilter() === 'running'"
               [class.ring-syslens-accent-green]="statusFilter() === 'running'">
            <p class="text-2xl font-bold text-syslens-accent-green">{{ summary()!.running }}</p>
            <p class="text-xs text-syslens-text-muted">Running</p>
          </div>
          <div class="card text-center cursor-pointer hover:bg-syslens-bg-hover transition-colors"
               (click)="setStatusFilter('stopped')"
               [class.ring-2]="statusFilter() === 'stopped'"
               [class.ring-syslens-accent-red]="statusFilter() === 'stopped'">
            <p class="text-2xl font-bold text-syslens-accent-red">{{ summary()!.stopped }}</p>
            <p class="text-xs text-syslens-text-muted">Stopped</p>
          </div>
          <div class="card text-center">
            <p class="text-2xl font-bold text-syslens-accent-yellow">{{ summary()!.startPending }}</p>
            <p class="text-xs text-syslens-text-muted">Starting</p>
          </div>
          <div class="card text-center">
            <p class="text-2xl font-bold text-syslens-accent-purple">{{ summary()!.stopPending }}</p>
            <p class="text-xs text-syslens-text-muted">Stopping</p>
          </div>
        </div>
      }

      <!-- Filters -->
      <div class="flex flex-wrap gap-4 items-center">
        <div class="flex-1 min-w-[200px]">
          <input
            type="text"
            [(ngModel)]="searchQuery"
            placeholder="Search services..."
            class="w-full px-3 py-2 bg-syslens-bg-secondary border border-syslens-border-primary rounded text-syslens-text-primary placeholder-syslens-text-muted focus:outline-none focus:ring-1 focus:ring-syslens-accent-blue"
          />
        </div>
        <div class="flex gap-2">
          <select
            [(ngModel)]="startupFilterValue"
            class="px-3 py-2 bg-syslens-bg-secondary border border-syslens-border-primary rounded text-syslens-text-primary focus:outline-none focus:ring-1 focus:ring-syslens-accent-blue">
            <option value="all">All Startup Types</option>
            <option value="Automatic">Automatic</option>
            <option value="Manual">Manual</option>
            <option value="Disabled">Disabled</option>
          </select>
          <button
            (click)="clearFilters()"
            class="px-3 py-2 text-sm text-syslens-text-secondary hover:text-syslens-text-primary transition-colors">
            Clear
          </button>
        </div>
      </div>

      <!-- Services Table -->
      <div class="card overflow-hidden">
        <div class="overflow-x-auto">
          <table class="w-full">
            <thead class="bg-syslens-bg-tertiary">
              <tr>
                <th class="px-4 py-3 text-left text-xs font-medium text-syslens-text-muted uppercase tracking-wider cursor-pointer hover:text-syslens-text-primary"
                    (click)="toggleSort('displayName')">
                  Name
                  @if (sortField() === 'displayName') {
                    <span class="ml-1">{{ sortAsc() ? '↑' : '↓' }}</span>
                  }
                </th>
                <th class="px-4 py-3 text-left text-xs font-medium text-syslens-text-muted uppercase tracking-wider cursor-pointer hover:text-syslens-text-primary"
                    (click)="toggleSort('status')">
                  Status
                  @if (sortField() === 'status') {
                    <span class="ml-1">{{ sortAsc() ? '↑' : '↓' }}</span>
                  }
                </th>
                <th class="px-4 py-3 text-left text-xs font-medium text-syslens-text-muted uppercase tracking-wider cursor-pointer hover:text-syslens-text-primary"
                    (click)="toggleSort('startupType')">
                  Startup
                  @if (sortField() === 'startupType') {
                    <span class="ml-1">{{ sortAsc() ? '↑' : '↓' }}</span>
                  }
                </th>
                <th class="px-4 py-3 text-left text-xs font-medium text-syslens-text-muted uppercase tracking-wider">
                  Account
                </th>
                <th class="px-4 py-3 text-left text-xs font-medium text-syslens-text-muted uppercase tracking-wider">
                  PID
                </th>
              </tr>
            </thead>
            <tbody class="divide-y divide-syslens-border-primary">
              @for (service of filteredServices(); track service.name) {
                <tr class="hover:bg-syslens-bg-hover transition-colors cursor-pointer"
                    (click)="selectService(service)"
                    [class.bg-syslens-bg-tertiary]="selectedService()?.name === service.name">
                  <td class="px-4 py-3">
                    <div class="font-medium text-syslens-text-primary">{{ service.displayName }}</div>
                    <div class="text-xs text-syslens-text-muted font-mono">{{ service.name }}</div>
                  </td>
                  <td class="px-4 py-3">
                    <span class="px-2 py-0.5 rounded text-xs font-medium" [class.bg-syslens-accent-green]="service.status === 'Running'" [class.text-white]="service.status === 'Running'" [class.bg-syslens-accent-red]="service.status === 'Stopped'" [class.text-syslens-accent-red]="service.status === 'Stopped'" [class.bg-syslens-accent-yellow]="service.status === 'StartPending' || service.status === 'StopPending'" [class.text-black]="service.status === 'StartPending' || service.status === 'StopPending'">{{ service.status }}</span>
                  </td>
                  <td class="px-4 py-3">
                    <span class="text-sm" [class.text-syslens-text-primary]="service.startupType === 'Automatic'" [class.text-syslens-text-secondary]="service.startupType === 'Manual'" [class.text-syslens-text-muted]="service.startupType === 'Disabled'">{{ service.startupType }}</span>
                  </td>
                  <td class="px-4 py-3 text-sm text-syslens-text-secondary font-mono truncate max-w-[150px]"
                      [title]="service.serviceAccount || ''">
                    {{ service.serviceAccount || '-' }}
                  </td>
                  <td class="px-4 py-3 text-sm text-syslens-text-muted font-mono">
                    {{ service.pid || '-' }}
                  </td>
                </tr>
              } @empty {
                <tr>
                  <td colspan="5" class="px-4 py-8 text-center text-syslens-text-muted">
                    @if (loading()) {
                      Loading services...
                    } @else {
                      No services found matching your criteria
                    }
                  </td>
                </tr>
              }
            </tbody>
          </table>
        </div>

        <!-- Pagination info -->
        <div class="px-4 py-3 bg-syslens-bg-tertiary border-t border-syslens-border-primary text-sm text-syslens-text-muted">
          Showing {{ filteredServices().length }} of {{ services().length }} services
        </div>
      </div>

      <!-- Service Details Panel -->
      @if (selectedService()) {
        <div class="card">
          <div class="flex items-center justify-between mb-4">
            <h2 class="text-lg font-semibold text-syslens-text-primary">{{ selectedService()!.displayName }}</h2>
            <button
              (click)="selectedService.set(null)"
              class="text-syslens-text-muted hover:text-syslens-text-primary transition-colors">
              <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>
          <dl class="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div>
              <dt class="text-xs text-syslens-text-muted">Service Name</dt>
              <dd class="text-syslens-text-primary font-mono text-sm">{{ selectedService()!.name }}</dd>
            </div>
            <div>
              <dt class="text-xs text-syslens-text-muted">Status</dt>
              <dd>
                <span class="px-2 py-0.5 rounded text-xs font-medium" [class.bg-syslens-accent-green]="selectedService()!.status === 'Running'" [class.text-white]="selectedService()!.status === 'Running'" [class.bg-syslens-accent-red]="selectedService()!.status === 'Stopped'" [class.bg-opacity-20]="selectedService()!.status === 'Stopped'">{{ selectedService()!.status }}</span>
              </dd>
            </div>
            <div>
              <dt class="text-xs text-syslens-text-muted">Startup Type</dt>
              <dd class="text-syslens-text-primary">{{ selectedService()!.startupType }}</dd>
            </div>
            <div>
              <dt class="text-xs text-syslens-text-muted">Process ID</dt>
              <dd class="text-syslens-text-primary font-mono">{{ selectedService()!.pid || 'Not running' }}</dd>
            </div>
            <div>
              <dt class="text-xs text-syslens-text-muted">Account</dt>
              <dd class="text-syslens-text-primary font-mono text-sm">{{ selectedService()!.serviceAccount || 'N/A' }}</dd>
            </div>
            @if (selectedService()!.binaryPath) {
              <div class="md:col-span-2">
                <dt class="text-xs text-syslens-text-muted">Binary Path</dt>
                <dd class="text-syslens-text-primary font-mono text-sm break-all">{{ selectedService()!.binaryPath }}</dd>
              </div>
            }
            @if (selectedService()!.description) {
              <div class="md:col-span-2">
                <dt class="text-xs text-syslens-text-muted">Description</dt>
                <dd class="text-syslens-text-secondary text-sm">{{ selectedService()!.description }}</dd>
              </div>
            }
          </dl>
        </div>
      }
    </div>
  `
})
export class ServicesComponent implements OnInit, OnDestroy {
  private serviceService = inject(ServiceService);
  private statusService = inject(StatusService);
  private destroy$ = new Subject<void>();

  // Data
  services = signal<ServiceInfo[]>([]);
  summary = signal<ServiceSummary | null>(null);
  loading = signal(false);
  selectedService = signal<ServiceInfo | null>(null);

  // Filters
  searchQuery = '';
  statusFilter = signal<StatusFilter>('all');
  startupFilterValue: StartupFilter = 'all';
  sortField = signal<'displayName' | 'status' | 'startupType'>('displayName');
  sortAsc = signal(true);

  // Computed filtered and sorted services
  filteredServices = computed(() => {
    let result = this.services();

    // Search filter
    if (this.searchQuery.trim()) {
      const query = this.searchQuery.toLowerCase();
      result = result.filter(s =>
        s.name.toLowerCase().includes(query) ||
        s.displayName.toLowerCase().includes(query) ||
        (s.description && s.description.toLowerCase().includes(query))
      );
    }

    // Status filter
    const status = this.statusFilter();
    if (status === 'running') {
      result = result.filter(s => s.status === 'Running');
    } else if (status === 'stopped') {
      result = result.filter(s => s.status === 'Stopped');
    }

    // Startup type filter
    if (this.startupFilterValue !== 'all') {
      result = result.filter(s => s.startupType === this.startupFilterValue);
    }

    // Sort
    const field = this.sortField();
    const asc = this.sortAsc();
    result = [...result].sort((a, b) => {
      const aVal = a[field];
      const bVal = b[field];
      const cmp = aVal.localeCompare(bVal);
      return asc ? cmp : -cmp;
    });

    return result;
  });

  ngOnInit(): void {
    this.loadServices();
    this.loadSummary();
  }

  ngOnDestroy(): void {
    this.destroy$.next();
    this.destroy$.complete();
  }

  private loadServices(): void {
    this.loading.set(true);
    this.statusService.startOperation('services-init', 'Loading services...');

    this.serviceService.getServices()
      .pipe(takeUntil(this.destroy$))
      .subscribe({
        next: (services) => {
          this.services.set(services);
          this.loading.set(false);
          this.statusService.endOperation('services-init');
        },
        error: () => {
          this.loading.set(false);
          this.statusService.endOperation('services-init');
        }
      });
  }

  private loadSummary(): void {
    this.serviceService.getServiceSummary()
      .pipe(takeUntil(this.destroy$))
      .subscribe(summary => this.summary.set(summary));
  }

  refreshServices(): void {
    this.serviceService.clearCache();
    this.loadServices();
    this.loadSummary();
  }

  setStatusFilter(filter: StatusFilter): void {
    if (this.statusFilter() === filter) {
      this.statusFilter.set('all');
    } else {
      this.statusFilter.set(filter);
    }
  }

  clearFilters(): void {
    this.searchQuery = '';
    this.statusFilter.set('all');
    this.startupFilterValue = 'all';
  }

  toggleSort(field: 'displayName' | 'status' | 'startupType'): void {
    if (this.sortField() === field) {
      this.sortAsc.set(!this.sortAsc());
    } else {
      this.sortField.set(field);
      this.sortAsc.set(true);
    }
  }

  selectService(service: ServiceInfo): void {
    if (this.selectedService()?.name === service.name) {
      this.selectedService.set(null);
    } else {
      this.selectedService.set(service);
    }
  }
}
