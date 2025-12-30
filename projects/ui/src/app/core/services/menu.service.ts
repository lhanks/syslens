import { Injectable, inject, signal, computed } from '@angular/core';
import { listen } from '@tauri-apps/api/event';
import { ViewSettingsService } from './view-settings.service';

/**
 * Service for handling native application menu events from Tauri.
 * Listens to menu events emitted from the Rust backend.
 */
@Injectable({
  providedIn: 'root'
})
export class MenuService {
  private viewSettings = inject(ViewSettingsService);

  // Sidebar visibility - now delegates to ViewSettingsService
  sidebarVisible = computed(() => this.viewSettings.rightSidebarVisible());

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

    // Listen for toggle sidebar events
    await listen('menu:toggle-sidebar', () => {
      this.viewSettings.toggleRightSidebar();
    });
    await listen('menu:toggle-left-sidebar', () => {
      this.viewSettings.toggleLeftSidebar();
    });

    // Listen for mini graph visibility toggles
    await listen('menu:toggle-cpu', () => {
      this.viewSettings.toggleMiniGraph('cpu');
    });
    await listen('menu:toggle-memory', () => {
      this.viewSettings.toggleMiniGraph('memory');
    });
    await listen('menu:toggle-disk', () => {
      this.viewSettings.toggleMiniGraph('disk');
    });
    await listen('menu:toggle-gpu', () => {
      this.viewSettings.toggleMiniGraph('gpu');
    });
    await listen('menu:toggle-network', () => {
      this.viewSettings.toggleMiniGraph('network');
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
    this.viewSettings.toggleRightSidebar();
  }

  /**
   * Show sidebar.
   */
  showSidebar(): void {
    this.viewSettings.setRightSidebarVisible(true);
  }

  /**
   * Hide sidebar.
   */
  hideSidebar(): void {
    this.viewSettings.setRightSidebarVisible(false);
  }

  /**
   * Close the about dialog.
   */
  closeAboutDialog(): void {
    this._aboutDialogOpen.set(false);
  }
}
