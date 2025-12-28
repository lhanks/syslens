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
