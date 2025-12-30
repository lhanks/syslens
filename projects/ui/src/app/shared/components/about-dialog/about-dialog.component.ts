import { Component, Input, Output, EventEmitter } from '@angular/core';
import { CommonModule } from '@angular/common';

@Component({
  selector: 'app-about-dialog',
  standalone: true,
  imports: [CommonModule],
  template: `
    @if (isOpen) {
      <div class="fixed inset-0 z-50 flex items-center justify-center">
        <!-- Backdrop -->
        <div class="absolute inset-0 bg-black/60 backdrop-blur-sm" (click)="close()"></div>

        <!-- Dialog -->
        <div class="relative bg-syslens-bg-secondary border border-syslens-border-primary rounded-xl shadow-2xl w-[400px] overflow-hidden">
          <!-- Header -->
          <div class="px-6 py-4 border-b border-syslens-border-primary">
            <h2 class="text-xl font-bold text-syslens-text-primary">About Syslens</h2>
          </div>

          <!-- Content -->
          <div class="p-6 space-y-4">
            <!-- Logo & Name -->
            <div class="flex items-center gap-4">
              <div class="w-16 h-16 rounded-xl bg-gradient-to-br from-syslens-accent-blue to-syslens-accent-purple flex items-center justify-center">
                <svg class="w-10 h-10 text-white" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <circle cx="12" cy="12" r="10" />
                  <path d="M12 6v6l4 2" />
                  <circle cx="12" cy="12" r="3" fill="currentColor" />
                </svg>
              </div>
              <div>
                <h3 class="text-2xl font-bold text-syslens-text-primary">Syslens</h3>
                <p class="text-syslens-text-secondary">System Information Dashboard</p>
              </div>
            </div>

            <!-- Version Info -->
            <div class="bg-syslens-bg-tertiary rounded-lg p-4 space-y-2">
              <div class="flex justify-between text-sm">
                <span class="text-syslens-text-muted">Version</span>
                <span class="text-syslens-text-primary font-mono">0.1.0</span>
              </div>
              <div class="flex justify-between text-sm">
                <span class="text-syslens-text-muted">Platform</span>
                <span class="text-syslens-text-primary font-mono">Windows</span>
              </div>
              <div class="flex justify-between text-sm">
                <span class="text-syslens-text-muted">Framework</span>
                <span class="text-syslens-text-primary font-mono">Tauri 2.0 + Angular</span>
              </div>
            </div>

            <!-- Description -->
            <p class="text-sm text-syslens-text-secondary">
              A modern desktop application for comprehensive system monitoring,
              hardware information, and real-time performance metrics.
            </p>

            <!-- Copyright -->
            <p class="text-xs text-syslens-text-muted text-center pt-2">
              Copyright 2025 Syslens. All rights reserved.
            </p>
          </div>

          <!-- Footer -->
          <div class="px-6 py-4 border-t border-syslens-border-primary flex justify-end">
            <button
              (click)="close()"
              class="px-4 py-2 bg-syslens-accent-blue text-white rounded-lg hover:bg-syslens-accent-blue/80 transition-colors"
            >
              Close
            </button>
          </div>
        </div>
      </div>
    }
  `
})
export class AboutDialogComponent {
  @Input() isOpen = false;
  @Output() closed = new EventEmitter<void>();

  close(): void {
    this.closed.emit();
  }
}
