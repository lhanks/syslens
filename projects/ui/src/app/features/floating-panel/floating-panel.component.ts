import { Component, inject, OnInit, signal } from '@angular/core';
import { ActivatedRoute } from '@angular/router';
import { Window } from '@tauri-apps/api/window';
import { DockService } from '@core/services';
import { DockPanelType } from '@shared/components/dock/dock.model';
import { PerformancePanelComponent, SystemInfoPanelComponent } from '@shared/components/dock/panels';

@Component({
  selector: 'app-floating-panel',
  standalone: true,
  imports: [PerformancePanelComponent, SystemInfoPanelComponent],
  template: `
    <div class="floating-container bg-syslens-bg-secondary flex flex-col h-screen">
      <!-- Drag Region / Header -->
      <div class="drag-region px-3 py-2 border-b border-syslens-border-primary flex items-center justify-between">
        <h1 class="text-sm font-semibold text-syslens-text-primary">{{ title() }}</h1>
        <button
          class="p-1.5 rounded hover:bg-syslens-bg-hover text-syslens-text-secondary hover:text-syslens-text-primary transition-colors"
          title="Dock panel"
          (click)="dockPanel()">
          <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
              d="M11 19l-7-7 7-7m8 14l-7-7 7-7" />
          </svg>
        </button>
      </div>

      <!-- Panel Content -->
      <div class="flex-1 overflow-y-auto">
        @switch (panelType()) {
          @case ('performance') {
            <app-performance-panel />
          }
          @case ('system-info') {
            <app-system-info-panel />
          }
        }
      </div>
    </div>
  `,
  styles: [`
    :host {
      display: block;
      height: 100vh;
    }

    .floating-container {
      -webkit-app-region: no-drag;
    }

    .drag-region {
      -webkit-app-region: drag;
    }

    .drag-region button {
      -webkit-app-region: no-drag;
    }
  `]
})
export class FloatingPanelComponent implements OnInit {
  private route = inject(ActivatedRoute);
  private dockService = inject(DockService);

  panelType = signal<DockPanelType>('performance');
  panelId = signal<string>('');
  title = signal<string>('Panel');

  ngOnInit(): void {
    const type = this.route.snapshot.paramMap.get('type') as DockPanelType;
    const id = this.route.snapshot.paramMap.get('id') || '';

    this.panelType.set(type || 'performance');
    this.panelId.set(id);

    // Set title based on type
    switch (type) {
      case 'performance':
        this.title.set('Performance');
        break;
      case 'system-info':
        this.title.set('System Info');
        break;
      default:
        this.title.set('Panel');
    }
  }

  async dockPanel(): Promise<void> {
    // Reattach panel to dock
    await this.dockService.reattachPanel(this.panelId());
    // Close this floating window
    const currentWindow = Window.getCurrent();
    await currentWindow.close();
  }
}
