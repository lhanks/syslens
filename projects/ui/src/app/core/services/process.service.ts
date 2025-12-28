import { Injectable, inject } from '@angular/core';
import { Observable, interval, switchMap, startWith } from 'rxjs';
import { TauriService } from './tauri.service';
import { ProcessInfo, ProcessSummary } from '../models/process.model';

/**
 * Service for retrieving process information.
 */
@Injectable({
  providedIn: 'root'
})
export class ProcessService {
  private tauri = inject(TauriService);

  /**
   * Get list of all running processes.
   */
  getProcesses(): Observable<ProcessInfo[]> {
    return this.tauri.invoke<ProcessInfo[]>('get_processes');
  }

  /**
   * Get process summary statistics.
   */
  getProcessSummary(): Observable<ProcessSummary> {
    return this.tauri.invoke<ProcessSummary>('get_process_summary');
  }

  /**
   * Poll for process list updates.
   * @param intervalMs Polling interval in milliseconds (default: 2000)
   */
  getProcessesPolling(intervalMs = 2000): Observable<ProcessInfo[]> {
    return interval(intervalMs).pipe(
      startWith(0),
      switchMap(() => this.getProcesses())
    );
  }

  /**
   * Poll for process summary updates.
   * @param intervalMs Polling interval in milliseconds (default: 2000)
   */
  getProcessSummaryPolling(intervalMs = 2000): Observable<ProcessSummary> {
    return interval(intervalMs).pipe(
      startWith(0),
      switchMap(() => this.getProcessSummary())
    );
  }
}
