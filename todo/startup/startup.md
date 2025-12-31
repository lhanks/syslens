## 0 New Tasks

** IMPORTANT ** Process the tasks below (in this section). Expand them if needed and put them in the appropriate sections below.
** IMPORTANT ** Do not remove this section. Just clear out the tasks once processed.

(none)

## Pending

- [ ] Hardware Device Images & Documentation System - [Spec](./../___ai_files/specs/_spec-hardware-images.md)
  - [x] Phase 1: Core infrastructure (ImageCache, data models, custom DB schema)
  - [x] Phase 2: GPU sources (TechPowerUp, ManufacturerSource, enhanced WikipediaSource)
  - [x] Phase 3: CPU sources (IntelArkSource, AMDProductSource, WikiChipSource)
  - [x] Phase 4: Frontend integration (image gallery, docs, drivers)
  - [x] Phase 5: Additional devices (motherboard, storage, memory, monitors)
  - [x] Phase 6: Polish & optimization
  - Phase 7: AI image generation fallback (future)

## Completed (Hardware UI)

- [x] Display device image next to device name on hardware cards
  - [x] Identified hardware card components (CPU, GPU sections)
  - [x] Added device image thumbnail to card layout using DeviceImageComponent
  - [x] Fetch images using existing enrichment service (DeviceInfoService.enrichDevice)

## Completed (Website)

- [x] Add website links/resources page
  - [x] Created `/resources` page with categorized links
  - [x] Added navigation link in header and footer
  - [x] Includes: GitHub, Releases, Vercel, Technology docs

## Completed (Status Bar)

- [x] Add current process CPU/MEM usage to status bar
  - [x] Created `SelfMetrics` model and `get_self_metrics` Rust command
  - [x] Added polling in MetricsHistoryService (1s interval)
  - [x] Status bar displays "App" with CPU % and memory usage
  - [x] Real-time updates alongside other system metrics

## Completed (Phase 6)

- [x] Hardware Device Images - Phase 6: Polish & Optimization
  - [x] Fixed all clippy warnings (-D warnings passes)
  - [x] Added #[allow(dead_code)] annotations for reserved client fields
  - [x] Modernized code: map_or -> is_some_and per clippy

## Completed (Phase 5)

- [x] Hardware Device Images - Phase 5: Additional Device Sources
  - [x] Created MotherboardSource with chipset/form factor parsing (ASUS, Gigabyte, MSI, ASRock)
  - [x] Created StorageSource with NVMe/SATA/HDD detection and performance estimates
  - [x] Created MemorySource with DDR4/DDR5 speed, capacity, and kit configuration parsing
  - [x] Created MonitorSource with resolution, refresh rate, panel type, and adaptive sync detection
  - [x] Added Monitor variant to DeviceType enum throughout codebase
  - [x] Registered all new sources in DeviceEnrichmentService
  - [x] Code compiles successfully (warnings only for unused client fields)

## Completed (Phase 4)

- [x] Hardware Device Images - Phase 4: Frontend Integration
  - [x] Updated Angular models for new image types (ImageEntry, ImageMetadata, ProductImages)
  - [x] Added device enrichment service methods to DeviceInfoService
  - [x] Created DeviceImageComponent for displaying cached/remote images
  - [x] Updated DeviceDetailModalComponent with enhanced Images tab
  - [x] Added image metadata display (source, fetched date, AI-generated badge)
  - [x] Gallery view with hover effects and type badges
  - [x] Loading placeholders and error states
  - [x] Angular and Rust both compile successfully

## Completed (Phase 3)

- [x] Hardware Device Images - Phase 3: CPU Sources

  - [x] IntelArkSource - Intel ARK database for Intel CPU specs (priority 5)
  - [x] AMDProductSource - AMD product database for AMD CPUs/GPUs (priority 5)
  - [x] WikiChipSource - WikiChip for detailed CPU architecture info (priority 15)
  - [x] Registered all sources in DeviceEnrichmentService
  - [x] Code passes cargo check and clippy

- [x] Marketing Website Screenshots

  - [x] Take screenshots of the running app using Chrome DevTools MCP
  - [x] Create creative screenshot presentation (angled, fading into distance)
  - [x] Update website with app screenshots

- [x] Add process icons to group headers in grouped view
- [x] Investigate process icons - verified working (482/664 processes have icons)
- [x] Fix View menu items - connected MenuService to DockService instead of ViewSettingsService
- [x] Make dock panels detachable as floating windows (pop-out button, Tauri WebviewWindow)

- [x] Add docking regions (left, right, top, bottom) - all resizable
  - [x] CSS Grid 4-region layout with DockContainerComponent
  - [x] Draggable dock sections between locations (HTML5 drag-drop)
  - [x] Stackable tabs with reorderable panels (DockRegionComponent)
  - [x] DockService for layout state + localStorage persistence
- [x] Dockable section types: System Info, System Performance (PerformancePanelComponent, SystemInfoPanelComponent)

- [x] Left sidebar detachable to floating window (pop-out button, Tauri WebviewWindow)
- [x] Navigation tabs moved to top bar (sidebar shows only mini graphs)
- [x] Left sidebar resizable (drag handle, 180-400px) and toggleable (Ctrl+N menu)
- [x] Fix Services tab hang (optimized WMI query from N+1 to single query)
- [x] Add Services tab (list Windows services with status, startup type, filtering, search)
- [x] Enable/disable network adapters via UI (netsh command)
- [x] Vendor badges with brand colors in sidebar details
- [x] Mini graph click navigation - System page shows CPU/Memory/Disk history
- [x] Disk activity line graph on System page
- [x] Network transfer rate font size reduced 10%
- [x] Remove performance graphs from System tab (per user request)
- [x] Add app icons to process list with fallback default
- [x] Remove Dashboard - System is now default view
- [x] Show vendor name in mini graphs (Intel, NVIDIA, DDR5, Samsung)
- [x] View > Show menu controls mini graph visibility
- [x] Right sidebar dockable (left/right), resizable, closeable
- [x] Persist view settings via ViewSettingsService
- [x] Machine name and uptime in sidebar header (replaced logo)
- [x] Native menu (File/View/Help) with Tauri
- [x] Process grouping with expandable tree
- [x] Network tab graphs populate from startup
- [x] Process file paths with ellipsis truncation
- [x] Per-adapter network graphs in sidebar
- [x] Active network adapters sorted to top
- [x] Network adapters poll every 10 seconds
- [x] Hardware documentation research (hybrid approach)
- [x] Process name styling improved
- [x] Process count right-aligned in grouped view
- [x] Network traffic labels fixed width (70px)
- [x] Logo removed from sidebar header
- [x] Resource summary removed from Processes page

## Future Development

- [ ] Set up code signing
- [ ] Tag v1.0.0 release
- [ ] Design hardware database and AI-enhanced features spec
- [ ] Cross-platform support (macOS, Linux)

### Hardware Documentation Database (Research Complete)

**Recommendation: Hybrid Approach** - Runtime web lookups with local caching

**Existing Infrastructure:**

- `KnowledgeStore` - Persistent storage for learned device info (learned_devices.json)
- `DeviceSource` trait - Plugin architecture for data sources
- `WikipediaSource` - Already implemented Wikipedia infobox parser

**Implementation Plan:**

1. Ship PCI/USB ID databases (~2MB compressed) for device identification
2. Use runtime web lookups (lazy, on-demand) for detailed specs
3. Cache fetched data in KnowledgeStore with confidence scoring
4. Multi-source merge logic already exists in device_sources/mod.rs

**Best Sources (no API keys):**

- TechPowerUp (GPUs, CPUs) - structured tables
- Intel ARK (Intel products) - official specs
- WikiChip (processors) - detailed architecture info
- AMD Product Database - official AMD specs
- Wikipedia - infoboxes with manufacturer, model, specs

---

**Archives:**

- [Phase 2 - December 30, 2025](./startup-archived-2025-12-30.md)
- [Phase 1 - December 29, 2025](./startup-archived-2025-12-29.md)
