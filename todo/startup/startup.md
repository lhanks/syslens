## 0 New Tasks

** IMPORTANT ** Process the tasks below (in this section). Expand them if needed and put them in the appropriate sections below.
** IMPORTANT ** Do not remove this section. Just clear out the tasks once processed.

## Pending

- [ ] Make left sidebar detachable (pop out to floating window)
- [ ] Add docking regions (left, right, top, bottom) - all resizable
  - [ ] Draggable dock sections between locations
  - [ ] Stackable sections with reorderable stack order
- [ ] Dockable section types: System Info, System Performance (CPU/Memory/Disk/Network)

## Completed (Phase 3)

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
