# Syslens Task List - Phase 2

## 0 New Tasks ** IMPORTANT ** DO NOT REMOVE THIS SECTION

** IMPORTANT ** Process the tasks below (in this section). Expand them if needed and put them in the appropriate sections below.
** IMPORTANT ** Do not remove this section. Just clear out the tasks once processed.

(No new tasks)

## In Progress

(No tasks currently in progress)

## UI/UX Improvements

### Status Bar

- [x] Change RAM display to show USED / TOTAL format (e.g., "100GB / 200GB")
- [x] Use fixed width for status bar values to prevent UI shifting
- [x] Add app version to bottom right of status bar

### Sidebars

- [x] Add port type summary to right sidebar (USB3, HDMI, DisplayPort counts)
- [x] Add GPU to left sidebar with CPU/MEM/DISK/NETWORK
- [x] Make sidebar devices clickable to open details dialog

### Dashboard

- [x] Show device names on Dashboard components (CPU, Memory, Disk, Network)
- [x] Make all boxes clickable - open details dialog if no obvious action

### Line Graphs

- [x] Fix vertical constraints - lines should not extend below graph bounds

## Storage Features

- [x] Fix Physical Disks tab - currently showing nothing
- [x] Show read/write speed and IOPS for physical disks
- [x] Report used/available space for network storage

## Network Features

- [x] Update Adapters tab to show 3 columns (add second details column)

## Details Dialogs

- [x] Display details dialog on click: monitors (specs/drivers/documentation)
- [x] Display details dialog on click: RAM modules (specs/drivers/documentation)

## System Features

- [x] Fix Restore Points tab - not showing any information

## Performance

- [x] Investigate 5-second startup pause that blocks UI
  - **Finding**: Multiple WMI connections created during startup (CPU, memory, GPU, disk, network info each create separate connections)
  - **Current mitigations**: Progressive loading, lazy SysInfoState, skeleton UI
  - **Further optimization**: Would require shared WMI connection pool or deferring non-critical info

## Hardware Database

- [x] Integrate USB Hardware ID database for device identification
- [x] Integrate PCI Hardware ID database for device identification
- [x] Create auto-update mechanism for hardware ID databases

## Recently Completed

(See archived task list)

## Future Development

- [ ] Set up code signing
- [ ] Tag v1.0.0 release
- [ ] Design hardware database and AI-enhanced features spec
- [ ] Cross-platform support (macOS, Linux)

---

**Archive:** [startup-archived-2025-12-29.md](./startup-archived-2025-12-29.md)
