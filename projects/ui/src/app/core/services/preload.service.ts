import { Injectable, inject } from '@angular/core';
import { Router, NavigationEnd } from '@angular/router';
import { Observable, forkJoin, filter, delay, of, catchError, Subject, takeUntil, Subscription } from 'rxjs';

import { HardwareService } from './hardware.service';
import { SystemService } from './system.service';
import { NetworkService } from './network.service';
import { StorageService } from './storage.service';

/**
 * Preload priority levels for data loading.
 */
export enum PreloadPriority {
  HIGH = 0,      // Current view - immediate
  MEDIUM = 1,    // Related views - after current loads
  LOW = 2        // Background preload - with delay
}

/**
 * Service for progressive data loading.
 * - Loads current view data first
 * - Preloads other views in the background
 * - Prioritizes user-clicked tabs
 */
@Injectable({
  providedIn: 'root'
})
export class PreloadService {
  private router = inject(Router);
  private hardwareService = inject(HardwareService);
  private systemService = inject(SystemService);
  private networkService = inject(NetworkService);
  private storageService = inject(StorageService);

  private preloadedViews = new Set<string>();
  private preloadCancel$ = new Subject<void>();
  private isPreloading = false;
  private currentSubscriptions: Subscription[] = [];

  constructor() {
    // Listen for navigation events to manage preloading
    this.router.events.pipe(
      filter((event): event is NavigationEnd => event instanceof NavigationEnd)
    ).subscribe((event) => {
      this.onNavigate(event.urlAfterRedirects);
    });
  }

  /**
   * Called when navigation occurs.
   * Cancels pending preloads and starts preloading for new route.
   */
  private onNavigate(url: string): void {
    // Cancel any pending background preloads
    this.cancelPendingPreloads();

    // Mark current view as loaded
    const view = this.extractViewName(url);
    if (view) {
      this.preloadedViews.add(view);
    }

    // Start background preloading after a short delay
    this.startBackgroundPreload(view);
  }

  /**
   * Extract view name from URL.
   */
  private extractViewName(url: string): string {
    const match = url.match(/^\/?([^?#/]+)/);
    return match?.[1] || 'dashboard';
  }

  /**
   * Cancel pending background preloads.
   * Called when user navigates to prioritize their action.
   */
  cancelPendingPreloads(): void {
    this.preloadCancel$.next();
    this.currentSubscriptions.forEach(sub => sub.unsubscribe());
    this.currentSubscriptions = [];
    this.isPreloading = false;
  }

  /**
   * Start background preloading of other views.
   * Uses a delay to avoid interfering with current view loading.
   */
  private startBackgroundPreload(currentView: string): void {
    if (this.isPreloading) return;

    const viewsToPreload = this.getViewsToPreload(currentView);
    if (viewsToPreload.length === 0) return;

    this.isPreloading = true;

    // Wait 500ms before starting background preload
    const sub = of(null).pipe(
      delay(500),
      takeUntil(this.preloadCancel$)
    ).subscribe(() => {
      this.preloadViews(viewsToPreload);
    });
    this.currentSubscriptions.push(sub);
  }

  /**
   * Get list of views to preload based on priority.
   */
  private getViewsToPreload(currentView: string): string[] {
    const allViews = ['dashboard', 'hardware', 'storage', 'network', 'system'];

    // Filter out already preloaded views
    const notPreloaded = allViews.filter(v => !this.preloadedViews.has(v));

    // Prioritize related views
    const priority: Record<string, string[]> = {
      'dashboard': ['hardware', 'storage', 'network', 'system'],
      'hardware': ['dashboard', 'system', 'storage', 'network'],
      'storage': ['dashboard', 'hardware', 'system', 'network'],
      'network': ['dashboard', 'system', 'hardware', 'storage'],
      'system': ['dashboard', 'hardware', 'network', 'storage']
    };

    const orderedViews = priority[currentView] || allViews;
    return orderedViews.filter(v => notPreloaded.includes(v));
  }

  /**
   * Preload data for specified views in sequence.
   */
  private preloadViews(views: string[]): void {
    views.forEach((view, index) => {
      // Stagger preloads by 200ms each
      const sub = of(null).pipe(
        delay(index * 200),
        takeUntil(this.preloadCancel$)
      ).subscribe(() => {
        this.preloadViewData(view);
      });
      this.currentSubscriptions.push(sub);
    });
  }

  /**
   * Preload static data for a specific view.
   */
  private preloadViewData(view: string): void {
    if (this.preloadedViews.has(view)) return;

    const preloadObs = this.getPreloadObservable(view);
    if (preloadObs) {
      const sub = preloadObs.pipe(
        catchError(() => of(null))
      ).subscribe(() => {
        this.preloadedViews.add(view);
      });
      this.currentSubscriptions.push(sub);
    }
  }

  /**
   * Get the preload observable for a specific view.
   * Only preloads static data, not real-time metrics.
   */
  private getPreloadObservable(view: string): Observable<unknown> | null {
    switch (view) {
      case 'dashboard':
        return forkJoin([
          this.hardwareService.getCpuInfo(),
          this.hardwareService.getMemoryInfo(),
          this.systemService.getDeviceInfo(),
          this.systemService.getOsInfo()
        ]);

      case 'hardware':
        return forkJoin([
          this.hardwareService.getCpuInfo(),
          this.hardwareService.getMemoryInfo(),
          this.hardwareService.getGpuInfo(),
          this.hardwareService.getMotherboardInfo(),
          this.hardwareService.getMonitors()
        ]);

      case 'storage':
        return forkJoin([
          this.storageService.getPhysicalDisks(),
          this.storageService.getVolumes(),
          this.storageService.getNetworkDrives()
        ]);

      case 'network':
        return forkJoin([
          this.networkService.getNetworkAdapters(),
          this.networkService.getRoutingTable()
        ]);

      case 'system':
        return forkJoin([
          this.systemService.getDeviceInfo(),
          this.systemService.getOsInfo(),
          this.systemService.getBiosInfo()
        ]);

      default:
        return null;
    }
  }

  /**
   * Check if a view's data has been preloaded.
   */
  isViewPreloaded(view: string): boolean {
    return this.preloadedViews.has(view);
  }

  /**
   * Force preload a specific view with high priority.
   * Used when user hovers over navigation.
   */
  priorityPreload(view: string): void {
    if (this.preloadedViews.has(view)) return;

    // Cancel other preloads and immediately preload this view
    this.cancelPendingPreloads();
    this.preloadViewData(view);
  }

  /**
   * Clear all preloaded data caches.
   */
  clearPreloadCache(): void {
    this.preloadedViews.clear();
    this.hardwareService.clearCache();
    this.systemService.clearCache();
  }
}
