import { Component, OnInit, inject } from '@angular/core';
import { RouterOutlet } from '@angular/router';
import { SidebarComponent, RightSidebarComponent, StatusBarComponent } from './shared/components';
import { PreloadService, StateService, MetricsHistoryService } from '@core/services';

@Component({
  selector: 'app-root',
  standalone: true,
  imports: [RouterOutlet, SidebarComponent, RightSidebarComponent, StatusBarComponent],
  template: `
    <div class="flex flex-col h-screen bg-syslens-bg-primary">
      <div class="flex flex-1 overflow-hidden">
        <app-sidebar />
        <main class="flex-1 overflow-auto">
          <router-outlet />
        </main>
        <app-right-sidebar />
      </div>
      <app-status-bar />
    </div>
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

  ngOnInit(): void {
    // Initialize state persistence and restore last route
    this.stateService.initialize();
    this.stateService.restoreLastRoute();

    // Start continuous metrics collection
    this.metricsHistoryService.start();
  }
}
