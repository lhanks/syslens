import { Component, OnInit, OnDestroy, inject, signal, computed } from '@angular/core';
import { CommonModule } from '@angular/common';
import { Subject, takeUntil, interval } from 'rxjs';
import { startWith } from 'rxjs/operators';

import { NetworkService, StatusService } from '@core/services';
import { NetworkAdapter, AdapterStats, NetworkConnection, Route } from '@core/models';
import { BytesPipe } from '@shared/pipes';
import { LineGraphComponent } from '@shared/components';

const MAX_HISTORY_POINTS = 60;

interface AdapterTrafficHistory {
  downloadHistory: number[];
  uploadHistory: number[];
  downloadSpeed: number;
  uploadSpeed: number;
  maxSpeed: number;
  previousStats: { bytesReceived: number; bytesSent: number; timestamp: number } | null;
}

@Component({
  selector: 'app-network',
  standalone: true,
  imports: [CommonModule, BytesPipe, LineGraphComponent],
  template: `
    <div class="p-6 space-y-6">
      <!-- Header -->
      <div>
        <h1 class="text-2xl font-bold text-syslens-text-primary">Network</h1>
        <p class="text-syslens-text-secondary">Network adapters and configuration</p>
      </div>

      <!-- Network Adapters -->
      <section>
        <h2 class="section-title">Network Adapters</h2>
        <div class="grid gap-4">
          @for (adapter of adapters; track adapter.id) {
            <div class="card">
              <!-- Header -->
              <div class="flex items-start justify-between mb-4">
                <div class="flex items-start gap-3">
                  <div class="status-dot mt-2" [class.online]="adapter.status === 'Up'" [class.offline]="adapter.status !== 'Up'"></div>
                  <div>
                    <h3 class="font-semibold text-syslens-text-primary">{{ adapter.name }}</h3>
                    <p class="text-sm text-syslens-text-muted">{{ adapter.description }}</p>
                  </div>
                </div>
                <span class="px-2 py-1 text-xs rounded bg-syslens-bg-tertiary text-syslens-text-secondary">
                  {{ adapter.adapterType }}
                </span>
              </div>

              @if (adapter.status === 'Up') {
                <!-- Main content: Graph left, IP info right -->
                <div class="flex gap-6">
                  <!-- Left: Traffic Graph with stats -->
                  <div class="flex-1 min-w-0">
                    @if (adapterTrafficHistory[adapter.id]; as traffic) {
                      <!-- Speed indicators -->
                      <div class="flex items-center gap-6 mb-3">
                        <div class="flex items-center gap-2">
                          <span class="text-syslens-accent-green text-lg">↓</span>
                          <div>
                            <p class="font-mono text-lg font-bold text-syslens-accent-green" style="min-width: 9ch;">{{ traffic.downloadSpeed | bytes }}/s</p>
                            <p class="text-xs text-syslens-text-muted">Download</p>
                          </div>
                        </div>
                        <div class="flex items-center gap-2">
                          <span class="text-syslens-accent-blue text-lg">↑</span>
                          <div>
                            <p class="font-mono text-lg font-bold text-syslens-accent-blue" style="min-width: 9ch;">{{ traffic.uploadSpeed | bytes }}/s</p>
                            <p class="text-xs text-syslens-text-muted">Upload</p>
                          </div>
                        </div>
                      </div>
                      <!-- Graph -->
                      <div class="h-20 bg-syslens-bg-tertiary/30 rounded-lg p-2">
                        <app-line-graph
                          [series1]="traffic.downloadHistory"
                          [series2]="traffic.uploadHistory"
                          [maxValue]="traffic.maxSpeed"
                          [width]="400"
                          [height]="64"
                          series1Color="syslens-accent-green"
                          series2Color="syslens-accent-blue"
                        />
                      </div>
                      <!-- Total stats below graph -->
                      @if (adapterStats[adapter.id]; as stats) {
                        <div class="flex gap-6 mt-2 text-xs text-syslens-text-muted">
                          <span>Total Received: <span class="text-syslens-accent-green font-mono">{{ stats.bytesReceived | bytes }}</span></span>
                          <span>Total Sent: <span class="text-syslens-accent-blue font-mono">{{ stats.bytesSent | bytes }}</span></span>
                        </div>
                      }
                    } @else {
                      <div class="h-20 bg-syslens-bg-tertiary/30 rounded-lg flex items-center justify-center text-syslens-text-muted text-sm">
                        Loading traffic data...
                      </div>
                    }
                  </div>

                  <!-- Right: IP Configuration -->
                  <div class="w-64 space-y-2 border-l border-syslens-border-primary pl-6">
                    @if (adapter.ipv4Config) {
                      <div>
                        <p class="text-xs text-syslens-text-muted">IPv4 Address</p>
                        <p class="font-mono text-sm text-syslens-text-primary">{{ adapter.ipv4Config.address }}</p>
                      </div>
                      <div>
                        <p class="text-xs text-syslens-text-muted">Subnet Mask</p>
                        <p class="font-mono text-sm text-syslens-text-primary">{{ adapter.ipv4Config.subnetMask }}</p>
                      </div>
                      @if (adapter.ipv4Config.defaultGateway) {
                        <div>
                          <p class="text-xs text-syslens-text-muted">Gateway</p>
                          <p class="font-mono text-sm text-syslens-text-primary">{{ adapter.ipv4Config.defaultGateway }}</p>
                        </div>
                      }
                    }
                    <div>
                      <p class="text-xs text-syslens-text-muted">MAC Address</p>
                      <p class="font-mono text-sm text-syslens-text-primary">{{ adapter.macAddress }}</p>
                    </div>
                    @if (adapter.speedMbps) {
                      <div>
                        <p class="text-xs text-syslens-text-muted">Link Speed</p>
                        <p class="font-mono text-sm text-syslens-text-primary">
                          {{ adapter.speedMbps >= 1000 ? (adapter.speedMbps / 1000) + ' Gbps' : adapter.speedMbps + ' Mbps' }}
                        </p>
                      </div>
                    }
                    @if (adapter.dnsConfig.servers.length > 0) {
                      <div>
                        <p class="text-xs text-syslens-text-muted">DNS Servers</p>
                        @for (dns of adapter.dnsConfig.servers; track dns) {
                          <p class="font-mono text-sm text-syslens-text-primary">{{ dns }}</p>
                        }
                      </div>
                    }
                  </div>
                </div>
              } @else {
                <p class="text-syslens-text-muted text-sm">Adapter is not connected</p>
              }
            </div>
          } @empty {
            <div class="card text-center text-syslens-text-muted py-8">
              No network adapters found
            </div>
          }
        </div>
      </section>

      <!-- Active Connections -->
      <section>
        <h2 class="section-title">Active Connections</h2>
        <div class="card overflow-x-auto">
          <table class="w-full text-sm">
            <thead>
              <tr class="text-left text-syslens-text-muted border-b border-syslens-border-primary">
                <th class="pb-2 font-medium">Protocol</th>
                <th class="pb-2 font-medium">Local Address</th>
                <th class="pb-2 font-medium">Remote Address</th>
                <th class="pb-2 font-medium">State</th>
                <th class="pb-2 font-medium">Process</th>
              </tr>
            </thead>
            <tbody class="font-mono text-xs">
              @for (conn of connections.slice(0, 20); track $index) {
                <tr class="border-b border-syslens-border-primary last:border-0">
                  <td class="py-2 text-syslens-text-secondary">{{ conn.protocol }}</td>
                  <td class="py-2 text-syslens-text-primary">{{ conn.localAddress }}:{{ conn.localPort }}</td>
                  <td class="py-2 text-syslens-text-primary">{{ conn.remoteAddress }}:{{ conn.remotePort }}</td>
                  <td class="py-2">
                    <span class="px-2 py-0.5 rounded text-xs"
                          [class.bg-syslens-accent-green]="conn.state === 'Established'"
                          [class.text-white]="conn.state === 'Established'"
                          [class.bg-syslens-bg-tertiary]="conn.state !== 'Established'"
                          [class.text-syslens-text-secondary]="conn.state !== 'Established'">
                      {{ conn.state }}
                    </span>
                  </td>
                  <td class="py-2 text-syslens-text-muted">{{ conn.processName || 'Unknown' }} ({{ conn.pid }})</td>
                </tr>
              } @empty {
                <tr>
                  <td colspan="5" class="py-8 text-center text-syslens-text-muted">
                    No active connections
                  </td>
                </tr>
              }
            </tbody>
          </table>
          @if (connections.length > 20) {
            <p class="mt-2 text-xs text-syslens-text-muted text-center">
              Showing 20 of {{ connections.length }} connections
            </p>
          }
        </div>
      </section>

      <!-- Routing Table -->
      <section>
        <h2 class="section-title">Routing Table</h2>
        <div class="card overflow-x-auto">
          <table class="w-full text-sm">
            <thead>
              <tr class="text-left text-syslens-text-muted border-b border-syslens-border-primary">
                <th class="pb-2 font-medium">Destination</th>
                <th class="pb-2 font-medium">Netmask</th>
                <th class="pb-2 font-medium">Gateway</th>
                <th class="pb-2 font-medium">Metric</th>
                <th class="pb-2 font-medium">Type</th>
              </tr>
            </thead>
            <tbody class="font-mono text-xs">
              @for (route of routes; track $index) {
                <tr class="border-b border-syslens-border-primary last:border-0">
                  <td class="py-2 text-syslens-text-primary">{{ route.destination }}</td>
                  <td class="py-2 text-syslens-text-secondary">{{ route.netmask }}</td>
                  <td class="py-2 text-syslens-text-primary">{{ route.gateway }}</td>
                  <td class="py-2 text-syslens-text-secondary">{{ route.metric }}</td>
                  <td class="py-2 text-syslens-text-muted">{{ route.routeType }}</td>
                </tr>
              } @empty {
                <tr>
                  <td colspan="5" class="py-8 text-center text-syslens-text-muted">
                    No routes found
                  </td>
                </tr>
              }
            </tbody>
          </table>
        </div>
      </section>
    </div>
  `
})
export class NetworkComponent implements OnInit, OnDestroy {
  private networkService = inject(NetworkService);
  private statusService = inject(StatusService);
  private destroy$ = new Subject<void>();

  adapters: NetworkAdapter[] = [];
  adapterStats: Record<string, AdapterStats> = {};
  adapterTrafficHistory: Record<string, AdapterTrafficHistory> = {};
  connections: NetworkConnection[] = [];
  routes: Route[] = [];

  ngOnInit(): void {
    this.loadNetworkData();
    this.startRealtimeUpdates();
  }

  ngOnDestroy(): void {
    this.destroy$.next();
    this.destroy$.complete();
  }

  getGraphWidth(): number {
    // Responsive width based on container
    return Math.min(window.innerWidth - 100, 800);
  }

  private loadNetworkData(): void {
    this.statusService.startOperation('network-init', 'Loading network information...');

    this.networkService.getNetworkAdapters()
      .pipe(takeUntil(this.destroy$))
      .subscribe(adapters => {
        this.adapters = adapters;
        this.statusService.endOperation('network-init');

        // Initialize traffic history for each adapter with initial data points
        adapters.filter(a => a.status === 'Up').forEach(adapter => {
          // Pre-fill with MAX_HISTORY_POINTS zeros so array length is constant
          // This prevents point spacing from changing as data fills in
          const initialHistory = new Array(MAX_HISTORY_POINTS).fill(0);
          this.adapterTrafficHistory[adapter.id] = {
            downloadHistory: [...initialHistory],
            uploadHistory: [...initialHistory],
            downloadSpeed: 0,
            uploadSpeed: 0,
            maxSpeed: 1,
            previousStats: null
          };
        });

        // Start polling stats for active adapters
        this.startAdapterStatsPolling(adapters.filter(a => a.status === 'Up'));
      });

    this.networkService.getRoutingTable()
      .pipe(takeUntil(this.destroy$))
      .subscribe(routes => {
        this.routes = routes;
      });
  }

  private startAdapterStatsPolling(adapters: NetworkAdapter[]): void {
    // Poll every second for smooth graphs
    interval(1000).pipe(
      startWith(0),
      takeUntil(this.destroy$)
    ).subscribe(() => {
      adapters.forEach(adapter => {
        this.networkService.getAdapterStats(adapter.id)
          .pipe(takeUntil(this.destroy$))
          .subscribe(stats => {
            this.adapterStats[adapter.id] = stats;
            this.updateTrafficHistory(adapter.id, stats);
          });
      });
    });
  }

  private updateTrafficHistory(adapterId: string, stats: AdapterStats): void {
    const history = this.adapterTrafficHistory[adapterId];
    if (!history) return;

    const now = Date.now();

    if (history.previousStats) {
      const timeDeltaSeconds = (now - history.previousStats.timestamp) / 1000;

      if (timeDeltaSeconds > 0) {
        const downloadDelta = stats.bytesReceived - history.previousStats.bytesReceived;
        const uploadDelta = stats.bytesSent - history.previousStats.bytesSent;

        // Calculate speeds (handle counter resets)
        const downloadSpeed = downloadDelta >= 0 ? downloadDelta / timeDeltaSeconds : 0;
        const uploadSpeed = uploadDelta >= 0 ? uploadDelta / timeDeltaSeconds : 0;

        const newDownloadSpeed = Math.round(downloadSpeed);
        const newUploadSpeed = Math.round(uploadSpeed);

        // Create new arrays (triggers Angular change detection)
        const newDownloadHistory = [...history.downloadHistory, newDownloadSpeed];
        const newUploadHistory = [...history.uploadHistory, newUploadSpeed];

        // Trim to max points
        if (newDownloadHistory.length > MAX_HISTORY_POINTS) {
          newDownloadHistory.shift();
        }
        if (newUploadHistory.length > MAX_HISTORY_POINTS) {
          newUploadHistory.shift();
        }

        // Update max speed for graph scaling
        const maxDown = Math.max(...newDownloadHistory, 1);
        const maxUp = Math.max(...newUploadHistory, 1);

        // Create new history object to trigger change detection
        this.adapterTrafficHistory[adapterId] = {
          downloadHistory: newDownloadHistory,
          uploadHistory: newUploadHistory,
          downloadSpeed: newDownloadSpeed,
          uploadSpeed: newUploadSpeed,
          maxSpeed: Math.max(maxDown, maxUp),
          previousStats: {
            bytesReceived: stats.bytesReceived,
            bytesSent: stats.bytesSent,
            timestamp: now
          }
        };
        return;
      }
    }

    // Store current stats for next iteration (first poll)
    this.adapterTrafficHistory[adapterId] = {
      ...history,
      previousStats: {
        bytesReceived: stats.bytesReceived,
        bytesSent: stats.bytesSent,
        timestamp: now
      }
    };
  }

  private startRealtimeUpdates(): void {
    this.networkService.getActiveConnectionsPolling()
      .pipe(takeUntil(this.destroy$))
      .subscribe(connections => {
        this.connections = connections;
      });
  }
}
