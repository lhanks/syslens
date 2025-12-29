import { Component, Input, computed, signal, OnDestroy, OnChanges, OnInit, SimpleChanges } from '@angular/core';
import { CommonModule } from '@angular/common';

export interface GraphDataPoint {
  value: number;
  label?: string;
}

// Color hex values matching Tailwind config
const COLOR_MAP: Record<string, string> = {
  'syslens-accent-blue': '#3b82f6',
  'syslens-accent-green': '#22c55e',
  'syslens-accent-yellow': '#eab308',
  'syslens-accent-red': '#ef4444',
  'syslens-accent-purple': '#8b5cf6',
  'syslens-accent-cyan': '#06b6d4',
};

// Default expected interval - will be auto-adjusted based on actual data timing
const DEFAULT_INTERVAL_MS = 1000;

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
            stroke="#333333"
            stroke-opacity="0.3"
          />
        }
      </g>

      <!-- Area fill for first series -->
      @if (series1AreaPath()) {
        <path
          [attr.d]="series1AreaPath()"
          [attr.fill]="getColor(series1Color)"
          fill-opacity="0.1"
        />
      }

      <!-- Area fill for second series -->
      @if (series2AreaPath()) {
        <path
          [attr.d]="series2AreaPath()"
          [attr.fill]="getColor(series2Color)"
          fill-opacity="0.1"
        />
      }

      <!-- Line for first series -->
      @if (series1Path()) {
        <path
          [attr.d]="series1Path()"
          fill="none"
          [attr.stroke]="getColor(series1Color)"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      }

      <!-- Line for second series -->
      @if (series2Path()) {
        <path
          [attr.d]="series2Path()"
          fill="none"
          [attr.stroke]="getColor(series2Color)"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
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
export class LineGraphComponent implements OnInit, OnDestroy, OnChanges {
  @Input() width = 200;
  @Input() height = 60;
  @Input() series1Color = 'syslens-accent-green';
  @Input() series2Color = 'syslens-accent-blue';
  @Input() series1: number[] = [];
  @Input() series2: number[] = [];
  @Input() maxValue = 1;

  // Animation state
  private animationFrameId: number | null = null;
  private scrollOffset = signal(0);

  // Current data
  private currentSeries1 = signal<number[]>([]);
  private currentSeries2 = signal<number[]>([]);

  // Timing for smooth scrolling - auto-detected from actual data updates
  private lastUpdateTime = 0;
  private actualIntervalMs = DEFAULT_INTERVAL_MS;
  private updateCount = 0;

  // Track previous data to detect actual changes (not just reference changes)
  private prevSeries1Length = 0;
  private prevSeries1LastValue: number | undefined;

  ngOnInit(): void {
    this.startAnimationLoop();
  }

  ngOnChanges(changes: SimpleChanges): void {
    if (changes['series1'] || changes['series2']) {
      const newSeries1 = this.series1 || [];
      const newSeries2 = this.series2 || [];

      // Detect if data actually changed (not just reference)
      // Check if the last value changed or array shifted
      const dataActuallyChanged = this.detectDataChange(newSeries1);

      if (dataActuallyChanged) {
        const now = performance.now();

        // Calculate actual interval from timing of real data updates
        if (this.lastUpdateTime > 0) {
          const elapsed = now - this.lastUpdateTime;
          // Only update interval if it's reasonable (between 100ms and 10s)
          if (elapsed >= 100 && elapsed <= 10000) {
            // Smooth the interval estimate with exponential moving average
            this.actualIntervalMs = this.updateCount < 3
              ? elapsed  // Use raw value for first few updates
              : this.actualIntervalMs * 0.7 + elapsed * 0.3;
          }
        }

        this.lastUpdateTime = now;
        this.updateCount++;

        // Update tracking state
        this.prevSeries1Length = newSeries1.length;
        this.prevSeries1LastValue = newSeries1[newSeries1.length - 1];
      }

      // Always update the current data
      this.currentSeries1.set([...newSeries1]);
      this.currentSeries2.set([...newSeries2]);
    }
  }

  ngOnDestroy(): void {
    if (this.animationFrameId !== null) {
      cancelAnimationFrame(this.animationFrameId);
      this.animationFrameId = null;
    }
  }

  /**
   * Detect if the data actually changed (new value added, array shifted)
   * vs just a reference change with same data
   */
  private detectDataChange(newSeries: number[]): boolean {
    if (newSeries.length === 0) return false;

    // First data
    if (this.prevSeries1Length === 0) return true;

    // Length changed
    if (newSeries.length !== this.prevSeries1Length) return true;

    // Last value changed (indicates new data point after shift)
    const newLastValue = newSeries[newSeries.length - 1];
    if (newLastValue !== this.prevSeries1LastValue) return true;

    return false;
  }

  gridLines = computed(() => {
    const lines: number[] = [];
    const count = 3;
    for (let i = 1; i < count; i++) {
      lines.push((this.height / count) * i);
    }
    return lines;
  });

  // Paths are computed from current data + scroll offset
  series1Path = computed(() => this.generateScrollingPath(this.currentSeries1(), this.scrollOffset()));
  series2Path = computed(() => this.generateScrollingPath(this.currentSeries2(), this.scrollOffset()));

  series1AreaPath = computed(() => this.generateScrollingAreaPath(this.currentSeries1(), this.scrollOffset()));
  series2AreaPath = computed(() => this.generateScrollingAreaPath(this.currentSeries2(), this.scrollOffset()));

  private startAnimationLoop(): void {
    const animate = (): void => {
      this.updateScrollOffset();
      this.animationFrameId = requestAnimationFrame(animate);
    };
    this.animationFrameId = requestAnimationFrame(animate);
  }

  private updateScrollOffset(): void {
    if (this.lastUpdateTime === 0) {
      this.scrollOffset.set(0);
      return;
    }

    const now = performance.now();
    const elapsed = now - this.lastUpdateTime;

    // Calculate scroll progress based on auto-detected interval
    // Clamp between 0 and 1 to prevent over-scrolling
    const scrollProgress = Math.min(Math.max(elapsed / this.actualIntervalMs, 0), 1);

    this.scrollOffset.set(scrollProgress);
  }

  private generateScrollingPath(data: number[], scrollProgress: number): string {
    if (data.length < 2) return '';

    const points = this.dataToScrollingPoints(data, scrollProgress);
    return this.catmullRomPath(points);
  }

  private generateScrollingAreaPath(data: number[], scrollProgress: number): string {
    if (data.length < 2) return '';

    const points = this.dataToScrollingPoints(data, scrollProgress);
    const linePath = this.catmullRomPath(points);

    // Close the area path
    const lastPoint = points[points.length - 1];
    const firstPoint = points[0];

    return `${linePath} L ${lastPoint.x} ${this.height} L ${firstPoint.x} ${this.height} Z`;
  }

  private dataToScrollingPoints(data: number[], scrollProgress: number): { x: number; y: number }[] {
    const maxVal = Math.max(this.maxValue, 1);
    const padding = 2;
    const effectiveHeight = this.height - padding * 2;
    const numPoints = data.length;

    if (numPoints < 2) return [];

    // Point spacing
    const pointSpacing = this.width / (numPoints - 1);

    // Scroll offset in pixels - points shift LEFT as time progresses
    const pixelOffset = scrollProgress * pointSpacing;

    const points: { x: number; y: number }[] = [];

    for (let i = 0; i < numPoints; i++) {
      // Each point shifts left by the scroll offset
      const x = (i * pointSpacing) - pixelOffset;
      const y = padding + effectiveHeight - (data[i] / maxVal) * effectiveHeight;
      points.push({ x, y });
    }

    return points;
  }

  private catmullRomPath(points: { x: number; y: number }[]): string {
    if (points.length < 2) return '';
    if (points.length === 2) {
      return `M ${points[0].x} ${points[0].y} L ${points[1].x} ${points[1].y}`;
    }

    const tension = 0.3;
    let path = `M ${points[0].x} ${points[0].y}`;

    for (let i = 0; i < points.length - 1; i++) {
      const p0 = points[Math.max(0, i - 1)];
      const p1 = points[i];
      const p2 = points[i + 1];
      const p3 = points[Math.min(points.length - 1, i + 2)];

      const cp1x = p1.x + (p2.x - p0.x) * tension;
      const cp1y = p1.y + (p2.y - p0.y) * tension;
      const cp2x = p2.x - (p3.x - p1.x) * tension;
      const cp2y = p2.y - (p3.y - p1.y) * tension;

      path += ` C ${cp1x} ${cp1y}, ${cp2x} ${cp2y}, ${p2.x} ${p2.y}`;
    }

    return path;
  }

  getColor(colorName: string): string {
    return COLOR_MAP[colorName] ?? '#22c55e';
  }
}
