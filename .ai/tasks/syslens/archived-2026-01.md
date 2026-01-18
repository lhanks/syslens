# Syslens Archived Tasks - January 2026

**Archived:** 2026-01-01

---

## Completed

- [x] AMD CPU image enrichment for Ryzen processors (commit: 50c2fdc)

  - [x] Fixed `normalize_for_slug()` to strip "-Core Processor" suffix
  - [x] Added `build_direct_urls()` for proper AMD product URL patterns
  - [x] Added `extract_ryzen_series()` for 9000/7000/5000 series detection
  - [x] Prioritized og:image meta tag for reliable image extraction
  - [x] Added comprehensive tests for new functions

- [x] JIRA Integration Setup

  - [x] Created Syslens Atlassian instance (syslens.atlassian.net)
  - [x] Created SL project for issue tracking
  - [x] Configured credentials in ~/.env (JIRA_SYSLENS_*)
  - [x] Created .claude/jira.json for project config

- [x] Migrated to ___ai_files/ structure
  - [x] Removed legacy todo/ directory
  - [x] Created specs/, reports/, tasks/ structure

### Archived 2026-01-02

- [x] [SL-1] Set up JIRA task list integration
  - [x] Create jira-tasks.md to sync with JIRA issues
  - [x] Enable automated task tracking between local development and JIRA

- [x] Hardware Device Images & Documentation System - [Spec](../specs/_spec-hardware-images.md)
  - [x] Phase 1: Core infrastructure (ImageCache, data models, custom DB schema)
  - [x] Phase 2: GPU sources (TechPowerUp, ManufacturerSource, enhanced WikipediaSource)
  - [x] Phase 3: CPU sources (IntelArkSource, AMDProductSource, WikiChipSource)
  - [x] Phase 4: Frontend integration (image gallery, docs, drivers)
  - [x] Phase 5: Additional devices (motherboard, storage, memory, monitors)
  - [x] Phase 6: Polish & optimization

- [x] Integrate comprehensive PCI/USB ID databases
  - [x] Downloaded pci.ids (41K lines, 1.5MB) from pci-ids.ucw.cz
  - [x] Downloaded usb.ids (25K lines, 730KB) from linux-usb.org
  - [x] Fixed UTF-8 encoding issues in usb.ids
  - [x] Integrated databases via include_str! at compile time
  - [x] PCI: 3000+ vendors, 5000+ devices loaded at startup
  - [x] USB: 3500+ vendors, 15000+ products loaded at startup
  - [x] Updated tests to verify comprehensive database loading

- [x] Display device image next to device name on hardware cards
- [x] Add website links/resources page
- [x] Add current process CPU/MEM usage to status bar
- [x] Hardware Device Images - Phase 6: Polish & Optimization
- [x] Hardware Device Images - Phase 5: Additional Device Sources
- [x] Hardware Device Images - Phase 4: Frontend Integration
- [x] Hardware Device Images - Phase 3: CPU Sources
- [x] Marketing Website Screenshots
- [x] Add process icons to group headers in grouped view
- [x] Fix View menu items
- [x] Make dock panels detachable as floating windows
- [x] Add docking regions (left, right, top, bottom)
- [x] Left sidebar detachable to floating window
- [x] Navigation tabs moved to top bar
- [x] Left sidebar resizable and toggleable
- [x] Fix Services tab hang
- [x] Add Services tab
- [x] Enable/disable network adapters via UI
- [x] All UI polish tasks (see startup.md for details)
