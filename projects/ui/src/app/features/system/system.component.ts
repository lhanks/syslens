import { Component, OnInit, OnDestroy, inject } from '@angular/core';
import { CommonModule } from '@angular/common';
import { Subject, takeUntil } from 'rxjs';

import { SystemService, StatusService } from '@core/services';
import { DeviceInfo, BiosInfo, BootConfig, OsInfo, SystemUptime, UserInfo } from '@core/models';
import { UptimePipe } from '@shared/pipes';

@Component({
  selector: 'app-system',
  standalone: true,
  imports: [CommonModule, UptimePipe],
  template: `
    <div class="p-6 space-y-6">
      <!-- Header -->
      <div>
        <h1 class="text-2xl font-bold text-syslens-text-primary">System</h1>
        <p class="text-syslens-text-secondary">Device and operating system information</p>
      </div>

      <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <!-- Device Information -->
        <section class="card">
          <h2 class="section-title">Device Information</h2>
          @if (deviceInfo) {
            <dl class="space-y-3">
              <div class="flex justify-between">
                <dt class="text-syslens-text-muted">Computer Name</dt>
                <dd class="text-syslens-text-primary font-medium">{{ deviceInfo.computerName }}</dd>
              </div>
              <div class="flex justify-between">
                <dt class="text-syslens-text-muted">Manufacturer</dt>
                <dd class="text-syslens-text-primary">{{ deviceInfo.manufacturer }}</dd>
              </div>
              <div class="flex justify-between">
                <dt class="text-syslens-text-muted">Model</dt>
                <dd class="text-syslens-text-primary">{{ deviceInfo.model }}</dd>
              </div>
              <div class="flex justify-between">
                <dt class="text-syslens-text-muted">System Type</dt>
                <dd class="text-syslens-text-primary">{{ deviceInfo.systemType }}</dd>
              </div>
              @if (deviceInfo.serialNumber) {
                <div class="flex justify-between">
                  <dt class="text-syslens-text-muted">Serial Number</dt>
                  <dd class="text-syslens-text-primary font-mono text-sm">{{ deviceInfo.serialNumber }}</dd>
                </div>
              }
            </dl>
          }
        </section>

        <!-- Operating System -->
        <section class="card">
          <h2 class="section-title">Operating System</h2>
          @if (osInfo) {
            <dl class="space-y-3">
              <div class="flex justify-between">
                <dt class="text-syslens-text-muted">Name</dt>
                <dd class="text-syslens-text-primary font-medium">{{ osInfo.name }}</dd>
              </div>
              <div class="flex justify-between">
                <dt class="text-syslens-text-muted">Version</dt>
                <dd class="text-syslens-text-primary">{{ osInfo.version }}</dd>
              </div>
              <div class="flex justify-between">
                <dt class="text-syslens-text-muted">Build</dt>
                <dd class="text-syslens-text-primary font-mono text-sm">{{ osInfo.build }}</dd>
              </div>
              <div class="flex justify-between">
                <dt class="text-syslens-text-muted">Architecture</dt>
                <dd class="text-syslens-text-primary">{{ osInfo.architecture }}</dd>
              </div>
              <div class="flex justify-between">
                <dt class="text-syslens-text-muted">Install Date</dt>
                <dd class="text-syslens-text-primary">{{ osInfo.installDate | date:'medium' }}</dd>
              </div>
              <div class="flex justify-between">
                <dt class="text-syslens-text-muted">Activation</dt>
                <dd>
                  <span class="px-2 py-0.5 rounded text-xs"
                        [class.bg-syslens-accent-green]="osInfo.activationStatus === 'Activated'"
                        [class.text-white]="osInfo.activationStatus === 'Activated'"
                        [class.bg-syslens-accent-yellow]="osInfo.activationStatus !== 'Activated'"
                        [class.text-black]="osInfo.activationStatus !== 'Activated'">
                    {{ osInfo.activationStatus }}
                  </span>
                </dd>
              </div>
            </dl>
          }
        </section>

        <!-- BIOS/Firmware -->
        <section class="card">
          <h2 class="section-title">BIOS / Firmware</h2>
          @if (biosInfo) {
            <dl class="space-y-3">
              <div class="flex justify-between">
                <dt class="text-syslens-text-muted">Vendor</dt>
                <dd class="text-syslens-text-primary">{{ biosInfo.vendor }}</dd>
              </div>
              <div class="flex justify-between">
                <dt class="text-syslens-text-muted">Version</dt>
                <dd class="text-syslens-text-primary font-mono text-sm">{{ biosInfo.version }}</dd>
              </div>
              <div class="flex justify-between">
                <dt class="text-syslens-text-muted">Release Date</dt>
                <dd class="text-syslens-text-primary">{{ biosInfo.releaseDate }}</dd>
              </div>
              <div class="flex justify-between">
                <dt class="text-syslens-text-muted">Secure Boot</dt>
                <dd>
                  <span class="px-2 py-0.5 rounded text-xs"
                        [class.bg-syslens-accent-green]="biosInfo.secureBoot"
                        [class.text-white]="biosInfo.secureBoot"
                        [class.bg-syslens-bg-tertiary]="!biosInfo.secureBoot"
                        [class.text-syslens-text-secondary]="!biosInfo.secureBoot">
                    {{ biosInfo.secureBoot ? 'Enabled' : 'Disabled' }}
                  </span>
                </dd>
              </div>
              @if (biosInfo.tpmVersion) {
                <div class="flex justify-between">
                  <dt class="text-syslens-text-muted">TPM</dt>
                  <dd class="text-syslens-text-primary">{{ biosInfo.tpmVersion }} ({{ biosInfo.tpmStatus }})</dd>
                </div>
              }
            </dl>
          }
        </section>

        <!-- Boot Configuration -->
        <section class="card">
          <h2 class="section-title">Boot Configuration</h2>
          @if (bootConfig) {
            <dl class="space-y-3">
              <div class="flex justify-between">
                <dt class="text-syslens-text-muted">Boot Mode</dt>
                <dd class="text-syslens-text-primary font-medium">{{ bootConfig.bootMode }}</dd>
              </div>
              <div class="flex justify-between">
                <dt class="text-syslens-text-muted">Boot Device</dt>
                <dd class="text-syslens-text-primary text-sm truncate max-w-[200px]" [title]="bootConfig.bootDevice">
                  {{ bootConfig.bootDevice }}
                </dd>
              </div>
              <div class="flex justify-between">
                <dt class="text-syslens-text-muted">Fast Startup</dt>
                <dd class="text-syslens-text-primary">{{ bootConfig.fastStartup ? 'Enabled' : 'Disabled' }}</dd>
              </div>
              <div class="flex justify-between">
                <dt class="text-syslens-text-muted">Last Boot</dt>
                <dd class="text-syslens-text-primary">{{ bootConfig.lastBootTime | date:'medium' }}</dd>
              </div>
              <div class="flex justify-between">
                <dt class="text-syslens-text-muted">Boot Duration</dt>
                <dd class="text-syslens-text-primary">{{ bootConfig.bootDurationSeconds }}s</dd>
              </div>
            </dl>
          }
        </section>

        <!-- System Uptime -->
        <section class="card">
          <h2 class="section-title">System Uptime</h2>
          @if (uptime) {
            <div class="text-center py-4">
              <p class="text-4xl font-mono font-bold text-syslens-accent-blue">
                {{ uptime.uptimeSeconds | uptime }}
              </p>
              <p class="text-syslens-text-muted mt-2">System running since boot</p>
            </div>
            <dl class="mt-4 space-y-3 pt-4 border-t border-syslens-border-primary">
              @if (uptime.lastShutdown) {
                <div class="flex justify-between">
                  <dt class="text-syslens-text-muted">Last Shutdown</dt>
                  <dd class="text-syslens-text-primary">{{ uptime.lastShutdown | date:'medium' }}</dd>
                </div>
              }
              <div class="flex justify-between">
                <dt class="text-syslens-text-muted">Restart Pending</dt>
                <dd>
                  <span class="px-2 py-0.5 rounded text-xs"
                        [class.bg-syslens-accent-yellow]="uptime.restartPending"
                        [class.text-black]="uptime.restartPending"
                        [class.bg-syslens-bg-tertiary]="!uptime.restartPending"
                        [class.text-syslens-text-secondary]="!uptime.restartPending">
                    {{ uptime.restartPending ? 'Yes' : 'No' }}
                  </span>
                </dd>
              </div>
            </dl>
          }
        </section>

        <!-- Current User -->
        <section class="card">
          <h2 class="section-title">Current User</h2>
          @if (userInfo) {
            <dl class="space-y-3">
              <div class="flex justify-between">
                <dt class="text-syslens-text-muted">Username</dt>
                <dd class="text-syslens-text-primary font-medium">{{ userInfo.username }}</dd>
              </div>
              <div class="flex justify-between">
                <dt class="text-syslens-text-muted">Profile Path</dt>
                <dd class="text-syslens-text-primary text-sm truncate max-w-[200px]" [title]="userInfo.userProfile">
                  {{ userInfo.userProfile }}
                </dd>
              </div>
              <div class="flex justify-between">
                <dt class="text-syslens-text-muted">Admin Status</dt>
                <dd>
                  <span class="px-2 py-0.5 rounded text-xs"
                        [class.bg-syslens-accent-purple]="userInfo.isAdmin"
                        [class.text-white]="userInfo.isAdmin"
                        [class.bg-syslens-bg-tertiary]="!userInfo.isAdmin"
                        [class.text-syslens-text-secondary]="!userInfo.isAdmin">
                    {{ userInfo.isAdmin ? 'Administrator' : 'Standard User' }}
                  </span>
                </dd>
              </div>
              <div class="flex justify-between">
                <dt class="text-syslens-text-muted">Login Time</dt>
                <dd class="text-syslens-text-primary">{{ userInfo.loginTime | date:'medium' }}</dd>
              </div>
            </dl>
          }
        </section>
      </div>
    </div>
  `
})
export class SystemComponent implements OnInit, OnDestroy {
  private systemService = inject(SystemService);
  private statusService = inject(StatusService);
  private destroy$ = new Subject<void>();

  deviceInfo: DeviceInfo | null = null;
  biosInfo: BiosInfo | null = null;
  bootConfig: BootConfig | null = null;
  osInfo: OsInfo | null = null;
  uptime: SystemUptime | null = null;
  userInfo: UserInfo | null = null;

  ngOnInit(): void {
    this.loadSystemData();
    this.startRealtimeUpdates();
  }

  ngOnDestroy(): void {
    this.destroy$.next();
    this.destroy$.complete();
  }

  private loadSystemData(): void {
    this.statusService.startOperation('system-init', 'Loading system information...');

    this.systemService.getDeviceInfo()
      .pipe(takeUntil(this.destroy$))
      .subscribe(info => {
        this.deviceInfo = info;
        this.statusService.endOperation('system-init');
      });

    this.systemService.getBiosInfo()
      .pipe(takeUntil(this.destroy$))
      .subscribe(info => this.biosInfo = info);

    this.systemService.getBootConfig()
      .pipe(takeUntil(this.destroy$))
      .subscribe(config => this.bootConfig = config);

    this.systemService.getOsInfo()
      .pipe(takeUntil(this.destroy$))
      .subscribe(info => this.osInfo = info);

    this.systemService.getUserInfo()
      .pipe(takeUntil(this.destroy$))
      .subscribe(info => this.userInfo = info);
  }

  private startRealtimeUpdates(): void {
    this.systemService.getUptime()
      .pipe(takeUntil(this.destroy$))
      .subscribe(uptime => this.uptime = uptime);
  }
}
