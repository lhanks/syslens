import { Pipe, PipeTransform } from '@angular/core';

/**
 * Pipe to format decimal numbers with a specified precision.
 * Removes trailing zeros for cleaner display.
 * Usage: {{ value | decimal }} or {{ value | decimal:2 }}
 */
@Pipe({
  name: 'decimal',
  standalone: true
})
export class DecimalPipe implements PipeTransform {
  transform(value: number | null | undefined, precision = 1): string {
    if (value === null || value === undefined || isNaN(value)) {
      return '0';
    }

    const fixed = value.toFixed(precision);
    // Remove trailing zeros after decimal point, but keep at least one decimal for non-integers
    const trimmed = fixed.replace(/(\.\d*?)0+$/, '$1').replace(/\.$/, '');
    return trimmed || '0';
  }
}
