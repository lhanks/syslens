import { Injectable, signal, computed } from '@angular/core';

export interface NetworkDataPoint {
  timestamp: number;
  downloadSpeed: number;
  uploadSpeed: number;
}

const MAX_HISTORY_POINTS = 60; // 60 seconds of history

// Pre-fill with zeros so array length is constant from the start
// This prevents point spacing from changing as data fills in
const INITIAL_HISTORY: NetworkDataPoint[] = Array.from({ length: MAX_HISTORY_POINTS }, () => ({
  timestamp: 0,
  downloadSpeed: 0,
  uploadSpeed: 0
}));

/**
 * Service for tracking network speed history for graphing.
 */
@Injectable({
  providedIn: 'root'
})
export class NetworkHistoryService {
  private history = signal<NetworkDataPoint[]>([...INITIAL_HISTORY]);

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

      // Keep only the last MAX_HISTORY_POINTS by removing from front
      if (newPoints.length > MAX_HISTORY_POINTS) {
        newPoints.shift();
      }
      return newPoints;
    });
  }

  /**
   * Clear all history data (reset to initial zeros).
   */
  clear(): void {
    this.history.set([...INITIAL_HISTORY]);
  }
}
