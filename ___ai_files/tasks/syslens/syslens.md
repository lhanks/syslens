## 0 New Tasks

** IMPORTANT ** Process the tasks below (in this section). Expand them if needed and put them in the appropriate sections below.
** IMPORTANT ** Do not remove this section. Just clear out the tasks once processed.

(none)

## Pending

- archive
- work on jira tasks

## In Progress

(none)

## Completed (January 2026)

- [x] AMD CPU image enrichment for Ryzen processors (commit: 50c2fdc)

  - [x] Fixed `normalize_for_slug()` to strip "-Core Processor" suffix
  - [x] Added `build_direct_urls()` for proper AMD product URL patterns
  - [x] Added `extract_ryzen_series()` for 9000/7000/5000 series detection
  - [x] Prioritized og:image meta tag for reliable image extraction
  - [x] Added comprehensive tests for new functions

- [x] JIRA Integration Setup

  - [x] Created Syslens Atlassian instance (syslens.atlassian.net)
  - [x] Created SL project for issue tracking
  - [x] Configured credentials in ~/.env (JIRA*SYSLENS*\*)
  - [x] Created .claude/jira.json for project config

- [x] Migrated to \_\_\_ai_files/ structure
  - [x] Removed legacy todo/ directory
  - [x] Created specs/, reports/, tasks/ structure

## Future Development

- [ ] Set up code signing
- [ ] Tag v1.0.0 release
- [ ] AI image generation fallback (Phase 7)
- [ ] Cross-platform support (macOS, Linux)
- [ ] Intel CPU image enrichment (Intel ARK)
- [ ] NVIDIA GPU image enrichment

---

**Project Info:**

- **JIRA:** https://syslens.atlassian.net/projects/SL
- **Issue Format:** SL-1, SL-2, SL-3...
- **Tech Stack:** Tauri 2.0 (Rust) + Angular 21 + Tailwind CSS
- **Repository:** C:\dev\syslens

**Related:**

- [JIRA Task List](./jira-tasks.md) - Active JIRA issues
- [Hardware Images Spec](../specs/_spec-hardware-images.md)
- [Startup Task List (archived)](./startup/startup.md) - Pre-JIRA development history
