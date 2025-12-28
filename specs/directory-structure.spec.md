# Syslens Directory Structure Specification

## Project Root Structure

```
syslens/
├── .claude/                    # Claude Code configuration
│   └── CLAUDE.md              # Project-specific instructions
├── .github/                    # GitHub configuration
│   └── workflows/
│       └── ci.yml             # CI/CD pipeline
├── bin/                        # Build and development scripts
│   ├── build                  # Build entire project
│   ├── build-backend          # Build Rust/Tauri backend only
│   ├── build-ui               # Build Angular frontend only
│   ├── dev                    # Start development servers
│   ├── test                   # Run all tests
│   ├── lint                   # Run linting
│   ├── deploy                 # Deployment script
│   ├── db-migrate             # N/A for this project
│   └── db-seed                # N/A for this project
├── etc/                        # Configuration files
│   └── .env                   # Environment variables
├── projects/                   # Application source code
│   ├── ui/                    # Angular frontend
│   └── backend/               # Rust/Tauri backend
├── specs/                      # Project specifications
│   ├── application.spec.md
│   ├── directory-structure.spec.md
│   ├── networking-config.spec.md
│   ├── system-config.spec.md
│   ├── hardware-config.spec.md
│   └── storage-config.spec.md
├── spec/                       # Original prompt
│   └── app-prompt.md
├── .editorconfig              # Editor configuration
├── .gitignore                 # Git ignore rules
└── README.md                  # Project documentation
```

## Frontend Structure (projects/ui/)

```
projects/ui/
├── src/
│   ├── app/
│   │   ├── core/                      # Core module (singleton services)
│   │   │   ├── services/
│   │   │   │   ├── tauri.service.ts   # Tauri IPC wrapper
│   │   │   │   ├── system.service.ts  # System info service
│   │   │   │   ├── network.service.ts # Network info service
│   │   │   │   ├── hardware.service.ts # Hardware info service
│   │   │   │   └── storage.service.ts # Storage info service
│   │   │   └── models/
│   │   │       ├── system.model.ts
│   │   │       ├── network.model.ts
│   │   │       ├── hardware.model.ts
│   │   │       └── storage.model.ts
│   │   ├── features/                  # Feature modules
│   │   │   ├── dashboard/
│   │   │   │   ├── dashboard.component.ts
│   │   │   │   └── dashboard.component.html
│   │   │   ├── network/
│   │   │   │   ├── network.component.ts
│   │   │   │   ├── network.component.html
│   │   │   │   └── components/
│   │   │   │       ├── adapter-card.component.ts
│   │   │   │       └── ip-config.component.ts
│   │   │   ├── system/
│   │   │   │   ├── system.component.ts
│   │   │   │   ├── system.component.html
│   │   │   │   └── components/
│   │   │   │       ├── device-info.component.ts
│   │   │   │       └── bios-info.component.ts
│   │   │   ├── hardware/
│   │   │   │   ├── hardware.component.ts
│   │   │   │   ├── hardware.component.html
│   │   │   │   └── components/
│   │   │   │       ├── cpu-card.component.ts
│   │   │   │       ├── memory-card.component.ts
│   │   │   │       └── gpu-card.component.ts
│   │   │   └── storage/
│   │   │       ├── storage.component.ts
│   │   │       ├── storage.component.html
│   │   │       └── components/
│   │   │           ├── drive-card.component.ts
│   │   │           └── partition-list.component.ts
│   │   ├── shared/                    # Shared components
│   │   │   ├── components/
│   │   │   │   ├── info-card.component.ts
│   │   │   │   ├── data-table.component.ts
│   │   │   │   ├── progress-bar.component.ts
│   │   │   │   ├── copy-button.component.ts
│   │   │   │   └── loading-spinner.component.ts
│   │   │   ├── pipes/
│   │   │   │   ├── bytes.pipe.ts      # Format bytes to KB/MB/GB
│   │   │   │   └── uptime.pipe.ts     # Format uptime duration
│   │   │   └── directives/
│   │   │       └── tooltip.directive.ts
│   │   ├── app.component.ts
│   │   ├── app.component.html
│   │   ├── app.config.ts
│   │   └── app.routes.ts
│   ├── assets/
│   │   ├── icons/
│   │   └── images/
│   ├── styles/
│   │   └── tailwind.css
│   ├── index.html
│   └── main.ts
├── angular.json
├── package.json
├── tailwind.config.js
├── tsconfig.json
└── tsconfig.app.json
```

## Backend Structure (projects/backend/)

```
projects/backend/
├── src/
│   ├── commands/                      # Tauri command handlers
│   │   ├── mod.rs
│   │   ├── system.rs                  # System info commands
│   │   ├── network.rs                 # Network info commands
│   │   ├── hardware.rs                # Hardware info commands
│   │   └── storage.rs                 # Storage info commands
│   ├── collectors/                    # Data collection modules
│   │   ├── mod.rs
│   │   ├── system_collector.rs
│   │   ├── network_collector.rs
│   │   ├── hardware_collector.rs
│   │   └── storage_collector.rs
│   ├── models/                        # Data models
│   │   ├── mod.rs
│   │   ├── system.rs
│   │   ├── network.rs
│   │   ├── hardware.rs
│   │   └── storage.rs
│   ├── lib.rs                         # Library exports
│   └── main.rs                        # Application entry point
├── src-tauri/
│   ├── tauri.conf.json               # Tauri configuration
│   ├── capabilities/
│   │   └── default.json              # Permission capabilities
│   └── icons/                        # Application icons
├── tests/
│   ├── system_tests.rs
│   ├── network_tests.rs
│   ├── hardware_tests.rs
│   └── storage_tests.rs
├── Cargo.toml
└── Cargo.lock
```

## Build Output Structure

```
projects/backend/target/
├── debug/                            # Debug builds
│   └── syslens.exe
└── release/                          # Release builds
    ├── syslens.exe
    └── bundle/
        ├── msi/                      # Windows MSI installer
        └── nsis/                     # Windows NSIS installer

projects/ui/dist/
└── syslens/                          # Angular production build
    ├── index.html
    ├── main.js
    ├── polyfills.js
    └── styles.css
```

## Configuration Files

### Root Level
- `.editorconfig` - Editor settings (indent, line endings)
- `.gitignore` - Git ignore patterns
- `README.md` - Project documentation

### Frontend (projects/ui/)
- `angular.json` - Angular CLI configuration
- `package.json` - Node.js dependencies
- `tailwind.config.js` - Tailwind CSS configuration
- `tsconfig.json` - TypeScript configuration

### Backend (projects/backend/)
- `Cargo.toml` - Rust dependencies
- `tauri.conf.json` - Tauri application configuration

## Naming Conventions

### Files
- Angular components: `kebab-case.component.ts`
- Angular services: `kebab-case.service.ts`
- Rust modules: `snake_case.rs`
- Spec files: `kebab-case.spec.md`

### Code
- Angular: camelCase for variables/functions, PascalCase for classes/interfaces
- Rust: snake_case for variables/functions, PascalCase for types/structs
