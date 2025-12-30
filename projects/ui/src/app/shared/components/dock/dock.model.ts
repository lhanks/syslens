/**
 * Types of panels that can be docked
 */
export type DockPanelType = 'performance' | 'system-info';

/**
 * Dock region positions
 */
export type DockRegionPosition = 'top' | 'left' | 'right' | 'bottom';

/**
 * A panel that can be placed in a dock region
 */
export interface DockPanel {
  id: string;
  type: DockPanelType;
  title: string;
  isActive: boolean;
  order: number;
}

/**
 * A dock region that can contain multiple panels
 */
export interface DockRegion {
  position: DockRegionPosition;
  panels: DockPanel[];
  size: number;
  isCollapsed: boolean;
  minSize: number;
  maxSize: number;
}

/**
 * The complete dock layout configuration
 */
export interface DockLayout {
  regions: Record<DockRegionPosition, DockRegion>;
}

/**
 * Drag state for tracking panel moves
 */
export interface DockDragState {
  isDragging: boolean;
  panelId: string | null;
  sourceRegion: DockRegionPosition | null;
  currentDropZone: DockRegionPosition | null;
}

/**
 * Default sizes for dock regions
 */
export const DOCK_DEFAULTS = {
  left: { size: 256, minSize: 180, maxSize: 400 },
  right: { size: 288, minSize: 200, maxSize: 500 },
  top: { size: 200, minSize: 100, maxSize: 400 },
  bottom: { size: 200, minSize: 100, maxSize: 400 },
} as const;

/**
 * Create default empty layout
 */
export function createDefaultLayout(): DockLayout {
  return {
    regions: {
      top: {
        position: 'top',
        panels: [],
        size: DOCK_DEFAULTS.top.size,
        isCollapsed: true,
        minSize: DOCK_DEFAULTS.top.minSize,
        maxSize: DOCK_DEFAULTS.top.maxSize,
      },
      left: {
        position: 'left',
        panels: [
          { id: 'performance-1', type: 'performance', title: 'Performance', isActive: true, order: 0 },
        ],
        size: DOCK_DEFAULTS.left.size,
        isCollapsed: false,
        minSize: DOCK_DEFAULTS.left.minSize,
        maxSize: DOCK_DEFAULTS.left.maxSize,
      },
      right: {
        position: 'right',
        panels: [
          { id: 'system-info-1', type: 'system-info', title: 'System Info', isActive: true, order: 0 },
        ],
        size: DOCK_DEFAULTS.right.size,
        isCollapsed: false,
        minSize: DOCK_DEFAULTS.right.minSize,
        maxSize: DOCK_DEFAULTS.right.maxSize,
      },
      bottom: {
        position: 'bottom',
        panels: [],
        size: DOCK_DEFAULTS.bottom.size,
        isCollapsed: true,
        minSize: DOCK_DEFAULTS.bottom.minSize,
        maxSize: DOCK_DEFAULTS.bottom.maxSize,
      },
    },
  };
}
