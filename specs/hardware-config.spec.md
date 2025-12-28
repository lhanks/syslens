# Syslens Hardware Configuration Specification

## Overview

The hardware configuration module provides detailed information about the physical components of the system, including CPU, memory, GPU, motherboard, and peripheral devices. This module includes both static hardware information and real-time performance metrics.

## Data Categories

### 1. CPU (Processor)

Central Processing Unit information and real-time metrics.

#### Static Information

| Field | Description | Source (Windows) |
|-------|-------------|------------------|
| Name | Processor name/model | WMI `Win32_Processor.Name` |
| Manufacturer | CPU manufacturer (Intel, AMD) | WMI `Win32_Processor.Manufacturer` |
| Architecture | x86, x64, ARM64 | WMI `Win32_Processor.Architecture` |
| Family | Processor family | WMI `Win32_Processor.Family` |
| Model | CPU model number | CPUID |
| Stepping | CPU stepping/revision | CPUID |
| Physical Cores | Number of physical cores | WMI `Win32_Processor.NumberOfCores` |
| Logical Processors | Number of logical processors | WMI `Win32_Processor.NumberOfLogicalProcessors` |
| Base Clock | Base clock speed (GHz) | WMI `Win32_Processor.MaxClockSpeed` |
| Max Turbo | Maximum turbo frequency | Manufacturer-specific |
| L1 Cache | L1 cache size | CPUID |
| L2 Cache | L2 cache size | WMI `Win32_Processor.L2CacheSize` |
| L3 Cache | L3 cache size | WMI `Win32_Processor.L3CacheSize` |
| Socket | CPU socket type | WMI `Win32_Processor.SocketDesignation` |
| TDP | Thermal Design Power | Manufacturer data |
| Virtualization | VT-x/AMD-V support | WMI `Win32_Processor.VirtualizationFirmwareEnabled` |

#### Real-time Metrics

| Field | Description | Update Frequency |
|-------|-------------|------------------|
| Total Usage | Overall CPU utilization (%) | 1 second |
| Per-Core Usage | Usage per logical processor (%) | 1 second |
| Current Clock | Current clock speed (GHz) | 1 second |
| Temperature | CPU temperature (°C) | 1 second |
| Power Draw | Current power consumption (W) | 1 second |
| Processes | Number of running processes | 5 seconds |
| Threads | Number of running threads | 5 seconds |

### 2. Memory (RAM)

System memory information and usage.

#### Static Information

| Field | Description | Source (Windows) |
|-------|-------------|------------------|
| Total Installed | Total physical memory | `GlobalMemoryStatusEx` |
| Usable | Memory available to OS | `GlobalMemoryStatusEx` |
| Form Factor | DIMM, SO-DIMM, etc. | WMI `Win32_PhysicalMemory.FormFactor` |
| Memory Type | DDR4, DDR5, etc. | WMI `Win32_PhysicalMemory.SMBIOSMemoryType` |
| Speed | Memory speed (MHz) | WMI `Win32_PhysicalMemory.Speed` |
| Configured Speed | Actual operating speed | WMI `Win32_PhysicalMemory.ConfiguredClockSpeed` |
| Slots Used | Number of populated slots | WMI count |
| Slots Total | Total memory slots | WMI `Win32_PhysicalMemoryArray` |
| Max Capacity | Maximum supported memory | WMI `Win32_PhysicalMemoryArray.MaxCapacity` |
| Channel Mode | Single/Dual/Quad channel | Calculated |

#### Per-DIMM Information

| Field | Description |
|-------|-------------|
| Slot | Physical slot location |
| Manufacturer | DIMM manufacturer |
| Part Number | Memory part number |
| Serial Number | DIMM serial number |
| Capacity | Module capacity |
| Bank Label | Memory bank identifier |

#### Real-time Metrics

| Field | Description | Update Frequency |
|-------|-------------|------------------|
| Used | Memory in use | 1 second |
| Available | Memory available | 1 second |
| Usage (%) | Percentage used | 1 second |
| Committed | Committed memory | 1 second |
| Cached | Cached memory | 1 second |
| Paged Pool | Paged pool size | 5 seconds |
| Non-Paged Pool | Non-paged pool size | 5 seconds |

### 3. GPU (Graphics)

Graphics processing unit information.

#### Static Information

| Field | Description | Source (Windows) |
|-------|-------------|------------------|
| Name | GPU name/model | WMI `Win32_VideoController.Name` |
| Manufacturer | GPU manufacturer | WMI `Win32_VideoController.AdapterCompatibility` |
| VRAM | Video memory size | WMI `Win32_VideoController.AdapterRAM` |
| Driver Version | Graphics driver version | WMI `Win32_VideoController.DriverVersion` |
| Driver Date | Driver release date | WMI `Win32_VideoController.DriverDate` |
| Resolution | Current display resolution | WMI `Win32_VideoController.CurrentHorizontalResolution` |
| Refresh Rate | Display refresh rate | WMI `Win32_VideoController.CurrentRefreshRate` |
| Bit Depth | Color bit depth | WMI `Win32_VideoController.CurrentBitsPerPixel` |
| DAC Type | Digital-to-Analog Converter | WMI `Win32_VideoController.AdapterDACType` |

#### Real-time Metrics (if available)

| Field | Description | Update Frequency |
|-------|-------------|------------------|
| GPU Usage | GPU utilization (%) | 1 second |
| VRAM Usage | Video memory usage | 1 second |
| Temperature | GPU temperature (°C) | 1 second |
| Fan Speed | Fan speed (RPM/%) | 1 second |
| Power Draw | Power consumption (W) | 1 second |

### 4. Motherboard

System board information.

| Field | Description | Source (Windows) |
|-------|-------------|------------------|
| Manufacturer | Board manufacturer | WMI `Win32_BaseBoard.Manufacturer` |
| Model | Board model/name | WMI `Win32_BaseBoard.Product` |
| Version | Board version/revision | WMI `Win32_BaseBoard.Version` |
| Serial Number | Board serial number | WMI `Win32_BaseBoard.SerialNumber` |
| Chipset | Chipset information | Registry/WMI |
| BIOS Chip | BIOS chip type | WMI |

### 5. Audio Devices

Audio hardware information.

| Field | Description |
|-------|-------------|
| Name | Audio device name |
| Manufacturer | Device manufacturer |
| Status | Device status |
| Default Device | Is default playback/recording |
| Type | Playback/Recording/Both |

### 6. Input Devices

Keyboard, mouse, and other input devices.

| Field | Description |
|-------|-------------|
| Name | Device name |
| Type | Keyboard/Mouse/Touchpad/etc. |
| Connection | USB/Bluetooth/PS2 |
| Status | Connected/Disconnected |

### 7. USB Devices

Connected USB devices and hubs.

| Field | Description |
|-------|-------------|
| Name | Device name |
| Manufacturer | Device manufacturer |
| VID/PID | Vendor ID / Product ID |
| USB Version | USB 2.0/3.0/3.1/3.2 |
| Port | Connected port |
| Power | Power draw (mA) |

### 8. PCI Devices

PCI/PCIe devices and expansion cards.

| Field | Description |
|-------|-------------|
| Name | Device name |
| Vendor | Device vendor |
| Device ID | PCI device ID |
| Class | Device class |
| Slot | Physical slot |
| Link Speed | PCIe link speed |
| Link Width | PCIe lane width |

## UI Components

### CPU Dashboard Card
- Large usage gauge (circular or arc)
- Per-core mini bars
- Temperature with color coding
- Clock speed display
- Model name header

### Memory Dashboard Card
- Used/Free bar chart
- DIMM slot visualization
- Detailed breakdown (cached, committed)
- Upgrade recommendation hint

### GPU Dashboard Card
- GPU usage gauge
- VRAM usage bar
- Temperature (if available)
- Resolution and refresh rate

### Hardware Tree View
- Collapsible tree of all hardware
- Category grouping
- Quick status indicators

### Device List Tables
- Sortable tables for USB, PCI, Audio
- Status indicators
- Connection type icons

## Rust Implementation Notes

### Primary Crates
- `sysinfo` - CPU, memory basics
- `windows` crate - Windows-specific APIs
- `wmi` crate - WMI queries
- `raw-cpuid` - Detailed CPU information

### Data Collection Examples

```rust
use sysinfo::{System, CpuRefreshKind, MemoryRefreshKind};

#[derive(Serialize)]
pub struct CpuInfo {
    pub name: String,
    pub manufacturer: String,
    pub cores_physical: u32,
    pub cores_logical: u32,
    pub base_clock_mhz: u32,
    pub cache_l1: u64,
    pub cache_l2: u64,
    pub cache_l3: u64,
    pub architecture: String,
    pub socket: String,
}

#[derive(Serialize)]
pub struct CpuMetrics {
    pub total_usage: f32,
    pub per_core_usage: Vec<f32>,
    pub current_clock_mhz: u32,
    pub temperature_celsius: Option<f32>,
}

#[derive(Serialize)]
pub struct MemoryInfo {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub usage_percent: f32,
    pub dimms: Vec<DimmInfo>,
}

#[tauri::command]
pub fn get_cpu_info() -> CpuInfo {
    // Static CPU information
}

#[tauri::command]
pub fn get_cpu_metrics() -> CpuMetrics {
    // Real-time CPU metrics
}

#[tauri::command]
pub fn get_memory_info() -> MemoryInfo {
    // Memory information and usage
}

#[tauri::command]
pub fn get_gpu_info() -> Vec<GpuInfo> {
    // GPU information (may be multiple)
}
```

## Refresh Behavior

| Data Type | Default Refresh | Configurable |
|-----------|-----------------|--------------|
| CPU Static Info | On startup | No |
| CPU Metrics | 1 second | Yes |
| Memory Static | On startup | No |
| Memory Usage | 1 second | Yes |
| GPU Static | On startup | No |
| GPU Metrics | 1 second | Yes |
| Device Lists | On demand | No |

## Export Format

### JSON Export
```json
{
  "timestamp": "2024-01-15T10:30:00Z",
  "cpu": {
    "name": "Intel Core i7-12700K",
    "manufacturer": "Intel",
    "coresPhysical": 12,
    "coresLogical": 20,
    "baseClockMhz": 3600,
    "cacheL3Kb": 25600,
    "currentUsage": 15.5,
    "temperatureCelsius": 45
  },
  "memory": {
    "totalGb": 32,
    "usedGb": 12.5,
    "availableGb": 19.5,
    "usagePercent": 39,
    "type": "DDR5",
    "speedMhz": 5200,
    "dimms": [
      {"slot": "DIMM1", "sizeGb": 16, "manufacturer": "G.Skill"},
      {"slot": "DIMM2", "sizeGb": 16, "manufacturer": "G.Skill"}
    ]
  },
  "gpu": [
    {
      "name": "NVIDIA GeForce RTX 4080",
      "vramGb": 16,
      "driverVersion": "545.84",
      "usage": 5,
      "temperatureCelsius": 38
    }
  ]
}
```

## Notes

- Temperature and power metrics may not be available on all systems
- GPU metrics beyond basic info require manufacturer-specific APIs
- Some information requires elevated privileges
- Virtual machines may report limited/different hardware info
