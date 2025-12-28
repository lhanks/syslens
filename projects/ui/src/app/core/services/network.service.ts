import { Injectable, inject } from '@angular/core';
import { Observable, interval, switchMap, startWith, shareReplay } from 'rxjs';
import { TauriService } from './tauri.service';
import {
  NetworkAdapter,
  AdapterStats,
  NetworkConnection,
  Route
} from '../models/network.model';

/**
 * Service for retrieving network configuration and statistics.
 */
@Injectable({
  providedIn: 'root'
})
export class NetworkService {
  private tauri = inject(TauriService);

  // Cached observables for real-time data
  private adaptersCache$: Observable<NetworkAdapter[]> | null = null;

  /**
   * Get all network adapters with their configuration.
   */
  getNetworkAdapters(): Observable<NetworkAdapter[]> {
    return this.tauri.invoke<NetworkAdapter[]>('get_network_adapters');
  }

  /**
   * Get cached network adapters (refreshed every 30 seconds).
   */
  getNetworkAdaptersCached(): Observable<NetworkAdapter[]> {
    if (!this.adaptersCache$) {
      this.adaptersCache$ = interval(30000).pipe(
        startWith(0),
        switchMap(() => this.getNetworkAdapters()),
        shareReplay(1)
      );
    }
    return this.adaptersCache$;
  }

  /**
   * Get real-time statistics for a specific adapter.
   * @param adapterId - The adapter identifier
   */
  getAdapterStats(adapterId: string): Observable<AdapterStats> {
    return this.tauri.invoke<AdapterStats>('get_adapter_stats', { adapterId });
  }

  /**
   * Get adapter statistics with polling (every second).
   * @param adapterId - The adapter identifier
   */
  getAdapterStatsPolling(adapterId: string): Observable<AdapterStats> {
    return interval(1000).pipe(
      startWith(0),
      switchMap(() => this.getAdapterStats(adapterId))
    );
  }

  /**
   * Get all active network connections.
   */
  getActiveConnections(): Observable<NetworkConnection[]> {
    return this.tauri.invoke<NetworkConnection[]>('get_active_connections');
  }

  /**
   * Get active connections with polling (every 5 seconds).
   */
  getActiveConnectionsPolling(): Observable<NetworkConnection[]> {
    return interval(5000).pipe(
      startWith(0),
      switchMap(() => this.getActiveConnections()),
      shareReplay(1)
    );
  }

  /**
   * Get the system routing table.
   */
  getRoutingTable(): Observable<Route[]> {
    return this.tauri.invoke<Route[]>('get_routing_table');
  }

  /**
   * Listen for network adapter change events.
   */
  onAdapterChange(): Observable<NetworkAdapter[]> {
    return this.tauri.listen<NetworkAdapter[]>('network-adapter-changed');
  }
}
