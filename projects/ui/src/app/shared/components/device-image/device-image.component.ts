import { Component, Input, OnChanges, SimpleChanges, signal, computed } from '@angular/core';
import { CommonModule } from '@angular/common';
import { convertFileSrc } from '@tauri-apps/api/core';
import { ImageEntry } from '@core/models/device-info.model';

/**
 * Component for displaying device images with support for cached local files.
 * Handles both remote URLs and cached file paths, converting local paths
 * to Tauri asset URIs.
 */
@Component({
  selector: 'app-device-image',
  standalone: true,
  imports: [CommonModule],
  template: `
    <div
      class="relative overflow-hidden"
      [class]="containerClass"
      [style.width]="width"
      [style.height]="height"
    >
      <!-- Loading placeholder -->
      @if (loading()) {
        <div class="absolute inset-0 flex items-center justify-center bg-syslens-bg-tertiary animate-pulse">
          <svg class="w-8 h-8 text-syslens-text-muted" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
              d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z"
            />
          </svg>
        </div>
      }

      <!-- Error placeholder -->
      @if (hasError()) {
        <div class="absolute inset-0 flex flex-col items-center justify-center bg-syslens-bg-tertiary text-syslens-text-muted">
          <svg class="w-8 h-8 mb-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
              d="M6 18L18 6M6 6l12 12"
            />
          </svg>
          <span class="text-xs">Image unavailable</span>
        </div>
      }

      <!-- Actual image -->
      @if (imageSrc()) {
        <img
          [src]="imageSrc()"
          [alt]="alt"
          [class]="imageClass"
          (load)="onImageLoad()"
          (error)="onImageError()"
          [class.opacity-0]="loading()"
          [class.opacity-100]="!loading() && !hasError()"
          class="transition-opacity duration-200"
        />
      }

      <!-- Image type badge -->
      @if (showTypeBadge && imageEntry && imageEntry.imageType && imageEntry.imageType !== 'Product') {
        <span class="absolute bottom-1 right-1 px-1.5 py-0.5 text-[10px] bg-black/60 text-white rounded">
          {{ imageEntry!.imageType }}
        </span>
      }
    </div>
  `,
})
export class DeviceImageComponent implements OnChanges {
  /** Image URL (remote) */
  @Input() src?: string;

  /** Cached file path (local) */
  @Input() cachedPath?: string;

  /** Full image entry object */
  @Input() imageEntry?: ImageEntry;

  /** Alt text for accessibility */
  @Input() alt = 'Device image';

  /** Container CSS class */
  @Input() containerClass = 'rounded bg-syslens-bg-tertiary';

  /** Image CSS class */
  @Input() imageClass = 'w-full h-full object-contain';

  /** Fixed width */
  @Input() width?: string;

  /** Fixed height */
  @Input() height?: string;

  /** Show image type badge */
  @Input() showTypeBadge = false;

  /** Prefer cached path over remote URL */
  @Input() preferCached = true;

  // State signals
  loading = signal(true);
  hasError = signal(false);

  // Computed image source
  imageSrc = computed(() => {
    const entry = this.imageEntry;
    const cached = entry?.cachedPath || this.cachedPath;
    const remote = entry?.url || this.src;

    if (this.preferCached && cached) {
      // Convert local file path to Tauri asset URI
      return convertFileSrc(cached);
    }

    if (remote) {
      return remote;
    }

    if (cached) {
      return convertFileSrc(cached);
    }

    return null;
  });

  ngOnChanges(changes: SimpleChanges): void {
    // Reset state when inputs change
    if (changes['src'] || changes['cachedPath'] || changes['imageEntry']) {
      this.loading.set(true);
      this.hasError.set(false);
    }
  }

  onImageLoad(): void {
    this.loading.set(false);
    this.hasError.set(false);
  }

  onImageError(): void {
    this.loading.set(false);
    this.hasError.set(true);

    // Try fallback to remote URL if cached path failed
    if (this.preferCached && this.cachedPath && this.src) {
      this.preferCached = false;
    }
  }
}
