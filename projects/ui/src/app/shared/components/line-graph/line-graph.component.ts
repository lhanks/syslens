import { Component, Input, computed, signal } from '@angular/core';
import { CommonModule } from '@angular/common';

export interface GraphDataPoint {
  value: number;
  label?: string;
}

@Component({
  selector: 'app-line-graph',
  standalone: true,
  imports: [CommonModule],
  template: `
    <svg [attr.width]="width" [attr.height]="height" class="overflow-visible">
      <!-- Background grid lines -->
      <g class="grid-lines">
        @for (y of gridLines(); track y) {
          <line
            [attr.x1]="0"
            [attr.y1]="y"
            [attr.x2]="width"
            [attr.y2]="y"
            stroke="currentColor"
            class="text-syslens-border"
            stroke-opacity="0.3"
          />
        }
      </g>

      <!-- Area fill for first series -->
      @if (series1Path()) {
        <path
          [attr.d]="series1AreaPath()"
          [class]="'fill-' + series1Color + '/10'"
          class="transition-all duration-300"
        />
      }

      <!-- Area fill for second series -->
      @if (series2Path()) {
        <path
          [attr.d]="series2AreaPath()"
          [class]="'fill-' + series2Color + '/10'"
          class="transition-all duration-300"
        />
      }

      <!-- Line for first series -->
      @if (series1Path()) {
        <path
          [attr.d]="series1Path()"
          fill="none"
          [class]="'stroke-' + series1Color"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
          class="transition-all duration-300"
        />
      }

      <!-- Line for second series -->
      @if (series2Path()) {
        <path
          [attr.d]="series2Path()"
          fill="none"
          [class]="'stroke-' + series2Color"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
          class="transition-all duration-300"
        />
      }
    </svg>
  `,
  styles: [`
    :host {
      display: block;
    }
  `]
})
export class LineGraphComponent {
  @Input() width = 200;
  @Input() height = 60;
  @Input() series1Color = 'syslens-accent-green';
  @Input() series2Color = 'syslens-accent-blue';

  private _series1 = signal<number[]>([]);
  private _series2 = signal<number[]>([]);
  private _maxValue = signal<number>(1);

  @Input() set series1(value: number[]) {
    this._series1.set(value);
  }

  @Input() set series2(value: number[]) {
    this._series2.set(value);
  }

  @Input() set maxValue(value: number) {
    this._maxValue.set(Math.max(value, 1));
  }

  gridLines = computed(() => {
    const lines: number[] = [];
    const count = 3;
    for (let i = 1; i < count; i++) {
      lines.push((this.height / count) * i);
    }
    return lines;
  });

  series1Path = computed(() => this.generatePath(this._series1()));
  series2Path = computed(() => this.generatePath(this._series2()));

  series1AreaPath = computed(() => this.generateAreaPath(this._series1()));
  series2AreaPath = computed(() => this.generateAreaPath(this._series2()));

  private generatePath(data: number[]): string {
    if (data.length < 2) return '';

    const points = this.dataToPoints(data);
    return points.map((p, i) => `${i === 0 ? 'M' : 'L'} ${p.x} ${p.y}`).join(' ');
  }

  private generateAreaPath(data: number[]): string {
    if (data.length < 2) return '';

    const points = this.dataToPoints(data);
    const linePath = points.map((p, i) => `${i === 0 ? 'M' : 'L'} ${p.x} ${p.y}`).join(' ');

    // Close the path along the bottom
    const lastX = points[points.length - 1].x;
    const firstX = points[0].x;

    return `${linePath} L ${lastX} ${this.height} L ${firstX} ${this.height} Z`;
  }

  private dataToPoints(data: number[]): { x: number; y: number }[] {
    const maxVal = this._maxValue();
    const padding = 2;
    const effectiveHeight = this.height - padding * 2;

    return data.map((value, index) => ({
      x: (index / Math.max(data.length - 1, 1)) * this.width,
      y: padding + effectiveHeight - (value / maxVal) * effectiveHeight
    }));
  }
}
