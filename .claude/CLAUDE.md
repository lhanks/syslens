# Syslens Project Instructions

## Project Overview

Syslens is a desktop system information dashboard for Windows, built with **Tauri 2.0** (Rust backend) and **Angular 21** (TypeScript frontend). The application provides comprehensive system monitoring, hardware details, network configuration, storage information, and process/service management.

- **Version**: 0.0.2
- **Package**: com.syslens.desktop
- **Window Size**: 1280x800 (min 900x600)
- **Platform**: Windows (primary target)

## Directory Structure

```
syslens/
├── .ai/                    # AI task/bug tracking system
├── .claude/                # Claude configuration (this file)
├── .github/workflows/      # CI/CD pipelines (ci.yml, build-release.yml)
├── .vscode/                # IDE configuration
├── bin/                    # Helper scripts (build, dev, setup, etc.)
├── docs/                   # Documentation (IDE_SETUP, TAURI_IPC_COMMANDS, etc.)
├── projects/
│   ├── ui/                 # Angular frontend application
│   ├── backend/            # Rust/Tauri backend
│   ├── website/            # Marketing website
│   └── branding/           # Brand assets
├── specs/                  # Project specifications (read-only reference)
└── README.md
```

## Development Commands

### Using Helper Scripts (Recommended)
```bash
./bin/run-app              # Run dev server (frontend + Tauri)
./bin/build                # Build complete app
./bin/build-ui             # Build Angular only
./bin/build-backend        # Build Rust only
./bin/build-installer      # Create Windows installer
./bin/dev                  # Development mode
./bin/lint                 # Run linting
./bin/test                 # Run tests
./bin/setup                # Initial project setup
```

### Frontend (Angular)
```bash
cd projects/ui
npm start          # Start dev server on port 4200
npm run build      # Production build
npm test           # Run tests (Karma/Jasmine)
npm run lint       # Run ESLint
```

### Backend (Rust)
```bash
cd projects/backend
cargo tauri dev    # Start Tauri in dev mode
cargo tauri build  # Production build
cargo test         # Run tests
cargo clippy       # Run linter
```

## Frontend Architecture

### Directory Layout (`projects/ui/src/app/`)

```
app/
├── core/
│   ├── services/           # Business logic and Tauri IPC
│   │   ├── tauri.service.ts        # Tauri invoke wrapper
│   │   ├── system.service.ts       # System info collection
│   │   ├── hardware.service.ts     # CPU, GPU, memory data
│   │   ├── network.service.ts      # Network adapter data
│   │   ├── storage.service.ts      # Disk/volume data
│   │   ├── process.service.ts      # Process listing
│   │   ├── service.service.ts      # Windows services
│   │   ├── device-info.service.ts  # Device enrichment
│   │   ├── metrics-history.service.ts  # Real-time metrics
│   │   ├── data-cache.service.ts   # Response caching
│   │   ├── state.service.ts        # Route persistence
│   │   ├── dock.service.ts         # Dock/sidebar state
│   │   ├── view-settings.service.ts # UI view preferences
│   │   └── status.service.ts       # Operation status
│   └── models/             # TypeScript interfaces
│       ├── system.model.ts         # DeviceInfo, OsInfo, etc.
│       ├── hardware.model.ts       # CpuInfo, GpuInfo, etc.
│       ├── network.model.ts        # NetworkAdapter, etc.
│       ├── storage.model.ts        # PhysicalDisk, Volume, etc.
│       ├── process.model.ts        # Process details
│       └── service.model.ts        # Windows service info
├── features/               # Route-based feature components (lazy-loaded)
│   ├── system/             # System overview
│   ├── hardware/           # Hardware details
│   ├── network/            # Network configuration
│   ├── storage/            # Storage information
│   ├── processes/          # Running processes
│   ├── services/           # Windows services
│   ├── restore-points/     # System restore points
│   ├── floating-sidebar/   # Floating performance monitor
│   └── floating-panel/     # Detail panels
├── shared/                 # Reusable components
│   ├── top-bar/            # Navigation and title bar
│   ├── sidebar/            # Left navigation
│   ├── right-sidebar/      # Details panel
│   ├── status-bar/         # Footer status
│   ├── dock/               # 4-region layout system
│   ├── stat-card/          # Statistics card
│   ├── progress-ring/      # Circular progress indicator
│   ├── line-graph/         # Real-time metrics graph
│   └── pipes/              # Data formatting (bytes, uptime, decimal)
└── app.routes.ts           # Routing configuration
```

### Key Conventions

- **Standalone Components**: All components use `standalone: true`
- **Angular Signals**: Use signals for reactive state management
- **Lazy Loading**: Routes are lazy-loaded for code splitting
- **Path Aliases**: Use `@core/`, `@features/`, `@shared/` imports
- **Template Syntax**: Use `@if`, `@for` (new Angular control flow)
- **Tailwind CSS**: Utility-first styling with custom dark theme
- **Service Injection**: All services use `@Injectable({ providedIn: 'root' })`
- **RxJS Cleanup**: Use `takeUntil()` or `DestroyRef` for subscription cleanup

### TypeScript Configuration

- **Path Aliases**: `@core/*`, `@features/*`, `@shared/*` (defined in tsconfig.json)
- **Strict Mode**: `strict: true`, `noImplicitReturns`, `noFallthroughCasesInSwitch`
- **Target**: ES2022

## Backend Architecture

### Directory Layout (`projects/backend/src/`)

```
src/
├── main.rs                 # Tauri app builder, menu, plugins
├── lib.rs                  # Module re-exports
├── state.rs                # SysInfoState - shared system cache
├── commands/               # Tauri IPC handlers
│   ├── system.rs           # Device, BIOS, OS, uptime, user
│   ├── hardware.rs         # CPU, GPU, memory, motherboard
│   ├── storage.rs          # Disks, partitions, volumes
│   ├── network.rs          # Adapters, connections, routing
│   ├── process.rs          # Running processes
│   ├── service.rs          # Windows services
│   └── device_info.rs      # Device enrichment/caching
├── collectors/             # Data aggregation
│   ├── system.rs           # SystemCollector
│   ├── hardware.rs         # HardwareCollector (WMI/Registry)
│   ├── network.rs          # NetworkCollector
│   ├── storage.rs          # StorageCollector
│   ├── process.rs          # ProcessCollector
│   └── service.rs          # ServiceCollector
├── models/                 # Serde-serialized data types
│   ├── system.rs           # DeviceInfo, BiosInfo, OsInfo, etc.
│   ├── hardware.rs         # CpuInfo, CpuMetrics, GpuInfo, etc.
│   ├── network.rs          # NetworkAdapter, ActiveConnection, etc.
│   ├── storage.rs          # PhysicalDisk, Partition, Volume, etc.
│   ├── process.rs          # Process, ProcessMetrics
│   ├── service.rs          # Service, ServiceSummary
│   └── device_info.rs      # EnrichedDeviceInfo, CachedDeviceInfo
├── services/               # Business logic
│   ├── cache_manager.rs    # Generic caching mechanism
│   ├── device_enrichment.rs # Device details enrichment
│   ├── image_cache.rs      # Image caching system
│   ├── internet_fetcher.rs # HTTP fetching with caching
│   ├── local_database.rs   # Persistent storage
│   └── device_sources/     # External data sources
│       ├── intelark.rs     # Intel ARK
│       ├── amdproduct.rs   # AMD Product database
│       ├── techpowerup.rs  # TechPowerUp databases
│       └── ...             # Other sources
└── hwids/                  # Hardware ID databases (USB/PCI)
```

### Key Conventions

- **Commands**: Decorate with `#[tauri::command]`, one file per feature area
- **Collectors**: Dedicated collector classes that gather specific system data
- **Models**: All data types derive `Serialize, Deserialize` via serde
- **State Sharing**: Use `State<SysInfoState>` for shared system cache
- **Windows-Specific**: Use `#[cfg(target_os = "windows")]` for conditional compilation
- **Error Handling**: Use `Result<T, E>` with `thiserror` crate
- **Logging**: Use `log::info!()`, `log::debug!()`, `log::trace!()`
- **Naming**: Follow Rust conventions (snake_case)

### Key Dependencies

```toml
# Tauri & IPC
tauri = "2.0"
tauri-plugin-shell = "2.0"
tauri-plugin-fs = "2.0"
tauri-plugin-dialog = "2.0"

# System Info
sysinfo = "0.31"
windows = "0.58"      # Windows API
wmi = "0.14"          # WMI queries
winreg = "0.55"       # Windows Registry

# Async
tokio = { version = "1.0", features = ["full"] }
async-trait = "0.1"

# Data
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = { version = "0.4", features = ["serde"] }

# HTTP
reqwest = { version = "0.12", features = ["rustls-tls"] }

# Error Handling
thiserror = "1.0"
anyhow = "1.0"
```

## Tauri IPC Pattern

### Frontend (TypeScript)
```typescript
import { invoke } from '@tauri-apps/api/core';

// Simple call
const cpuInfo = await invoke<CpuInfo>('get_cpu_info');

// With arguments
const process = await invoke<Process>('get_process', { pid: 1234 });

// Using TauriService wrapper
constructor(private tauri: TauriService) {}
const result = await this.tauri.invoke<CpuInfo>('get_cpu_info');
```

### Backend (Rust)
```rust
use tauri::State;
use crate::state::SysInfoState;

#[tauri::command]
pub fn get_cpu_info(state: State<SysInfoState>) -> CpuInfo {
    let collector = HardwareCollector::new();
    collector.collect_cpu_info()
}

#[tauri::command]
pub async fn get_device_info(name: String) -> Result<DeviceInfo, String> {
    // Async command with parameters
}
```

### Command Registration (main.rs)
```rust
.invoke_handler(tauri::generate_handler![
    commands::system::get_device_info,
    commands::system::get_os_info,
    commands::hardware::get_cpu_info,
    // ... more commands
])
```

## Available IPC Commands

### System Commands
- `get_device_info` - Computer name, manufacturer, model, serial
- `get_bios_info` - BIOS vendor, version, secure boot, TPM
- `get_boot_config` - Boot mode, device, duration, fast startup
- `get_os_info` - OS name, version, build, architecture
- `get_uptime` - Uptime seconds, last shutdown
- `get_user_info` - Current user, admin status
- `get_domain_info` - Domain/workgroup membership
- `get_restore_points` - System restore points
- `generate_system_report` - Complete system snapshot

### Hardware Commands
- `get_cpu_info` / `get_cpu_metrics` - CPU specs and real-time usage
- `get_memory_info` / `get_memory_metrics` - Memory details and usage
- `get_gpu_info` / `get_gpu_metrics` - GPU specs and usage
- `get_motherboard_info` - Motherboard details
- `get_usb_devices` - USB device list
- `get_audio_devices` - Audio devices
- `get_monitors` - Display information

### Storage Commands
- `get_physical_disks` - Disk drives
- `get_partitions` - Disk partitions
- `get_volumes` - Mounted volumes
- `get_disk_health` - S.M.A.R.T. status
- `get_disk_performance` - Read/write IOPS

### Network Commands
- `get_network_adapters` - Network adapters
- `get_adapter_stats` - Real-time statistics
- `get_active_connections` - TCP/UDP connections
- `get_routing_table` - Routing table entries

### Process/Service Commands
- `get_processes` - Running processes
- `get_process_summary` - Process statistics
- `kill_process` - Terminate process
- `get_services` - Windows services
- `get_service_summary` - Service statistics

## Testing

### Frontend
- **Framework**: Jasmine + Karma
- **Runner**: Chrome Headless
- **Command**: `npm test` (in projects/ui)
- **Coverage**: HTML reports in `coverage/syslens-ui/`

### Backend
- **Framework**: Rust built-in test framework
- **Command**: `cargo test --all` (in projects/backend)

## Build Outputs

- **Frontend**: `projects/ui/dist/syslens-ui/browser/`
- **Backend**: `projects/backend/target/release/`
- **Installer**: Created via `cargo tauri build` (MSI/NSIS bundles)

## CI/CD Workflows

### CI (`ci.yml`)
Runs on push to master/main or PRs:
1. Frontend lint (ESLint + TypeScript)
2. Frontend test (Karma + Jasmine)
3. Frontend build (production)
4. Backend lint (Clippy)
5. Backend build + tests
6. Format check (rustfmt)

### Build Release (`build-release.yml`)
Runs on tag push (v*) or manual trigger:
- Builds Windows MSI and NSIS bundles
- Creates draft GitHub release with artifacts

## Code Style

### EditorConfig
- UTF-8 encoding, LF line endings
- 2-space indent (TypeScript, HTML, CSS, JSON)
- 4-space indent (Rust)
- Trim trailing whitespace

### Tailwind Theme Colors
```js
// Dark theme palette (tailwind.config.js)
background: { primary: '#0f0f0f', secondary: '#1a1a1a', tertiary: '#252525' }
border: { primary: '#333333', secondary: '#404040' }
text: { primary: '#e5e5e5', secondary: '#a0a0a0', muted: '#666666' }
accent: { blue, green, yellow, red, purple, cyan, orange }
```

## Important Files Reference

### Configuration
- `projects/ui/angular.json` - Angular build configuration
- `projects/ui/tsconfig.json` - TypeScript configuration
- `projects/ui/tailwind.config.js` - Tailwind theme and plugins
- `projects/backend/Cargo.toml` - Rust dependencies
- `projects/backend/tauri.conf.json` - Tauri app configuration

### Entry Points
- `projects/ui/src/main.ts` - Angular bootstrap
- `projects/ui/src/app/app.routes.ts` - Frontend routing
- `projects/backend/src/main.rs` - Tauri app setup and menu

### Core Services
- `projects/ui/src/app/core/services/tauri.service.ts` - IPC wrapper
- `projects/backend/src/state.rs` - Shared system state cache

## Specifications Reference

The `specs/` directory contains detailed specifications (read-only reference):
- `application.spec.md` - Main application specification
- `hardware-config.spec.md` - Hardware features spec
- `networking-config.spec.md` - Network features spec
- `storage-config.spec.md` - Storage features spec
- `system-config.spec.md` - System features spec
- `_spec-device-info.md` - Device enrichment specification

## Documentation

The `docs/` directory contains additional documentation:
- `IDE_SETUP.md` - IDE configuration guide
- `TAURI_IPC_COMMANDS.md` - Complete IPC command reference
- `RELEASE_CHECKLIST.md` - Release process guide
