# Syslens System Configuration Specification

## Overview

The system configuration module provides detailed device identification, BIOS/UEFI information, boot configuration, and operating system details. This information is essential for system identification, support, and troubleshooting.

## Data Categories

### 1. Device Identification

Core device information that uniquely identifies the machine.

| Field | Description | Source (Windows) |
|-------|-------------|------------------|
| **Name** | Computer hostname | `COMPUTERNAME` env var, `GetComputerNameEx` |
| **Manufacturer** | System manufacturer (e.g., Dell, HP, Lenovo) | WMI `Win32_ComputerSystem.Manufacturer` |
| **Model** | System model name/number | WMI `Win32_ComputerSystem.Model` |
| **Serial Number** | Unique system serial number | WMI `Win32_BIOS.SerialNumber` |
| System SKU | Product SKU identifier | WMI `Win32_ComputerSystemProduct.SKUNumber` |
| System Family | Product family/line | WMI `Win32_ComputerSystem.SystemFamily` |
| System Type | Architecture (x64-based PC) | WMI `Win32_ComputerSystem.SystemType` |
| UUID | Universally unique identifier | WMI `Win32_ComputerSystemProduct.UUID` |

### 2. BIOS Information

Basic Input/Output System details.

| Field | Description | Source (Windows) |
|-------|-------------|------------------|
| **BIOS Version** | BIOS version string | WMI `Win32_BIOS.SMBIOSBIOSVersion` |
| BIOS Vendor | BIOS manufacturer | WMI `Win32_BIOS.Manufacturer` |
| BIOS Release Date | BIOS release date | WMI `Win32_BIOS.ReleaseDate` |
| SMBIOS Version | SMBIOS specification version | WMI `Win32_BIOS.SMBIOSMajorVersion/MinorVersion` |
| BIOS Characteristics | Supported BIOS features | WMI `Win32_BIOS.BiosCharacteristics` |
| EC Version | Embedded Controller version | Manufacturer-specific |

### 3. UEFI Information

Unified Extensible Firmware Interface details.

| Field | Description | Source (Windows) |
|-------|-------------|------------------|
| **UEFI Version** | UEFI firmware version | `GetFirmwareEnvironmentVariable` |
| UEFI Enabled | Whether UEFI is active (vs Legacy BIOS) | `GetFirmwareType` |
| Secure Boot Status | Secure Boot enabled/disabled | Registry/WMI |
| Secure Boot Mode | Setup/User mode | Registry |
| Platform Key | PK enrolled status | UEFI variables |
| KEK | Key Exchange Key status | UEFI variables |

### 4. Firmware Version

Consolidated firmware information.

| Field | Description | Source (Windows) |
|-------|-------------|------------------|
| **Firmware Version** | Primary firmware version (BIOS or UEFI) | Combined from BIOS/UEFI |
| Firmware Type | BIOS, UEFI, or Unknown | `GetFirmwareType` |
| Firmware Vendor | Firmware provider | WMI |
| Last Update | Firmware last modified date | WMI/Registry |

### 5. Boot Configuration

System boot settings and configuration.

| Field | Description | Source (Windows) |
|-------|-------------|------------------|
| **Boot Mode** | UEFI or Legacy | `GetFirmwareType` |
| **Boot Order** | Ordered list of boot devices | UEFI variables / `bcdedit` |
| **Boot Priority** | Current boot priority settings | UEFI variables |
| Boot Drive | Current boot device | `bcdedit` |
| Boot Partition | Boot partition identifier | `bcdedit` |
| Windows Boot Loader | Boot loader path | `bcdedit` |
| Recovery Enabled | Recovery options available | `bcdedit` |
| Safe Boot | Safe mode configuration | `bcdedit` |
| Fast Startup | Fast startup enabled | Power settings |
| Hibernation | Hibernation enabled | `powercfg` |

### 6. Operating System

Windows operating system details.

| Field | Description | Source (Windows) |
|-------|-------------|------------------|
| OS Name | Full OS name | WMI `Win32_OperatingSystem.Caption` |
| OS Version | Version number (e.g., 10.0.22631) | `RtlGetVersion` |
| OS Build | Build number | Registry |
| OS Edition | Edition (Home, Pro, Enterprise) | WMI |
| OS Architecture | 32-bit or 64-bit | WMI |
| Install Date | Original installation date | WMI |
| Last Boot | Last system boot time | WMI |
| Uptime | Time since last boot | Calculated |
| Registered User | Registered owner | Registry |
| Organization | Registered organization | Registry |
| Product ID | Windows product ID | Registry |
| Product Key | Last 5 digits of product key | WMI (partial) |

### 7. Windows Update

Update and patch information.

| Field | Description |
|-------|-------------|
| Last Update Check | When updates were last checked |
| Last Update Install | When updates were last installed |
| Pending Updates | Number of pending updates |
| Update History | Recent update history |
| Windows Update Service | Service status |

### 8. Time and Locale

System time and regional settings.

| Field | Description |
|-------|-------------|
| Time Zone | Current time zone |
| UTC Offset | Offset from UTC |
| DST Active | Daylight saving time status |
| System Locale | System locale setting |
| Input Locale | Keyboard/input language |
| Date Format | Short/long date format |
| Time Format | Time format (12/24 hour) |

## UI Components

### Device Identity Card
- Prominent display of Name, Manufacturer, Model
- Serial number with copy button
- System UUID for inventory systems

### BIOS/UEFI Panel
- Clear indication of BIOS vs UEFI mode
- Firmware version with release date
- Secure Boot status indicator (green shield / red warning)

### Boot Configuration Panel
- Visual boot order display
- Current boot device highlighted
- Boot mode indicator

### OS Information Card
- Windows logo with edition
- Version and build number
- Uptime counter (live updating)

### Timeline View
- Install date
- Last boot time
- Last update

## Rust Implementation Notes

### Primary Sources
- `windows` crate for Windows API calls
- WMI queries via `wmi` crate
- Registry access via `winreg` crate

### Data Collection Examples

```rust
use windows::Win32::System::SystemInformation::*;
use wmi::{COMLibrary, WMIConnection};

#[derive(Serialize)]
pub struct DeviceInfo {
    pub name: String,
    pub manufacturer: String,
    pub model: String,
    pub serial_number: String,
    pub firmware_version: String,
    pub bios_version: String,
    pub uefi_version: Option<String>,
    pub boot_mode: BootMode,
    pub boot_order: Vec<BootDevice>,
    pub boot_priority: Vec<String>,
}

#[derive(Serialize)]
pub enum BootMode {
    Legacy,
    UEFI,
    Unknown,
}

#[tauri::command]
pub fn get_device_info() -> DeviceInfo {
    // WMI query for Win32_ComputerSystem, Win32_BIOS
}

#[tauri::command]
pub fn get_boot_configuration() -> BootConfig {
    // GetFirmwareType, UEFI variables, bcdedit parsing
}

#[tauri::command]
pub fn get_os_info() -> OsInfo {
    // RtlGetVersion, WMI Win32_OperatingSystem
}
```

### WMI Queries

```sql
-- Device Information
SELECT Manufacturer, Model, SystemType, SystemFamily
FROM Win32_ComputerSystem

-- BIOS Information
SELECT SMBIOSBIOSVersion, Manufacturer, ReleaseDate, SerialNumber
FROM Win32_BIOS

-- OS Information
SELECT Caption, Version, BuildNumber, OSArchitecture, InstallDate, LastBootUpTime
FROM Win32_OperatingSystem
```

## Refresh Behavior

| Data Type | Default Refresh | Configurable |
|-----------|-----------------|--------------|
| Device Info | On startup only | No |
| BIOS/UEFI Info | On startup only | No |
| Boot Config | On startup only | No |
| OS Info | On startup only | No |
| Uptime | 1 second | Yes |
| Time/Locale | 1 minute | No |

## Required Fields Summary

As specified in the requirements, these fields MUST be prominently displayed:

1. **Name** - Computer hostname
2. **Manufacturer** - System manufacturer
3. **Model** - System model
4. **Serial Number** - Unique serial number
5. **Firmware Version** - Primary firmware version
6. **BIOS Version** - BIOS version string
7. **UEFI Version** - UEFI version (if applicable)
8. **Boot Mode** - UEFI or Legacy
9. **Boot Order** - List of boot devices in order
10. **Boot Priority** - Priority settings for boot devices

## Export Format

### JSON Export
```json
{
  "timestamp": "2024-01-15T10:30:00Z",
  "device": {
    "name": "DESKTOP-ABC123",
    "manufacturer": "Dell Inc.",
    "model": "XPS 15 9520",
    "serialNumber": "ABC1234567",
    "uuid": "12345678-1234-1234-1234-123456789ABC"
  },
  "firmware": {
    "type": "UEFI",
    "version": "1.15.0",
    "biosVersion": "1.15.0",
    "uefiVersion": "2.7",
    "secureBootEnabled": true
  },
  "boot": {
    "mode": "UEFI",
    "order": ["Windows Boot Manager", "USB Drive", "Network"],
    "currentDevice": "Windows Boot Manager"
  },
  "os": {
    "name": "Windows 11 Pro",
    "version": "10.0.22631",
    "architecture": "64-bit",
    "installDate": "2023-06-15T14:30:00Z",
    "uptime": "5 days, 3 hours, 22 minutes"
  }
}
```
