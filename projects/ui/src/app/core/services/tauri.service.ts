import { Injectable } from '@angular/core';
import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { Observable, from, Subject, shareReplay } from 'rxjs';

/**
 * Base service for Tauri IPC communication.
 * Provides methods to invoke Tauri commands and listen to events.
 */
@Injectable({
  providedIn: 'root'
})
export class TauriService {
  private eventListeners = new Map<string, UnlistenFn>();

  /**
   * Invoke a Tauri command with optional arguments.
   * @param command - The command name to invoke
   * @param args - Optional arguments to pass to the command
   * @returns Observable of the command result
   */
  invoke<T>(command: string, args?: Record<string, unknown>): Observable<T> {
    return from(invoke<T>(command, args));
  }

  /**
   * Listen to a Tauri event and return an Observable.
   * @param event - The event name to listen to
   * @returns Observable that emits event payloads
   */
  listen<T>(event: string): Observable<T> {
    const subject = new Subject<T>();

    listen<T>(event, (e) => {
      subject.next(e.payload);
    }).then((unlisten) => {
      // Store the unlisten function for cleanup
      const existing = this.eventListeners.get(event);
      if (existing) {
        existing();
      }
      this.eventListeners.set(event, unlisten);
    });

    return subject.asObservable().pipe(shareReplay(1));
  }

  /**
   * Stop listening to a specific event.
   * @param event - The event name to stop listening to
   */
  unlisten(event: string): void {
    const unlisten = this.eventListeners.get(event);
    if (unlisten) {
      unlisten();
      this.eventListeners.delete(event);
    }
  }

  /**
   * Clean up all event listeners.
   */
  cleanup(): void {
    this.eventListeners.forEach((unlisten) => unlisten());
    this.eventListeners.clear();
  }
}
