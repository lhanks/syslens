import { Component, inject } from '@angular/core';
import { CommonModule } from '@angular/common';
import { StatusService } from '@core/services/status.service';

@Component({
  selector: 'app-status-bar',
  standalone: true,
  imports: [CommonModule],
  template: `
    <div class="h-6 px-3 flex items-center justify-between bg-syslens-bg-tertiary border-t border-syslens-border text-xs">
      <div class="flex items-center gap-2">
        @if (statusService.isLoading()) {
          <div class="w-2 h-2 rounded-full bg-syslens-accent-blue animate-pulse"></div>
        } @else {
          <div class="w-2 h-2 rounded-full bg-syslens-accent-green"></div>
        }
        <span class="text-syslens-text-secondary">{{ statusService.currentStatus() }}</span>
      </div>
      @if (statusService.operationCount() > 1) {
        <span class="text-syslens-text-muted">
          {{ statusService.operationCount() }} operations
        </span>
      }
    </div>
  `
})
export class StatusBarComponent {
  statusService = inject(StatusService);
}
