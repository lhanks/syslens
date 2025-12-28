import { Pipe, PipeTransform } from '@angular/core';

/**
 * Pipe to format uptime seconds into human-readable format.
 * Usage: {{ uptimeSeconds | uptime }} -> "2d 5h 30m 15s"
 */
@Pipe({
  name: 'uptime',
  standalone: true
})
export class UptimePipe implements PipeTransform {
  transform(seconds: number | null | undefined, format: 'full' | 'short' = 'full'): string {
    if (seconds === null || seconds === undefined || isNaN(seconds) || seconds < 0) {
      return '0s';
    }

    const days = Math.floor(seconds / 86400);
    const hours = Math.floor((seconds % 86400) / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const secs = Math.floor(seconds % 60);

    if (format === 'short') {
      if (days > 0) return `${days}d ${hours}h`;
      if (hours > 0) return `${hours}h ${minutes}m`;
      if (minutes > 0) return `${minutes}m ${secs}s`;
      return `${secs}s`;
    }

    const parts: string[] = [];
    if (days > 0) parts.push(`${days}d`);
    if (hours > 0) parts.push(`${hours}h`);
    if (minutes > 0) parts.push(`${minutes}m`);
    if (secs > 0 || parts.length === 0) parts.push(`${secs}s`);

    return parts.join(' ');
  }
}
