# Syslens Storage Configuration Specification

## Overview

The storage configuration module provides comprehensive information about physical drives, partitions, volumes, and storage health. This includes both traditional HDDs and modern SSDs/NVMe drives.

## Data Categories

### 1. Physical Drives

Information about physical storage devices.

| Field | Description | Source (Windows) |
|-------|-------------|------------------|
| Model | Drive model name | WMI `Win32_DiskDrive.Model` |
| Manufacturer | Drive manufacturer | WMI `Win32_DiskDrive.Manufacturer` |
| Serial Number | Drive serial number | WMI `Win32_DiskDrive.SerialNumber` |
| Firmware Version | Drive firmware | WMI `Win32_DiskDrive.FirmwareRevision` |
| Interface | SATA, NVMe, USB, etc. | WMI `Win32_DiskDrive.InterfaceType` |
| Media Type | HDD, SSD, NVMe | WMI `MSFT_PhysicalDisk.MediaType` |
| Bus Type | SATA, NVMe, USB, RAID | WMI `MSFT_PhysicalDisk.BusType` |
| Capacity | Total drive capacity | WMI `Win32_DiskDrive.Size` |
| Bytes Per Sector | Sector size | WMI `Win32_DiskDrive.BytesPerSector` |
| Partitions | Number of partitions | WMI `Win32_DiskDrive.Partitions` |
| Status | Operational status | WMI `Win32_DiskDrive.Status` |
| Device ID | Physical device path | WMI `Win32_DiskDrive.DeviceID` |

### 2. Drive Health (S.M.A.R.T.)

Self-Monitoring, Analysis and Reporting Technology data.

| Field | Description | Threshold |
|-------|-------------|-----------|
| Health Status | Overall drive health | OK/Warning/Critical |
| Power On Hours | Total hours powered on | Informational |
| Power Cycle Count | Number of power cycles | Informational |
| Temperature | Current temperature (°C) | < 50°C nominal |
| Reallocated Sectors | Bad sectors remapped | 0 = healthy |
| Pending Sectors | Sectors awaiting reallocation | 0 = healthy |
| Uncorrectable Errors | Read/write errors | 0 = healthy |
| Wear Leveling (SSD) | SSD wear percentage | < 90% = healthy |
| Total Bytes Written | Lifetime writes (TB) | Drive-specific |
| Total Bytes Read | Lifetime reads (TB) | Informational |

### 3. Partitions

Partition information for each physical drive.

| Field | Description | Source (Windows) |
|-------|-------------|------------------|
| Index | Partition number | WMI `Win32_DiskPartition.Index` |
| Type | Partition type (Primary, Extended, Logical) | WMI `Win32_DiskPartition.Type` |
| Size | Partition size | WMI `Win32_DiskPartition.Size` |
| Offset | Byte offset on disk | WMI `Win32_DiskPartition.StartingOffset` |
| Boot Partition | Is bootable | WMI `Win32_DiskPartition.Bootable` |
| System Partition | Contains boot files | WMI `Win32_DiskPartition.PrimaryPartition` |
| GPT Type | GPT partition GUID | WMI `MSFT_Partition.GptType` |

### 4. Volumes / Logical Drives

Mounted volumes and their properties.

| Field | Description | Source (Windows) |
|-------|-------------|------------------|
| Drive Letter | Assigned drive letter | WMI `Win32_LogicalDisk.DeviceID` |
| Label | Volume label | WMI `Win32_LogicalDisk.VolumeName` |
| File System | NTFS, FAT32, exFAT, ReFS | WMI `Win32_LogicalDisk.FileSystem` |
| Total Size | Volume capacity | WMI `Win32_LogicalDisk.Size` |
| Free Space | Available space | WMI `Win32_LogicalDisk.FreeSpace` |
| Used Space | Space in use | Calculated |
| Usage Percent | Percentage used | Calculated |
| Drive Type | Fixed, Removable, Network, CD-ROM | WMI `Win32_LogicalDisk.DriveType` |
| Compressed | Compression enabled | WMI `Win32_LogicalDisk.Compressed` |
| Serial Number | Volume serial number | WMI `Win32_LogicalDisk.VolumeSerialNumber` |

### 5. Real-time Metrics

Storage performance metrics.

| Field | Description | Update Frequency |
|-------|-------------|------------------|
| Read Speed | Current read speed (MB/s) | 1 second |
| Write Speed | Current write speed (MB/s) | 1 second |
| Queue Depth | I/O queue length | 1 second |
| Active Time | Disk busy percentage | 1 second |
| Response Time | Average I/O latency (ms) | 1 second |
| IOPS | I/O operations per second | 1 second |

### 6. Optical Drives

CD/DVD/Blu-ray drive information.

| Field | Description |
|-------|-------------|
| Name | Drive model |
| Drive Letter | Assigned letter |
| Media Loaded | Disc present |
| Media Type | CD, DVD, BD |
| Capabilities | Read/Write capabilities |

### 7. Removable Storage

USB drives, SD cards, and external drives.

| Field | Description |
|-------|-------------|
| Name | Device name |
| Drive Letter | Assigned letter |
| Capacity | Total capacity |
| Free Space | Available space |
| File System | File system type |
| Connection | USB 2.0/3.0/3.1 |

### 8. Storage Spaces (if configured)

Windows Storage Spaces information.

| Field | Description |
|-------|-------------|
| Pool Name | Storage pool name |
| Health | Pool health status |
| Capacity | Total pool capacity |
| Allocated | Space allocated |
| Resiliency | Mirror/Parity/Simple |
| Virtual Disks | Associated virtual disks |

## UI Components

### Drive Overview Cards
- Card per physical drive
- Health indicator (green/yellow/red)
- Capacity bar
- Type icon (HDD/SSD/NVMe)
- Temperature display

### Volume Usage Bars
- Horizontal bar per volume
- Color coding for usage levels
  - Green: < 70%
  - Yellow: 70-85%
  - Red: > 85%
- Drive letter and label

### S.M.A.R.T. Health Panel
- Expandable health details
- Warning indicators
- Historical trend (if available)
- Predicted failure warning

### Storage Performance Graph
- Real-time read/write speed
- Dual-line chart
- Configurable time window

### Partition Map Visualization
- Visual representation of disk layout
- Partition sizes proportional
- Color coding by type

### Drive Hierarchy Tree
- Physical Drive
  - Partition 1
    - Volume C:
  - Partition 2
    - Volume D:

## Rust Implementation Notes

### Primary Crates
- `sysinfo` - Basic disk information
- `windows` crate - Windows APIs
- `wmi` crate - WMI queries for detailed info
- Performance counters for real-time metrics

### Data Collection Examples

```rust
use sysinfo::{Disks, DiskKind};

#[derive(Serialize)]
pub struct PhysicalDrive {
    pub model: String,
    pub serial_number: String,
    pub firmware: String,
    pub interface: DriveInterface,
    pub media_type: MediaType,
    pub capacity_bytes: u64,
    pub partitions: Vec<Partition>,
    pub health: DriveHealth,
}

#[derive(Serialize)]
pub enum DriveInterface {
    SATA,
    NVMe,
    USB,
    SCSI,
    Unknown,
}

#[derive(Serialize)]
pub enum MediaType {
    HDD,
    SSD,
    NVMe,
    Unknown,
}

#[derive(Serialize)]
pub struct DriveHealth {
    pub status: HealthStatus,
    pub temperature_celsius: Option<u32>,
    pub power_on_hours: Option<u64>,
    pub reallocated_sectors: Option<u32>,
    pub wear_level_percent: Option<u8>,
}

#[derive(Serialize)]
pub struct Volume {
    pub drive_letter: String,
    pub label: String,
    pub file_system: String,
    pub total_bytes: u64,
    pub free_bytes: u64,
    pub used_bytes: u64,
    pub usage_percent: f32,
    pub drive_type: DriveType,
}

#[derive(Serialize)]
pub struct StorageMetrics {
    pub drive_id: String,
    pub read_bytes_per_sec: u64,
    pub write_bytes_per_sec: u64,
    pub queue_depth: f32,
    pub active_time_percent: f32,
}

#[tauri::command]
pub fn get_physical_drives() -> Vec<PhysicalDrive> {
    // Enumerate physical drives with details
}

#[tauri::command]
pub fn get_volumes() -> Vec<Volume> {
    // Get all mounted volumes
}

#[tauri::command]
pub fn get_drive_health(drive_id: String) -> DriveHealth {
    // S.M.A.R.T. data for specific drive
}

#[tauri::command]
pub fn get_storage_metrics() -> Vec<StorageMetrics> {
    // Real-time performance metrics
}
```

### WMI Queries

```sql
-- Physical Drives
SELECT Model, Manufacturer, SerialNumber, FirmwareRevision,
       InterfaceType, Size, BytesPerSector, Partitions, Status
FROM Win32_DiskDrive

-- Partitions
SELECT Index, Type, Size, StartingOffset, Bootable
FROM Win32_DiskPartition

-- Logical Disks (Volumes)
SELECT DeviceID, VolumeName, FileSystem, Size, FreeSpace,
       DriveType, Compressed, VolumeSerialNumber
FROM Win32_LogicalDisk

-- S.M.A.R.T. Data (requires elevated)
SELECT * FROM MSStorageDriver_ATAPISmartData

-- Storage Subsystem (Windows 8+)
SELECT * FROM MSFT_PhysicalDisk
SELECT * FROM MSFT_Disk
SELECT * FROM MSFT_Partition
SELECT * FROM MSFT_Volume
```

## Refresh Behavior

| Data Type | Default Refresh | Configurable |
|-----------|-----------------|--------------|
| Physical Drives | On startup | No |
| Partitions | On startup | No |
| Volume List | On startup | Manual refresh |
| Volume Usage | 30 seconds | Yes |
| Drive Health | 5 minutes | Yes |
| Performance Metrics | 1 second | Yes |

## Warning Thresholds

| Metric | Warning | Critical |
|--------|---------|----------|
| Volume Usage | > 85% | > 95% |
| Temperature | > 45°C | > 55°C |
| Reallocated Sectors | > 0 | > 100 |
| Pending Sectors | > 0 | > 10 |
| SSD Wear Level | > 80% | > 95% |

## Export Format

### JSON Export
```json
{
  "timestamp": "2024-01-15T10:30:00Z",
  "physicalDrives": [
    {
      "model": "Samsung SSD 980 PRO 1TB",
      "serialNumber": "S5XXXX123456",
      "interface": "NVMe",
      "mediaType": "NVMe",
      "capacityGb": 931.5,
      "health": {
        "status": "OK",
        "temperatureCelsius": 35,
        "powerOnHours": 1250,
        "wearLevelPercent": 2
      }
    },
    {
      "model": "WDC WD4005FZBX-00K5WB0",
      "serialNumber": "WD-WXXXXXX",
      "interface": "SATA",
      "mediaType": "HDD",
      "capacityGb": 3725.9,
      "health": {
        "status": "OK",
        "temperatureCelsius": 32,
        "powerOnHours": 8500,
        "reallocatedSectors": 0
      }
    }
  ],
  "volumes": [
    {
      "driveLetter": "C:",
      "label": "System",
      "fileSystem": "NTFS",
      "totalGb": 237.0,
      "freeGb": 89.5,
      "usedGb": 147.5,
      "usagePercent": 62.2
    },
    {
      "driveLetter": "D:",
      "label": "Data",
      "fileSystem": "NTFS",
      "totalGb": 3725.9,
      "freeGb": 2100.3,
      "usedGb": 1625.6,
      "usagePercent": 43.6
    }
  ]
}
```

## Notes

- S.M.A.R.T. data access may require administrator privileges
- Some NVMe drives require specific APIs for health data
- USB/external drives may have limited S.M.A.R.T. support
- Performance metrics require Windows Performance Counters
- Storage Spaces information only available if feature is configured
