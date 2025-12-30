import { Component, OnInit, OnDestroy, inject, signal } from '@angular/core';
import { CommonModule } from '@angular/common';
import { Subject, takeUntil } from 'rxjs';

import { SystemService } from '@core/services';
import { RestorePoint, RestorePointType } from '@core/models';

@Component({
  selector: 'app-restore-points',
  standalone: true,
  imports: [CommonModule],
  template: `
    <div class="p-6 space-y-6">
      <!-- Header -->
      <div>
        <h1 class="text-2xl font-bold text-syslens-text-primary">System Restore Points</h1>
        <p class="text-syslens-text-secondary">Windows recovery checkpoints</p>
      </div>

      <!-- Loading State -->
      @if (isLoading()) {
        <div class="card flex items-center justify-center py-12">
          <div class="animate-spin w-8 h-8 border-2 border-syslens-accent-blue border-t-transparent rounded-full"></div>
        </div>
      } @else {
        <!-- Restore Points List -->
        <section>
          <h2 class="section-title">Available Restore Points</h2>

          @if (restorePoints().length === 0) {
            <div class="card text-center py-12">
              <div class="w-16 h-16 rounded-full bg-syslens-bg-tertiary flex items-center justify-center mx-auto mb-4">
                <svg class="w-8 h-8 text-syslens-text-muted" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                    d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
              </div>
              <p class="text-syslens-text-muted mb-2">No restore points found</p>
              <p class="text-sm text-syslens-text-muted">System Protection may be disabled or no restore points have been created yet.</p>
            </div>
          } @else {
            <div class="space-y-3">
              @for (point of restorePoints(); track point.sequenceNumber) {
                <div class="card hover:border-syslens-accent-blue transition-colors">
                  <div class="flex items-start gap-4">
                    <!-- Icon -->
                    <div class="w-10 h-10 rounded-lg flex items-center justify-center flex-shrink-0"
                         [class]="getTypeIconClass(point.restorePointType)">
                      <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        @switch (getTypeCategory(point.restorePointType)) {
                          @case ('app') {
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                              d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" />
                          }
                          @case ('driver') {
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                              d="M9 3v2m6-2v2M9 19v2m6-2v2M5 9H3m2 6H3m18-6h-2m2 6h-2M7 19h10a2 2 0 002-2V7a2 2 0 00-2-2H7a2 2 0 00-2 2v10a2 2 0 002 2zM9 9h6v6H9V9z" />
                          }
                          @case ('update') {
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                              d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                          }
                          @case ('manual') {
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                              d="M5 5a2 2 0 012-2h10a2 2 0 012 2v16l-7-3.5L5 21V5z" />
                          }
                          @default {
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                              d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
                          }
                        }
                      </svg>
                    </div>

                    <!-- Content -->
                    <div class="flex-1 min-w-0">
                      <div class="flex items-start justify-between gap-4">
                        <div class="min-w-0">
                          <h3 class="font-medium text-syslens-text-primary truncate">{{ point.description }}</h3>
                          <p class="text-sm text-syslens-text-muted mt-0.5">{{ formatDate(point.creationTime) }}</p>
                        </div>
                        <div class="flex-shrink-0">
                          <span class="px-2 py-1 text-xs rounded"
                                [class]="getTypeBadgeClass(point.restorePointType)">
                            {{ getTypeLabel(point.restorePointType) }}
                          </span>
                        </div>
                      </div>
                      <p class="text-xs text-syslens-text-muted mt-2">Sequence #{{ point.sequenceNumber }}</p>
                    </div>
                  </div>
                </div>
              }
            </div>

            <!-- Summary -->
            <div class="mt-6 p-4 bg-syslens-bg-tertiary rounded-lg">
              <p class="text-sm text-syslens-text-secondary">
                <span class="font-medium text-syslens-text-primary">{{ restorePoints().length }}</span> restore point{{ restorePoints().length !== 1 ? 's' : '' }} available
              </p>
            </div>
          }
        </section>
      }
    </div>
  `,
  styles: [`
    :host {
      display: block;
      height: 100%;
      overflow-y: auto;
    }
  `]
})
export class RestorePointsComponent implements OnInit, OnDestroy {
  private systemService = inject(SystemService);
  private destroy$ = new Subject<void>();

  restorePoints = signal<RestorePoint[]>([]);
  isLoading = signal(true);

  ngOnInit(): void {
    this.loadRestorePoints();
  }

  ngOnDestroy(): void {
    this.destroy$.next();
    this.destroy$.complete();
  }

  private loadRestorePoints(): void {
    this.systemService.getRestorePoints().pipe(
      takeUntil(this.destroy$)
    ).subscribe({
      next: (points) => {
        // Sort by sequence number descending (newest first)
        this.restorePoints.set(points.sort((a, b) => b.sequenceNumber - a.sequenceNumber));
        this.isLoading.set(false);
      },
      error: () => {
        this.isLoading.set(false);
      }
    });
  }

  getTypeCategory(type: RestorePointType): string {
    switch (type) {
      case 'ApplicationInstall':
      case 'ApplicationUninstall':
        return 'app';
      case 'DeviceDriverInstall':
        return 'driver';
      case 'WindowsUpdate':
        return 'update';
      case 'ManualCheckpoint':
        return 'manual';
      default:
        return 'other';
    }
  }

  getTypeIconClass(type: RestorePointType): string {
    const base = 'text-white';
    switch (type) {
      case 'ApplicationInstall':
        return `${base} bg-syslens-accent-green`;
      case 'ApplicationUninstall':
        return `${base} bg-syslens-accent-red`;
      case 'DeviceDriverInstall':
        return `${base} bg-syslens-accent-purple`;
      case 'WindowsUpdate':
        return `${base} bg-syslens-accent-blue`;
      case 'ManualCheckpoint':
        return `${base} bg-syslens-accent-cyan`;
      default:
        return 'text-syslens-text-secondary bg-syslens-bg-tertiary';
    }
  }

  getTypeBadgeClass(type: RestorePointType): string {
    switch (type) {
      case 'ApplicationInstall':
        return 'bg-syslens-accent-green/20 text-syslens-accent-green';
      case 'ApplicationUninstall':
        return 'bg-syslens-accent-red/20 text-syslens-accent-red';
      case 'DeviceDriverInstall':
        return 'bg-syslens-accent-purple/20 text-syslens-accent-purple';
      case 'WindowsUpdate':
        return 'bg-syslens-accent-blue/20 text-syslens-accent-blue';
      case 'ManualCheckpoint':
        return 'bg-syslens-accent-cyan/20 text-syslens-accent-cyan';
      default:
        return 'bg-syslens-bg-tertiary text-syslens-text-secondary';
    }
  }

  getTypeLabel(type: RestorePointType): string {
    switch (type) {
      case 'ApplicationInstall':
        return 'App Install';
      case 'ApplicationUninstall':
        return 'App Uninstall';
      case 'DeviceDriverInstall':
        return 'Driver Install';
      case 'WindowsUpdate':
        return 'Windows Update';
      case 'ManualCheckpoint':
        return 'Manual';
      case 'ModifySettings':
        return 'Settings Change';
      case 'CancelledOperation':
        return 'Cancelled';
      case 'BackupRecovery':
        return 'Backup/Recovery';
      default:
        return 'Unknown';
    }
  }

  formatDate(dateStr: string): string {
    try {
      // Parse the date string (format: YYYY-MM-DD HH:MM:SS)
      const [datePart, timePart] = dateStr.split(' ');
      if (!datePart || !timePart) return dateStr;

      const [year, month, day] = datePart.split('-').map(Number);
      const [hours, minutes] = timePart.split(':').map(Number);

      const date = new Date(year, month - 1, day, hours, minutes);

      return date.toLocaleString(undefined, {
        year: 'numeric',
        month: 'short',
        day: 'numeric',
        hour: '2-digit',
        minute: '2-digit'
      });
    } catch {
      return dateStr;
    }
  }
}
