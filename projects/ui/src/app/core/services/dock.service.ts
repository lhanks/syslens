import { Injectable, signal, computed, effect } from '@angular/core';
import {
  DockLayout,
  DockRegion,
  DockRegionPosition,
  DockPanel,
  DockDragState,
  DockPanelType,
  createDefaultLayout,
  DOCK_DEFAULTS,
} from '../../shared/components/dock/dock.model';

const STORAGE_KEY = 'syslens_dock_layout';
const DETACHED_PANELS_KEY = 'syslens_detached_panels';

/** Tracks a detached (floating) panel */
export interface DetachedPanel {
  panelId: string;
  type: DockPanelType;
  title: string;
  originalRegion: DockRegionPosition;
  windowLabel: string;
}

/**
 * Service for managing dock layout and drag-drop state.
 * Provides signals for reactive UI updates and persists layout to localStorage.
 */
@Injectable({
  providedIn: 'root',
})
export class DockService {
  // Layout state
  private _layout = signal<DockLayout>(createDefaultLayout());

  // Drag state
  private _dragState = signal<DockDragState>({
    isDragging: false,
    panelId: null,
    sourceRegion: null,
    currentDropZone: null,
  });

  // Detached panels state
  private _detachedPanels = signal<DetachedPanel[]>([]);

  // Public read-only signals
  readonly layout = this._layout.asReadonly();
  readonly dragState = this._dragState.asReadonly();
  readonly detachedPanels = this._detachedPanels.asReadonly();

  // Computed signals for each region
  readonly topRegion = computed(() => this._layout().regions.top);
  readonly leftRegion = computed(() => this._layout().regions.left);
  readonly rightRegion = computed(() => this._layout().regions.right);
  readonly bottomRegion = computed(() => this._layout().regions.bottom);

  // Computed: is dragging?
  readonly isDragging = computed(() => this._dragState().isDragging);

  // Computed: current drop zone
  readonly currentDropZone = computed(() => this._dragState().currentDropZone);

  constructor() {
    this.loadLayout();
    this.loadDetachedPanels();

    // Auto-save layout when it changes
    effect(() => {
      this.saveLayout();
    });
  }

  /**
   * Get a region by position
   */
  getRegion(position: DockRegionPosition): DockRegion {
    return this._layout().regions[position];
  }

  /**
   * Set region size
   */
  setRegionSize(position: DockRegionPosition, size: number): void {
    const defaults = DOCK_DEFAULTS[position];
    const clampedSize = Math.max(defaults.minSize, Math.min(defaults.maxSize, size));

    this._layout.update((layout) => ({
      ...layout,
      regions: {
        ...layout.regions,
        [position]: {
          ...layout.regions[position],
          size: clampedSize,
        },
      },
    }));
  }

  /**
   * Toggle region collapsed state
   */
  toggleRegionCollapsed(position: DockRegionPosition): void {
    this._layout.update((layout) => ({
      ...layout,
      regions: {
        ...layout.regions,
        [position]: {
          ...layout.regions[position],
          isCollapsed: !layout.regions[position].isCollapsed,
        },
      },
    }));
  }

  /**
   * Set region collapsed state
   */
  setRegionCollapsed(position: DockRegionPosition, collapsed: boolean): void {
    this._layout.update((layout) => ({
      ...layout,
      regions: {
        ...layout.regions,
        [position]: {
          ...layout.regions[position],
          isCollapsed: collapsed,
        },
      },
    }));
  }

  /**
   * Set active panel in a region
   */
  setActivePanel(position: DockRegionPosition, panelId: string): void {
    this._layout.update((layout) => ({
      ...layout,
      regions: {
        ...layout.regions,
        [position]: {
          ...layout.regions[position],
          panels: layout.regions[position].panels.map((p) => ({
            ...p,
            isActive: p.id === panelId,
          })),
        },
      },
    }));
  }

  /**
   * Start dragging a panel
   */
  startDrag(panelId: string, sourceRegion: DockRegionPosition): void {
    this._dragState.set({
      isDragging: true,
      panelId,
      sourceRegion,
      currentDropZone: null,
    });
  }

  /**
   * Update current drop zone during drag
   */
  setDropZone(zone: DockRegionPosition | null): void {
    this._dragState.update((state) => ({
      ...state,
      currentDropZone: zone,
    }));
  }

  /**
   * End drag operation
   */
  endDrag(): void {
    this._dragState.set({
      isDragging: false,
      panelId: null,
      sourceRegion: null,
      currentDropZone: null,
    });
  }

  /**
   * Move a panel from one region to another
   */
  movePanel(panelId: string, fromRegion: DockRegionPosition, toRegion: DockRegionPosition): void {
    if (fromRegion === toRegion) {
      return;
    }

    this._layout.update((layout) => {
      const sourceRegion = layout.regions[fromRegion];
      const targetRegion = layout.regions[toRegion];

      const panel = sourceRegion.panels.find((p) => p.id === panelId);
      if (!panel) {
        return layout;
      }

      // Remove from source
      const newSourcePanels = sourceRegion.panels.filter((p) => p.id !== panelId);

      // Add to target with new order
      const newTargetPanels = [
        ...targetRegion.panels,
        { ...panel, order: targetRegion.panels.length, isActive: true },
      ];

      // Deactivate other panels in target
      const finalTargetPanels = newTargetPanels.map((p) => ({
        ...p,
        isActive: p.id === panelId,
      }));

      return {
        ...layout,
        regions: {
          ...layout.regions,
          [fromRegion]: {
            ...sourceRegion,
            panels: newSourcePanels,
            // Collapse if no panels left
            isCollapsed: newSourcePanels.length === 0,
          },
          [toRegion]: {
            ...targetRegion,
            panels: finalTargetPanels,
            // Expand if was collapsed
            isCollapsed: false,
          },
        },
      };
    });
  }

  /**
   * Reorder panels within a region
   */
  reorderPanels(position: DockRegionPosition, panelIds: string[]): void {
    this._layout.update((layout) => ({
      ...layout,
      regions: {
        ...layout.regions,
        [position]: {
          ...layout.regions[position],
          panels: layout.regions[position].panels
            .sort((a, b) => panelIds.indexOf(a.id) - panelIds.indexOf(b.id))
            .map((p, i) => ({ ...p, order: i })),
        },
      },
    }));
  }

  /**
   * Add a new panel to a region
   */
  addPanel(position: DockRegionPosition, type: DockPanelType, title: string): string {
    const id = `${type}-${Date.now()}`;

    this._layout.update((layout) => {
      const region = layout.regions[position];
      const newPanel: DockPanel = {
        id,
        type,
        title,
        isActive: true,
        order: region.panels.length,
      };

      // Deactivate other panels
      const updatedPanels = region.panels.map((p) => ({ ...p, isActive: false }));

      return {
        ...layout,
        regions: {
          ...layout.regions,
          [position]: {
            ...region,
            panels: [...updatedPanels, newPanel],
            isCollapsed: false,
          },
        },
      };
    });

    return id;
  }

  /**
   * Remove a panel from a region
   */
  removePanel(position: DockRegionPosition, panelId: string): void {
    this._layout.update((layout) => {
      const region = layout.regions[position];
      const newPanels = region.panels.filter((p) => p.id !== panelId);

      // If we removed the active panel, activate the first remaining panel
      if (newPanels.length > 0 && !newPanels.some((p) => p.isActive)) {
        newPanels[0].isActive = true;
      }

      return {
        ...layout,
        regions: {
          ...layout.regions,
          [position]: {
            ...region,
            panels: newPanels,
            isCollapsed: newPanels.length === 0,
          },
        },
      };
    });
  }

  /**
   * Reset layout to defaults
   */
  resetLayout(): void {
    this._layout.set(createDefaultLayout());
  }

  /**
   * Detach a panel from the dock into a floating window
   */
  async detachPanel(position: DockRegionPosition, panelId: string): Promise<void> {
    const region = this._layout().regions[position];
    const panel = region.panels.find((p) => p.id === panelId);
    if (!panel) return;

    // Generate unique window label
    const windowLabel = `floating-panel-${panelId}`;

    // Check if already detached
    if (this._detachedPanels().some((p) => p.panelId === panelId)) {
      // Focus existing window
      try {
        const { WebviewWindow } = await import('@tauri-apps/api/webviewWindow');
        const existingWindow = await WebviewWindow.getByLabel(windowLabel);
        if (existingWindow) {
          await existingWindow.setFocus();
        }
      } catch (e) {
        console.error('Failed to focus existing window:', e);
      }
      return;
    }

    // Create floating window
    try {
      const { WebviewWindow } = await import('@tauri-apps/api/webviewWindow');

      const floatingWindow = new WebviewWindow(windowLabel, {
        url: `/floating-panel/${panel.type}/${panelId}`,
        title: panel.title,
        width: 300,
        height: 400,
        minWidth: 200,
        minHeight: 250,
        decorations: false,
        transparent: false,
        alwaysOnTop: true,
        resizable: true,
        x: 100,
        y: 100,
      });

      // Track detached panel
      const detached: DetachedPanel = {
        panelId,
        type: panel.type,
        title: panel.title,
        originalRegion: position,
        windowLabel,
      };

      floatingWindow.once('tauri://created', () => {
        // Add to detached panels
        this._detachedPanels.update((panels) => [...panels, detached]);
        // Remove from dock
        this.removePanel(position, panelId);
        this.saveDetachedPanels();
      });

      floatingWindow.once('tauri://error', (e) => {
        console.error('Failed to create floating panel window:', e);
      });

      // Listen for window close to re-dock
      floatingWindow.once('tauri://close-requested', async () => {
        await this.reattachPanel(panelId);
      });
    } catch (e) {
      console.error('Failed to create floating window:', e);
    }
  }

  /**
   * Reattach a detached panel back to its original dock region
   */
  async reattachPanel(panelId: string): Promise<void> {
    const detached = this._detachedPanels().find((p) => p.panelId === panelId);
    if (!detached) return;

    // Add panel back to original region
    this.addPanel(detached.originalRegion, detached.type, detached.title);

    // Remove from detached list
    this._detachedPanels.update((panels) => panels.filter((p) => p.panelId !== panelId));
    this.saveDetachedPanels();
  }

  /**
   * Check if a panel is detached
   */
  isPanelDetached(panelId: string): boolean {
    return this._detachedPanels().some((p) => p.panelId === panelId);
  }

  /**
   * Get detached panel by ID
   */
  getDetachedPanel(panelId: string): DetachedPanel | undefined {
    return this._detachedPanels().find((p) => p.panelId === panelId);
  }

  /**
   * Load layout from localStorage
   */
  private loadLayout(): void {
    try {
      const stored = localStorage.getItem(STORAGE_KEY);
      if (stored) {
        const parsed = JSON.parse(stored) as DockLayout;
        // Merge with defaults to handle missing properties
        const layout = createDefaultLayout();
        for (const pos of ['top', 'left', 'right', 'bottom'] as DockRegionPosition[]) {
          if (parsed.regions[pos]) {
            layout.regions[pos] = {
              ...layout.regions[pos],
              ...parsed.regions[pos],
            };
          }
        }
        this._layout.set(layout);
      }
    } catch {
      // Use default layout if localStorage fails
    }
  }

  /**
   * Save layout to localStorage
   */
  private saveLayout(): void {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(this._layout()));
    } catch {
      // Ignore localStorage errors
    }
  }

  /**
   * Load detached panels from localStorage
   */
  private loadDetachedPanels(): void {
    try {
      const stored = localStorage.getItem(DETACHED_PANELS_KEY);
      if (stored) {
        const panels = JSON.parse(stored) as DetachedPanel[];
        this._detachedPanels.set(panels);
      }
    } catch {
      // Use empty array if localStorage fails
    }
  }

  /**
   * Save detached panels to localStorage
   */
  private saveDetachedPanels(): void {
    try {
      localStorage.setItem(DETACHED_PANELS_KEY, JSON.stringify(this._detachedPanels()));
    } catch {
      // Ignore localStorage errors
    }
  }
}
