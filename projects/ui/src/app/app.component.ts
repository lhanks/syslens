import { Component, OnInit, inject, computed } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterOutlet } from '@angular/router';
import { SidebarComponent, RightSidebarComponent, StatusBarComponent, AboutDialogComponent } from './shared/components';
import { PreloadService, StateService, MetricsHistoryService, MenuService } from '@core/services';

@Component({
  selector: 'app-root',
  standalone: true,
  imports: [CommonModule, RouterOutlet, SidebarComponent, RightSidebarComponent, StatusBarComponent, AboutDialogComponent],
  template: `
    <div class="flex flex-col h-screen bg-syslens-bg-primary">
      <div class="flex flex-1 overflow-hidden">
        <app-sidebar />
        <main class="flex-1 overflow-auto">
          <router-outlet />
        </main>
        @if (sidebarVisible()) {
          <app-right-sidebar />
        }
      </div>
      <app-status-bar />
    </div>

    <!-- About Dialog -->
    <app-about-dialog
      [isOpen]="aboutDialogOpen()"
      (closed)="menuService.closeAboutDialog()"
    />
  `,
  styles: [`
    :host {
      display: block;
      height: 100vh;
    }
  `]
})
export class AppComponent implements OnInit {
  title = 'Syslens';

  private preloadService = inject(PreloadService);
  private stateService = inject(StateService);
  private metricsHistoryService = inject(MetricsHistoryService);
  menuService = inject(MenuService);

  // Expose menu service signals to template
  sidebarVisible = computed(() => this.menuService.sidebarVisible());
  aboutDialogOpen = computed(() => this.menuService.aboutDialogOpen());

  ngOnInit(): void {
    // Initialize state persistence and restore last route
    this.stateService.initialize();
    this.stateService.restoreLastRoute();

    // Start continuous metrics collection
    this.metricsHistoryService.start();

    // Initialize menu event listeners
    this.menuService.init();
  }
}
