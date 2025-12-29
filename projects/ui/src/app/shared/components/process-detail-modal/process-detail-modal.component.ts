import { Component, EventEmitter, Input, Output } from '@angular/core';
import { CommonModule } from '@angular/common';
import { ProcessInfo } from '@core/models/process.model';

type TabType = 'overview' | 'resources' | 'hierarchy';

@Component({
  selector: 'app-process-detail-modal',
  standalone: true,
  imports: [CommonModule],
  template: `
    @if (isOpen && process) {
      <div class="fixed inset-0 z-50 flex items-center justify-center">
        <!-- Backdrop -->
        <div
          class="absolute inset-0 bg-black/70 backdrop-blur-sm"
          (click)="close()"
        ></div>

        <!-- Modal Content -->
        <div class="relative bg-syslens-bg-secondary rounded-lg shadow-xl w-full max-w-2xl max-h-[90vh] m-4 overflow-hidden flex flex-col">
          <!-- Header -->
          <div class="flex items-center justify-between p-4 border-b border-syslens-border-primary">
            <div class="flex items-center gap-3">
              <h2 class="text-xl font-semibold text-syslens-text-primary">
                Process Details
              </h2>
              <span
                class="px-2 py-0.5 text-xs rounded-full"
                [class.bg-syslens-accent-green]="process.status === 'Run'"
                [class.text-white]="process.status === 'Run'"
                [class.bg-syslens-accent-yellow]="process.status === 'Sleep'"
                [class.text-black]="process.status === 'Sleep'"
                [class.bg-syslens-bg-tertiary]="process.status !== 'Run' && process.status !== 'Sleep'"
                [class.text-syslens-text-secondary]="process.status !== 'Run' && process.status !== 'Sleep'"
              >
                {{ process.status }}
              </span>
            </div>
            <button
              (click)="close()"
              class="p-1 rounded hover:bg-syslens-bg-tertiary text-syslens-text-muted hover:text-syslens-text-primary transition-colors"
            >
              <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>

          <!-- Tabs -->
          <div class="flex border-b border-syslens-border-primary">
            <button
              (click)="activeTab = 'overview'"
              class="px-4 py-3 text-sm font-medium transition-colors"
              [class.text-syslens-accent-blue]="activeTab === 'overview'"
              [class.border-b-2]="activeTab === 'overview'"
              [class.border-syslens-accent-blue]="activeTab === 'overview'"
              [class.text-syslens-text-secondary]="activeTab !== 'overview'"
            >
              Overview
            </button>
            <button
              (click)="activeTab = 'resources'"
              class="px-4 py-3 text-sm font-medium transition-colors"
              [class.text-syslens-accent-blue]="activeTab === 'resources'"
              [class.border-b-2]="activeTab === 'resources'"
              [class.border-syslens-accent-blue]="activeTab === 'resources'"
              [class.text-syslens-text-secondary]="activeTab !== 'resources'"
            >
              Resources
            </button>
            <button
              (click)="activeTab = 'hierarchy'"
              class="px-4 py-3 text-sm font-medium transition-colors"
              [class.text-syslens-accent-blue]="activeTab === 'hierarchy'"
              [class.border-b-2]="activeTab === 'hierarchy'"
              [class.border-syslens-accent-blue]="activeTab === 'hierarchy'"
              [class.text-syslens-text-secondary]="activeTab !== 'hierarchy'"
            >
              Hierarchy
            </button>
          </div>

          <!-- Tab Content -->
          <div class="flex-1 overflow-y-auto p-4">
            <!-- Overview Tab -->
            @if (activeTab === 'overview') {
              <div class="space-y-4">
                <!-- Process Identity -->
                <div class="bg-syslens-bg-tertiary rounded-lg p-4">
                  <h3 class="text-sm font-medium text-syslens-text-muted mb-3">Process Identity</h3>
                  <div class="grid grid-cols-2 gap-4">
                    <div>
                      <p class="text-xs text-syslens-text-muted">Name</p>
                      <p class="text-sm text-syslens-text-primary font-medium">{{ process.name }}</p>
                    </div>
                    <div>
                      <p class="text-xs text-syslens-text-muted">PID</p>
                      <p class="text-sm text-syslens-text-primary font-mono">{{ process.pid }}</p>
                    </div>
                    <div>
                      <p class="text-xs text-syslens-text-muted">Parent PID</p>
                      <p class="text-sm text-syslens-text-primary font-mono">
                        {{ process.parentPid ?? 'None' }}
                        @if (process.parentPid && parentProcess) {
                          <span class="text-syslens-text-muted"> ({{ parentProcess.name }})</span>
                        }
                      </p>
                    </div>
                    <div>
                      <p class="text-xs text-syslens-text-muted">User</p>
                      <p class="text-sm text-syslens-text-primary">{{ process.user || 'N/A' }}</p>
                    </div>
                  </div>
                </div>

                <!-- Command Line -->
                <div class="bg-syslens-bg-tertiary rounded-lg p-4">
                  <h3 class="text-sm font-medium text-syslens-text-muted mb-3">Command Line</h3>
                  <div class="bg-syslens-bg-primary rounded p-3 overflow-x-auto">
                    <code class="text-xs text-syslens-text-primary font-mono whitespace-pre-wrap break-all">
                      {{ process.command || process.name }}
                    </code>
                  </div>
                </div>

                <!-- Timing -->
                <div class="bg-syslens-bg-tertiary rounded-lg p-4">
                  <h3 class="text-sm font-medium text-syslens-text-muted mb-3">Timing</h3>
                  <div class="grid grid-cols-2 gap-4">
                    <div>
                      <p class="text-xs text-syslens-text-muted">Started</p>
                      <p class="text-sm text-syslens-text-primary">{{ formatStartTime(process.startTime) }}</p>
                    </div>
                    <div>
                      <p class="text-xs text-syslens-text-muted">Uptime</p>
                      <p class="text-sm text-syslens-text-primary">{{ formatUptime(process.startTime) }}</p>
                    </div>
                  </div>
                </div>
              </div>
            }

            <!-- Resources Tab -->
            @if (activeTab === 'resources') {
              <div class="space-y-4">
                <!-- CPU -->
                <div class="bg-syslens-bg-tertiary rounded-lg p-4">
                  <h3 class="text-sm font-medium text-syslens-text-muted mb-3">CPU Usage</h3>
                  <div class="flex items-center gap-4">
                    <div class="flex-1">
                      <div class="h-3 bg-syslens-bg-primary rounded-full overflow-hidden">
                        <div
                          class="h-full rounded-full transition-all"
                          [style.width.%]="Math.min(process.cpuUsage, 100)"
                          [class.bg-syslens-accent-green]="process.cpuUsage < 25"
                          [class.bg-syslens-accent-yellow]="process.cpuUsage >= 25 && process.cpuUsage < 75"
                          [class.bg-syslens-accent-red]="process.cpuUsage >= 75"
                        ></div>
                      </div>
                    </div>
                    <span class="text-2xl font-bold font-mono text-syslens-text-primary min-w-[80px] text-right">
                      {{ process.cpuUsage.toFixed(1) }}%
                    </span>
                  </div>
                </div>

                <!-- Memory -->
                <div class="bg-syslens-bg-tertiary rounded-lg p-4">
                  <h3 class="text-sm font-medium text-syslens-text-muted mb-3">Memory</h3>
                  <div class="grid grid-cols-2 gap-4">
                    <div>
                      <p class="text-xs text-syslens-text-muted">Physical Memory</p>
                      <p class="text-lg font-bold font-mono text-syslens-text-primary">{{ formatBytes(process.memoryBytes) }}</p>
                    </div>
                    <div>
                      <p class="text-xs text-syslens-text-muted">Virtual Memory</p>
                      <p class="text-lg font-bold font-mono text-syslens-text-primary">{{ formatBytes(process.virtualMemoryBytes) }}</p>
                    </div>
                  </div>
                </div>

                <!-- Disk I/O -->
                <div class="bg-syslens-bg-tertiary rounded-lg p-4">
                  <h3 class="text-sm font-medium text-syslens-text-muted mb-3">Disk I/O (Total)</h3>
                  <div class="grid grid-cols-2 gap-4">
                    <div>
                      <p class="text-xs text-syslens-text-muted">Read</p>
                      <p class="text-lg font-bold font-mono text-syslens-accent-green">{{ formatBytes(process.diskReadBytes) }}</p>
                    </div>
                    <div>
                      <p class="text-xs text-syslens-text-muted">Written</p>
                      <p class="text-lg font-bold font-mono text-syslens-accent-blue">{{ formatBytes(process.diskWriteBytes) }}</p>
                    </div>
                  </div>
                </div>
              </div>
            }

            <!-- Hierarchy Tab -->
            @if (activeTab === 'hierarchy') {
              <div class="space-y-4">
                <!-- Parent Process -->
                <div class="bg-syslens-bg-tertiary rounded-lg p-4">
                  <h3 class="text-sm font-medium text-syslens-text-muted mb-3">Parent Process</h3>
                  @if (parentProcess) {
                    <div
                      class="p-3 bg-syslens-bg-primary rounded-lg cursor-pointer hover:bg-syslens-bg-hover transition-colors"
                      (click)="viewParent()"
                    >
                      <div class="flex items-center justify-between">
                        <div>
                          <p class="text-sm font-medium text-syslens-text-primary">{{ parentProcess.name }}</p>
                          <p class="text-xs text-syslens-text-muted font-mono">PID: {{ parentProcess.pid }}</p>
                        </div>
                        <svg class="w-5 h-5 text-syslens-text-muted" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
                        </svg>
                      </div>
                    </div>
                  } @else if (process.parentPid) {
                    <p class="text-sm text-syslens-text-muted">
                      Parent PID {{ process.parentPid }} (process not found)
                    </p>
                  } @else {
                    <p class="text-sm text-syslens-text-muted">No parent process (system process)</p>
                  }
                </div>

                <!-- Process Chain -->
                <div class="bg-syslens-bg-tertiary rounded-lg p-4">
                  <h3 class="text-sm font-medium text-syslens-text-muted mb-3">Process Chain</h3>
                  <div class="flex items-center gap-2 text-sm overflow-x-auto pb-2">
                    @for (ancestor of processChain; track ancestor.pid; let last = $last) {
                      <span
                        class="px-2 py-1 rounded bg-syslens-bg-primary text-syslens-text-secondary font-mono text-xs whitespace-nowrap cursor-pointer hover:bg-syslens-bg-hover"
                        (click)="viewProcess(ancestor)"
                      >
                        {{ ancestor.name }}
                      </span>
                      @if (!last) {
                        <svg class="w-4 h-4 text-syslens-text-muted flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
                        </svg>
                      }
                    }
                    <span class="px-2 py-1 rounded bg-syslens-accent-blue text-white font-mono text-xs whitespace-nowrap">
                      {{ process.name }}
                    </span>
                  </div>
                </div>
              </div>
            }
          </div>

          <!-- Footer -->
          <div class="p-3 border-t border-syslens-border-primary bg-syslens-bg-tertiary text-xs text-syslens-text-muted flex justify-between items-center">
            <span>PID: {{ process.pid }}</span>
            <div class="flex gap-2">
              <button
                (click)="requestKill()"
                class="px-3 py-1 bg-syslens-accent-red/20 text-syslens-accent-red rounded hover:bg-syslens-accent-red/30 transition-colors"
                title="Terminate this process"
              >
                End Process
              </button>
              <button
                (click)="requestRefresh()"
                class="px-3 py-1 bg-syslens-bg-primary rounded hover:bg-syslens-bg-secondary transition-colors"
              >
                Refresh
              </button>
            </div>
          </div>
        </div>
      </div>
    }
  `,
})
export class ProcessDetailModalComponent {
  @Input() isOpen = false;
  @Input() process: ProcessInfo | null = null;
  @Input() allProcesses: ProcessInfo[] = [];

  @Output() closed = new EventEmitter<void>();
  @Output() refreshRequested = new EventEmitter<number>();
  @Output() processSelected = new EventEmitter<ProcessInfo>();
  @Output() killRequested = new EventEmitter<number>();

  activeTab: TabType = 'overview';
  Math = Math;

  get parentProcess(): ProcessInfo | null {
    if (!this.process?.parentPid) return null;
    return this.allProcesses.find((p) => p.pid === this.process!.parentPid) || null;
  }

  get processChain(): ProcessInfo[] {
    const chain: ProcessInfo[] = [];
    let current = this.parentProcess;

    while (current && chain.length < 10) {
      chain.unshift(current);
      current = this.allProcesses.find((p) => p.pid === current!.parentPid) || null;
    }

    return chain;
  }

  close(): void {
    this.isOpen = false;
    this.activeTab = 'overview';
    this.closed.emit();
  }

  requestRefresh(): void {
    if (this.process) {
      this.refreshRequested.emit(this.process.pid);
    }
  }

  requestKill(): void {
    if (this.process) {
      // Confirm before killing
      const confirmed = confirm(
        `Are you sure you want to end "${this.process.name}" (PID: ${this.process.pid})?\n\nThis may cause data loss if the process has unsaved work.`
      );
      if (confirmed) {
        this.killRequested.emit(this.process.pid);
      }
    }
  }

  viewParent(): void {
    if (this.parentProcess) {
      this.processSelected.emit(this.parentProcess);
    }
  }

  viewProcess(process: ProcessInfo): void {
    this.processSelected.emit(process);
  }

  formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
  }

  formatStartTime(timestamp: number): string {
    if (!timestamp) return 'Unknown';
    const date = new Date(timestamp * 1000);
    return date.toLocaleString();
  }

  formatUptime(startTimestamp: number): string {
    if (!startTimestamp) return 'Unknown';
    const now = Date.now();
    const start = startTimestamp * 1000;
    const diff = now - start;

    if (diff < 0) return 'Unknown';

    const seconds = Math.floor(diff / 1000);
    const minutes = Math.floor(seconds / 60);
    const hours = Math.floor(minutes / 60);
    const days = Math.floor(hours / 24);

    if (days > 0) {
      return `${days}d ${hours % 24}h ${minutes % 60}m`;
    } else if (hours > 0) {
      return `${hours}h ${minutes % 60}m ${seconds % 60}s`;
    } else if (minutes > 0) {
      return `${minutes}m ${seconds % 60}s`;
    } else {
      return `${seconds}s`;
    }
  }
}
