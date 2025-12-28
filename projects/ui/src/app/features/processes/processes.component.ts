import { Component, OnInit, OnDestroy, inject, signal, computed } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { Subject, takeUntil } from 'rxjs';

import { ProcessService, StatusService, HardwareService, StorageService, NetworkHistoryService } from '@core/services';
import { ProcessInfo, ProcessSummary, CpuMetrics, MemoryInfo, MemoryMetrics, DiskPerformance } from '@core/models';
import { BytesPipe } from '@shared/pipes';

type SortColumn = 'name' | 'pid' | 'cpuUsage' | 'memoryBytes' | 'status';
type SortDirection = 'asc' | 'desc';

@Component({
  selector: 'app-processes',
  standalone: true,
  imports: [CommonModule, FormsModule, BytesPipe],
  template: `
    <div class="p-6 space-y-6">
      <!-- Header -->
      <div class="flex items-start justify-between">
        <div>
          <h1 class="text-2xl font-bold text-syslens-text-primary">Processes</h1>
          <p class="text-syslens-text-secondary">Running processes and system activity</p>
        </div>

        <!-- Process Summary Stats -->
        @if (summary()) {
          <div class="flex gap-6 text-sm">
            <div class="text-center">
              <p class="text-2xl font-bold text-syslens-text-primary">{{ summary()!.totalCount }}</p>
              <p class="text-xs text-syslens-text-muted">Total</p>
            </div>
            <div class="text-center">
              <p class="text-2xl font-bold text-syslens-accent-green">{{ summary()!.runningCount }}</p>
              <p class="text-xs text-syslens-text-muted">Running</p>
            </div>
            <div class="text-center">
              <p class="text-2xl font-bold text-syslens-accent-yellow">{{ summary()!.sleepingCount }}</p>
              <p class="text-xs text-syslens-text-muted">Sleeping</p>
            </div>
          </div>
        }
      </div>

      <!-- System Resource Summary -->
      <div class="grid grid-cols-2 lg:grid-cols-4 gap-4">
        <!-- CPU -->
        <div class="card">
          <div class="flex items-center gap-3">
            <div class="w-10 h-10 rounded-lg bg-syslens-accent-blue/20 flex items-center justify-center">
              <svg class="w-5 h-5 text-syslens-accent-blue" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                  d="M9 3v2m6-2v2M9 19v2m6-2v2M5 9H3m2 6H3m18-6h-2m2 6h-2M7 19h10a2 2 0 002-2V7a2 2 0 00-2-2H7a2 2 0 00-2 2v10a2 2 0 002 2zM9 9h6v6H9V9z" />
              </svg>
            </div>
            <div class="flex-1">
              <p class="text-xs text-syslens-text-muted">CPU</p>
              <p class="text-lg font-bold text-syslens-text-primary">{{ cpuUsage().toFixed(1) }}%</p>
            </div>
          </div>
          <div class="mt-2 h-1.5 bg-syslens-bg-tertiary rounded-full overflow-hidden">
            <div class="h-full bg-syslens-accent-blue rounded-full transition-all"
                 [style.width.%]="cpuUsage()"></div>
          </div>
        </div>

        <!-- Memory -->
        <div class="card">
          <div class="flex items-center gap-3">
            <div class="w-10 h-10 rounded-lg bg-syslens-accent-purple/20 flex items-center justify-center">
              <svg class="w-5 h-5 text-syslens-accent-purple" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                  d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
              </svg>
            </div>
            <div class="flex-1">
              <p class="text-xs text-syslens-text-muted">Memory</p>
              <p class="text-lg font-bold text-syslens-text-primary">{{ memoryUsage().toFixed(1) }}%</p>
            </div>
          </div>
          <div class="mt-2 h-1.5 bg-syslens-bg-tertiary rounded-full overflow-hidden">
            <div class="h-full bg-syslens-accent-purple rounded-full transition-all"
                 [style.width.%]="memoryUsage()"></div>
          </div>
        </div>

        <!-- Disk -->
        <div class="card">
          <div class="flex items-center gap-3">
            <div class="w-10 h-10 rounded-lg bg-syslens-accent-cyan/20 flex items-center justify-center">
              <svg class="w-5 h-5 text-syslens-accent-cyan" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                  d="M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4" />
              </svg>
            </div>
            <div class="flex-1">
              <p class="text-xs text-syslens-text-muted">Disk</p>
              <p class="text-lg font-bold text-syslens-text-primary">{{ diskActivity().toFixed(0) }}%</p>
            </div>
          </div>
          <div class="mt-2 h-1.5 bg-syslens-bg-tertiary rounded-full overflow-hidden">
            <div class="h-full bg-syslens-accent-cyan rounded-full transition-all"
                 [style.width.%]="diskActivity()"></div>
          </div>
        </div>

        <!-- Network -->
        <div class="card">
          <div class="flex items-center gap-3">
            <div class="w-10 h-10 rounded-lg bg-syslens-accent-green/20 flex items-center justify-center">
              <svg class="w-5 h-5 text-syslens-accent-green" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                  d="M8 16l2.879-2.879m0 0a3 3 0 104.243-4.242 3 3 0 00-4.243 4.242zM21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
            </div>
            <div class="flex-1">
              <p class="text-xs text-syslens-text-muted">Network</p>
              <div class="flex gap-2 text-sm">
                <span class="text-syslens-accent-green">↓{{ networkDown() | bytes }}/s</span>
                <span class="text-syslens-accent-blue">↑{{ networkUp() | bytes }}/s</span>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Search and Filters -->
      <div class="card">
        <div class="flex gap-4">
          <div class="flex-1">
            <input
              type="text"
              [ngModel]="searchTerm()"
              (ngModelChange)="searchTerm.set($event)"
              placeholder="Search processes..."
              class="w-full px-3 py-2 bg-syslens-bg-tertiary border border-syslens-border-primary rounded-lg
                     text-syslens-text-primary placeholder-syslens-text-muted
                     focus:outline-none focus:border-syslens-accent-blue"
            />
          </div>
          <div class="flex items-center gap-2 text-sm text-syslens-text-muted">
            <span>{{ filteredProcesses().length }} processes</span>
          </div>
        </div>
      </div>

      <!-- Process Table -->
      <div class="card overflow-hidden">
        <div class="overflow-x-auto">
          <table class="w-full">
            <thead>
              <tr class="border-b border-syslens-border-primary">
                <th class="table-header cursor-pointer" (click)="toggleSort('name')">
                  <div class="flex items-center gap-1">
                    Name
                    @if (sortColumn() === 'name') {
                      <span>{{ sortDirection() === 'asc' ? '↑' : '↓' }}</span>
                    }
                  </div>
                </th>
                <th class="table-header cursor-pointer" (click)="toggleSort('pid')">
                  <div class="flex items-center gap-1">
                    PID
                    @if (sortColumn() === 'pid') {
                      <span>{{ sortDirection() === 'asc' ? '↑' : '↓' }}</span>
                    }
                  </div>
                </th>
                <th class="table-header cursor-pointer" (click)="toggleSort('cpuUsage')">
                  <div class="flex items-center gap-1">
                    CPU
                    @if (sortColumn() === 'cpuUsage') {
                      <span>{{ sortDirection() === 'asc' ? '↑' : '↓' }}</span>
                    }
                  </div>
                </th>
                <th class="table-header cursor-pointer" (click)="toggleSort('memoryBytes')">
                  <div class="flex items-center gap-1">
                    Memory
                    @if (sortColumn() === 'memoryBytes') {
                      <span>{{ sortDirection() === 'asc' ? '↑' : '↓' }}</span>
                    }
                  </div>
                </th>
                <th class="table-header cursor-pointer" (click)="toggleSort('status')">
                  <div class="flex items-center gap-1">
                    Status
                    @if (sortColumn() === 'status') {
                      <span>{{ sortDirection() === 'asc' ? '↑' : '↓' }}</span>
                    }
                  </div>
                </th>
                <th class="table-header">User</th>
              </tr>
            </thead>
            <tbody>
              @for (process of paginatedProcesses(); track process.pid) {
                <tr class="border-b border-syslens-border-primary hover:bg-syslens-bg-hover transition-colors">
                  <td class="table-cell">
                    <div class="flex flex-col">
                      <span class="text-syslens-text-primary font-medium truncate max-w-[200px]" [title]="process.name">
                        {{ process.name }}
                      </span>
                      @if (process.command && process.command !== process.name) {
                        <span class="text-xs text-syslens-text-muted truncate max-w-[200px]" [title]="process.command">
                          {{ process.command }}
                        </span>
                      }
                    </div>
                  </td>
                  <td class="table-cell font-mono text-syslens-text-secondary">{{ process.pid }}</td>
                  <td class="table-cell">
                    <div class="flex items-center gap-2">
                      <div class="w-12 h-1.5 bg-syslens-bg-tertiary rounded-full overflow-hidden">
                        <div class="h-full rounded-full transition-all"
                             [style.width.%]="Math.min(process.cpuUsage, 100)"
                             [class.bg-syslens-accent-green]="process.cpuUsage < 25"
                             [class.bg-syslens-accent-yellow]="process.cpuUsage >= 25 && process.cpuUsage < 75"
                             [class.bg-syslens-accent-red]="process.cpuUsage >= 75">
                        </div>
                      </div>
                      <span class="font-mono text-xs text-syslens-text-secondary w-12 text-right">
                        {{ process.cpuUsage.toFixed(1) }}%
                      </span>
                    </div>
                  </td>
                  <td class="table-cell font-mono text-syslens-text-secondary">
                    {{ process.memoryBytes | bytes }}
                  </td>
                  <td class="table-cell">
                    <span class="px-2 py-0.5 text-xs rounded"
                          [class.bg-syslens-accent-green]="process.status === 'Run'"
                          [class.text-white]="process.status === 'Run'"
                          [class.bg-syslens-accent-yellow]="process.status === 'Sleep'"
                          [class.text-black]="process.status === 'Sleep'"
                          [class.bg-syslens-bg-tertiary]="process.status !== 'Run' && process.status !== 'Sleep'"
                          [class.text-syslens-text-secondary]="process.status !== 'Run' && process.status !== 'Sleep'">
                      {{ process.status }}
                    </span>
                  </td>
                  <td class="table-cell text-syslens-text-muted truncate max-w-[100px]" [title]="process.user || 'N/A'">
                    {{ process.user || 'N/A' }}
                  </td>
                </tr>
              } @empty {
                <tr>
                  <td colspan="6" class="table-cell text-center text-syslens-text-muted py-8">
                    No processes found
                  </td>
                </tr>
              }
            </tbody>
          </table>
        </div>

        <!-- Pagination -->
        @if (totalPages() > 1) {
          <div class="flex items-center justify-between px-4 py-3 border-t border-syslens-border-primary">
            <div class="text-sm text-syslens-text-muted">
              Showing {{ startIndex() + 1 }}-{{ endIndex() }} of {{ filteredProcesses().length }}
            </div>
            <div class="flex gap-2">
              <button
                (click)="prevPage()"
                [disabled]="currentPage() === 0"
                class="px-3 py-1 text-sm rounded border border-syslens-border-primary
                       hover:bg-syslens-bg-hover disabled:opacity-50 disabled:cursor-not-allowed
                       text-syslens-text-secondary"
              >
                Previous
              </button>
              <span class="px-3 py-1 text-sm text-syslens-text-secondary">
                {{ currentPage() + 1 }} / {{ totalPages() }}
              </span>
              <button
                (click)="nextPage()"
                [disabled]="currentPage() >= totalPages() - 1"
                class="px-3 py-1 text-sm rounded border border-syslens-border-primary
                       hover:bg-syslens-bg-hover disabled:opacity-50 disabled:cursor-not-allowed
                       text-syslens-text-secondary"
              >
                Next
              </button>
            </div>
          </div>
        }
      </div>
    </div>
  `,
  styles: [`
    .table-header {
      @apply px-4 py-3 text-left text-xs font-medium text-syslens-text-muted uppercase tracking-wider;
    }
    .table-cell {
      @apply px-4 py-3 text-sm;
    }
  `]
})
export class ProcessesComponent implements OnInit, OnDestroy {
  private processService = inject(ProcessService);
  private statusService = inject(StatusService);
  private hardwareService = inject(HardwareService);
  private storageService = inject(StorageService);
  private networkHistoryService = inject(NetworkHistoryService);
  private destroy$ = new Subject<void>();

  Math = Math;

  // Process data
  processes = signal<ProcessInfo[]>([]);
  summary = signal<ProcessSummary | null>(null);
  searchTerm = signal('');
  sortColumn = signal<SortColumn>('cpuUsage');
  sortDirection = signal<SortDirection>('desc');
  currentPage = signal(0);
  pageSize = 50;

  // System metrics
  private cpuMetrics = signal<CpuMetrics | null>(null);
  private memoryInfo = signal<MemoryInfo | null>(null);
  private memoryMetrics = signal<MemoryMetrics | null>(null);
  private diskPerformance = signal<DiskPerformance[]>([]);

  cpuUsage = computed(() => this.cpuMetrics()?.totalUsage ?? 0);

  memoryUsage = computed(() => {
    const info = this.memoryInfo();
    const metrics = this.memoryMetrics();
    if (!info || !metrics) return 0;
    return (metrics.inUseBytes / info.totalBytes) * 100;
  });

  diskActivity = computed(() => {
    const perf = this.diskPerformance();
    if (perf.length === 0) return 0;
    // Return the max active time across all disks
    return Math.max(...perf.map(d => d.activeTimePercent));
  });

  networkDown = computed(() => {
    const points = this.networkHistoryService.dataPoints();
    if (points.length === 0) return 0;
    return points[points.length - 1].downloadSpeed;
  });

  networkUp = computed(() => {
    const points = this.networkHistoryService.dataPoints();
    if (points.length === 0) return 0;
    return points[points.length - 1].uploadSpeed;
  });

  filteredProcesses = computed(() => {
    const term = this.searchTerm().toLowerCase();
    let filtered = this.processes();

    if (term) {
      filtered = filtered.filter(p =>
        p.name.toLowerCase().includes(term) ||
        p.command.toLowerCase().includes(term) ||
        p.pid.toString().includes(term) ||
        (p.user?.toLowerCase().includes(term) ?? false)
      );
    }

    // Sort
    const col = this.sortColumn();
    const dir = this.sortDirection();
    return [...filtered].sort((a, b) => {
      let aVal: string | number = a[col];
      let bVal: string | number = b[col];

      if (typeof aVal === 'string') {
        aVal = aVal.toLowerCase();
        bVal = (bVal as string).toLowerCase();
      }

      if (aVal < bVal) return dir === 'asc' ? -1 : 1;
      if (aVal > bVal) return dir === 'asc' ? 1 : -1;
      return 0;
    });
  });

  totalPages = computed(() => Math.ceil(this.filteredProcesses().length / this.pageSize));
  startIndex = computed(() => this.currentPage() * this.pageSize);
  endIndex = computed(() => Math.min(this.startIndex() + this.pageSize, this.filteredProcesses().length));

  paginatedProcesses = computed(() => {
    return this.filteredProcesses().slice(this.startIndex(), this.endIndex());
  });

  ngOnInit(): void {
    this.loadInitialData();
    this.startPolling();
  }

  ngOnDestroy(): void {
    this.destroy$.next();
    this.destroy$.complete();
  }

  toggleSort(column: SortColumn): void {
    if (this.sortColumn() === column) {
      this.sortDirection.set(this.sortDirection() === 'asc' ? 'desc' : 'asc');
    } else {
      this.sortColumn.set(column);
      this.sortDirection.set('desc');
    }
  }

  prevPage(): void {
    if (this.currentPage() > 0) {
      this.currentPage.set(this.currentPage() - 1);
    }
  }

  nextPage(): void {
    if (this.currentPage() < this.totalPages() - 1) {
      this.currentPage.set(this.currentPage() + 1);
    }
  }

  private loadInitialData(): void {
    this.statusService.startOperation('processes-init', 'Loading processes...');

    this.processService.getProcesses()
      .pipe(takeUntil(this.destroy$))
      .subscribe(processes => {
        this.processes.set(processes);
        this.statusService.endOperation('processes-init');
      });

    this.processService.getProcessSummary()
      .pipe(takeUntil(this.destroy$))
      .subscribe(summary => this.summary.set(summary));

    // Load system metrics
    this.hardwareService.getMemoryInfo()
      .pipe(takeUntil(this.destroy$))
      .subscribe(info => this.memoryInfo.set(info));
  }

  private startPolling(): void {
    // Process polling
    this.processService.getProcessesPolling(3000)
      .pipe(takeUntil(this.destroy$))
      .subscribe(processes => this.processes.set(processes));

    this.processService.getProcessSummaryPolling(3000)
      .pipe(takeUntil(this.destroy$))
      .subscribe(summary => this.summary.set(summary));

    // System metrics polling
    this.hardwareService.getCpuMetricsPolling()
      .pipe(takeUntil(this.destroy$))
      .subscribe(metrics => this.cpuMetrics.set(metrics));

    this.hardwareService.getMemoryMetricsPolling()
      .pipe(takeUntil(this.destroy$))
      .subscribe(metrics => this.memoryMetrics.set(metrics));

    this.storageService.getDiskPerformancePolling()
      .pipe(takeUntil(this.destroy$))
      .subscribe(perf => this.diskPerformance.set(perf));
  }
}
