# Syslens Startup Task List

Project: Syslens - Desktop System Information Dashboard
Tech Stack: Tauri 2.0 (Rust) + Angular 21 + Tailwind CSS

---

## 0 New Tasks
** IMPORTANT ** Process the tasks below (in this section).  Expand them if needed and put them in the appropriate sections below.
** IMPORTANT ** Do not remove this section. Just clear out the tasks once processed.

- For device information, we want to display deep detail about the device. We need to add more information to the device information. Explore ways to find information on the internet if necessary.
- Add a status bar to show the current task being executed. Mainly interested in feedback while the app is capturing data.
- Keep track of historical data. Add network traffic graph.
- Add a process list (like task manager) tab.  

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
- [x] 2.5 Create run-app-prod script for production builds with bundled assets
- [ ] 2.3 Configure Tauri build for Windows installer
- [ ] 2.4 Set up code signing (optional, for later)

## 3. Backend Implementation (Rust)

- [x] 3.1 Implement system info collector (device name, manufacturer, model, serial, BIOS/UEFI)
- [x] 3.2 Implement network info collector (adapters, IP config, DNS, statistics)
- [x] 3.3 Implement hardware info collector (CPU, memory, GPU)
- [x] 3.4 Implement storage info collector (drives, partitions, volumes, S.M.A.R.T.)
- [x] 3.5 Create Tauri commands for all collectors
- [ ] 3.6 Add real-time metrics polling
- [x] 3.14 Fix memory speed to show XMP/rated speed instead of JEDEC base speed
- [x] 3.15 Implement Windows monitor/display information collector (WMI + GDI fallback)
- [x] 3.7 Enhance device information collection (manufacturer, model, serial, BIOS/UEFI for all devices)
- [x] 3.8 Add complete IP configuration data (ipconfig /all) for network connections
- [x] 3.9 Add GPU vendor information (manufacturer, model, driver version, driver link)
- [x] 3.10 Add Memory vendor information (manufacturer, part number, serial, speed)
- [x] 3.11 Add detailed Motherboard information (manufacturer, model, version, serial, configuration, BIOS info)
- [x] 3.12 Gather motherboard image URL from manufacturer website based on model
- [x] 3.13 Add driver/documentation links for all hardware devices

## 4. Frontend Implementation (Angular)

- [x] 4.1 Set up Tailwind CSS and base styles
- [x] 4.19 Dim primary text color for reduced eye strain (#ffffff → #e5e5e5)
- [x] 4.2 Create Tauri service wrapper for IPC
- [x] 4.3 Build dashboard layout with navigation
- [x] 4.4 Implement System Info feature component
- [x] 4.5 Implement Network Info feature component
- [x] 4.6 Implement Hardware Info feature component
- [x] 4.7 Implement Storage Info feature component
- [ ] 4.8 Add real-time data refresh with signals
- [x] 4.18 Fix network stats real-time updates on dashboard (download/upload speeds)
- [x] 4.9 Create shared UI components (info cards, progress bars, copy buttons)
- [x] 4.10 Format numeric values to show 1 decimal place (avoid long digit strings)
- [x] 4.11 Add click-through from summary to detailed information views
- [x] 4.12 Implement progressive data loading (prioritize current tab, preload others)
- [x] 4.13 Display GPU vendor information in Hardware Info component
- [x] 4.14 Display Memory vendor information in Hardware Info component
- [x] 4.15 Display detailed Motherboard information in Hardware Info component
- [x] 4.16 Add clickable driver/documentation links for all devices
- [x] 4.17 Display motherboard image if available
- [x] 4.20 Parallelize dashboard data loading for faster startup (~500-1000ms improvement)
- [x] 4.21 Restore last active route on app restart (state persistence)

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

- Total Tasks: 52
- Completed: 40
- Remaining: 12

Last Updated: 2025-12-28
