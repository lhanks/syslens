import { Injectable, signal, effect } from '@angular/core';

/**
 * Sidebar dock position
 */
export type SidebarPosition = 'left' | 'right';

/**
 * View settings interface
 */
export interface ViewSettings {
  // Left sidebar settings
  leftSidebarVisible: boolean;
  leftSidebarWidth: number;
  leftSidebarDetached: boolean;

  // Right sidebar settings
  rightSidebarVisible: boolean;
  rightSidebarPosition: SidebarPosition;
  rightSidebarWidth: number;

  // Left sidebar resource visibility (CPU/MEM/DISK/NETWORK)
  showCpuMiniGraph: boolean;
  showMemoryMiniGraph: boolean;
  showDiskMiniGraph: boolean;
  showGpuMiniGraph: boolean;
  showNetworkMiniGraph: boolean;
}

const STORAGE_KEY = 'syslens_view_settings';

const DEFAULT_SETTINGS: ViewSettings = {
  leftSidebarVisible: true,
  leftSidebarWidth: 256, // 16rem = 256px (w-64)
  leftSidebarDetached: false,

  rightSidebarVisible: true,
  rightSidebarPosition: 'right',
  rightSidebarWidth: 288, // 18rem = 288px

  showCpuMiniGraph: true,
  showMemoryMiniGraph: true,
  showDiskMiniGraph: true,
  showGpuMiniGraph: true,
  showNetworkMiniGraph: true,
};

/**
 * Service for persisting and managing view/layout settings.
 * Saves settings to localStorage and provides reactive signals.
 */
@Injectable({
  providedIn: 'root'
})
export class ViewSettingsService {
  // Reactive signals for each setting
  private _leftSidebarVisible = signal(DEFAULT_SETTINGS.leftSidebarVisible);
  private _leftSidebarWidth = signal(DEFAULT_SETTINGS.leftSidebarWidth);
  private _leftSidebarDetached = signal(DEFAULT_SETTINGS.leftSidebarDetached);

  private _rightSidebarVisible = signal(DEFAULT_SETTINGS.rightSidebarVisible);
  private _rightSidebarPosition = signal<SidebarPosition>(DEFAULT_SETTINGS.rightSidebarPosition);
  private _rightSidebarWidth = signal(DEFAULT_SETTINGS.rightSidebarWidth);

  private _showCpuMiniGraph = signal(DEFAULT_SETTINGS.showCpuMiniGraph);
  private _showMemoryMiniGraph = signal(DEFAULT_SETTINGS.showMemoryMiniGraph);
  private _showDiskMiniGraph = signal(DEFAULT_SETTINGS.showDiskMiniGraph);
  private _showGpuMiniGraph = signal(DEFAULT_SETTINGS.showGpuMiniGraph);
  private _showNetworkMiniGraph = signal(DEFAULT_SETTINGS.showNetworkMiniGraph);

  // Public read-only signals
  leftSidebarVisible = this._leftSidebarVisible.asReadonly();
  leftSidebarWidth = this._leftSidebarWidth.asReadonly();
  leftSidebarDetached = this._leftSidebarDetached.asReadonly();

  rightSidebarVisible = this._rightSidebarVisible.asReadonly();
  rightSidebarPosition = this._rightSidebarPosition.asReadonly();
  rightSidebarWidth = this._rightSidebarWidth.asReadonly();

  showCpuMiniGraph = this._showCpuMiniGraph.asReadonly();
  showMemoryMiniGraph = this._showMemoryMiniGraph.asReadonly();
  showDiskMiniGraph = this._showDiskMiniGraph.asReadonly();
  showGpuMiniGraph = this._showGpuMiniGraph.asReadonly();
  showNetworkMiniGraph = this._showNetworkMiniGraph.asReadonly();

  constructor() {
    this.loadSettings();

    // Auto-save settings when any signal changes
    effect(() => {
      this.saveSettings();
    });
  }

  /**
   * Toggle left sidebar visibility
   */
  toggleLeftSidebar(): void {
    this._leftSidebarVisible.set(!this._leftSidebarVisible());
  }

  /**
   * Set left sidebar visibility
   */
  setLeftSidebarVisible(visible: boolean): void {
    this._leftSidebarVisible.set(visible);
  }

  /**
   * Set left sidebar width
   */
  setLeftSidebarWidth(width: number): void {
    // Clamp width between 180 and 400
    this._leftSidebarWidth.set(Math.max(180, Math.min(400, width)));
  }

  /**
   * Set left sidebar detached state
   */
  setLeftSidebarDetached(detached: boolean): void {
    this._leftSidebarDetached.set(detached);
  }

  /**
   * Detach left sidebar to floating window
   */
  async detachLeftSidebar(): Promise<void> {
    // Import dynamically to avoid issues in browser-only mode
    const { WebviewWindow } = await import('@tauri-apps/api/webviewWindow');

    // Check if window already exists
    const existingWindow = await WebviewWindow.getByLabel('sidebar-float');
    if (existingWindow) {
      await existingWindow.setFocus();
      return;
    }

    // Create new floating window
    const floatingWindow = new WebviewWindow('sidebar-float', {
      url: '/floating-sidebar',
      title: 'Performance Monitor',
      width: 280,
      height: 400,
      minWidth: 200,
      minHeight: 300,
      decorations: false,
      transparent: false,
      alwaysOnTop: true,
      resizable: true,
      x: 100,
      y: 100,
    });

    // Listen for window creation
    floatingWindow.once('tauri://created', () => {
      this._leftSidebarDetached.set(true);
    });

    floatingWindow.once('tauri://error', (e) => {
      console.error('Failed to create floating sidebar window:', e);
    });

    // Listen for window close to re-dock
    floatingWindow.once('tauri://close-requested', () => {
      this._leftSidebarDetached.set(false);
    });
  }

  /**
   * Toggle right sidebar visibility
   */
  toggleRightSidebar(): void {
    this._rightSidebarVisible.set(!this._rightSidebarVisible());
  }

  /**
   * Set right sidebar visibility
   */
  setRightSidebarVisible(visible: boolean): void {
    this._rightSidebarVisible.set(visible);
  }

  /**
   * Set right sidebar position (dock side)
   */
  setRightSidebarPosition(position: SidebarPosition): void {
    this._rightSidebarPosition.set(position);
  }

  /**
   * Set right sidebar width
   */
  setRightSidebarWidth(width: number): void {
    // Clamp width between 200 and 500
    this._rightSidebarWidth.set(Math.max(200, Math.min(500, width)));
  }

  /**
   * Toggle a specific mini graph visibility
   */
  toggleMiniGraph(type: 'cpu' | 'memory' | 'disk' | 'gpu' | 'network'): void {
    switch (type) {
      case 'cpu':
        this._showCpuMiniGraph.set(!this._showCpuMiniGraph());
        break;
      case 'memory':
        this._showMemoryMiniGraph.set(!this._showMemoryMiniGraph());
        break;
      case 'disk':
        this._showDiskMiniGraph.set(!this._showDiskMiniGraph());
        break;
      case 'gpu':
        this._showGpuMiniGraph.set(!this._showGpuMiniGraph());
        break;
      case 'network':
        this._showNetworkMiniGraph.set(!this._showNetworkMiniGraph());
        break;
    }
  }

  /**
   * Set a specific mini graph visibility
   */
  setMiniGraphVisible(type: 'cpu' | 'memory' | 'disk' | 'gpu' | 'network', visible: boolean): void {
    switch (type) {
      case 'cpu':
        this._showCpuMiniGraph.set(visible);
        break;
      case 'memory':
        this._showMemoryMiniGraph.set(visible);
        break;
      case 'disk':
        this._showDiskMiniGraph.set(visible);
        break;
      case 'gpu':
        this._showGpuMiniGraph.set(visible);
        break;
      case 'network':
        this._showNetworkMiniGraph.set(visible);
        break;
    }
  }

  /**
   * Load settings from localStorage
   */
  private loadSettings(): void {
    try {
      const stored = localStorage.getItem(STORAGE_KEY);
      if (stored) {
        const settings: Partial<ViewSettings> = JSON.parse(stored);

        if (settings.leftSidebarVisible !== undefined) {
          this._leftSidebarVisible.set(settings.leftSidebarVisible);
        }
        if (settings.leftSidebarWidth !== undefined) {
          this._leftSidebarWidth.set(settings.leftSidebarWidth);
        }
        if (settings.leftSidebarDetached !== undefined) {
          this._leftSidebarDetached.set(settings.leftSidebarDetached);
        }
        if (settings.rightSidebarVisible !== undefined) {
          this._rightSidebarVisible.set(settings.rightSidebarVisible);
        }
        if (settings.rightSidebarPosition !== undefined) {
          this._rightSidebarPosition.set(settings.rightSidebarPosition);
        }
        if (settings.rightSidebarWidth !== undefined) {
          this._rightSidebarWidth.set(settings.rightSidebarWidth);
        }
        if (settings.showCpuMiniGraph !== undefined) {
          this._showCpuMiniGraph.set(settings.showCpuMiniGraph);
        }
        if (settings.showMemoryMiniGraph !== undefined) {
          this._showMemoryMiniGraph.set(settings.showMemoryMiniGraph);
        }
        if (settings.showDiskMiniGraph !== undefined) {
          this._showDiskMiniGraph.set(settings.showDiskMiniGraph);
        }
        if (settings.showGpuMiniGraph !== undefined) {
          this._showGpuMiniGraph.set(settings.showGpuMiniGraph);
        }
        if (settings.showNetworkMiniGraph !== undefined) {
          this._showNetworkMiniGraph.set(settings.showNetworkMiniGraph);
        }
      }
    } catch {
      // Use defaults if localStorage isn't available or parse fails
    }
  }

  /**
   * Save current settings to localStorage
   */
  private saveSettings(): void {
    try {
      const settings: ViewSettings = {
        leftSidebarVisible: this._leftSidebarVisible(),
        leftSidebarWidth: this._leftSidebarWidth(),
        leftSidebarDetached: this._leftSidebarDetached(),
        rightSidebarVisible: this._rightSidebarVisible(),
        rightSidebarPosition: this._rightSidebarPosition(),
        rightSidebarWidth: this._rightSidebarWidth(),
        showCpuMiniGraph: this._showCpuMiniGraph(),
        showMemoryMiniGraph: this._showMemoryMiniGraph(),
        showDiskMiniGraph: this._showDiskMiniGraph(),
        showGpuMiniGraph: this._showGpuMiniGraph(),
        showNetworkMiniGraph: this._showNetworkMiniGraph(),
      };
      localStorage.setItem(STORAGE_KEY, JSON.stringify(settings));
    } catch {
      // Ignore localStorage errors
    }
  }
}
