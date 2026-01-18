import { Component, inject, computed } from '@angular/core';
import { CommonModule } from '@angular/common';
import { DockService } from '@core/services/dock.service';
import { DockRegionComponent } from './dock-region.component';

/**
 * Main dock container using CSS Grid for 4-region layout.
 * Layout:
 *   ┌────────────────────────────────┐
 *   │           TOP REGION           │
 *   ├────────┬────────────┬──────────┤
 *   │  LEFT  │    MAIN    │  RIGHT   │
 *   │ REGION │   (slot)   │  REGION  │
 *   ├────────┴────────────┴──────────┤
 *   │          BOTTOM REGION         │
 *   └────────────────────────────────┘
 */
@Component({
  selector: 'app-dock-container',
  standalone: true,
  imports: [CommonModule, DockRegionComponent],
  template: `
    <div
      class="dock-container"
      [style.grid-template-columns]="gridTemplateColumns()"
      [style.grid-template-rows]="gridTemplateRows()">

      <!-- Top Region -->
      <div class="dock-top" [class.hidden]="isRegionHidden('top')">
        <app-dock-region position="top" />
      </div>

      <!-- Left Region -->
      <div class="dock-left" [class.hidden]="isRegionHidden('left')">
        <app-dock-region position="left" />
      </div>

      <!-- Main Content (slot) -->
      <div class="dock-main">
        <ng-content></ng-content>
      </div>

      <!-- Right Region -->
      <div class="dock-right" [class.hidden]="isRegionHidden('right')">
        <app-dock-region position="right" />
      </div>

      <!-- Bottom Region -->
      <div class="dock-bottom" [class.hidden]="isRegionHidden('bottom')">
        <app-dock-region position="bottom" />
      </div>

      <!-- Global Drop Zones (shown during drag) -->
      @if (isDragging()) {
        <!-- Top drop zone -->
        @if (canDropAt('top')) {
          <div
            class="drop-zone drop-zone-top"
            [class.active]="isDropTarget('top')"
            (dragover)="onDragOver($event, 'top')"
            (dragleave)="onDragLeave()"
            (drop)="onDrop($event, 'top')">
            <span class="drop-zone-label">Drop here</span>
          </div>
        }

        <!-- Bottom drop zone -->
        @if (canDropAt('bottom')) {
          <div
            class="drop-zone drop-zone-bottom"
            [class.active]="isDropTarget('bottom')"
            (dragover)="onDragOver($event, 'bottom')"
            (dragleave)="onDragLeave()"
            (drop)="onDrop($event, 'bottom')">
            <span class="drop-zone-label">Drop here</span>
          </div>
        }

        <!-- Left drop zone -->
        @if (canDropAt('left')) {
          <div
            class="drop-zone drop-zone-left"
            [class.active]="isDropTarget('left')"
            (dragover)="onDragOver($event, 'left')"
            (dragleave)="onDragLeave()"
            (drop)="onDrop($event, 'left')">
            <span class="drop-zone-label">Drop here</span>
          </div>
        }

        <!-- Right drop zone -->
        @if (canDropAt('right')) {
          <div
            class="drop-zone drop-zone-right"
            [class.active]="isDropTarget('right')"
            (dragover)="onDragOver($event, 'right')"
            (dragleave)="onDragLeave()"
            (drop)="onDrop($event, 'right')">
            <span class="drop-zone-label">Drop here</span>
          </div>
        }
      }
    </div>
  `,
  styles: [`
    :host {
      display: block;
      height: 100%;
      width: 100%;
    }

    .dock-container {
      display: grid;
      position: relative;
      height: 100%;
      width: 100%;
      grid-template-areas:
        'top top top'
        'left main right'
        'bottom bottom bottom';
    }

    .dock-top {
      grid-area: top;
    }

    .dock-left {
      grid-area: left;
    }

    .dock-main {
      grid-area: main;
      overflow: auto;
      min-width: 0;
      min-height: 0;
    }

    .dock-right {
      grid-area: right;
    }

    .dock-bottom {
      grid-area: bottom;
    }

    .hidden {
      display: none !important;
    }

    /* Drop zones */
    .drop-zone {
      position: absolute;
      display: flex;
      align-items: center;
      justify-content: center;
      background: rgba(59, 130, 246, 0.1);
      border: 2px dashed rgba(59, 130, 246, 0.3);
      transition: all 0.15s;
      z-index: 100;
      pointer-events: auto;
    }

    .drop-zone.active {
      background: rgba(59, 130, 246, 0.2);
      border-color: rgba(59, 130, 246, 0.6);
    }

    .drop-zone-label {
      font-size: 11px;
      font-weight: 500;
      color: var(--syslens-accent-blue);
      opacity: 0;
      transition: opacity 0.15s;
    }

    .drop-zone.active .drop-zone-label {
      opacity: 1;
    }

    .drop-zone-top {
      top: 0;
      left: 0;
      right: 0;
      height: 40px;
    }

    .drop-zone-bottom {
      bottom: 0;
      left: 0;
      right: 0;
      height: 40px;
    }

    .drop-zone-left {
      top: 40px;
      bottom: 40px;
      left: 0;
      width: 40px;
    }

    .drop-zone-right {
      top: 40px;
      bottom: 40px;
      right: 0;
      width: 40px;
    }
  `]
})
export class DockContainerComponent {
  private dockService = inject(DockService);

  // Computed grid template based on region states
  gridTemplateColumns = computed(() => {
    // Access signals to trigger recomputation when regions change
    this.dockService.leftRegion();
    this.dockService.rightRegion();

    const leftSize = this.getRegionSize('left');
    const rightSize = this.getRegionSize('right');

    return `${leftSize} 1fr ${rightSize}`;
  });

  gridTemplateRows = computed(() => {
    const topSize = this.getRegionSize('top');
    const bottomSize = this.getRegionSize('bottom');

    return `${topSize} 1fr ${bottomSize}`;
  });

  isDragging = computed(() => this.dockService.isDragging());

  private getRegionSize(position: 'top' | 'left' | 'right' | 'bottom'): string {
    const region = this.dockService.getRegion(position);

    // If no panels, don't reserve space
    if (region.panels.length === 0) {
      return '0';
    }

    // If collapsed, show minimal expand button
    if (region.isCollapsed) {
      return position === 'left' || position === 'right' ? '24px' : '24px';
    }

    // Return actual size
    return `${region.size}px`;
  }

  isRegionHidden(position: 'top' | 'left' | 'right' | 'bottom'): boolean {
    const region = this.dockService.getRegion(position);
    return region.panels.length === 0;
  }

  canDropAt(position: 'top' | 'left' | 'right' | 'bottom'): boolean {
    const dragState = this.dockService.dragState();
    // Can drop anywhere except the source region
    return dragState.sourceRegion !== position;
  }

  isDropTarget(position: 'top' | 'left' | 'right' | 'bottom'): boolean {
    return this.dockService.currentDropZone() === position;
  }

  onDragOver(event: DragEvent, position: 'top' | 'left' | 'right' | 'bottom'): void {
    event.preventDefault();
    if (event.dataTransfer) {
      event.dataTransfer.dropEffect = 'move';
    }
    this.dockService.setDropZone(position);
  }

  onDragLeave(): void {
    this.dockService.setDropZone(null);
  }

  onDrop(event: DragEvent, position: 'top' | 'left' | 'right' | 'bottom'): void {
    event.preventDefault();
    const dragState = this.dockService.dragState();
    if (dragState.panelId && dragState.sourceRegion) {
      this.dockService.movePanel(dragState.panelId, dragState.sourceRegion, position);
    }
    this.dockService.endDrag();
  }
}
