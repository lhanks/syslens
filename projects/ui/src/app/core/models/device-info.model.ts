/**
 * Device information models for deep device lookup.
 */

/** Device category */
export type DeviceType = 'Cpu' | 'Gpu' | 'Motherboard' | 'Memory' | 'Storage' | 'Monitor';

/** Data source for device information */
export type DataSource =
  | 'LocalDatabase'
  | 'ManufacturerWebsite'
  | 'ThirdPartyDatabase'
  | 'AiAgent'
  | 'Cache';

/** Deep device information with internet-sourced data */
export interface DeviceDeepInfo {
  deviceId: string;
  deviceType: DeviceType;
  identifier: DeviceIdentifier;
  specifications?: DeviceSpecifications;
  drivers?: DriverInfo;
  documentation?: DocumentationLinks;
  images?: ProductImages;
  metadata: DataMetadata;
}

/** Device identifying information */
export interface DeviceIdentifier {
  manufacturer: string;
  model: string;
  partNumber?: string;
  serialNumber?: string;
  hardwareIds: string[];
}

/** Detailed device specifications */
export interface DeviceSpecifications {
  specs: Record<string, string>;
  categories: SpecCategory[];
  description?: string;
  releaseDate?: string;
  eolDate?: string;
}

/** Specification category for organized display */
export interface SpecCategory {
  name: string;
  specs: SpecItem[];
}

/** Individual specification item */
export interface SpecItem {
  label: string;
  value: string;
  unit?: string;
}

/** Driver information and links */
export interface DriverInfo {
  installedVersion?: string;
  latestVersion?: string;
  downloadUrl?: string;
  releaseDate?: string;
  releaseNotesUrl?: string;
  driverPageUrl?: string;
  updateAvailable: boolean;
}

/** Documentation and support links */
export interface DocumentationLinks {
  productPage?: string;
  supportPage?: string;
  manuals: DocumentLink[];
  datasheets: DocumentLink[];
  firmwareUpdates: FirmwareLink[];
}

/** Link to a document (manual, datasheet, etc.) */
export interface DocumentLink {
  title: string;
  url: string;
  fileType: string;
  language?: string;
}

/** Link to firmware/BIOS update */
export interface FirmwareLink {
  title: string;
  version: string;
  url: string;
  releaseDate?: string;
}

/** Image type for categorizing product images */
export type ImageType = 'Product' | 'Packaging' | 'Installation' | 'Diagram' | 'DieShot' | 'Other';

/** A single image entry with metadata */
export interface ImageEntry {
  url: string;
  cachedPath?: string;
  imageType: ImageType;
  description?: string;
  width?: number;
  height?: number;
}

/** Image metadata for tracking source and freshness */
export interface ImageMetadata {
  fetchedAt: string;
  source: string;
  aiGenerated: boolean;
  cacheKey: string;
  fileSize?: number;
}

/** Product images with comprehensive metadata */
export interface ProductImages {
  primaryImage?: string;
  primaryImageCached?: string;
  gallery: ImageEntry[];
  thumbnail?: string;
  thumbnailCached?: string;
  metadata?: ImageMetadata;
}

/** Legacy product images for backwards compatibility */
export interface LegacyProductImages {
  primaryImage?: string;
  gallery: string[];
  thumbnail?: string;
}

/** Data source and freshness metadata */
export interface DataMetadata {
  source: DataSource;
  lastUpdated: string;
  expiresAt: string;
  sourceUrl?: string;
  aiConfidence?: number;
}

/** Database statistics response */
export interface DatabaseStats {
  databaseVersion: string;
  databaseLastUpdated: string;
  cpuCount: number;
  gpuCount: number;
  motherboardCount: number;
  memoryCount: number;
  storageCount: number;
  cacheTotalEntries: number;
  cacheValidEntries: number;
  cacheExpiredEntries: number;
}

// =============================================================================
// Device Enrichment Types (Phase 4)
// =============================================================================

/** Enriched device information with all available data from multiple sources */
export interface EnrichedDeviceInfo {
  identifier: DeviceIdentifier;
  deviceType: DeviceType;
  images?: ProductImages;
  specs: Record<string, string>;
  categories: SpecCategory[];
  description?: string;
  releaseDate?: string;
  productPage?: string;
  supportPage?: string;
  documentation?: DocumentationLinks;
  drivers?: DriverInfo;
  sources: string[];
  confidence: number;
  fetchedAt?: string;
  fromCache: boolean;
}

/** Information about a data source */
export interface EnrichmentSource {
  name: string;
  priority: number;
}

/** Image cache response */
export interface ImageCacheResponse {
  cacheKey: string;
  filePath: string;
  isCached: boolean;
  thumbnailPath?: string;
}

/** Image cache statistics */
export interface ImageCacheStats {
  hits: number;
  misses: number;
  downloads: number;
  downloadFailures: number;
  totalBytesCached: number;
  cachedCount: number;
  totalSizeBytes: number;
  cacheDir: string;
}

/** Cleanup response */
export interface CleanupResponse {
  imagesRemoved: number;
}
