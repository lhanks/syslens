// System configuration models

export interface DeviceInfo {
  computerName: string;
  deviceName: string;
  manufacturer: string;
  model: string;
  systemType: string;
  serialNumber: string;
  productId: string | null;
  systemSku: string | null;
}

export interface BiosInfo {
  vendor: string;
  version: string;
  firmwareVersion: string;
  releaseDate: string;
  uefiVersion: string | null;
  secureBoot: boolean;
  tpmVersion: string | null;
  tpmStatus: TpmStatus;
}

export type TpmStatus = 'Enabled' | 'Disabled' | 'NotPresent' | 'Unknown';

export interface BootConfig {
  bootMode: BootMode;
  secureBootEnabled: boolean;
  bootDevice: string;
  bootOrder: string[];
  bootPriority: string;
  fastStartup: boolean;
  hibernation: boolean;
  lastBootTime: string;
  bootDurationSeconds: number;
}

export type BootMode = 'UEFI' | 'Legacy';

export interface OsInfo {
  name: string;
  version: string;
  build: string;
  architecture: string;
  installDate: string;
  lastUpdate: string | null;
  activationStatus: ActivationStatus;
  productKey: string | null;
}

export type ActivationStatus = 'Activated' | 'NotActivated' | 'GracePeriod' | 'Unknown';

export interface SystemUptime {
  uptimeSeconds: number;
  lastShutdown: string | null;
  restartPending: boolean;
  sleepCount: number;
}

export interface DomainInfo {
  domain: string | null;
  workgroup: string | null;
  domainRole: DomainRole;
  adSite: string | null;
  logonServer: string | null;
}

export type DomainRole = 'Workstation' | 'MemberWorkstation' | 'StandaloneServer' | 'MemberServer' | 'BackupDomainController' | 'PrimaryDomainController';

export interface UserInfo {
  username: string;
  userSid: string;
  userProfile: string;
  isAdmin: boolean;
  loginTime: string;
}

export interface RestorePoint {
  sequenceNumber: number;
  description: string;
  restorePointType: RestorePointType;
  creationTime: string;
}

export type RestorePointType =
  | 'ApplicationInstall'
  | 'ApplicationUninstall'
  | 'ModifySettings'
  | 'CancelledOperation'
  | 'BackupRecovery'
  | 'DeviceDriverInstall'
  | 'ManualCheckpoint'
  | 'WindowsUpdate'
  | 'Unknown';

// Import types for SystemReport
import type {
  CpuInfo,
  GpuInfo,
  MemoryInfo,
  MotherboardInfo,
  UsbDevice,
  AudioDevice,
  Monitor,
} from './hardware.model';
import type { PhysicalDisk, Volume } from './storage.model';
import type { NetworkAdapter } from './network.model';
import type { ServiceSummary } from './service.model';

/** Complete system report containing all static system information */
export interface SystemReport {
  /** Report metadata */
  reportGeneratedAt: string;
  syslensVersion: string;

  /** System information */
  deviceInfo: DeviceInfo;
  biosInfo: BiosInfo;
  bootConfig: BootConfig;
  osInfo: OsInfo;
  uptime: SystemUptime;
  domainInfo: DomainInfo;
  userInfo: UserInfo;

  /** Hardware information */
  cpuInfo: CpuInfo;
  memoryInfo: MemoryInfo;
  gpuInfo: GpuInfo[];
  motherboardInfo: MotherboardInfo;
  usbDevices: UsbDevice[];
  audioDevices: AudioDevice[];
  monitors: Monitor[];

  /** Storage information */
  physicalDisks: PhysicalDisk[];
  volumes: Volume[];

  /** Network information */
  networkAdapters: NetworkAdapter[];

  /** Services summary */
  servicesSummary: ServiceSummary;
}
