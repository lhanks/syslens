# Syslens Startup Task List

Project: Syslens - Desktop System Information Dashboard
Tech Stack: Tauri 2.0 (Rust) + Angular 21 + Tailwind CSS

---

## 1. Setup

- [x] 1.1 Review project specs in specs/ folder
- [x] 1.2 Install frontend dependencies (npm install in projects/ui)
- [x] 1.3 Install backend dependencies (cargo build in projects/backend)
- [x] 1.4 Verify Angular dev server runs (npm start)
- [x] 1.5 Verify Tauri dev mode works (cargo tauri dev)
- [ ] 1.6 Set up IDE extensions (Rust Analyzer, Angular Language Service)

## 2. Infrastructure

- [ ] 2.1 Configure GitHub Actions CI pipeline
- [x] 2.2 Set up build scripts in bin/ folder
- [ ] 2.3 Configure Tauri build for Windows installer
- [ ] 2.4 Set up code signing (optional, for later)

## 3. Backend Implementation (Rust)

- [x] 3.1 Implement system info collector (device name, manufacturer, model, serial, BIOS/UEFI)
- [x] 3.2 Implement network info collector (adapters, IP config, DNS, statistics)
- [x] 3.3 Implement hardware info collector (CPU, memory, GPU)
- [x] 3.4 Implement storage info collector (drives, partitions, volumes, S.M.A.R.T.)
- [x] 3.5 Create Tauri commands for all collectors
- [ ] 3.6 Add real-time metrics polling

## 4. Frontend Implementation (Angular)

- [x] 4.1 Set up Tailwind CSS and base styles
- [x] 4.2 Create Tauri service wrapper for IPC
- [x] 4.3 Build dashboard layout with navigation
- [x] 4.4 Implement System Info feature component
- [x] 4.5 Implement Network Info feature component
- [x] 4.6 Implement Hardware Info feature component
- [x] 4.7 Implement Storage Info feature component
- [ ] 4.8 Add real-time data refresh with signals
- [x] 4.9 Create shared UI components (info cards, progress bars, copy buttons)

## 5. Quality

- [ ] 5.1 Set up Rust test framework with unit tests
- [ ] 5.2 Set up Angular test framework (Jasmine/Karma)
- [ ] 5.3 Configure ESLint for frontend
- [ ] 5.4 Configure Clippy for backend
- [ ] 5.5 Add pre-commit hooks (lint, format)

## 6. Documentation

- [x] 6.1 Complete README with build instructions
- [ ] 6.2 Document Tauri IPC commands
- [ ] 6.3 Add inline code documentation

## 7. Release

- [ ] 7.1 Test Windows build and installer
- [ ] 7.2 Create release checklist
- [ ] 7.3 Tag v1.0.0 release

---

## Progress

- Total Tasks: 32
- Completed: 20
- Remaining: 12

Last Updated: 2025-12-28
