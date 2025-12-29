# Tauri IPC Commands Reference

This document describes all Tauri IPC commands available in Syslens for communication between the Angular frontend and Rust backend.

## Overview

Syslens exposes 36 Tauri commands organized into 6 categories:

| Category | Commands | Description |
|----------|----------|-------------|
| [System](#system-commands) | 7 | Device info, BIOS, OS, uptime |
| [Hardware](#hardware-commands) | 10 | CPU, memory, GPU, motherboard, monitors |
| [Network](#network-commands) | 4 | Adapters, connections, routing |
| [Storage](#storage-commands) | 6 | Disks, partitions, volumes, health |
| [Process](#process-commands) | 2 | Process list and summary |
| [Device Info](#device-info-commands) | 7 | Deep device lookup with internet fetch |

## Usage

### Frontend (TypeScript)

```typescript
import { invoke } from '@tauri-apps/api/core';

// Simple command (no parameters)
const cpuInfo = await invoke<CpuInfo>('get_cpu_info');

// Command with parameters
const adapterStats = await invoke<AdapterStats>('get_adapter_stats', {
  adapterId: 'ethernet-0'
});

// Async command
const deviceInfo = await invoke<DeviceDeepInfo>('get_device_deep_info', {
  deviceId: 'cpu-intel-i7-12700k',
  deviceType: 'Cpu'
});
```

---

## System Commands

### `get_device_info`

Returns device identification information.

**Parameters:** None

**Returns:** `DeviceInfo`

```typescript
interface DeviceInfo {
  computerName: string;
  deviceName: string;
  manufacturer: string;
  model: string;
  systemType: string;
  serialNumber: string;
  productId?: string;
  systemSku?: string;
}
```

**Example:**
```typescript
const device = await invoke<DeviceInfo>('get_device_info');
console.log(device.computerName); // "DESKTOP-ABC123"
```

---

### `get_bios_info`

Returns BIOS/UEFI firmware information.

**Parameters:** None

**Returns:** `BiosInfo`

```typescript
interface BiosInfo {
  vendor: string;
  version: string;
  firmwareVersion: string;
  releaseDate: string;
  uefiVersion?: string;
  secureBoot: boolean;
  tpmVersion?: string;
  tpmStatus: 'Enabled' | 'Disabled' | 'NotPresent' | 'Unknown';
}
```

---

### `get_boot_config`

Returns boot configuration details.

**Parameters:** None

**Returns:** `BootConfig`

```typescript
interface BootConfig {
  bootMode: 'UEFI' | 'Legacy';
  secureBootEnabled: boolean;
  bootDevice: string;
  bootOrder: string[];
  bootPriority: string;
  fastStartup: boolean;
  hibernation: boolean;
  lastBootTime: string;
  bootDurationSeconds: number;
}
```

---

### `get_os_info`

Returns operating system information.

**Parameters:** None

**Returns:** `OsInfo`

```typescript
interface OsInfo {
  name: string;
  version: string;
  build: string;
  architecture: string;
  installDate: string;
  lastUpdate?: string;
  activationStatus: 'Activated' | 'NotActivated' | 'GracePeriod' | 'Unknown';
  productKey?: string;
}
```

---

### `get_uptime`

Returns system uptime information. Called frequently for real-time updates.

**Parameters:** None

**Returns:** `SystemUptime`

```typescript
interface SystemUptime {
  uptimeSeconds: number;
  lastShutdown?: string;
  restartPending: boolean;
  sleepCount: number;
}
```

---

### `get_domain_info`

Returns domain/workgroup information.

**Parameters:** None

**Returns:** `DomainInfo`

```typescript
interface DomainInfo {
  domain?: string;
  workgroup?: string;
  domainRole: 'Workstation' | 'MemberWorkstation' | 'StandaloneServer' |
              'MemberServer' | 'BackupDomainController' | 'PrimaryDomainController';
  adSite?: string;
  logonServer?: string;
}
```

---

### `get_user_info`

Returns current user information.

**Parameters:** None

**Returns:** `UserInfo`

```typescript
interface UserInfo {
  username: string;
  userSid: string;
  userProfile: string;
  isAdmin: boolean;
  loginTime: string;
}
```

---

## Hardware Commands

### `get_cpu_info`

Returns CPU static information.

**Parameters:** None

**Returns:** `CpuInfo`

```typescript
interface CpuInfo {
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
  cache: {
    l1DataKb: number;
    l1InstructionKb: number;
    l2Kb: number;
    l3Kb: number;
  };
  socket: string;
  tdpWatts?: number;
}
```

---

### `get_cpu_metrics`

Returns real-time CPU metrics. Called frequently (every 1 second).

**Parameters:** None

**Returns:** `CpuMetrics`

```typescript
interface CpuMetrics {
  totalUsage: number;        // 0-100
  perCoreUsage: number[];    // Array of 0-100 values
  currentClockMhz: number;
  temperature?: number;      // Celsius
  powerDraw?: number;        // Watts
}
```

---

### `get_memory_info`

Returns memory static information.

**Parameters:** None

**Returns:** `MemoryInfo`

```typescript
interface MemoryInfo {
  totalBytes: number;
  usableBytes: number;
  memoryType: string;
  speedMhz: number;
  slotsUsed: number;
  slotsTotal: number;
  maxCapacityBytes: number;
  modules: MemoryModule[];
}

interface MemoryModule {
  slot: string;
  capacityBytes: number;
  manufacturer: string;
  partNumber: string;
  serialNumber: string;
  speedMhz: number;
  configuredSpeedMhz: number;
}
```

---

### `get_memory_metrics`

Returns real-time memory metrics. Called frequently (every 1 second).

**Parameters:** None

**Returns:** `MemoryMetrics`

```typescript
interface MemoryMetrics {
  inUseBytes: number;
  availableBytes: number;
  committedBytes: number;
  cachedBytes: number;
  pagedPoolBytes: number;
  nonPagedPoolBytes: number;
}
```

---

### `get_gpu_info`

Returns GPU information for all graphics adapters.

**Parameters:** None

**Returns:** `GpuInfo[]`

```typescript
interface GpuInfo {
  id: string;
  name: string;
  manufacturer: string;
  driverVersion: string;
  driverDate: string;
  driverLink?: string;
  vramBytes: number;
  currentResolution: string;
  refreshRateHz: number;
  adapterType: 'Discrete' | 'Integrated';
  pnpDeviceId?: string;
}
```

---

### `get_gpu_metrics`

Returns real-time GPU metrics. Called frequently (every 1 second).

**Parameters:** None

**Returns:** `GpuMetrics[]`

```typescript
interface GpuMetrics {
  gpuId: string;
  usagePercent: number;
  vramUsedBytes: number;
  temperature?: number;
  clockMhz?: number;
  fanSpeedPercent?: number;
  powerDraw?: number;
}
```

---

### `get_motherboard_info`

Returns motherboard information.

**Parameters:** None

**Returns:** `MotherboardInfo`

```typescript
interface MotherboardInfo {
  manufacturer: string;
  product: string;
  version: string;
  serialNumber: string;
  chipset?: string;
  biosVendor?: string;
  biosVersion?: string;
  biosReleaseDate?: string;
  supportUrl?: string;
  imageUrl?: string;
}
```

---

### `get_usb_devices`

Returns connected USB devices.

**Parameters:** None

**Returns:** `UsbDevice[]`

```typescript
interface UsbDevice {
  name: string;
  manufacturer?: string;
  vid: string;
  pid: string;
  port: string;
  speed: 'Low' | 'Full' | 'High' | 'Super' | 'SuperPlus' | 'Unknown';
  isBusPowered: boolean;
}
```

---

### `get_audio_devices`

Returns audio playback and recording devices.

**Parameters:** None

**Returns:** `AudioDevice[]`

```typescript
interface AudioDevice {
  id: string;
  name: string;
  deviceType: 'Playback' | 'Recording';
  isDefault: boolean;
  status: 'Active' | 'Disabled' | 'NotPresent';
}
```

---

### `get_monitors`

Returns connected monitors/displays.

**Parameters:** None

**Returns:** `Monitor[]`

```typescript
interface Monitor {
  id: string;
  name: string;
  manufacturer?: string;
  resolution: string;
  sizeInches?: number;
  connection: string;
  hdrSupport: boolean;
  refreshRateHz: number;
}
```

---

## Network Commands

### `get_network_adapters`

Returns all network adapters with configuration.

**Parameters:** None

**Returns:** `NetworkAdapter[]`

```typescript
interface NetworkAdapter {
  id: string;
  name: string;
  description: string;
  adapterType: 'Ethernet' | 'WiFi' | 'Virtual' | 'Loopback' | 'Unknown';
  macAddress: string;
  status: 'Up' | 'Down' | 'Disconnected' | 'Unknown';
  speedMbps?: number;
  mtu: number;
  ipv4Config?: Ipv4Config;
  ipv6Config?: Ipv6Config;
  dnsConfig: DnsConfig;
}

interface Ipv4Config {
  address: string;
  subnetMask: string;
  defaultGateway?: string;
  dhcpEnabled: boolean;
  dhcpServer?: string;
  leaseObtained?: string;
  leaseExpires?: string;
}
```

---

### `get_adapter_stats`

Returns network statistics for a specific adapter.

**Parameters:**
- `adapterId: string` - The adapter ID from `get_network_adapters`

**Returns:** `AdapterStats | null`

```typescript
interface AdapterStats {
  adapterId: string;
  bytesSent: number;
  bytesReceived: number;
  packetsSent: number;
  packetsReceived: number;
  errors: number;
  discards: number;
  timestamp: number;
}
```

**Example:**
```typescript
const stats = await invoke<AdapterStats | null>('get_adapter_stats', {
  adapterId: 'ethernet-0'
});
```

---

### `get_active_connections`

Returns active network connections (TCP/UDP).

**Parameters:** None

**Returns:** `NetworkConnection[]`

```typescript
interface NetworkConnection {
  protocol: 'TCP' | 'UDP';
  localAddress: string;
  localPort: number;
  remoteAddress: string;
  remotePort: number;
  state: 'Listen' | 'Established' | 'TimeWait' | 'CloseWait' | 'Closed' |
         'SynSent' | 'SynReceived' | 'FinWait1' | 'FinWait2' | 'LastAck' |
         'Closing' | 'Unknown';
  processName?: string;
  pid: number;
}
```

---

### `get_routing_table`

Returns the system routing table.

**Parameters:** None

**Returns:** `Route[]`

```typescript
interface Route {
  destination: string;
  netmask: string;
  gateway: string;
  interfaceId: string;
  metric: number;
  routeType: 'Local' | 'Remote' | 'Default';
}
```

---

## Storage Commands

### `get_physical_disks`

Returns physical disk information.

**Parameters:** None

**Returns:** `PhysicalDisk[]`

```typescript
interface PhysicalDisk {
  deviceId: number;
  model: string;
  manufacturer: string;
  serialNumber: string;
  mediaType: 'HDD' | 'SSD' | 'NVMe' | 'Removable' | 'Unknown';
  interfaceType: 'SATA' | 'NVMe' | 'USB' | 'SCSI' | 'Unknown';
  sizeBytes: number;
  partitionStyle: 'GPT' | 'MBR' | 'RAW';
  status: string;
  firmware: string;
}
```

---

### `get_partitions`

Returns partitions for a specific disk.

**Parameters:**
- `diskId: number` - The disk device ID from `get_physical_disks`

**Returns:** `Partition[]`

```typescript
interface Partition {
  partitionNumber: number;
  diskId: number;
  partitionType: string;
  sizeBytes: number;
  offsetBytes: number;
  isBootable: boolean;
  isActive: boolean;
}
```

---

### `get_volumes`

Returns logical volumes (drive letters).

**Parameters:** None

**Returns:** `Volume[]`

```typescript
interface Volume {
  driveLetter?: string;
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
```

---

### `get_disk_health`

Returns S.M.A.R.T. health data for a disk.

**Parameters:**
- `diskId: number` - The disk device ID

**Returns:** `DiskHealth`

```typescript
interface DiskHealth {
  deviceId: number;
  status: 'Good' | 'Warning' | 'Critical' | 'Unknown';
  temperatureCelsius?: number;
  powerOnHours?: number;
  powerCycles?: number;
  wearLevelPercent?: number;
  smartAttributes: SmartAttribute[];
}

interface SmartAttribute {
  id: number;
  name: string;
  current: number;
  worst: number;
  threshold: number;
  rawValue: string;
}
```

---

### `get_disk_performance`

Returns real-time disk I/O metrics. Called frequently (every 1 second).

**Parameters:** None

**Returns:** `DiskPerformance[]`

```typescript
interface DiskPerformance {
  deviceId: number;
  readBytesPerSec: number;
  writeBytesPerSec: number;
  readIops: number;
  writeIops: number;
  queueDepth: number;
  activeTimePercent: number;
}
```

---

### `get_network_drives`

Returns mapped network drives.

**Parameters:** None

**Returns:** `NetworkDrive[]`

```typescript
interface NetworkDrive {
  driveLetter: string;
  uncPath: string;
  server: string;
  shareName: string;
  status: 'Connected' | 'Disconnected' | 'Unknown';
}
```

---

## Process Commands

### `get_processes`

Returns list of all running processes.

**Parameters:** None

**Returns:** `ProcessInfo[]`

```typescript
interface ProcessInfo {
  pid: number;
  parentPid?: number;
  name: string;
  cpuUsage: number;        // 0-100
  memoryBytes: number;
  virtualMemoryBytes: number;
  status: string;
  user?: string;
  command: string;
  startTime: number;       // Unix timestamp
  diskReadBytes: number;
  diskWriteBytes: number;
}
```

---

### `get_process_summary`

Returns summary statistics for all processes.

**Parameters:** None

**Returns:** `ProcessSummary`

```typescript
interface ProcessSummary {
  totalCount: number;
  runningCount: number;
  sleepingCount: number;
  totalCpuUsage: number;
  totalMemoryBytes: number;
}
```

---

## Device Info Commands

These commands provide deep device information with internet lookup capabilities.

### `get_device_deep_info`

Fetches comprehensive device information from multiple sources.

**Parameters:**
- `deviceId: string` - Unique device identifier
- `deviceType: DeviceType` - Device category

**Returns:** `DeviceDeepInfo`

```typescript
type DeviceType = 'Cpu' | 'Gpu' | 'Motherboard' | 'Memory' | 'Storage';

interface DeviceDeepInfo {
  deviceId: string;
  deviceType: DeviceType;
  identifier: {
    manufacturer: string;
    model: string;
    partNumber?: string;
    serialNumber?: string;
    hardwareIds: string[];
  };
  specifications?: {
    specs: Record<string, string>;
    categories: SpecCategory[];
    description?: string;
    releaseDate?: string;
    eolDate?: string;
  };
  drivers?: {
    installedVersion?: string;
    latestVersion?: string;
    downloadUrl?: string;
    releaseDate?: string;
    releaseNotesUrl?: string;
    driverPageUrl?: string;
    updateAvailable: boolean;
  };
  documentation?: {
    productPage?: string;
    supportPage?: string;
    manuals: DocumentLink[];
    datasheets: DocumentLink[];
    firmwareUpdates: FirmwareLink[];
  };
  images?: {
    primaryImage?: string;
    gallery: string[];
    thumbnail?: string;
  };
  metadata: {
    source: 'LocalDatabase' | 'ManufacturerWebsite' | 'ThirdPartyDatabase' | 'AiAgent' | 'Cache';
    lastUpdated: string;
    expiresAt: string;
    sourceUrl?: string;
    aiConfidence?: number;
  };
}
```

**Example:**
```typescript
const cpuDetails = await invoke<DeviceDeepInfo>('get_device_deep_info', {
  deviceId: 'cpu-intel-core-i7-12700k',
  deviceType: 'Cpu'
});
```

---

### `search_device_info`

Searches for device information by manufacturer and model.

**Parameters:**
- `manufacturer: string` - Device manufacturer
- `model: string` - Device model name

**Returns:** `DeviceDeepInfo | null`

---

### `get_cached_devices`

Returns all cached device information.

**Parameters:** None

**Returns:** `DeviceDeepInfo[]`

---

### `clear_device_cache`

Clears cached device information.

**Parameters:**
- `deviceId?: string` - Optional specific device to clear
- `deviceType?: DeviceType` - Optional device type to clear

**Returns:** `void`

---

### `cleanup_device_cache`

Removes expired cache entries.

**Parameters:** None

**Returns:** `number` - Number of entries removed

---

### `get_device_database_stats`

Returns statistics about the device database and cache.

**Parameters:** None

**Returns:** `DatabaseStatsResponse`

```typescript
interface DatabaseStatsResponse {
  cacheEntries: number;
  localDbEntries: number;
  cacheSize: number;
}
```

---

## Real-Time Polling

The following commands are designed for frequent polling (every 1 second):

| Command | Purpose |
|---------|---------|
| `get_cpu_metrics` | CPU usage, clock speed, temperature |
| `get_memory_metrics` | Memory usage |
| `get_gpu_metrics` | GPU usage, VRAM, temperature |
| `get_disk_performance` | Disk I/O throughput |
| `get_uptime` | System uptime |
| `get_processes` | Process list with CPU/memory |
| `get_adapter_stats` | Network throughput |

Use `log::trace!` for these to avoid log spam (they use trace-level logging).

---

## Error Handling

Commands return errors as rejected promises. Handle them with try/catch:

```typescript
try {
  const info = await invoke<DeviceDeepInfo>('get_device_deep_info', {
    deviceId: 'unknown-device',
    deviceType: 'Cpu'
  });
} catch (error) {
  console.error('Failed to fetch device info:', error);
}
```

---

## Command Registration

Commands are registered in `src/main.rs`:

```rust
tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![
        // System
        commands::system::get_device_info,
        commands::system::get_bios_info,
        // ... etc
    ])
```

---

*Last updated: 2025-12-29*
