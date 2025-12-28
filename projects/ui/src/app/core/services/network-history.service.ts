import { Injectable, signal, computed } from '@angular/core';

export interface NetworkDataPoint {
  timestamp: number;
  downloadSpeed: number;
  uploadSpeed: number;
}

const MAX_HISTORY_POINTS = 60; // 60 seconds of history

/**
 * Service for tracking network speed history for graphing.
 */
@Injectable({
  providedIn: 'root'
})
export class NetworkHistoryService {
  private history = signal<NetworkDataPoint[]>([]);

  /** Get the current history data points */
  dataPoints = computed(() => this.history());

  /** Get the maximum download speed in history (for scaling) */
  maxDownload = computed(() => {
    const points = this.history();
    if (points.length === 0) return 1;
    return Math.max(...points.map(p => p.downloadSpeed), 1);
  });

  /** Get the maximum upload speed in history (for scaling) */
  maxUpload = computed(() => {
    const points = this.history();
    if (points.length === 0) return 1;
    return Math.max(...points.map(p => p.uploadSpeed), 1);
  });

  /** Get the overall max speed for unified scaling */
  maxSpeed = computed(() => Math.max(this.maxDownload(), this.maxUpload()));

  /**
   * Add a new data point to the history.
   */
  addDataPoint(downloadSpeed: number, uploadSpeed: number): void {
    this.history.update(points => {
      const newPoint: NetworkDataPoint = {
        timestamp: Date.now(),
        downloadSpeed,
        uploadSpeed
      };

      const newPoints = [...points, newPoint];

      // Keep only the last MAX_HISTORY_POINTS
      if (newPoints.length > MAX_HISTORY_POINTS) {
        return newPoints.slice(-MAX_HISTORY_POINTS);
      }
      return newPoints;
    });
  }

  /**
   * Clear all history data.
   */
  clear(): void {
    this.history.set([]);
  }
}
