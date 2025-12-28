# Syslens Application Specification

## Overview

**Name:** Syslens
**Type:** Desktop Application
**Platform:** Windows (primary), with potential for cross-platform support
**Technology Stack:** Tauri 2.0 (Rust backend) + Angular 21 (frontend)

## Purpose

Syslens is a modern desktop application that provides a comprehensive dashboard for viewing machine configuration information. It serves as an enhanced alternative to command-line tools like `ipconfig`, `systeminfo`, and `wmic`, presenting all system data in a real-time, visually appealing dashboard.

**Tagline:** "ipconfig on steroids"

## Target Users

- Personal use (initial release)
- Power users who need quick access to system information
- IT professionals for troubleshooting
- Developers needing system configuration details

## Core Objectives

1. **Comprehensive Information** - Display all relevant system, network, hardware, and storage information in one place
2. **Real-time Updates** - Provide live data that updates automatically
3. **Modern UI** - Clean, intuitive dashboard interface using Tailwind CSS
4. **Performance** - Lightweight application with minimal system impact
5. **Extensibility** - Platform for building additional debugging/visualization tools

## Feature Categories

### 1. Networking Information
See: [networking-config.spec.md](./networking-config.spec.md)
- Network adapters and interfaces
- IP configuration (IPv4/IPv6)
- DNS settings
- Gateway information
- MAC addresses
- Network statistics

### 2. System Information
See: [system-config.spec.md](./system-config.spec.md)
- Device identification
- BIOS/UEFI information
- Boot configuration
- Operating system details
- Firmware versions

### 3. Hardware Information
See: [hardware-config.spec.md](./hardware-config.spec.md)
- CPU details and real-time usage
- Memory (RAM) configuration and usage
- GPU information
- Motherboard details
- Peripheral devices

### 4. Storage Information
See: [storage-config.spec.md](./storage-config.spec.md)
- Physical drives
- Partitions and volumes
- Storage usage and health
- File systems

## Non-Functional Requirements

### Performance
- Application startup time: < 2 seconds
- Dashboard refresh rate: Configurable (default 1 second for real-time data)
- Memory footprint: < 100MB RAM
- CPU usage: < 2% when idle

### Usability
- Single-window dashboard interface
- Tabbed or card-based navigation between categories
- Search/filter capability for finding specific information
- Copy-to-clipboard for any data field
- Export capabilities (JSON, CSV)

### Security
- No network communication required for core functionality
- No data collection or telemetry
- Read-only access to system information (no modifications)

## Technology Decisions

### Frontend (Angular 21)
- Standalone components
- Angular Signals for reactive state management
- Tailwind CSS for styling
- @tauri-apps/api for IPC communication

### Backend (Rust/Tauri 2.0)
- `sysinfo` crate for cross-platform system information
- `windows` crate for Windows-specific APIs
- `serde` for JSON serialization
- Tauri commands for IPC

## Future Enhancements (Out of Scope for v1.0)

1. **Analysis Mode**
   - AI-powered system analysis
   - Performance recommendations
   - Upgrade suggestions with pricing (low/medium/high tiers)

2. **Software Audit**
   - Installed software inventory
   - Update availability
   - Vulnerability scanning
   - Performance analysis
   - Optimization recommendations

3. **Cross-Platform Support**
   - macOS support
   - Linux support

## Success Criteria

- All specified information categories are displayed
- Real-time data updates work reliably
- Application is stable and responsive
- User can easily navigate and find information
- Data can be exported for documentation/troubleshooting
