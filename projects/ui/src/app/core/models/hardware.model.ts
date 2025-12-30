// Hardware configuration models

export interface CpuInfo {
  name: string;
  manufacturer: string;
  architecture: string;
  family: string;
  model: string;
  stepping: string;
  physicalCores: number;
  logicalProcessors: number;
  baseClockMhz: number;
  maxClockMhz: number;
  cache: CacheInfo;
  socket: string;
  tdpWatts: number | null;
}

export interface CacheInfo {
  l1DataKb: number;
  l1InstructionKb: number;
  l2Kb: number;
  l3Kb: number;
}

export interface CpuMetrics {
  totalUsage: number;
  perCoreUsage: number[];
  currentClockMhz: number;
  temperature: number | null;
  powerDraw: number | null;
}

export interface MemoryInfo {
  totalBytes: number;
  usableBytes: number;
  memoryType: string;
  speedMhz: number;
  slotsUsed: number;
  slotsTotal: number;
  maxCapacityBytes: number;
  modules: MemoryModule[];
}

export interface MemoryModule {
  slot: string;
  capacityBytes: number;
  manufacturer: string;
  partNumber: string;
  serialNumber: string;
  speedMhz: number;
  configuredSpeedMhz: number;
}

export interface MemoryMetrics {
  inUseBytes: number;
  availableBytes: number;
  committedBytes: number;
  cachedBytes: number;
  pagedPoolBytes: number;
  nonPagedPoolBytes: number;
}

export interface GpuInfo {
  id: string;
  name: string;
  manufacturer: string;
  driverVersion: string;
  driverDate: string;
  driverLink: string | null;
  vramBytes: number;
  currentResolution: string;
  refreshRateHz: number;
  adapterType: 'Discrete' | 'Integrated';
  pnpDeviceId: string | null;
}

export interface GpuMetrics {
  gpuId: string;
  usagePercent: number;
  vramUsedBytes: number;
  temperature: number | null;
  clockMhz: number | null;
  fanSpeedPercent: number | null;
  powerDraw: number | null;
}

export interface MotherboardInfo {
  manufacturer: string;
  product: string;
  version: string;
  serialNumber: string;
  chipset: string | null;
  formFactor: string | null;
  biosVendor: string | null;
  biosVersion: string | null;
  biosReleaseDate: string | null;
  bootMode: string | null;
  secureBoot: boolean | null;
  tpmVersion: string | null;
  supportUrl: string | null;
  imageUrl: string | null;
}

export interface UsbDevice {
  name: string;
  manufacturer: string | null;
  vid: string;
  pid: string;
  port: string;
  speed: UsbSpeed;
  isBusPowered: boolean;
}

export type UsbSpeed = 'Low' | 'Full' | 'High' | 'Super' | 'SuperPlus' | 'Unknown';

export interface AudioDevice {
  id: string;
  name: string;
  deviceType: 'Playback' | 'Recording';
  isDefault: boolean;
  status: 'Active' | 'Disabled' | 'NotPresent';
}

export interface Monitor {
  id: string;
  name: string;
  manufacturer: string | null;
  resolution: string;
  sizeInches: number | null;
  connection: string;
  hdrSupport: boolean;
  refreshRateHz: number;
}
