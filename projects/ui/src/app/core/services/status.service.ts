import { Injectable, signal, computed } from '@angular/core';

export interface StatusItem {
  id: string;
  message: string;
  startTime: number;
}

/**
 * Service for tracking and displaying operation status.
 * Uses Angular signals for reactive updates.
 */
@Injectable({
  providedIn: 'root'
})
export class StatusService {
  private activeOperations = signal<Map<string, StatusItem>>(new Map());

  /** Current status message (most recent operation, or idle) */
  currentStatus = computed(() => {
    const ops = this.activeOperations();
    if (ops.size === 0) {
      return 'Ready';
    }
    // Return the most recent operation's message
    const items = Array.from(ops.values());
    const latest = items.reduce((a, b) => a.startTime > b.startTime ? a : b);
    return latest.message;
  });

  /** Whether any operations are in progress */
  isLoading = computed(() => this.activeOperations().size > 0);

  /** Number of active operations */
  operationCount = computed(() => this.activeOperations().size);

  /**
   * Start tracking an operation.
   * @param id Unique identifier for the operation
   * @param message Status message to display
   */
  startOperation(id: string, message: string): void {
    this.activeOperations.update(ops => {
      const newMap = new Map(ops);
      newMap.set(id, { id, message, startTime: Date.now() });
      return newMap;
    });
  }

  /**
   * Complete an operation and stop tracking it.
   * @param id Unique identifier for the operation
   */
  endOperation(id: string): void {
    this.activeOperations.update(ops => {
      const newMap = new Map(ops);
      newMap.delete(id);
      return newMap;
    });
  }

  /**
   * Update the message for an existing operation.
   * @param id Unique identifier for the operation
   * @param message New status message
   */
  updateOperation(id: string, message: string): void {
    this.activeOperations.update(ops => {
      const existing = ops.get(id);
      if (existing) {
        const newMap = new Map(ops);
        newMap.set(id, { ...existing, message });
        return newMap;
      }
      return ops;
    });
  }

  /**
   * Clear all active operations.
   */
  clear(): void {
    this.activeOperations.set(new Map());
  }
}
