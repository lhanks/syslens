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
