# Syslens Task List - Phase 3

## 0 New Tasks ** IMPORTANT ** DO NOT REMOVE THIS SECTION

** IMPORTANT ** Process the tasks below (in this section). Expand them if needed and put them in the appropriate sections below.
** IMPORTANT ** Do not remove this section. Just clear out the tasks once processed.

- Add a menu (File,View,Help) to the app.
- Make the right sidebar a window that can be dragged and dropped to the side of the screen. It can be turned off and on (via the view menu).
- Add app icon to the process list. Make sure apps have a default.
- Make the process name in the process list more readable. It is too dim.
- In the process list, make the process could right aligned/remove background color.
- Change the network traffic mini graphs. Fix the width of all labels so that the screen doesn't shift around when the values change.
- Remove SysLens logo from the top left of the screen.
- Add option to hide/show the different items: CPU/MEM/DISK/NETWORK on the left sidebar in the view menu.
- add ability to enable/disable network adapters.
- make dockaable sidebar resizeable.
- Be s ure to remember all the view settings

<!-- Process name in process list now uses text-syslens-text-primary font-medium (was too dim) -->
<!-- Resource summary (CPU/MEM/DISK/NETWORK) removed from Processes page -->
<!-- Hardware documentation research completed - hybrid approach recommended -->
<!-- Network adapters now poll every 10 seconds to detect enable/disable changes -->
<!-- Active network adapters are sorted to the top of the list -->
<!-- Per-adapter network graphs now displayed in Dashboard, Processes, and Sidebar -->
<!-- Process file paths now display with ellipsis in middle: C:\Windows\...\process.exe -->
<!-- Network tab graphs now populate from app startup via MetricsHistoryService -->
<!-- Process grouping with expandable tree already implemented in processes.component.ts -->

## In Progress

(No tasks currently in progress)

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
