import { Component, OnInit, OnDestroy, inject, signal, computed } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { Subject, takeUntil } from 'rxjs';
import { invoke } from '@tauri-apps/api/core';

import { ProcessService, StatusService } from '@core/services';
import { ProcessInfo, ProcessSummary } from '@core/models';
import { BytesPipe } from '@shared/pipes';
import { ProcessDetailModalComponent } from '@shared/components';

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
  imports: [CommonModule, FormsModule, BytesPipe, ProcessDetailModalComponent],
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
                          <span class="text-xs text-syslens-text-muted" [title]="process.command">
                            {{ truncatePath(process.command, 45) }}
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
                        <span class="text-syslens-text-primary font-medium flex-1">{{ group.name }}</span>
                        <span class="text-sm text-syslens-text-secondary font-mono">
                          ({{ group.processes.length }})
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
                              <span class="text-syslens-text-primary font-medium truncate max-w-[180px]" [title]="process.name">
                                {{ process.name }}
                              </span>
                              @if (process.command && process.command !== process.name) {
                                <span class="text-xs text-syslens-text-muted" [title]="process.command">
                                  {{ truncatePath(process.command, 40) }}
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
  private destroy$ = new Subject<void>();

  Math = Math;

  /**
   * Truncate a file path showing beginning and end with ellipsis in middle.
   * Example: C:\Windows\System32\...\process.exe
   */
  truncatePath(path: string, maxLength: number = 40): string {
    if (!path || path.length <= maxLength) return path;

    // Find the last path separator for the filename
    const lastSepIndex = Math.max(path.lastIndexOf('\\'), path.lastIndexOf('/'));
    const filename = lastSepIndex >= 0 ? path.slice(lastSepIndex + 1) : path;

    // If filename alone is too long, truncate it
    if (filename.length >= maxLength - 5) {
      return filename.slice(0, maxLength - 3) + '...';
    }

    // Calculate how much of the beginning we can show
    const ellipsis = '...';
    const beginLength = maxLength - filename.length - ellipsis.length - 1; // -1 for separator

    if (beginLength <= 3) {
      // Not enough room for meaningful beginning, just show filename
      return ellipsis + path.slice(lastSepIndex);
    }

    // Get the beginning portion (include drive letter and start of path)
    const separator = path.includes('\\') ? '\\' : '/';
    const beginning = path.slice(0, beginLength);

    return beginning + ellipsis + separator + filename;
  }

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
