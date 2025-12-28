import { Component, inject } from '@angular/core';
import { RouterOutlet } from '@angular/router';
import { SidebarComponent } from './shared/components/sidebar/sidebar.component';
import { PreloadService } from '@core/services';

@Component({
  selector: 'app-root',
  standalone: true,
  imports: [RouterOutlet, SidebarComponent],
  template: `
    <div class="flex h-screen bg-syslens-bg-primary">
      <app-sidebar />
      <main class="flex-1 overflow-auto">
        <router-outlet />
      </main>
    </div>
  `,
  styles: [`
    :host {
      display: block;
      height: 100vh;
    }
  `]
})
export class AppComponent {
  title = 'Syslens';

  // Inject PreloadService to initialize background preloading
  private preloadService = inject(PreloadService);
}
