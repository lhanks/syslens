import { Component, Input, computed, input } from '@angular/core';
import { CommonModule } from '@angular/common';

@Component({
  selector: 'app-progress-ring',
  standalone: true,
  imports: [CommonModule],
  template: `
    <div class="relative inline-flex items-center justify-center">
      <svg [attr.width]="size()" [attr.height]="size()" class="transform -rotate-90">
        <!-- Background circle -->
        <circle
          [attr.cx]="center()"
          [attr.cy]="center()"
          [attr.r]="radius()"
          [attr.stroke-width]="strokeWidth()"
          class="fill-none stroke-syslens-bg-tertiary"
        />
        <!-- Progress circle -->
        <circle
          [attr.cx]="center()"
          [attr.cy]="center()"
          [attr.r]="radius()"
          [attr.stroke-width]="strokeWidth()"
          [attr.stroke-dasharray]="circumference()"
          [attr.stroke-dashoffset]="dashOffset()"
          class="fill-none transition-all duration-300"
          [ngClass]="strokeClass()"
          stroke-linecap="round"
        />
      </svg>
      <div class="absolute inset-0 flex flex-col items-center justify-center">
        <span class="text-lg font-semibold font-mono text-syslens-text-primary">
          {{ value() }}%
        </span>
        @if (label()) {
          <span class="text-xs text-syslens-text-muted">{{ label() }}</span>
        }
      </div>
    </div>
  `
})
export class ProgressRingComponent {
  size = input(120);
  strokeWidth = input(8);
  value = input(0);
  label = input('');
  colorClass = input('stroke-syslens-accent-blue');

  center = computed(() => this.size() / 2);
  radius = computed(() => (this.size() - this.strokeWidth()) / 2);
  circumference = computed(() => 2 * Math.PI * this.radius());
  dashOffset = computed(() => {
    const progress = Math.min(100, Math.max(0, this.value()));
    return this.circumference() - (progress / 100) * this.circumference();
  });

  strokeClass = computed(() => {
    const v = this.value();
    if (v >= 90) return 'stroke-syslens-accent-red';
    if (v >= 75) return 'stroke-syslens-accent-yellow';
    return this.colorClass();
  });
}
