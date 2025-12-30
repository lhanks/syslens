import { Injectable, inject } from '@angular/core';
import { Observable, interval, switchMap, startWith, shareReplay, of, tap, concat } from 'rxjs';
import { TauriService } from './tauri.service';
import { DataCacheService, CacheKeys } from './data-cache.service';
import { NetworkAdapter, AdapterStats, NetworkConnection, Route } from '../models/network.model';

/**
 * Service for retrieving network configuration and statistics.
 * Uses cache-first pattern: returns cached data immediately, then refreshes in background.
 */
@Injectable({
  providedIn: 'root',
})
export class NetworkService {
  private tauri = inject(TauriService);
  private cache = inject(DataCacheService);

  // Cached observable for static data
  private adaptersCache$: Observable<NetworkAdapter[]> | null = null;

  // Cached observable for polling
  private adaptersPolling$: Observable<NetworkAdapter[]> | null = null;

  /**
   * Get all network adapters with their configuration.
   * Returns cached data immediately if available, then fetches fresh data.
   */
  getNetworkAdapters(): Observable<NetworkAdapter[]> {
    if (!this.adaptersCache$) {
      const cached = this.cache.load<NetworkAdapter[]>(CacheKeys.NETWORK_ADAPTERS);
      const fetch$ = this.tauri.invoke<NetworkAdapter[]>('get_network_adapters').pipe(
        tap((data) => this.cache.save(CacheKeys.NETWORK_ADAPTERS, data))
      );

      this.adaptersCache$ = cached
        ? concat(of(cached), fetch$).pipe(shareReplay(1))
        : fetch$.pipe(shareReplay(1));
    }
    return this.adaptersCache$;
  }

  /**
   * Get cached network adapters (refreshed every 30 seconds).
   */
  getNetworkAdaptersCached(): Observable<NetworkAdapter[]> {
    if (!this.adaptersPolling$) {
      this.adaptersPolling$ = interval(30000).pipe(
        startWith(0),
        switchMap(() => this.tauri.invoke<NetworkAdapter[]>('get_network_adapters')),
        tap((data) => this.cache.save(CacheKeys.NETWORK_ADAPTERS, data)),
        shareReplay(1)
      );
    }
    return this.adaptersPolling$;
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

  /**
   * Enable or disable a network adapter.
   * Requires administrator privileges on Windows.
   * @param adapterName - The adapter friendly name
   * @param enabled - Whether to enable (true) or disable (false) the adapter
   */
  setAdapterEnabled(adapterName: string, enabled: boolean): Observable<boolean> {
    return this.tauri.invoke<boolean>('set_adapter_enabled', { adapterName, enabled });
  }

  /**
   * Clear cached network data (both in-memory and persistent).
   */
  clearCache(): void {
    // Clear observable cache
    this.adaptersCache$ = null;
    this.adaptersPolling$ = null;

    // Clear persistent cache
    this.cache.clear(CacheKeys.NETWORK_ADAPTERS);
  }
}
