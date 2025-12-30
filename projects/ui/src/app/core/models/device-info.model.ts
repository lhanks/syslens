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

/** Product images */
export interface ProductImages {
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
