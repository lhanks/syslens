import { Pipe, PipeTransform } from '@angular/core';

/**
 * Pipe to format bytes into human-readable format.
 * Usage: {{ byteValue | bytes }} or {{ byteValue | bytes:'MB' }}
 */
@Pipe({
  name: 'bytes',
  standalone: true
})
export class BytesPipe implements PipeTransform {
  private units = ['B', 'KB', 'MB', 'GB', 'TB', 'PB'];

  transform(bytes: number | null | undefined, targetUnit?: string, precision = 2): string {
    if (bytes === null || bytes === undefined || isNaN(bytes)) {
      return '0 B';
    }

    if (bytes === 0) {
      return '0 B';
    }

    // If a target unit is specified, convert to that unit
    if (targetUnit) {
      const unitIndex = this.units.indexOf(targetUnit.toUpperCase());
      if (unitIndex !== -1) {
        const value = bytes / Math.pow(1024, unitIndex);
        return `${value.toFixed(precision)} ${this.units[unitIndex]}`;
      }
    }

    // Auto-select the best unit
    const i = Math.floor(Math.log(Math.abs(bytes)) / Math.log(1024));
    const index = Math.min(i, this.units.length - 1);
    const value = bytes / Math.pow(1024, index);

    return `${value.toFixed(precision)} ${this.units[index]}`;
  }
}
