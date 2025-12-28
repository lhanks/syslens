import { Component, OnInit, OnDestroy, inject, signal, computed } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { Subject, takeUntil } from 'rxjs';

import { ProcessService, StatusService } from '@core/services';
import { ProcessInfo, ProcessSummary } from '@core/models';
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

        <!-- Summary Stats -->
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
  private destroy$ = new Subject<void>();

  Math = Math;

  processes = signal<ProcessInfo[]>([]);
  summary = signal<ProcessSummary | null>(null);
  searchTerm = signal('');
  sortColumn = signal<SortColumn>('cpuUsage');
  sortDirection = signal<SortDirection>('desc');
  currentPage = signal(0);
  pageSize = 50;

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
  }

  private startPolling(): void {
    this.processService.getProcessesPolling(3000)
      .pipe(takeUntil(this.destroy$))
      .subscribe(processes => this.processes.set(processes));

    this.processService.getProcessSummaryPolling(3000)
      .pipe(takeUntil(this.destroy$))
      .subscribe(summary => this.summary.set(summary));
  }
}
