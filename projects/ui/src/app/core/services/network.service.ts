import { Injectable, inject } from '@angular/core';
import {
  Observable,
  interval,
  switchMap,
  startWith,
  shareReplay,
  BehaviorSubject,
  of,
  tap,
  catchError,
} from 'rxjs';
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

  // BehaviorSubject for cache-first pattern
  private adapters$ = new BehaviorSubject<NetworkAdapter[] | null>(null);

  // Track if fresh data has been fetched
  private adaptersFetched = false;

  // Cached observable for polling
  private adaptersPolling$: Observable<NetworkAdapter[]> | null = null;

  constructor() {
    this.loadCachedData();
  }

  /**
   * Load cached network data on startup
   */
  private loadCachedData(): void {
    const adapters = this.cache.load<NetworkAdapter[]>(CacheKeys.NETWORK_ADAPTERS);
    if (adapters) this.adapters$.next(adapters);
  }

  /**
   * Get all network adapters with their configuration.
   * Returns cached data immediately, then fetches fresh data in background.
   */
  getNetworkAdapters(): Observable<NetworkAdapter[]> {
    if (!this.adaptersFetched) {
      this.adaptersFetched = true;
      this.tauri
        .invoke<NetworkAdapter[]>('get_network_adapters')
        .pipe(
          tap((data) => {
            this.cache.save(CacheKeys.NETWORK_ADAPTERS, data);
            this.adapters$.next(data);
          }),
          catchError((err) => {
            console.error('Failed to fetch network adapters:', err);
            return of(null);
          })
        )
        .subscribe();
    }

    return this.adapters$.asObservable().pipe(
      switchMap((data) =>
        data ? of(data) : this.tauri.invoke<NetworkAdapter[]>('get_network_adapters')
      ),
      shareReplay(1)
    );
  }

  /**
   * Get cached network adapters (refreshed every 30 seconds).
   */
  getNetworkAdaptersCached(): Observable<NetworkAdapter[]> {
    if (!this.adaptersPolling$) {
      this.adaptersPolling$ = interval(30000).pipe(
        startWith(0),
        switchMap(() => this.tauri.invoke<NetworkAdapter[]>('get_network_adapters')),
        tap((data) => {
          this.cache.save(CacheKeys.NETWORK_ADAPTERS, data);
          this.adapters$.next(data);
        }),
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
   * Clear cached network data (both in-memory and persistent).
   */
  clearCache(): void {
    // Reset fetch flag
    this.adaptersFetched = false;
    this.adaptersPolling$ = null;

    // Clear in-memory cache
    this.adapters$.next(null);

    // Clear persistent cache
    this.cache.clear(CacheKeys.NETWORK_ADAPTERS);
  }
}
