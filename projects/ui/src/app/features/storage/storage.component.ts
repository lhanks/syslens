import { Component, OnInit, OnDestroy, inject } from '@angular/core';
import { CommonModule } from '@angular/common';
import { Subject, takeUntil } from 'rxjs';

import { StorageService } from '@core/services';
import { PhysicalDisk, Volume, DiskHealth, DiskPerformance, NetworkDrive } from '@core/models';
import { BytesPipe } from '@shared/pipes';

@Component({
  selector: 'app-storage',
  standalone: true,
  imports: [CommonModule, BytesPipe],
  template: `
    <div class="p-6 space-y-6">
      <!-- Header -->
      <div>
        <h1 class="text-2xl font-bold text-syslens-text-primary">Storage</h1>
        <p class="text-syslens-text-secondary">Disks, volumes, and storage health</p>
      </div>

      <!-- Volumes Overview -->
      <section>
        <h2 class="section-title">Volumes</h2>
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          @for (volume of volumes; track volume.driveLetter) {
            <div class="card">
              <div class="flex items-start justify-between mb-3">
                <div class="flex items-center gap-3">
                  <div class="w-10 h-10 rounded-lg bg-syslens-bg-tertiary flex items-center justify-center">
                    <span class="text-lg font-bold text-syslens-accent-cyan">{{ volume.driveLetter || '?' }}</span>
                  </div>
                  <div>
                    <h3 class="font-medium text-syslens-text-primary">{{ volume.label || 'Local Disk' }}</h3>
                    <p class="text-xs text-syslens-text-muted">{{ volume.fileSystem }}</p>
                  </div>
                </div>
                <div class="flex gap-1">
                  @if (volume.isSystem) {
                    <span class="px-1.5 py-0.5 text-xs bg-syslens-accent-blue text-white rounded">System</span>
                  }
                  @if (volume.isEncrypted) {
                    <span class="px-1.5 py-0.5 text-xs bg-syslens-accent-green text-white rounded">Encrypted</span>
                  }
                </div>
              </div>

              <!-- Capacity Bar -->
              <div class="mb-2">
                <div class="progress-bar h-3">
                  <div class="progress-fill"
                       [style.width.%]="volume.percentUsed"
                       [class.bg-syslens-accent-cyan]="volume.percentUsed < 75"
                       [class.bg-syslens-accent-yellow]="volume.percentUsed >= 75 && volume.percentUsed < 90"
                       [class.bg-syslens-accent-red]="volume.percentUsed >= 90">
                  </div>
                </div>
              </div>

              <div class="flex justify-between text-sm">
                <span class="text-syslens-text-muted">{{ volume.usedBytes | bytes }} used</span>
                <span class="text-syslens-text-secondary">{{ volume.freeBytes | bytes }} free</span>
              </div>
              <p class="text-xs text-syslens-text-muted mt-1">{{ volume.totalBytes | bytes }} total</p>
            </div>
          } @empty {
            <div class="card text-center text-syslens-text-muted py-8 col-span-full">
              No volumes found
            </div>
          }
        </div>
      </section>

      <!-- Physical Disks -->
      <section>
        <h2 class="section-title">Physical Disks</h2>
        <div class="space-y-4">
          @for (disk of disks; track disk.deviceId) {
            <div class="card">
              <div class="flex flex-col lg:flex-row lg:items-start gap-4">
                <!-- Disk Info -->
                <div class="flex-1">
                  <div class="flex items-start justify-between">
                    <div>
                      <h3 class="font-medium text-syslens-text-primary">{{ disk.model }}</h3>
                      <p class="text-sm text-syslens-text-muted">{{ disk.manufacturer }}</p>
                    </div>
                    <div class="flex gap-2">
                      <span class="px-2 py-0.5 text-xs rounded"
                            [class.bg-syslens-accent-purple]="disk.mediaType === 'NVMe'"
                            [class.text-white]="disk.mediaType === 'NVMe'"
                            [class.bg-syslens-accent-blue]="disk.mediaType === 'SSD'"
                            [class.text-white]="disk.mediaType === 'SSD'"
                            [class.bg-syslens-bg-tertiary]="disk.mediaType !== 'NVMe' && disk.mediaType !== 'SSD'"
                            [class.text-syslens-text-secondary]="disk.mediaType !== 'NVMe' && disk.mediaType !== 'SSD'">
                        {{ disk.mediaType }}
                      </span>
                      <span class="px-2 py-0.5 text-xs bg-syslens-bg-tertiary text-syslens-text-secondary rounded">
                        {{ disk.interfaceType }}
                      </span>
                    </div>
                  </div>

                  <div class="mt-4 grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
                    <div>
                      <p class="text-xs text-syslens-text-muted">Capacity</p>
                      <p class="text-syslens-text-primary font-medium">{{ disk.sizeBytes | bytes }}</p>
                    </div>
                    <div>
                      <p class="text-xs text-syslens-text-muted">Partition Style</p>
                      <p class="text-syslens-text-primary">{{ disk.partitionStyle }}</p>
                    </div>
                    <div>
                      <p class="text-xs text-syslens-text-muted">Status</p>
                      <p class="text-syslens-text-primary">{{ disk.status }}</p>
                    </div>
                    <div>
                      <p class="text-xs text-syslens-text-muted">Firmware</p>
                      <p class="text-syslens-text-primary font-mono text-xs">{{ disk.firmware }}</p>
                    </div>
                  </div>
                </div>

                <!-- Health Status -->
                @if (diskHealthMap[disk.deviceId]; as health) {
                  <div class="lg:w-48 p-3 bg-syslens-bg-tertiary rounded-lg">
                    <div class="flex items-center gap-2 mb-2">
                      <div class="w-3 h-3 rounded-full"
                           [class.bg-syslens-accent-green]="health.status === 'Good'"
                           [class.bg-syslens-accent-yellow]="health.status === 'Warning'"
                           [class.bg-syslens-accent-red]="health.status === 'Critical'"
                           [class.bg-syslens-text-muted]="health.status === 'Unknown'">
                      </div>
                      <span class="text-sm font-medium text-syslens-text-primary">{{ health.status }}</span>
                    </div>
                    @if (health.temperatureCelsius !== null) {
                      <p class="text-xs text-syslens-text-muted">
                        Temp: <span class="text-syslens-text-secondary">{{ health.temperatureCelsius }}C</span>
                      </p>
                    }
                    @if (health.powerOnHours !== null) {
                      <p class="text-xs text-syslens-text-muted">
                        Power On: <span class="text-syslens-text-secondary">{{ health.powerOnHours | number }} hrs</span>
                      </p>
                    }
                    @if (health.wearLevelPercent !== null) {
                      <p class="text-xs text-syslens-text-muted">
                        Wear: <span class="text-syslens-text-secondary">{{ health.wearLevelPercent }}%</span>
                      </p>
                    }
                  </div>
                }
              </div>

              <!-- Performance Metrics -->
              @if (diskPerfMap[disk.deviceId]; as perf) {
                <div class="mt-4 pt-4 border-t border-syslens-border-primary">
                  <p class="text-xs text-syslens-text-muted mb-2">Performance</p>
                  <div class="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
                    <div>
                      <p class="text-xs text-syslens-text-muted">Read</p>
                      <p class="font-mono text-syslens-accent-green">{{ perf.readBytesPerSec | bytes }}/s</p>
                    </div>
                    <div>
                      <p class="text-xs text-syslens-text-muted">Write</p>
                      <p class="font-mono text-syslens-accent-blue">{{ perf.writeBytesPerSec | bytes }}/s</p>
                    </div>
                    <div>
                      <p class="text-xs text-syslens-text-muted">IOPS (R/W)</p>
                      <p class="font-mono text-syslens-text-primary">{{ perf.readIops }} / {{ perf.writeIops }}</p>
                    </div>
                    <div>
                      <p class="text-xs text-syslens-text-muted">Active Time</p>
                      <p class="font-mono text-syslens-text-primary">{{ perf.activeTimePercent | number:'1.0-0' }}%</p>
                    </div>
                  </div>
                </div>
              }
            </div>
          } @empty {
            <div class="card text-center text-syslens-text-muted py-8">
              No physical disks found
            </div>
          }
        </div>
      </section>

      <!-- Network Drives -->
      @if (networkDrives.length > 0) {
        <section>
          <h2 class="section-title">Network Drives</h2>
          <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
            @for (drive of networkDrives; track drive.driveLetter) {
              <div class="card">
                <div class="flex items-center gap-3">
                  <div class="w-10 h-10 rounded-lg bg-syslens-bg-tertiary flex items-center justify-center">
                    <span class="text-lg font-bold text-syslens-accent-blue">{{ drive.driveLetter }}</span>
                  </div>
                  <div class="flex-1 min-w-0">
                    <h3 class="font-medium text-syslens-text-primary truncate">{{ drive.shareName }}</h3>
                    <p class="text-sm text-syslens-text-muted truncate" [title]="drive.uncPath">{{ drive.uncPath }}</p>
                  </div>
                  <span class="px-2 py-0.5 text-xs rounded"
                        [class.bg-syslens-accent-green]="drive.status === 'Connected'"
                        [class.text-white]="drive.status === 'Connected'"
                        [class.bg-syslens-accent-red]="drive.status === 'Disconnected'"
                        [class.text-white]="drive.status === 'Disconnected'"
                        [class.bg-syslens-bg-tertiary]="drive.status === 'Unknown'"
                        [class.text-syslens-text-secondary]="drive.status === 'Unknown'">
                    {{ drive.status }}
                  </span>
                </div>
              </div>
            }
          </div>
        </section>
      }
    </div>
  `
})
export class StorageComponent implements OnInit, OnDestroy {
  private storageService = inject(StorageService);
  private destroy$ = new Subject<void>();

  disks: PhysicalDisk[] = [];
  volumes: Volume[] = [];
  diskHealthMap: Record<number, DiskHealth> = {};
  diskPerfMap: Record<number, DiskPerformance> = {};
  networkDrives: NetworkDrive[] = [];

  ngOnInit(): void {
    this.loadStorageData();
    this.startRealtimeUpdates();
  }

  ngOnDestroy(): void {
    this.destroy$.next();
    this.destroy$.complete();
  }

  private loadStorageData(): void {
    this.storageService.getPhysicalDisks()
      .pipe(takeUntil(this.destroy$))
      .subscribe(disks => {
        this.disks = disks;
        // Load health for each disk
        disks.forEach(disk => {
          this.storageService.getDiskHealth(disk.deviceId)
            .pipe(takeUntil(this.destroy$))
            .subscribe(health => {
              this.diskHealthMap[disk.deviceId] = health;
            });
        });
      });

    this.storageService.getVolumes()
      .pipe(takeUntil(this.destroy$))
      .subscribe(volumes => this.volumes = volumes);

    this.storageService.getNetworkDrives()
      .pipe(takeUntil(this.destroy$))
      .subscribe(drives => this.networkDrives = drives);
  }

  private startRealtimeUpdates(): void {
    this.storageService.getDiskPerformancePolling()
      .pipe(takeUntil(this.destroy$))
      .subscribe(perfList => {
        this.diskPerfMap = {};
        perfList.forEach(p => this.diskPerfMap[p.deviceId] = p);
      });

    // Refresh volumes periodically
    this.storageService.getVolumesPolling()
      .pipe(takeUntil(this.destroy$))
      .subscribe(volumes => this.volumes = volumes);
  }
}
