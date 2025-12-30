import { Injectable, inject, signal } from '@angular/core';
import { listen } from '@tauri-apps/api/event';

/**
 * Service for handling native application menu events from Tauri.
 * Listens to menu events emitted from the Rust backend.
 */
@Injectable({
  providedIn: 'root'
})
export class MenuService {
  // Sidebar visibility state
  private _sidebarVisible = signal(true);
  sidebarVisible = this._sidebarVisible.asReadonly();

  // About dialog state
  private _aboutDialogOpen = signal(false);
  aboutDialogOpen = this._aboutDialogOpen.asReadonly();

  // Refresh event counter (components can watch this to trigger refresh)
  private _refreshTrigger = signal(0);
  refreshTrigger = this._refreshTrigger.asReadonly();

  private isInitialized = false;

  /**
   * Initialize menu event listeners.
   * Safe to call multiple times - only initializes once.
   */
  async init(): Promise<void> {
    if (this.isInitialized) return;
    this.isInitialized = true;

    // Listen for toggle sidebar event
    await listen('menu:toggle-sidebar', () => {
      this._sidebarVisible.set(!this._sidebarVisible());
    });

    // Listen for refresh event
    await listen('menu:refresh', () => {
      this._refreshTrigger.set(this._refreshTrigger() + 1);
    });

    // Listen for about dialog event
    await listen('menu:about', () => {
      this._aboutDialogOpen.set(true);
    });
  }

  /**
   * Toggle sidebar visibility programmatically.
   */
  toggleSidebar(): void {
    this._sidebarVisible.set(!this._sidebarVisible());
  }

  /**
   * Show sidebar.
   */
  showSidebar(): void {
    this._sidebarVisible.set(true);
  }

  /**
   * Hide sidebar.
   */
  hideSidebar(): void {
    this._sidebarVisible.set(false);
  }

  /**
   * Close the about dialog.
   */
  closeAboutDialog(): void {
    this._aboutDialogOpen.set(false);
  }
}
