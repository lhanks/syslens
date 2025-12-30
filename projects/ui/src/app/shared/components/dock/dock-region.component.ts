import {
  Component,
  Input,
  inject,
  computed,
  signal,
  HostListener,
  ViewChild,
  ElementRef,
  AfterViewInit
} from '@angular/core';
import { CommonModule } from '@angular/common';
import { DockService } from '@core/services/dock.service';
import { DockRegionPosition, DockPanel } from './dock.model';
import { PerformancePanelComponent } from './panels/performance-panel.component';
import { SystemInfoPanelComponent } from './panels/system-info-panel.component';

/**
 * A dock region that can contain multiple panels with tabs.
 * Supports resizing and drag-drop between regions.
 */
@Component({
  selector: 'app-dock-region',
  standalone: true,
  imports: [CommonModule, PerformancePanelComponent, SystemInfoPanelComponent],
  template: `
    @if (!region().isCollapsed && region().panels.length > 0) {
      <div
        class="dock-region flex flex-col h-full bg-syslens-bg-secondary"
        [class.border-r]="position === 'left'"
        [class.border-l]="position === 'right'"
        [class.border-b]="position === 'top'"
        [class.border-t]="position === 'bottom'"
        [class.border-syslens-border-primary]="true"
        [style.width.px]="isHorizontal ? region().size : null"
        [style.height.px]="!isHorizontal ? region().size : null"
        [style.min-width.px]="isHorizontal ? region().minSize : null"
        [style.min-height.px]="!isHorizontal ? region().minSize : null">

        <!-- Tab Header -->
        <div class="tab-header flex items-center border-b border-syslens-border-primary bg-syslens-bg-tertiary px-1">
          <!-- Tabs -->
          <div class="flex-1 flex items-center gap-0.5 min-w-0 overflow-x-auto">
            @for (panel of region().panels; track panel.id) {
              <button
                class="tab-button flex items-center gap-1 px-2 py-1.5 text-xs font-medium rounded-t transition-colors whitespace-nowrap"
                [class.bg-syslens-bg-secondary]="panel.isActive"
                [class.text-syslens-text-primary]="panel.isActive"
                [class.text-syslens-text-muted]="!panel.isActive"
                [class.hover:text-syslens-text-secondary]="!panel.isActive"
                [class.hover:bg-syslens-bg-hover]="!panel.isActive"
                draggable="true"
                (click)="selectPanel(panel.id)"
                (dragstart)="onDragStart($event, panel)"
                (dragend)="onDragEnd()">
                <span>{{ panel.title }}</span>
              </button>
            }
          </div>

          <!-- Region controls -->
          <div class="flex items-center gap-0.5 ml-1">
            <!-- Pop-out button -->
            <button
              class="p-1 rounded hover:bg-syslens-bg-hover text-syslens-text-muted hover:text-syslens-text-secondary transition-colors"
              title="Pop out to floating window"
              (click)="popOutActivePanel()">
              <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                  d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
              </svg>
            </button>
            <!-- Collapse button -->
            <button
              class="p-1 rounded hover:bg-syslens-bg-hover text-syslens-text-muted hover:text-syslens-text-secondary transition-colors"
              title="Collapse"
              (click)="collapse()">
              <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                @if (isHorizontal) {
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                    d="M11 19l-7-7 7-7m8 14l-7-7 7-7" />
                } @else {
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                    d="M5 11l7-7 7 7M5 19l7-7 7 7" />
                }
              </svg>
            </button>
          </div>
        </div>

        <!-- Panel Content -->
        <div class="panel-content flex-1 overflow-hidden">
          @for (panel of region().panels; track panel.id) {
            @if (panel.isActive) {
              @switch (panel.type) {
                @case ('performance') {
                  <app-performance-panel />
                }
                @case ('system-info') {
                  <app-system-info-panel />
                }
              }
            }
          }
        </div>

        <!-- Resize Handle -->
        <div
          #resizeHandle
          class="resize-handle"
          [class.resize-handle-horizontal]="isHorizontal"
          [class.resize-handle-vertical]="!isHorizontal"
          [class.left-0]="position === 'right'"
          [class.right-0]="position === 'left'"
          [class.top-0]="position === 'bottom'"
          [class.bottom-0]="position === 'top'"
          (mousedown)="startResize($event)">
        </div>

        <!-- Drop Zone Indicator -->
        @if (isDropTarget()) {
          <div class="absolute inset-0 border-2 border-dashed border-syslens-accent-blue bg-syslens-accent-blue/10 pointer-events-none z-10"></div>
        }
      </div>
    } @else if (region().isCollapsed && region().panels.length > 0) {
      <!-- Collapsed state - show expand button -->
      <div
        class="collapsed-region flex items-center justify-center cursor-pointer hover:bg-syslens-bg-hover transition-colors"
        [class.border-r]="position === 'left'"
        [class.border-l]="position === 'right'"
        [class.border-b]="position === 'top'"
        [class.border-t]="position === 'bottom'"
        [class.border-syslens-border-primary]="true"
        [class.w-6]="isHorizontal"
        [class.h-6]="!isHorizontal"
        (click)="expand()"
        (dragover)="onDragOver($event)"
        (dragleave)="onDragLeave()"
        (drop)="onDrop($event)">
        <svg class="w-4 h-4 text-syslens-text-muted" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          @if (position === 'left') {
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 5l7 7-7 7" />
          } @else if (position === 'right') {
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 19l-7-7 7-7" />
          } @else if (position === 'top') {
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
          } @else {
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 15l7-7 7 7" />
          }
        </svg>
      </div>
    }
  `,
  styles: [`
    :host {
      display: contents;
    }

    .dock-region {
      position: relative;
    }

    .tab-header {
      min-height: 28px;
    }

    .tab-button {
      cursor: pointer;
      border-bottom: 2px solid transparent;
    }

    .tab-button[class*="bg-syslens-bg-secondary"] {
      border-bottom-color: var(--syslens-accent-blue);
    }

    .resize-handle {
      position: absolute;
      background: transparent;
      transition: background-color 0.15s;
      z-index: 10;
    }

    .resize-handle-horizontal {
      width: 4px;
      height: 100%;
      top: 0;
      cursor: ew-resize;
    }

    .resize-handle-vertical {
      height: 4px;
      width: 100%;
      left: 0;
      cursor: ns-resize;
    }

    .resize-handle:hover,
    .resize-handle.resizing {
      background: rgba(59, 130, 246, 0.5);
    }

    .collapsed-region {
      background: var(--syslens-bg-secondary);
    }
  `]
})
export class DockRegionComponent implements AfterViewInit {
  @Input({ required: true }) position!: DockRegionPosition;

  @ViewChild('resizeHandle') resizeHandle?: ElementRef<HTMLDivElement>;

  private dockService = inject(DockService);

  // Resize state
  private isResizing = signal(false);
  private resizeStartPos = 0;
  private resizeStartSize = 0;

  // Computed region data
  region = computed(() => this.dockService.getRegion(this.position));

  // Is this a horizontal region (left/right) or vertical (top/bottom)?
  get isHorizontal(): boolean {
    return this.position === 'left' || this.position === 'right';
  }

  // Is this region currently a drop target?
  isDropTarget = computed(() => {
    const dragState = this.dockService.dragState();
    return dragState.isDragging && dragState.currentDropZone === this.position;
  });

  ngAfterViewInit(): void {
    // Add resize handle styling class
  }

  selectPanel(panelId: string): void {
    this.dockService.setActivePanel(this.position, panelId);
  }

  collapse(): void {
    this.dockService.setRegionCollapsed(this.position, true);
  }

  expand(): void {
    this.dockService.setRegionCollapsed(this.position, false);
  }

  popOutActivePanel(): void {
    const activePanel = this.region().panels.find((p) => p.isActive);
    if (activePanel) {
      this.dockService.detachPanel(this.position, activePanel.id);
    }
  }

  // Drag-drop handlers
  onDragStart(event: DragEvent, panel: DockPanel): void {
    if (event.dataTransfer) {
      event.dataTransfer.effectAllowed = 'move';
      event.dataTransfer.setData('text/plain', panel.id);
    }
    this.dockService.startDrag(panel.id, this.position);
  }

  onDragEnd(): void {
    this.dockService.endDrag();
  }

  onDragOver(event: DragEvent): void {
    event.preventDefault();
    if (event.dataTransfer) {
      event.dataTransfer.dropEffect = 'move';
    }
    this.dockService.setDropZone(this.position);
  }

  onDragLeave(): void {
    this.dockService.setDropZone(null);
  }

  onDrop(event: DragEvent): void {
    event.preventDefault();
    const dragState = this.dockService.dragState();
    if (dragState.panelId && dragState.sourceRegion) {
      this.dockService.movePanel(dragState.panelId, dragState.sourceRegion, this.position);
    }
    this.dockService.endDrag();
  }

  // Resize handlers
  startResize(event: MouseEvent): void {
    event.preventDefault();
    this.isResizing.set(true);
    this.resizeStartPos = this.isHorizontal ? event.clientX : event.clientY;
    this.resizeStartSize = this.region().size;

    document.addEventListener('mousemove', this.onMouseMove);
    document.addEventListener('mouseup', this.onMouseUp);
    document.body.style.cursor = this.isHorizontal ? 'ew-resize' : 'ns-resize';
    document.body.style.userSelect = 'none';
  }

  private onMouseMove = (event: MouseEvent): void => {
    if (!this.isResizing()) return;

    const currentPos = this.isHorizontal ? event.clientX : event.clientY;
    const delta = currentPos - this.resizeStartPos;

    // Calculate new size based on position
    let newSize: number;
    if (this.position === 'left' || this.position === 'top') {
      newSize = this.resizeStartSize + delta;
    } else {
      newSize = this.resizeStartSize - delta;
    }

    this.dockService.setRegionSize(this.position, newSize);
  };

  private onMouseUp = (): void => {
    this.isResizing.set(false);
    document.removeEventListener('mousemove', this.onMouseMove);
    document.removeEventListener('mouseup', this.onMouseUp);
    document.body.style.cursor = '';
    document.body.style.userSelect = '';
  };
}
