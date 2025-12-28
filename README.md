# Syslens

A modern desktop application that provides a comprehensive dashboard for viewing machine configuration information, including networking, system, hardware, and storage details.

## Overview

Syslens is built with:
- **Frontend**: Angular 21 with Tailwind CSS
- **Backend**: Rust with Tauri 2.0
- **Platform**: Windows (primary), with potential for cross-platform support

Think of it as `ipconfig` on steroids - all the networking information you need, plus comprehensive hardware, system, and storage details in a modern, real-time dashboard.

## Features

- **System Overview**: Device identification, BIOS/UEFI info, OS details
- **Network Configuration**: Adapter info, IP settings, DNS, active connections
- **Hardware Monitoring**: CPU, memory, GPU with real-time metrics
- **Storage Information**: Disk drives, partitions, volumes, health status
- **Real-time Updates**: Live monitoring of system metrics

## Project Structure

```
syslens/
├── specs/                    # Project specifications
│   ├── application.spec.md   # Main application spec
│   ├── networking-config.spec.md
│   ├── system-config.spec.md
│   ├── hardware-config.spec.md
│   └── storage-config.spec.md
├── projects/
│   ├── ui/                   # Angular frontend
│   └── backend/              # Rust/Tauri backend
└── README.md
```

## Prerequisites

### Required Software

1. **Node.js** (v20 or later)
   - Download from: https://nodejs.org/

2. **Rust** (latest stable)
   - Install via rustup: https://rustup.rs/
   ```bash
   # Windows (PowerShell)
   winget install Rustlang.Rustup

   # Or download installer from rustup.rs
   ```

3. **Tauri Prerequisites** (Windows)
   - Microsoft Visual Studio C++ Build Tools
   - WebView2 (usually pre-installed on Windows 10/11)

   See: https://tauri.app/start/prerequisites/

## Getting Started

### 1. Install Dependencies

```bash
# Install frontend dependencies
cd projects/ui
npm install

# Verify Rust is installed
cargo --version
rustc --version
```

### 2. Development Mode

```bash
# Terminal 1: Start Angular dev server
cd projects/ui
npm start

# Terminal 2: Start Tauri dev mode
cd projects/backend
cargo tauri dev
```

### 3. Build for Production

```bash
# Build the complete application
cd projects/backend
cargo tauri build
```

The built application will be in `projects/backend/target/release/`.

## Development

### Frontend (Angular)

```bash
cd projects/ui

# Start dev server
npm start

# Run tests
npm test

# Build
npm run build
```

### Backend (Rust)

```bash
cd projects/backend

# Check code
cargo check

# Run tests
cargo test

# Build
cargo build --release
```

## Architecture

### Frontend Components

- `DashboardComponent`: Main overview with system stats
- `NetworkComponent`: Network adapter and connection details
- `SystemComponent`: Device and OS information
- `HardwareComponent`: CPU, memory, GPU details
- `StorageComponent`: Disk and volume information

### Backend Commands (Tauri IPC)

- `get_device_info()`: Device identification
- `get_cpu_info()`, `get_cpu_metrics()`: CPU data
- `get_memory_info()`, `get_memory_metrics()`: Memory data
- `get_network_adapters()`, `get_adapter_stats()`: Network data
- `get_volumes()`, `get_physical_disks()`: Storage data

## Roadmap

### Core Features
- [ ] Complete network configuration display
- [ ] Add real-time performance graphs
- [ ] Implement S.M.A.R.T. disk health monitoring
- [ ] Add dark/light theme toggle
- [ ] Export system report feature

### Future: Analysis Mode
- [ ] User-specified machine usage profile
- [ ] AI-powered system analysis based on usage
- [ ] Hardware upgrade recommendations with pricing tiers
- [ ] Internet research for component prices

### Future: Software Analysis
- [ ] Installed software inventory
- [ ] Software update status and recommendations
- [ ] Security vulnerability scanning
- [ ] Performance optimization suggestions

## License

MIT License - See LICENSE file for details.
