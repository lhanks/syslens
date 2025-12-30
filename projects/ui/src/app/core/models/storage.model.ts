// Storage configuration models

export interface PhysicalDisk {
  deviceId: number;
  model: string;
  manufacturer: string;
  serialNumber: string;
  mediaType: MediaType;
  interfaceType: InterfaceType;
  sizeBytes: number;
  partitionStyle: PartitionStyle;
  status: string;
  firmware: string;
}

export type MediaType = 'HDD' | 'SSD' | 'NVMe' | 'Removable' | 'Unknown';

export type InterfaceType = 'SATA' | 'NVMe' | 'USB' | 'SCSI' | 'Unknown';

export type PartitionStyle = 'GPT' | 'MBR' | 'RAW';

export interface Partition {
  partitionNumber: number;
  diskId: number;
  partitionType: string;
  sizeBytes: number;
  offsetBytes: number;
  isBootable: boolean;
  isActive: boolean;
}

export interface Volume {
  driveLetter: string | null;
  label: string;
  fileSystem: string;
  totalBytes: number;
  freeBytes: number;
  usedBytes: number;
  percentUsed: number;
  volumeSerial: string;
  isCompressed: boolean;
  isEncrypted: boolean;
  isSystem: boolean;
  isBoot: boolean;
}

export interface DiskHealth {
  deviceId: number;
  status: HealthStatus;
  temperatureCelsius: number | null;
  powerOnHours: number | null;
  powerCycles: number | null;
  wearLevelPercent: number | null;
  smartAttributes: SmartAttribute[];
}

export type HealthStatus = 'Good' | 'Warning' | 'Critical' | 'Unknown';

export interface SmartAttribute {
  id: number;
  name: string;
  current: number;
  worst: number;
  threshold: number;
  rawValue: string;
}

export interface DiskPerformance {
  deviceId: number;
  readBytesPerSec: number;
  writeBytesPerSec: number;
  readIops: number;
  writeIops: number;
  queueDepth: number;
  activeTimePercent: number;
}

export interface NetworkDrive {
  driveLetter: string;
  uncPath: string;
  server: string;
  shareName: string;
  status: 'Connected' | 'Disconnected' | 'Unknown';
  totalBytes: number | null;
  freeBytes: number | null;
  usedBytes: number | null;
}
