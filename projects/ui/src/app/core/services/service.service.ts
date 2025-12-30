import { Injectable, inject } from '@angular/core';
import { Observable, shareReplay } from 'rxjs';
import { TauriService } from './tauri.service';
import { ServiceInfo, ServiceSummary } from '../models/service.model';

/**
 * Service for retrieving Windows service information.
 */
@Injectable({
  providedIn: 'root',
})
export class ServiceService {
  private tauri = inject(TauriService);

  // Cached observable for services list
  private servicesCache$: Observable<ServiceInfo[]> | null = null;

  /**
   * Get all Windows services.
   */
  getServices(): Observable<ServiceInfo[]> {
    if (!this.servicesCache$) {
      this.servicesCache$ = this.tauri.invoke<ServiceInfo[]>('get_services').pipe(
        shareReplay(1)
      );
    }
    return this.servicesCache$;
  }

  /**
   * Get services with fresh data (bypasses cache).
   */
  getServicesFresh(): Observable<ServiceInfo[]> {
    this.servicesCache$ = null;
    return this.getServices();
  }

  /**
   * Get service summary statistics.
   */
  getServiceSummary(): Observable<ServiceSummary> {
    return this.tauri.invoke<ServiceSummary>('get_service_summary');
  }

  /**
   * Clear cached service data.
   */
  clearCache(): void {
    this.servicesCache$ = null;
  }
}
