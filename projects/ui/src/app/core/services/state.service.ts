import { Injectable, inject } from '@angular/core';
import { Router, NavigationEnd } from '@angular/router';
import { filter } from 'rxjs/operators';

const STORAGE_KEY = 'syslens_last_route';

/**
 * Service for persisting and restoring application state.
 * Saves the last active route and restores it on app restart.
 */
@Injectable({
  providedIn: 'root'
})
export class StateService {
  private router = inject(Router);

  /**
   * Initialize state persistence.
   * Call this once during app initialization.
   */
  initialize(): void {
    this.listenForRouteChanges();
  }

  /**
   * Get the last saved route, or default to 'dashboard'.
   */
  getLastRoute(): string {
    try {
      return localStorage.getItem(STORAGE_KEY) || 'dashboard';
    } catch {
      return 'dashboard';
    }
  }

  /**
   * Navigate to the last saved route.
   * Returns true if navigation was triggered.
   */
  restoreLastRoute(): boolean {
    const lastRoute = this.getLastRoute();
    if (lastRoute && lastRoute !== 'dashboard') {
      this.router.navigate([lastRoute]);
      return true;
    }
    return false;
  }

  /**
   * Listen for route changes and persist the current route.
   */
  private listenForRouteChanges(): void {
    this.router.events.pipe(
      filter((event): event is NavigationEnd => event instanceof NavigationEnd)
    ).subscribe(event => {
      this.saveRoute(event.urlAfterRedirects);
    });
  }

  /**
   * Save the current route to localStorage.
   */
  private saveRoute(url: string): void {
    try {
      // Extract the route path (remove leading slash)
      const route = url.replace(/^\//, '').split('?')[0];
      if (route) {
        localStorage.setItem(STORAGE_KEY, route);
      }
    } catch {
      // localStorage may not be available in some contexts
    }
  }
}
