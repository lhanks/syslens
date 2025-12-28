import { Component, Input } from '@angular/core';
import { CommonModule } from '@angular/common';

@Component({
  selector: 'app-stat-card',
  standalone: true,
  imports: [CommonModule],
  template: `
    <div class="card">
      <div class="flex items-start justify-between">
        <div class="flex-1">
          <p class="stat-label">{{ label }}</p>
          <p class="stat-value mt-1" [class]="valueClass">{{ value }}</p>
          @if (subValue) {
            <p class="text-xs text-syslens-text-muted mt-1">{{ subValue }}</p>
          }
        </div>
        @if (icon) {
          <div class="w-10 h-10 rounded-lg flex items-center justify-center"
               [ngClass]="iconBgClass">
            <span class="w-5 h-5" [innerHTML]="icon"></span>
          </div>
        }
      </div>
      @if (showProgress) {
        <div class="mt-3 progress-bar">
          <div class="progress-fill"
               [style.width.%]="progressValue"
               [ngClass]="progressClass">
          </div>
        </div>
      }
    </div>
  `
})
export class StatCardComponent {
  @Input() label = '';
  @Input() value = '';
  @Input() subValue = '';
  @Input() icon = '';
  @Input() iconBgClass = 'bg-syslens-bg-tertiary';
  @Input() valueClass = '';
  @Input() showProgress = false;
  @Input() progressValue = 0;
  @Input() progressClass = 'bg-syslens-accent-blue';
}
