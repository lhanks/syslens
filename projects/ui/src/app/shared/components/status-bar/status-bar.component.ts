import { Component, inject, computed, signal, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { StatusService } from '@core/services/status.service';
import { MetricsHistoryService } from '@core/services';
import { BytesPipe } from '@shared/pipes';
import { getVersion } from '@tauri-apps/api/app';

@Component({
  selector: 'app-status-bar',
  standalone: true,
  imports: [CommonModule, BytesPipe],
  template: `
    <div class="h-6 px-3 flex items-center justify-between bg-syslens-bg-tertiary border-t border-syslens-border text-xs">
      <!-- Left side: Status indicator and message -->
      <div class="flex items-center gap-2">
        @if (statusService.isLoading()) {
          <div class="w-2 h-2 rounded-full bg-syslens-accent-blue animate-pulse"></div>
        } @else {
          <div class="w-2 h-2 rounded-full bg-syslens-accent-green"></div>
        }
        <span class="text-syslens-text-secondary">{{ statusService.currentStatus() }}</span>
      </div>

      <!-- Right side: System metrics, IP, and version -->
      <div class="flex items-center gap-4">
        <!-- CPU -->
        <div class="flex items-center gap-1.5">
          <span class="text-syslens-text-muted">CPU</span>
          <span class="font-mono w-10 text-right" [class]="cpuColorClass()">{{ cpuUsage().toFixed(0) }}%</span>
        </div>

        <!-- Memory -->
        <div class="flex items-center gap-1.5">
          <span class="text-syslens-text-muted">RAM</span>
          <span class="font-mono w-28 text-right" [class]="memoryColorClass()">{{ memoryUsed() | bytes }} / {{ memoryTotal() | bytes }}</span>
        </div>

        <!-- Disk -->
        <div class="flex items-center gap-1.5">
          <span class="text-syslens-text-muted">Disk</span>
          <span class="font-mono w-10 text-right" [class]="diskColorClass()">{{ diskActivity().toFixed(0) }}%</span>
        </div>

        <!-- Network -->
        <div class="flex items-center gap-1.5">
          <span class="text-syslens-text-muted">Net</span>
          <span class="font-mono w-20 text-right text-syslens-accent-green">↓{{ networkDown() | bytes }}/s</span>
          <span class="font-mono w-20 text-right text-syslens-accent-blue">↑{{ networkUp() | bytes }}/s</span>
        </div>

        <!-- IP Address -->
        @if (metricsService.primaryIpAddress()) {
          <div class="flex items-center gap-1.5 border-l border-syslens-border-primary pl-4">
            <span class="text-syslens-text-muted">IP</span>
            <span class="font-mono text-syslens-text-secondary">{{ metricsService.primaryIpAddress() }}</span>
          </div>
        }

        <!-- App Version -->
        <div class="flex items-center gap-1.5 border-l border-syslens-border-primary pl-4">
          <span class="text-syslens-text-muted">v{{ appVersion() }}</span>
        </div>
      </div>
    </div>
  `
})
export class StatusBarComponent implements OnInit {
  statusService = inject(StatusService);
  metricsService = inject(MetricsHistoryService);

  // App version
  appVersion = signal('0.1.0');

  // System metrics from metrics service
  cpuUsage = computed(() => this.metricsService.cpuUsage());
  memoryUsage = computed(() => this.metricsService.memoryUsage());
  memoryUsed = computed(() => this.metricsService.memoryUsedBytes());
  memoryTotal = computed(() => this.metricsService.memoryTotalBytes());
  diskActivity = computed(() => this.metricsService.diskActivity());
  networkDown = computed(() => this.metricsService.networkDownSpeed());
  networkUp = computed(() => this.metricsService.networkUpSpeed());

  // Color classes based on usage levels
  cpuColorClass = computed(() => {
    const usage = this.cpuUsage();
    if (usage >= 80) return 'text-syslens-accent-red';
    if (usage >= 50) return 'text-syslens-accent-yellow';
    return 'text-syslens-accent-green';
  });

  memoryColorClass = computed(() => {
    const usage = this.memoryUsage();
    if (usage >= 85) return 'text-syslens-accent-red';
    if (usage >= 60) return 'text-syslens-accent-yellow';
    return 'text-syslens-accent-green';
  });

  diskColorClass = computed(() => {
    const usage = this.diskActivity();
    if (usage >= 80) return 'text-syslens-accent-red';
    if (usage >= 50) return 'text-syslens-accent-yellow';
    return 'text-syslens-accent-green';
  });

  ngOnInit(): void {
    this.loadAppVersion();
  }

  private async loadAppVersion(): Promise<void> {
    try {
      const version = await getVersion();
      this.appVersion.set(version);
    } catch {
      // Keep default version if Tauri API is not available (browser mode)
    }
  }
}
