import { Component, OnInit, OnDestroy, inject, signal, computed } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { Subject, takeUntil } from 'rxjs';
import { invoke } from '@tauri-apps/api/core';

import { ProcessService, StatusService, MetricsHistoryService } from '@core/services';
import { ProcessInfo, ProcessSummary } from '@core/models';
import { BytesPipe } from '@shared/pipes';
import { LineGraphComponent, ProcessDetailModalComponent } from '@shared/components';

type SortColumn = 'name' | 'pid' | 'cpuUsage' | 'memoryBytes' | 'status';
type SortDirection = 'asc' | 'desc';
type ViewMode = 'flat' | 'grouped';

interface ProcessGroup {
  name: string;
  processes: ProcessInfo[];
  totalCpu: number;
  totalMemory: number;
  expanded: boolean;
}

@Component({
  selector: 'app-processes',
  standalone: true,
  imports: [CommonModule, FormsModule, BytesPipe, LineGraphComponent, ProcessDetailModalComponent],
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
          <div class="flex items-center gap-3 mb-2">
            <div class="w-8 h-8 rounded-lg bg-syslens-accent-blue/20 flex items-center justify-center">
              <svg class="w-4 h-4 text-syslens-accent-blue" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                  d="M9 3v2m6-2v2M9 19v2m6-2v2M5 9H3m2 6H3m18-6h-2m2 6h-2M7 19h10a2 2 0 002-2V7a2 2 0 00-2-2H7a2 2 0 00-2 2v10a2 2 0 002 2zM9 9h6v6H9V9z" />
              </svg>
            </div>
            <div class="flex-1">
              <p class="text-xs text-syslens-text-muted">CPU</p>
              <p class="text-lg font-bold font-mono text-syslens-text-primary" style="min-width: 5ch;">{{ cpuUsage().toFixed(1) }}%</p>
            </div>
          </div>
          <app-line-graph
            [series1]="cpuHistory()"
            [maxValue]="100"
            [height]="40"
            series1Color="syslens-accent-blue"
            [showYAxis]="true"
            yAxisFormat="percent"
            [yAxisWidth]="40"
          />
        </div>

        <!-- Memory -->
        <div class="card">
          <div class="flex items-center gap-3 mb-2">
            <div class="w-8 h-8 rounded-lg bg-syslens-accent-purple/20 flex items-center justify-center">
              <svg class="w-4 h-4 text-syslens-accent-purple" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                  d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
              </svg>
            </div>
            <div class="flex-1">
              <p class="text-xs text-syslens-text-muted">Memory</p>
              <p class="text-sm font-bold font-mono text-syslens-text-primary"><span style="min-width: 6ch; display: inline-block; text-align: right;">{{ memoryUsedBytes() | bytes }}</span> / {{ memoryTotalBytes() | bytes }}</p>
            </div>
          </div>
          <app-line-graph
            [series1]="memoryHistory()"
            [maxValue]="memoryTotalBytes()"
            [height]="40"
            series1Color="syslens-accent-purple"
            [showYAxis]="true"
            yAxisFormat="bytes"
            [yAxisWidth]="40"
          />
        </div>

        <!-- Disk -->
        <div class="card">
          <div class="flex items-center gap-3 mb-2">
            <div class="w-8 h-8 rounded-lg bg-syslens-accent-cyan/20 flex items-center justify-center">
              <svg class="w-4 h-4 text-syslens-accent-cyan" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                  d="M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4" />
              </svg>
            </div>
            <div class="flex-1">
              <p class="text-xs text-syslens-text-muted">Disk</p>
              <p class="text-lg font-bold font-mono text-syslens-text-primary" style="min-width: 4ch;">{{ diskActivity().toFixed(0) }}%</p>
            </div>
          </div>
          <app-line-graph
            [series1]="diskHistory()"
            [maxValue]="100"
            [height]="40"
            series1Color="syslens-accent-cyan"
            [showYAxis]="true"
            yAxisFormat="percent"
            [yAxisWidth]="40"
          />
        </div>

        <!-- Network -->
        <div class="card">
          <div class="flex items-center gap-3 mb-2">
            <div class="w-8 h-8 rounded-lg bg-syslens-accent-green/20 flex items-center justify-center">
              <svg class="w-4 h-4 text-syslens-accent-green" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                  d="M8 16l2.879-2.879m0 0a3 3 0 104.243-4.242 3 3 0 00-4.243 4.242zM21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
            </div>
            <div class="flex-1">
              <p class="text-xs text-syslens-text-muted">Network</p>
              <div class="flex gap-2 text-sm font-mono">
                <span class="text-syslens-accent-green">↓<span style="min-width: 7ch; display: inline-block; text-align: right;">{{ networkDown() | bytes }}</span>/s</span>
                <span class="text-syslens-accent-blue">↑<span style="min-width: 7ch; display: inline-block; text-align: right;">{{ networkUp() | bytes }}</span>/s</span>
              </div>
            </div>
          </div>
          <app-line-graph
            [series1]="networkDownloadHistory()"
            [series2]="networkUploadHistory()"
            [maxValue]="networkMaxSpeed()"
            [height]="40"
            series1Color="syslens-accent-green"
            series2Color="syslens-accent-blue"
            [showYAxis]="true"
            yAxisFormat="bytes"
            [yAxisWidth]="40"
          />
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
          <button
            (click)="toggleViewMode()"
            class="px-3 py-2 rounded-lg border border-syslens-border-primary
                   hover:bg-syslens-bg-hover transition-colors flex items-center gap-2"
            [class.bg-syslens-accent-blue]="viewMode() === 'grouped'"
            [class.bg-syslens-bg-tertiary]="viewMode() === 'flat'"
            [class.text-white]="viewMode() === 'grouped'"
            [class.text-syslens-text-secondary]="viewMode() === 'flat'"
          >
            <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              @if (viewMode() === 'flat') {
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 10h16M4 14h16M4 18h16" />
              } @else {
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7h3m0 0V4m0 3v3m0 7h3m0 0v-3m0 3v3m8-10h5m-5 4h5m-5 4h5" />
              }
            </svg>
            <span class="text-sm">{{ viewMode() === 'flat' ? 'Group' : 'Flat' }}</span>
          </button>
          <div class="flex items-center gap-2 text-sm text-syslens-text-muted">
            <span>{{ filteredProcesses().length }} processes</span>
            @if (viewMode() === 'grouped') {
              <span class="text-syslens-text-muted">({{ groupedProcesses().length }} groups)</span>
            }
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
              @if (viewMode() === 'flat') {
                @for (process of paginatedProcesses(); track process.pid) {
                  <tr class="border-b border-syslens-border-primary hover:bg-syslens-bg-hover transition-colors cursor-pointer"
                      (click)="openProcessDetails(process)">
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
              } @else {
                <!-- Grouped View -->
                @for (group of groupedProcesses(); track group.name) {
                  <!-- Group Header Row -->
                  <tr class="border-b border-syslens-border-primary bg-syslens-bg-secondary hover:bg-syslens-bg-hover transition-colors cursor-pointer"
                      (click)="toggleGroup(group.name)">
                    <td class="table-cell">
                      <div class="flex items-center gap-2">
                        <svg class="w-4 h-4 text-syslens-text-muted transition-transform"
                             [class.rotate-90]="group.expanded"
                             fill="none" viewBox="0 0 24 24" stroke="currentColor">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
                        </svg>
                        <span class="text-syslens-text-primary font-medium">{{ group.name }}</span>
                        <span class="text-sm font-medium text-syslens-accent-blue bg-syslens-accent-blue/20 px-2 py-0.5 rounded-full">
                          {{ group.processes.length }}
                        </span>
                      </div>
                    </td>
                    <td class="table-cell font-mono text-syslens-text-muted">—</td>
                    <td class="table-cell">
                      <div class="flex items-center gap-2">
                        <div class="w-12 h-1.5 bg-syslens-bg-tertiary rounded-full overflow-hidden">
                          <div class="h-full rounded-full transition-all"
                               [style.width.%]="Math.min(group.totalCpu, 100)"
                               [class.bg-syslens-accent-green]="group.totalCpu < 25"
                               [class.bg-syslens-accent-yellow]="group.totalCpu >= 25 && group.totalCpu < 75"
                               [class.bg-syslens-accent-red]="group.totalCpu >= 75">
                          </div>
                        </div>
                        <span class="font-mono text-xs text-syslens-text-secondary w-12 text-right">
                          {{ group.totalCpu.toFixed(1) }}%
                        </span>
                      </div>
                    </td>
                    <td class="table-cell font-mono text-syslens-text-secondary">
                      {{ group.totalMemory | bytes }}
                    </td>
                    <td class="table-cell text-syslens-text-muted">—</td>
                    <td class="table-cell text-syslens-text-muted">—</td>
                  </tr>
                  <!-- Expanded Child Processes -->
                  @if (group.expanded) {
                    @for (process of group.processes; track process.pid; let last = $last; let i = $index) {
                      <tr class="border-b border-syslens-border-primary hover:bg-syslens-bg-hover transition-colors cursor-pointer"
                          (click)="openProcessDetails(process); $event.stopPropagation()">
                        <td class="table-cell">
                          <div class="flex items-start gap-0">
                            <!-- Tree connector -->
                            <div class="flex-shrink-0 w-8 flex items-center justify-center text-syslens-text-muted select-none">
                              <span class="font-mono text-sm">{{ last ? '└' : '├' }}</span>
                            </div>
                            <div class="flex flex-col">
                              <span class="text-syslens-text-secondary truncate max-w-[180px]" [title]="process.name">
                                {{ process.name }}
                              </span>
                              @if (process.command && process.command !== process.name) {
                                <span class="text-xs text-syslens-text-muted truncate max-w-[180px]" [title]="process.command">
                                  {{ process.command }}
                                </span>
                              }
                            </div>
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
                    }
                  }
                } @empty {
                  <tr>
                    <td colspan="6" class="table-cell text-center text-syslens-text-muted py-8">
                      No processes found
                    </td>
                  </tr>
                }
              }
            </tbody>
          </table>
        </div>

        <!-- Pagination (flat view only) -->
        @if (viewMode() === 'flat' && totalPages() > 1) {
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

    <!-- Process Detail Modal -->
    <app-process-detail-modal
      [isOpen]="modalOpen()"
      [process]="selectedProcess()"
      [allProcesses]="processes()"
      (closed)="closeModal()"
      (refreshRequested)="handleRefresh()"
      (processSelected)="selectProcess($event)"
      (killRequested)="handleKillProcess($event)"
    />
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
  metricsService = inject(MetricsHistoryService);
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

  // View mode for grouping (default to grouped view)
  viewMode = signal<ViewMode>('grouped');
  expandedGroups = signal<Set<string>>(new Set());

  // Modal state
  modalOpen = signal(false);
  selectedProcess = signal<ProcessInfo | null>(null);

  // Computed values from shared metrics service
  cpuUsage = computed(() => this.metricsService.cpuUsage());
  cpuHistory = computed(() => this.metricsService.cpuHistory());
  memoryUsage = computed(() => this.metricsService.memoryUsage());
  memoryHistory = computed(() => this.metricsService.memoryHistory());
  memoryUsedBytes = computed(() => this.metricsService.memoryUsedBytes());
  memoryTotalBytes = computed(() => this.metricsService.memoryTotalBytes());
  diskActivity = computed(() => this.metricsService.diskActivity());
  diskHistory = computed(() => this.metricsService.diskHistory());
  networkDown = computed(() => this.metricsService.networkDownSpeed());
  networkUp = computed(() => this.metricsService.networkUpSpeed());
  networkDownloadHistory = computed(() => this.metricsService.networkDownHistory());
  networkUploadHistory = computed(() => this.metricsService.networkUpHistory());
  networkMaxSpeed = computed(() => this.metricsService.networkMaxSpeed());

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

  // Group processes by name
  groupedProcesses = computed(() => {
    const processes = this.filteredProcesses();
    const groupMap = new Map<string, ProcessInfo[]>();

    for (const process of processes) {
      const name = process.name;
      if (!groupMap.has(name)) {
        groupMap.set(name, []);
      }
      groupMap.get(name)!.push(process);
    }

    const groups: ProcessGroup[] = [];
    const expanded = this.expandedGroups();

    for (const [name, procs] of groupMap) {
      groups.push({
        name,
        processes: procs,
        totalCpu: procs.reduce((sum, p) => sum + p.cpuUsage, 0),
        totalMemory: procs.reduce((sum, p) => sum + p.memoryBytes, 0),
        expanded: expanded.has(name)
      });
    }

    // Sort groups by the same criteria as individual processes
    const col = this.sortColumn();
    const dir = this.sortDirection();

    return groups.sort((a, b) => {
      let aVal: string | number;
      let bVal: string | number;

      switch (col) {
        case 'cpuUsage':
          aVal = a.totalCpu;
          bVal = b.totalCpu;
          break;
        case 'memoryBytes':
          aVal = a.totalMemory;
          bVal = b.totalMemory;
          break;
        case 'name':
          aVal = a.name.toLowerCase();
          bVal = b.name.toLowerCase();
          break;
        default:
          aVal = a.name.toLowerCase();
          bVal = b.name.toLowerCase();
      }

      if (aVal < bVal) return dir === 'asc' ? -1 : 1;
      if (aVal > bVal) return dir === 'asc' ? 1 : -1;
      return 0;
    });
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

  toggleViewMode(): void {
    this.viewMode.set(this.viewMode() === 'flat' ? 'grouped' : 'flat');
  }

  toggleGroup(groupName: string): void {
    const expanded = new Set(this.expandedGroups());
    if (expanded.has(groupName)) {
      expanded.delete(groupName);
    } else {
      expanded.add(groupName);
    }
    this.expandedGroups.set(expanded);
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

  openProcessDetails(process: ProcessInfo): void {
    this.selectedProcess.set(process);
    this.modalOpen.set(true);
  }

  closeModal(): void {
    this.modalOpen.set(false);
  }

  selectProcess(process: ProcessInfo): void {
    this.selectedProcess.set(process);
  }

  handleRefresh(): void {
    // Refresh the process list to get updated data
    this.processService.getProcesses()
      .pipe(takeUntil(this.destroy$))
      .subscribe(processes => {
        this.processes.set(processes);
        // Update selected process with fresh data
        const selected = this.selectedProcess();
        if (selected) {
          const updated = processes.find(p => p.pid === selected.pid);
          if (updated) {
            this.selectedProcess.set(updated);
          }
        }
      });
  }

  async handleKillProcess(pid: number): Promise<void> {
    try {
      this.statusService.startOperation('kill-process', `Ending process ${pid}...`);
      await invoke<boolean>('kill_process', { pid });

      // Close modal and refresh process list
      this.closeModal();
      this.handleRefresh();
      this.statusService.endOperation('kill-process');
    } catch (error) {
      this.statusService.endOperation('kill-process');
      // Show error to user
      const errorMessage = error instanceof Error ? error.message : String(error);
      alert(`Failed to end process: ${errorMessage}`);
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
  }

  private startPolling(): void {
    // Process polling only - system metrics are handled by MetricsHistoryService
    this.processService.getProcessesPolling(3000)
      .pipe(takeUntil(this.destroy$))
      .subscribe(processes => this.processes.set(processes));

    this.processService.getProcessSummaryPolling(3000)
      .pipe(takeUntil(this.destroy$))
      .subscribe(summary => this.summary.set(summary));
  }
}
