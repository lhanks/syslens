import { Component, OnInit, OnDestroy, inject } from '@angular/core';
import { CommonModule } from '@angular/common';
import { Subject, takeUntil } from 'rxjs';

import { NetworkService } from '@core/services';
import { NetworkAdapter, AdapterStats, NetworkConnection, Route } from '@core/models';
import { BytesPipe } from '@shared/pipes';

@Component({
  selector: 'app-network',
  standalone: true,
  imports: [CommonModule, BytesPipe],
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
              <div class="flex items-start justify-between">
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
                <div class="mt-4 grid grid-cols-2 md:grid-cols-4 gap-4">
                  <!-- IPv4 -->
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

                  <!-- MAC Address -->
                  <div>
                    <p class="text-xs text-syslens-text-muted">MAC Address</p>
                    <p class="font-mono text-sm text-syslens-text-primary">{{ adapter.macAddress }}</p>
                  </div>

                  <!-- Speed -->
                  @if (adapter.speedMbps) {
                    <div>
                      <p class="text-xs text-syslens-text-muted">Speed</p>
                      <p class="font-mono text-sm text-syslens-text-primary">
                        {{ adapter.speedMbps >= 1000 ? (adapter.speedMbps / 1000) + ' Gbps' : adapter.speedMbps + ' Mbps' }}
                      </p>
                    </div>
                  }
                </div>

                <!-- DNS Servers -->
                @if (adapter.dnsConfig.servers.length > 0) {
                  <div class="mt-4">
                    <p class="text-xs text-syslens-text-muted mb-1">DNS Servers</p>
                    <div class="flex flex-wrap gap-2">
                      @for (dns of adapter.dnsConfig.servers; track dns) {
                        <span class="px-2 py-1 text-xs font-mono rounded bg-syslens-bg-tertiary text-syslens-text-secondary">
                          {{ dns }}
                        </span>
                      }
                    </div>
                  </div>
                }

                <!-- Real-time Stats -->
                @if (adapterStats[adapter.id]; as stats) {
                  <div class="mt-4 pt-4 border-t border-syslens-border-primary">
                    <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
                      <div>
                        <p class="text-xs text-syslens-text-muted">Received</p>
                        <p class="font-mono text-sm text-syslens-accent-green">{{ stats.bytesReceived | bytes }}</p>
                      </div>
                      <div>
                        <p class="text-xs text-syslens-text-muted">Sent</p>
                        <p class="font-mono text-sm text-syslens-accent-blue">{{ stats.bytesSent | bytes }}</p>
                      </div>
                      <div>
                        <p class="text-xs text-syslens-text-muted">Packets In</p>
                        <p class="font-mono text-sm text-syslens-text-primary">{{ stats.packetsReceived | number }}</p>
                      </div>
                      <div>
                        <p class="text-xs text-syslens-text-muted">Packets Out</p>
                        <p class="font-mono text-sm text-syslens-text-primary">{{ stats.packetsSent | number }}</p>
                      </div>
                    </div>
                  </div>
                }
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
  private destroy$ = new Subject<void>();

  adapters: NetworkAdapter[] = [];
  adapterStats: Record<string, AdapterStats> = {};
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

  private loadNetworkData(): void {
    this.networkService.getNetworkAdapters()
      .pipe(takeUntil(this.destroy$))
      .subscribe(adapters => {
        this.adapters = adapters;
        // Start polling stats for active adapters
        adapters.filter(a => a.status === 'Up').forEach(adapter => {
          this.networkService.getAdapterStatsPolling(adapter.id)
            .pipe(takeUntil(this.destroy$))
            .subscribe(stats => {
              this.adapterStats[adapter.id] = stats;
            });
        });
      });

    this.networkService.getRoutingTable()
      .pipe(takeUntil(this.destroy$))
      .subscribe(routes => {
        this.routes = routes;
      });
  }

  private startRealtimeUpdates(): void {
    this.networkService.getActiveConnectionsPolling()
      .pipe(takeUntil(this.destroy$))
      .subscribe(connections => {
        this.connections = connections;
      });
  }
}
