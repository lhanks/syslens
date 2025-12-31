import { Component, EventEmitter, Input, Output, OnChanges, SimpleChanges, inject } from '@angular/core';
import { CommonModule } from '@angular/common';
import { DeviceInfoService } from '@core/services/device-info.service';
import {
  DeviceDeepInfo,
  DeviceType,
  ImageEntry,
} from '@core/models/device-info.model';
import { DeviceImageComponent } from '../device-image/device-image.component';

type TabType = 'specs' | 'drivers' | 'docs' | 'images';

@Component({
  selector: 'app-device-detail-modal',
  standalone: true,
  imports: [CommonModule, DeviceImageComponent],
  template: `
    @if (isOpen) {
      <div class="fixed inset-0 z-50 flex items-center justify-center">
        <!-- Backdrop -->
        <div
          class="absolute inset-0 bg-black/70 backdrop-blur-sm"
          (click)="close()"
        ></div>

        <!-- Modal Content -->
        <div class="relative bg-syslens-bg-secondary rounded-lg shadow-xl w-full max-w-4xl max-h-[90vh] m-4 overflow-hidden flex flex-col">
          <!-- Header -->
          <div class="flex items-center justify-between p-4 border-b border-syslens-border-primary">
            <div class="flex items-center gap-3">
              <h2 class="text-xl font-semibold text-syslens-text-primary">
                {{ deviceInfo?.identifier?.model || 'Device Details' }}
              </h2>
              @if (deviceInfo) {
                <span
                  class="px-2 py-0.5 text-xs rounded-full text-white"
                  [class]="getSourceBadgeClass()"
                >
                  {{ getSourceLabel() }}
                </span>
              }
            </div>
            <button
              (click)="close()"
              class="p-1 rounded hover:bg-syslens-bg-tertiary text-syslens-text-muted hover:text-syslens-text-primary transition-colors"
            >
              <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>

          <!-- Loading State -->
          @if (loading) {
            <div class="flex items-center justify-center p-12">
              <div class="flex flex-col items-center gap-4">
                <div class="w-10 h-10 border-2 border-syslens-accent-blue border-t-transparent rounded-full animate-spin"></div>
                <p class="text-syslens-text-secondary">Fetching device information...</p>
              </div>
            </div>
          }

          <!-- Error State -->
          @if (error && !loading) {
            <div class="flex items-center justify-center p-12">
              <div class="flex flex-col items-center gap-4 text-center">
                <svg class="w-12 h-12 text-syslens-accent-red" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
                <p class="text-syslens-text-primary">{{ error }}</p>
                <button
                  (click)="refresh()"
                  class="px-4 py-2 bg-syslens-accent-blue text-white rounded hover:bg-syslens-accent-blue/80 transition-colors"
                >
                  Try Again
                </button>
              </div>
            </div>
          }

          <!-- Content -->
          @if (deviceInfo && !loading) {
            <!-- Tabs -->
            <div class="flex border-b border-syslens-border-primary">
              <button
                (click)="activeTab = 'specs'"
                class="px-4 py-3 text-sm font-medium transition-colors"
                [class.text-syslens-accent-blue]="activeTab === 'specs'"
                [class.border-b-2]="activeTab === 'specs'"
                [class.border-syslens-accent-blue]="activeTab === 'specs'"
                [class.text-syslens-text-secondary]="activeTab !== 'specs'"
              >
                Specifications
              </button>
              <button
                (click)="activeTab = 'drivers'"
                class="px-4 py-3 text-sm font-medium transition-colors"
                [class.text-syslens-accent-blue]="activeTab === 'drivers'"
                [class.border-b-2]="activeTab === 'drivers'"
                [class.border-syslens-accent-blue]="activeTab === 'drivers'"
                [class.text-syslens-text-secondary]="activeTab !== 'drivers'"
              >
                Drivers
              </button>
              <button
                (click)="activeTab = 'docs'"
                class="px-4 py-3 text-sm font-medium transition-colors"
                [class.text-syslens-accent-blue]="activeTab === 'docs'"
                [class.border-b-2]="activeTab === 'docs'"
                [class.border-syslens-accent-blue]="activeTab === 'docs'"
                [class.text-syslens-text-secondary]="activeTab !== 'docs'"
              >
                Documentation
              </button>
              <button
                (click)="activeTab = 'images'"
                class="px-4 py-3 text-sm font-medium transition-colors"
                [class.text-syslens-accent-blue]="activeTab === 'images'"
                [class.border-b-2]="activeTab === 'images'"
                [class.border-syslens-accent-blue]="activeTab === 'images'"
                [class.text-syslens-text-secondary]="activeTab !== 'images'"
              >
                Images
              </button>
            </div>

            <!-- Tab Content -->
            <div class="flex-1 overflow-y-auto p-4">
              <!-- Specifications Tab -->
              @if (activeTab === 'specs') {
                <div class="space-y-6">
                  <!-- Device Identity -->
                  <div class="bg-syslens-bg-tertiary rounded-lg p-4">
                    <h3 class="text-sm font-medium text-syslens-text-muted mb-3">Device Identity</h3>
                    <div class="grid grid-cols-2 gap-4">
                      <div>
                        <p class="text-xs text-syslens-text-muted">Manufacturer</p>
                        <p class="text-sm text-syslens-text-primary">{{ deviceInfo.identifier.manufacturer }}</p>
                      </div>
                      <div>
                        <p class="text-xs text-syslens-text-muted">Model</p>
                        <p class="text-sm text-syslens-text-primary">{{ deviceInfo.identifier.model }}</p>
                      </div>
                      @if (deviceInfo.identifier.partNumber) {
                        <div>
                          <p class="text-xs text-syslens-text-muted">Part Number</p>
                          <p class="text-sm text-syslens-text-primary">{{ deviceInfo.identifier.partNumber }}</p>
                        </div>
                      }
                      @if (deviceInfo.specifications?.releaseDate) {
                        <div>
                          <p class="text-xs text-syslens-text-muted">Release Date</p>
                          <p class="text-sm text-syslens-text-primary">{{ deviceInfo.specifications!.releaseDate }}</p>
                        </div>
                      }
                    </div>
                  </div>

                  <!-- Specification Categories -->
                  @if (deviceInfo.specifications?.categories?.length) {
                    @for (category of deviceInfo.specifications!.categories; track category.name) {
                      <div class="bg-syslens-bg-tertiary rounded-lg p-4">
                        <h3 class="text-sm font-medium text-syslens-text-muted mb-3">{{ category.name }}</h3>
                        <div class="grid grid-cols-2 gap-3">
                          @for (spec of category.specs; track spec.label) {
                            <div>
                              <p class="text-xs text-syslens-text-muted">{{ spec.label }}</p>
                              <p class="text-sm text-syslens-text-primary">
                                {{ spec.value }}
                                @if (spec.unit) {
                                  <span class="text-syslens-text-muted">{{ spec.unit }}</span>
                                }
                              </p>
                            </div>
                          }
                        </div>
                      </div>
                    }
                  }

                  <!-- Flat Specs (fallback) -->
                  @if (!deviceInfo.specifications?.categories?.length && deviceInfo.specifications?.specs) {
                    <div class="bg-syslens-bg-tertiary rounded-lg p-4">
                      <h3 class="text-sm font-medium text-syslens-text-muted mb-3">Specifications</h3>
                      <div class="grid grid-cols-2 gap-3">
                        @for (item of getSpecsArray(); track item.key) {
                          <div>
                            <p class="text-xs text-syslens-text-muted">{{ item.key }}</p>
                            <p class="text-sm text-syslens-text-primary">{{ item.value }}</p>
                          </div>
                        }
                      </div>
                    </div>
                  }

                  @if (!deviceInfo.specifications) {
                    <div class="text-center text-syslens-text-muted py-8">
                      No specifications available
                    </div>
                  }
                </div>
              }

              <!-- Drivers Tab -->
              @if (activeTab === 'drivers') {
                <div class="space-y-4">
                  @if (deviceInfo.drivers) {
                    <div class="bg-syslens-bg-tertiary rounded-lg p-4">
                      <h3 class="text-sm font-medium text-syslens-text-muted mb-3">Driver Information</h3>
                      <div class="grid grid-cols-2 gap-4">
                        @if (deviceInfo.drivers.installedVersion) {
                          <div>
                            <p class="text-xs text-syslens-text-muted">Installed Version</p>
                            <p class="text-sm text-syslens-text-primary">{{ deviceInfo.drivers.installedVersion }}</p>
                          </div>
                        }
                        @if (deviceInfo.drivers.latestVersion) {
                          <div>
                            <p class="text-xs text-syslens-text-muted">Latest Version</p>
                            <p class="text-sm text-syslens-text-primary">{{ deviceInfo.drivers.latestVersion }}</p>
                          </div>
                        }
                        @if (deviceInfo.drivers.releaseDate) {
                          <div>
                            <p class="text-xs text-syslens-text-muted">Release Date</p>
                            <p class="text-sm text-syslens-text-primary">{{ deviceInfo.drivers.releaseDate }}</p>
                          </div>
                        }
                      </div>

                      @if (deviceInfo.drivers.updateAvailable) {
                        <div class="mt-4 p-3 bg-syslens-accent-yellow/10 border border-syslens-accent-yellow rounded-lg">
                          <p class="text-sm text-syslens-accent-yellow font-medium">Update Available</p>
                        </div>
                      }

                      <div class="mt-4 flex gap-3">
                        @if (deviceInfo.drivers.downloadUrl) {
                          <a
                            [href]="deviceInfo.drivers.downloadUrl"
                            target="_blank"
                            class="px-4 py-2 bg-syslens-accent-blue text-white rounded hover:bg-syslens-accent-blue/80 transition-colors text-sm"
                          >
                            Download Driver
                          </a>
                        }
                        @if (deviceInfo.drivers.driverPageUrl) {
                          <a
                            [href]="deviceInfo.drivers.driverPageUrl"
                            target="_blank"
                            class="px-4 py-2 bg-syslens-bg-primary text-syslens-text-primary rounded hover:bg-syslens-bg-primary/80 transition-colors text-sm"
                          >
                            Driver Page
                          </a>
                        }
                        @if (deviceInfo.drivers.releaseNotesUrl) {
                          <a
                            [href]="deviceInfo.drivers.releaseNotesUrl"
                            target="_blank"
                            class="px-4 py-2 bg-syslens-bg-primary text-syslens-text-primary rounded hover:bg-syslens-bg-primary/80 transition-colors text-sm"
                          >
                            Release Notes
                          </a>
                        }
                      </div>
                    </div>
                  } @else {
                    <div class="text-center text-syslens-text-muted py-8">
                      No driver information available
                    </div>
                  }
                </div>
              }

              <!-- Documentation Tab -->
              @if (activeTab === 'docs') {
                <div class="space-y-4">
                  @if (deviceInfo.documentation) {
                    <!-- Quick Links -->
                    <div class="flex gap-3 flex-wrap">
                      @if (deviceInfo.documentation.productPage) {
                        <a
                          [href]="deviceInfo.documentation.productPage"
                          target="_blank"
                          class="px-4 py-2 bg-syslens-accent-blue text-white rounded hover:bg-syslens-accent-blue/80 transition-colors text-sm"
                        >
                          Product Page
                        </a>
                      }
                      @if (deviceInfo.documentation.supportPage) {
                        <a
                          [href]="deviceInfo.documentation.supportPage"
                          target="_blank"
                          class="px-4 py-2 bg-syslens-bg-tertiary text-syslens-text-primary rounded hover:bg-syslens-bg-primary transition-colors text-sm"
                        >
                          Support Page
                        </a>
                      }
                    </div>

                    <!-- Manuals -->
                    @if (deviceInfo.documentation.manuals?.length) {
                      <div class="bg-syslens-bg-tertiary rounded-lg p-4">
                        <h3 class="text-sm font-medium text-syslens-text-muted mb-3">Manuals</h3>
                        <div class="space-y-2">
                          @for (doc of deviceInfo.documentation.manuals; track doc.url) {
                            <a
                              [href]="doc.url"
                              target="_blank"
                              class="flex items-center gap-2 p-2 rounded hover:bg-syslens-bg-primary transition-colors group"
                            >
                              <svg class="w-5 h-5 text-syslens-text-muted" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 21h10a2 2 0 002-2V9.414a1 1 0 00-.293-.707l-5.414-5.414A1 1 0 0012.586 3H7a2 2 0 00-2 2v14a2 2 0 002 2z" />
                              </svg>
                              <span class="text-sm text-syslens-text-primary group-hover:text-syslens-accent-blue">{{ doc.title }}</span>
                              <span class="text-xs text-syslens-text-muted">({{ doc.fileType }})</span>
                            </a>
                          }
                        </div>
                      </div>
                    }

                    <!-- Datasheets -->
                    @if (deviceInfo.documentation.datasheets?.length) {
                      <div class="bg-syslens-bg-tertiary rounded-lg p-4">
                        <h3 class="text-sm font-medium text-syslens-text-muted mb-3">Datasheets</h3>
                        <div class="space-y-2">
                          @for (doc of deviceInfo.documentation.datasheets; track doc.url) {
                            <a
                              [href]="doc.url"
                              target="_blank"
                              class="flex items-center gap-2 p-2 rounded hover:bg-syslens-bg-primary transition-colors group"
                            >
                              <svg class="w-5 h-5 text-syslens-text-muted" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                              </svg>
                              <span class="text-sm text-syslens-text-primary group-hover:text-syslens-accent-blue">{{ doc.title }}</span>
                              <span class="text-xs text-syslens-text-muted">({{ doc.fileType }})</span>
                            </a>
                          }
                        </div>
                      </div>
                    }

                    <!-- Firmware Updates -->
                    @if (deviceInfo.documentation.firmwareUpdates?.length) {
                      <div class="bg-syslens-bg-tertiary rounded-lg p-4">
                        <h3 class="text-sm font-medium text-syslens-text-muted mb-3">Firmware Updates</h3>
                        <div class="space-y-2">
                          @for (fw of deviceInfo.documentation.firmwareUpdates; track fw.url) {
                            <a
                              [href]="fw.url"
                              target="_blank"
                              class="flex items-center justify-between p-2 rounded hover:bg-syslens-bg-primary transition-colors group"
                            >
                              <div class="flex items-center gap-2">
                                <svg class="w-5 h-5 text-syslens-text-muted" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
                                </svg>
                                <span class="text-sm text-syslens-text-primary group-hover:text-syslens-accent-blue">{{ fw.title }}</span>
                              </div>
                              <div class="text-xs text-syslens-text-muted">
                                v{{ fw.version }}
                                @if (fw.releaseDate) {
                                  <span class="ml-2">{{ fw.releaseDate }}</span>
                                }
                              </div>
                            </a>
                          }
                        </div>
                      </div>
                    }
                  } @else {
                    <div class="text-center text-syslens-text-muted py-8">
                      No documentation available
                    </div>
                  }
                </div>
              }

              <!-- Images Tab -->
              @if (activeTab === 'images') {
                <div class="space-y-4">
                  @if (deviceInfo.images?.primaryImage || deviceInfo.images?.gallery?.length) {
                    <!-- Primary Image -->
                    @if (deviceInfo.images!.primaryImage) {
                      <div class="bg-syslens-bg-tertiary rounded-lg p-4 flex items-center justify-center">
                        <app-device-image
                          [src]="deviceInfo.images!.primaryImage"
                          [cachedPath]="deviceInfo.images!.primaryImageCached"
                          [alt]="deviceInfo.identifier.model"
                          containerClass="max-w-full max-h-96 rounded"
                          imageClass="max-w-full max-h-96 object-contain"
                        />
                      </div>
                    }

                    <!-- Image metadata -->
                    @if (deviceInfo.images!.metadata) {
                      <div class="text-xs text-syslens-text-muted flex items-center gap-4">
                        <span>Source: {{ deviceInfo.images!.metadata.source }}</span>
                        <span>Fetched: {{ formatFetchedAt(deviceInfo.images!.metadata.fetchedAt) }}</span>
                        @if (deviceInfo.images!.metadata.aiGenerated) {
                          <span class="px-1.5 py-0.5 bg-purple-500/20 text-purple-400 rounded">AI Generated</span>
                        }
                      </div>
                    }

                    <!-- Gallery -->
                    @if (deviceInfo.images!.gallery.length) {
                      <h4 class="text-sm font-medium text-syslens-text-muted mt-4">Gallery</h4>
                      <div class="grid grid-cols-3 gap-4">
                        @for (image of deviceInfo.images!.gallery; track getImageTrackKey(image, $index)) {
                          <div
                            class="bg-syslens-bg-tertiary rounded-lg p-2 cursor-pointer hover:ring-2 hover:ring-syslens-accent-blue transition-all"
                            (click)="openGalleryImage(image)"
                          >
                            <app-device-image
                              [imageEntry]="isImageEntry(image) ? image : undefined"
                              [src]="isImageEntry(image) ? undefined : image"
                              [alt]="deviceInfo.identifier.model"
                              containerClass="w-full h-32 rounded"
                              imageClass="w-full h-full object-contain"
                              [showTypeBadge]="true"
                            />
                          </div>
                        }
                      </div>
                    }
                  } @else {
                    <div class="text-center text-syslens-text-muted py-8">
                      <svg class="w-16 h-16 mx-auto mb-4 opacity-50" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5"
                          d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z"
                        />
                      </svg>
                      <p>No images available</p>
                      <p class="text-xs mt-2">Images will be fetched from product databases when available</p>
                    </div>
                  }
                </div>
              }
            </div>

            <!-- Footer with metadata -->
            <div class="p-3 border-t border-syslens-border-primary bg-syslens-bg-tertiary text-xs text-syslens-text-muted flex justify-between items-center">
              <span>Last updated: {{ getTimeSinceUpdate() }}</span>
              <button
                (click)="refresh()"
                class="px-3 py-1 bg-syslens-bg-primary rounded hover:bg-syslens-bg-secondary transition-colors"
              >
                Refresh
              </button>
            </div>
          }
        </div>
      </div>
    }
  `,
})
export class DeviceDetailModalComponent implements OnChanges {
  private deviceInfoService = inject(DeviceInfoService);

  @Input() isOpen = false;
  @Input() deviceId = '';
  @Input() deviceType: DeviceType = 'Cpu';

  @Output() closed = new EventEmitter<void>();

  deviceInfo: DeviceDeepInfo | null = null;
  loading = false;
  error: string | null = null;
  activeTab: TabType = 'specs';

  ngOnChanges(changes: SimpleChanges): void {
    // Reload when modal opens OR when deviceId/deviceType changes while modal is open
    const isOpenChanged = changes['isOpen'] && this.isOpen;
    const deviceIdChanged = changes['deviceId'] && this.isOpen;
    const deviceTypeChanged = changes['deviceType'] && this.isOpen;

    if ((isOpenChanged || deviceIdChanged || deviceTypeChanged) && this.deviceId) {
      this.loadDeviceInfo();
    }
  }

  loadDeviceInfo(forceRefresh = false): void {
    this.loading = true;
    this.error = null;

    this.deviceInfoService
      .getDeviceDeepInfo(this.deviceId, this.deviceType, forceRefresh)
      .subscribe({
        next: (info) => {
          this.deviceInfo = info;
          this.loading = false;

          // If no images available, try to enrich the device with web sources
          const hasImages = info.images?.primaryImage || (info.images?.gallery?.length ?? 0) > 0;
          if (!hasImages && info.identifier?.manufacturer && info.identifier?.model) {
            console.log('[DeviceDetailModal] No images found, attempting enrichment...');
            this.enrichDeviceImages(info.identifier.manufacturer, info.identifier.model);
          }
        },
        error: (err) => {
          console.error('Failed to load device info:', err);
          this.error = err?.message || 'Failed to load device information';
          this.loading = false;
        },
      });
  }

  /**
   * Attempt to enrich device with images from web sources.
   * This runs in the background after the main device info is loaded.
   */
  private enrichDeviceImages(manufacturer: string, model: string): void {
    this.deviceInfoService
      .enrichDevice(this.deviceType, manufacturer, model, false)
      .subscribe({
        next: (enrichedInfo) => {
          console.log('[DeviceDetailModal] Enrichment result:', enrichedInfo);
          console.log('[DeviceDetailModal] Enrichment images:', enrichedInfo.images);
          // Update images if enrichment found any
          const hasImages = enrichedInfo.images?.primaryImage || (enrichedInfo.images?.gallery?.length ?? 0) > 0;
          console.log('[DeviceDetailModal] Has images:', hasImages);
          if (hasImages) {
            if (this.deviceInfo) {
              console.log('[DeviceDetailModal] Updating deviceInfo.images with:', enrichedInfo.images);
              this.deviceInfo = {
                ...this.deviceInfo,
                images: enrichedInfo.images,
              };
              console.log('[DeviceDetailModal] Updated deviceInfo:', this.deviceInfo);
            }
          }
        },
        error: (err) => {
          // Don't show error to user, enrichment is optional enhancement
          console.warn('[DeviceDetailModal] Enrichment failed:', err);
        },
      });
  }

  refresh(): void {
    this.loadDeviceInfo(true);
  }

  close(): void {
    this.isOpen = false;
    this.closed.emit();
  }

  getSourceLabel(): string {
    if (!this.deviceInfo) return '';
    return this.deviceInfoService.getSourceLabel(this.deviceInfo.metadata.source);
  }

  getSourceBadgeClass(): string {
    if (!this.deviceInfo) return 'bg-gray-500';
    return this.deviceInfoService.getSourceBadgeClass(this.deviceInfo.metadata.source);
  }

  getTimeSinceUpdate(): string {
    if (!this.deviceInfo) return '';
    return this.deviceInfoService.getTimeSinceUpdate(this.deviceInfo.metadata.lastUpdated);
  }

  getSpecsArray(): { key: string; value: string }[] {
    if (!this.deviceInfo?.specifications?.specs) return [];
    return Object.entries(this.deviceInfo.specifications.specs).map(([key, value]) => ({
      key,
      value,
    }));
  }

  /**
   * Check if a gallery item is an ImageEntry object or a string URL.
   */
  isImageEntry(image: ImageEntry | string): image is ImageEntry {
    return typeof image === 'object' && 'url' in image;
  }

  /**
   * Get a tracking key for gallery images.
   */
  getImageTrackKey(image: ImageEntry | string, index: number): string {
    if (this.isImageEntry(image)) {
      return image.url || `image-${index}`;
    }
    return image || `image-${index}`;
  }

  /**
   * Format the fetched-at timestamp for display.
   */
  formatFetchedAt(fetchedAt: string): string {
    try {
      const date = new Date(fetchedAt);
      return date.toLocaleDateString();
    } catch {
      return fetchedAt;
    }
  }

  /**
   * Open a gallery image in a lightbox/expanded view.
   */
  openGalleryImage(image: ImageEntry | string): void {
    const url = this.isImageEntry(image) ? image.url : image;
    // For now, open in a new tab. Could be replaced with a lightbox later.
    if (url) {
      window.open(url, '_blank');
    }
  }
}
