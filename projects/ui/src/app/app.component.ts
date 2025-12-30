import { Component, OnInit, inject, computed } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterOutlet } from '@angular/router';
import { SidebarComponent, RightSidebarComponent, StatusBarComponent, TopBarComponent, AboutDialogComponent } from './shared/components';
import { PreloadService, StateService, MetricsHistoryService, MenuService, ViewSettingsService } from '@core/services';

@Component({
  selector: 'app-root',
  standalone: true,
  imports: [CommonModule, RouterOutlet, SidebarComponent, RightSidebarComponent, StatusBarComponent, TopBarComponent, AboutDialogComponent],
  template: `
    <div class="flex flex-col h-screen bg-syslens-bg-primary">
      <!-- Top navigation bar -->
      <app-top-bar />

      <div class="flex flex-1 overflow-hidden">
        <app-sidebar />

        <!-- Right sidebar docked to left side -->
        @if (sidebarVisible() && sidebarPosition() === 'left') {
          <app-right-sidebar [dockPosition]="'left'" />
        }

        <main class="flex-1 overflow-auto">
          <router-outlet />
        </main>

        <!-- Right sidebar docked to right side (default) -->
        @if (sidebarVisible() && sidebarPosition() === 'right') {
          <app-right-sidebar [dockPosition]="'right'" />
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
  private viewSettings = inject(ViewSettingsService);
  menuService = inject(MenuService);

  // Expose service signals to template
  sidebarVisible = computed(() => this.viewSettings.rightSidebarVisible());
  sidebarPosition = computed(() => this.viewSettings.rightSidebarPosition());
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
