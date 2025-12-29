# Syslens Startup Task List

Project: Syslens - Desktop System Information Dashboard
Tech Stack: Tauri 2.0 (Rust) + Angular 21 + Tailwind CSS

---

## 0 New Tasks
** IMPORTANT ** Process the tasks below (in this section).  Expand them if needed and put them in the appropriate sections below.
** IMPORTANT ** Do not remove this section. Just clear out the tasks once processed.

(no new tasks)

## 1. Setup

- [ ] 1.6 Set up IDE extensions (Rust Analyzer, Angular Language Service)

## 2. Infrastructure

- [x] 2.1 Configure GitHub Actions CI pipeline
- [x] 2.2 Add state persistence service (cache latest metrics/data for faster startup)
- [x] 2.3 Configure Tauri build for Windows installer
- [ ] 2.4 Set up code signing (optional, for later)

## 3. Backend Implementation (Rust)

- [x] 3.6 Add real-time metrics polling
- [x] 3.17 Fix display detection (EDID parsing for actual monitor model)
- [x] 3.18 Fix processor cache size reporting (WMI for L1/L2/L3 cache)

## 4. Frontend Implementation (Angular)

- [x] 4.8 Add real-time data refresh with signals
- [~] 4.27 Add deep device information with internet lookup (Phase 2 complete: internet fetching)
- [x] 4.29 Fix line graphs to have fixed height (prevent container resize on update)
- [x] 4.31 Add process details dialog (comprehensive process information on click)

## 5. Quality

- [x] 5.1 Set up Rust test framework with unit tests
- [x] 5.2 Set up Angular test framework (Jasmine/Karma)
- [x] 5.3 Configure ESLint for frontend
- [x] 5.4 Configure Clippy for backend
- [x] 5.5 Add pre-commit hooks (lint, format)

## 6. Documentation

- [ ] 6.2 Document Tauri IPC commands
- [ ] 6.3 Add inline code documentation

## 7. Release

- [ ] 7.1 Test Windows build and installer
- [ ] 7.2 Create release checklist
- [ ] 7.3 Tag v1.0.0 release

## 8. Branding

- [x] 8.1 Create app name
- [x] 8.2 Design logo and app icon
- [x] 8.3 Define color scheme
- [x] 8.4 Select typography/fonts
- [x] 8.5 Create style guide
- [x] 8.6 Generate brand assets (icons, splash screens)
- [x] 8.7 Create brand guidelines document

## 9. Future Enhancements (Requires Spec Design)

- [ ] 9.1 Get spec agent to design hardware database and AI-enhanced features
    - Local hardware device database (JSON) with images/specs/drivers/documentation
    - AI agent for searching device information not in database
    - Process history database with AI-powered usage pattern analysis
    - Settings dialog for API keys (AI agent, internet fetcher)

---

## Progress

- Total Tasks: 30
- Completed: 21
- Remaining: 9

Last Updated: 2025-12-29 (Session 6)

---

## Archive (Completed Tasks)

### Setup
- [x] 1.1 Review project specs in specs/ folder
- [x] 1.2 Install frontend dependencies (npm install in projects/ui)
- [x] 1.3 Install backend dependencies (cargo build in projects/backend)
- [x] 1.4 Verify Angular dev server runs (npm start)
- [x] 1.5 Verify Tauri dev mode works (cargo tauri dev)

### Infrastructure
- [x] 2.2 Set up build scripts in bin/ folder
- [x] 2.2 Add state persistence service (DataCacheService with localStorage + cache-first pattern)
- [x] 2.5 Create run-app-prod script for production builds with bundled assets

### Backend Implementation (Rust)
- [x] 3.1 Implement system info collector (device name, manufacturer, model, serial, BIOS/UEFI)
- [x] 3.2 Implement network info collector (adapters, IP config, DNS, statistics)
- [x] 3.3 Implement hardware info collector (CPU, memory, GPU)
- [x] 3.4 Implement storage info collector (drives, partitions, volumes, S.M.A.R.T.)
- [x] 3.5 Create Tauri commands for all collectors
- [x] 3.7 Enhance device information collection (manufacturer, model, serial, BIOS/UEFI for all devices)
- [x] 3.8 Add complete IP configuration data (ipconfig /all) for network connections
- [x] 3.9 Add GPU vendor information (manufacturer, model, driver version, driver link)
- [x] 3.10 Add Memory vendor information (manufacturer, part number, serial, speed)
- [x] 3.11 Add detailed Motherboard information (manufacturer, model, version, serial, configuration, BIOS info)
- [x] 3.12 Gather motherboard image URL from manufacturer website based on model
- [x] 3.13 Add driver/documentation links for all hardware devices
- [x] 3.14 Fix memory speed to show XMP/rated speed instead of JEDEC base speed
- [x] 3.15 Implement Windows monitor/display information collector (WMI + GDI fallback)

### Frontend Implementation (Angular)
- [x] 4.1 Set up Tailwind CSS and base styles
- [x] 4.2 Create Tauri service wrapper for IPC
- [x] 4.3 Build dashboard layout with navigation
- [x] 4.4 Implement System Info feature component
- [x] 4.5 Implement Network Info feature component
- [x] 4.6 Implement Hardware Info feature component
- [x] 4.7 Implement Storage Info feature component
- [x] 4.9 Create shared UI components (info cards, progress bars, copy buttons)
- [x] 4.10 Format numeric values to show 1 decimal place (avoid long digit strings)
- [x] 4.11 Add click-through from summary to detailed information views
- [x] 4.12 Implement progressive data loading (prioritize current tab, preload others)
- [x] 4.13 Display GPU vendor information in Hardware Info component
- [x] 4.14 Display Memory vendor information in Hardware Info component
- [x] 4.15 Display detailed Motherboard information in Hardware Info component
- [x] 4.16 Add clickable driver/documentation links for all devices
- [x] 4.17 Display motherboard image if available
- [x] 4.18 Fix network stats real-time updates on dashboard (download/upload speeds)
- [x] 4.19 Dim primary text color for reduced eye strain (#ffffff -> #e5e5e5)
- [x] 4.20 Parallelize dashboard data loading for faster startup (~500-1000ms improvement)
- [x] 4.21 Restore last active route on app restart (state persistence)
- [x] 4.22 Add status bar showing data capture operations in progress
- [x] 4.23 Add real-time network traffic graph on dashboard (60-second history)
- [x] 4.24 Add process list tab with real-time monitoring (sortable, searchable, paginated)
- [x] 4.25 Add system summary stats (CPU/Memory/Disk/Network) to processes page header
- [x] 4.26 Add graph history visualization for all metrics (CPU, memory, disk, network)
- [x] 4.28 Add network traffic graph to each adapter on network tab (with 60s history)
- [x] 4.30 Add smooth interpolated transitions to line graphs (Catmull-Rom splines + requestAnimationFrame)

### Backend Implementation (Rust)
- [x] 3.16 Fix CPU % normalization in process list (divide by core count)
- [x] 3.17 Fix display detection (EDID parsing from Windows Registry for actual monitor model)
- [x] 3.18 Fix processor cache size reporting (WMI Win32_Processor + Win32_CacheMemory for L1/L2/L3)

### Infrastructure
- [x] 2.1 Configure GitHub Actions CI pipeline (ESLint, Clippy, TypeScript, Rust build)
- [x] 2.3 Configure Tauri build for Windows installer (NSIS + MSI bundle targets)

### Backend Implementation (Rust)
- [x] 3.6 Add real-time metrics polling (MetricsHistoryService with 1-second polling)

### Frontend Implementation (Angular)
- [x] 4.8 Add real-time data refresh with signals (MetricsHistoryService uses Angular signals)
- [x] 4.29 Fix line graphs to have fixed height (consistent yAxisWidth=40 for alignment)
- [x] 4.31 Add process details dialog (ProcessDetailModalComponent with 3 tabs: Overview, Resources, Hierarchy)

### Quality
- [x] 5.1 Set up Rust test framework with unit tests (56 tests: models, collectors, services)
- [x] 5.2 Set up Angular test framework (47 tests: StatusService, DataCacheService, BytesPipe)
- [x] 5.3 Configure ESLint for frontend (ESLint 9 flat config with Angular plugins)
- [x] 5.4 Configure Clippy for backend (clippy.toml + fixed all warnings)
- [x] 5.5 Add pre-commit hooks (lint, format) with Husky

### Documentation
- [x] 6.1 Complete README with build instructions
