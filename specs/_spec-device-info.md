# Specification: Deep Device Information with Internet Lookup

> Status: Draft
> Created: 2025-12-28

## Overview

Extend Syslens hardware information capabilities with deep device intelligence through internet lookups, local caching, and AI-powered information retrieval. Provide a one-stop shop for device specifications, drivers, documentation, and images with offline fallback support.

## Problem Statement

Users currently need to visit multiple manufacturer websites to find:
- Detailed product specifications beyond basic WMI data
- Latest driver downloads and version checking
- Product documentation and manuals
- High-quality product images
- Benchmark data and performance comparisons

This spec provides comprehensive device information through intelligent internet lookup with local caching, fallback databases, and AI-assisted web scraping for hard-to-find information.

## Goals

- [ ] Fetch detailed device specs from manufacturer websites with 7-day cache
- [ ] Provide direct driver download links with 1-day version cache
- [ ] Display product images and PDF documentation links
- [ ] Maintain local JSON database for offline operation
- [ ] Integrate Claude API for AI-powered spec lookup
- [ ] Show data freshness with "last updated" timestamps
- [ ] Support CPU, GPU, Motherboard, Memory, and Storage devices

## Non-Goals

- Natural language query interface (Phase 2)
- Process history and usage pattern analysis (Phase 2)
- Benchmark score database integration (nice-to-have)
- Real-time driver installation (link only)
- Community reviews or ratings

## User Stories

### Story 1: View Deep Device Specifications
**As a** power user
**I want to** see comprehensive technical specifications for my hardware
**So that** I can make informed upgrade decisions without visiting multiple websites

**Acceptance Criteria:**
- [ ] Click on any supported device to view deep info modal/panel
- [ ] Specifications include all manufacturer-published technical data
- [ ] Data includes product images (if available)
- [ ] Shows "Last Updated" timestamp for fetched data
- [ ] Works offline with cached/local database data

### Story 2: Find Latest Drivers
**As an** IT professional
**I want to** quickly access the latest driver download for any device
**So that** I can keep systems up-to-date without manual searching

**Acceptance Criteria:**
- [ ] Direct download links to latest drivers
- [ ] Show installed vs. latest available version
- [ ] Highlight when updates are available
- [ ] Cache driver version info for 1 day
- [ ] Fallback to generic manufacturer driver page if specific link unavailable

### Story 3: Access Documentation
**As a** power user
**I want to** view product manuals and technical documentation
**So that** I can troubleshoot issues and understand capabilities

**Acceptance Criteria:**
- [ ] Links to PDF manuals and datasheets
- [ ] Support documentation URLs
- [ ] BIOS/firmware update links (motherboards)
- [ ] Quick access without leaving the application

### Story 4: Offline Operation
**As a** user with intermittent internet
**I want to** access cached device information offline
**So that** I can use the application without connectivity

**Acceptance Criteria:**
- [ ] Local JSON database contains common device specs
- [ ] Previously fetched data cached locally
- [ ] Clear indication when data is cached vs. live
- [ ] Graceful degradation when internet unavailable

### Story 5: AI-Assisted Information Retrieval
**As a** user with obscure/OEM hardware
**I want to** find specifications for devices not in standard databases
**So that** I can get information even for uncommon hardware

**Acceptance Criteria:**
- [ ] AI agent scrapes manufacturer websites for missing data
- [ ] Results stored in local database for future use
- [ ] Shows AI confidence level or source URL
- [ ] Manual retry option if AI lookup fails

## Technical Design

### Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         Angular Frontend                        │
│  ┌────────────────┐  ┌──────────────┐  ┌───────────────────┐  │
│  │ Device Detail  │  │ Driver Check │  │  Spec Viewer      │  │
│  │    Panel       │  │   Service    │  │   Component       │  │
│  └────────┬───────┘  └──────┬───────┘  └─────────┬─────────┘  │
│           │                  │                     │             │
│           └──────────────────┴─────────────────────┘             │
│                              │                                   │
└──────────────────────────────┼───────────────────────────────────┘
                               │ Tauri IPC
┌──────────────────────────────┼───────────────────────────────────┐
│                         Rust Backend                             │
│  ┌──────────────────────────┴────────────────────────────────┐  │
│  │              Device Information Service                     │  │
│  └─┬────────────┬──────────────┬──────────────┬──────────────┘  │
│    │            │              │              │                  │
│  ┌─▼──────┐  ┌─▼────────┐  ┌─▼────────┐  ┌──▼──────────────┐  │
│  │ Cache  │  │ Internet │  │   AI     │  │ Local Database  │  │
│  │Manager │  │ Fetcher  │  │  Agent   │  │    Manager      │  │
│  └────────┘  └──────────┘  └──────────┘  └─────────────────┘  │
│      │            │              │              │                │
│      │            │              │              │                │
│  ┌───▼────────────▼──────────────▼──────────────▼────────────┐  │
│  │           Device Information Aggregator                    │  │
│  └─────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
         │              │                    │
         │              │                    │
    ┌────▼────┐    ┌────▼─────────┐    ┌────▼────────────┐
    │  Local  │    │ Manufacturer │    │  Claude API     │
    │  Cache  │    │   Websites   │    │ (Anthropic)     │
    │(AppData)│    │              │    │                 │
    └─────────┘    └──────────────┘    └─────────────────┘
```

### Data Models

#### Extended Device Information

```rust
// projects/backend/src/models/device_info.rs

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Deep device information with internet-sourced data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceDeepInfo {
    /// Device identifier (matches hardware collector ID)
    pub device_id: String,

    /// Device category
    pub device_type: DeviceType,

    /// Basic identifying information
    pub identifier: DeviceIdentifier,

    /// Detailed specifications
    pub specifications: Option<DeviceSpecifications>,

    /// Driver information
    pub drivers: Option<DriverInfo>,

    /// Documentation links
    pub documentation: Option<DocumentationLinks>,

    /// Product images
    pub images: Option<ProductImages>,

    /// Data source and freshness
    pub metadata: DataMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum DeviceType {
    Cpu,
    Gpu,
    Motherboard,
    Memory,
    Storage,
}

/// Device identifying information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceIdentifier {
    pub manufacturer: String,
    pub model: String,
    pub part_number: Option<String>,
    pub serial_number: Option<String>,

    /// Hardware IDs (PCI ID, USB VID/PID, etc.)
    pub hardware_ids: Vec<String>,
}

/// Detailed device specifications
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceSpecifications {
    /// Raw specification key-value pairs
    pub specs: HashMap<String, String>,

    /// Categorized specifications for display
    pub categories: Vec<SpecCategory>,

    /// Marketing/product description
    pub description: Option<String>,

    /// Release date
    pub release_date: Option<String>,

    /// End of life date (if applicable)
    pub eol_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpecCategory {
    pub name: String,
    pub specs: Vec<SpecItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpecItem {
    pub label: String,
    pub value: String,
    pub unit: Option<String>,
}

/// Driver information and links
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriverInfo {
    /// Currently installed driver version
    pub installed_version: Option<String>,

    /// Latest available driver version
    pub latest_version: Option<String>,

    /// Direct download URL for latest driver
    pub download_url: Option<String>,

    /// Driver release date
    pub release_date: Option<String>,

    /// Release notes URL
    pub release_notes_url: Option<String>,

    /// Generic driver download page (fallback)
    pub driver_page_url: Option<String>,

    /// Update available flag
    pub update_available: bool,
}

/// Documentation and support links
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentationLinks {
    /// Product page URL
    pub product_page: Option<String>,

    /// Support/download page
    pub support_page: Option<String>,

    /// User manual PDF links
    pub manuals: Vec<DocumentLink>,

    /// Technical datasheets
    pub datasheets: Vec<DocumentLink>,

    /// BIOS/Firmware downloads (motherboards/storage)
    pub firmware_updates: Vec<FirmwareLink>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentLink {
    pub title: String,
    pub url: String,
    pub file_type: String, // "PDF", "HTML", etc.
    pub language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FirmwareLink {
    pub title: String,
    pub version: String,
    pub url: String,
    pub release_date: Option<String>,
}

/// Product images
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductImages {
    /// Primary product image URL
    pub primary_image: Option<String>,

    /// Additional product images
    pub gallery: Vec<String>,

    /// Thumbnail image
    pub thumbnail: Option<String>,
}

/// Data source and freshness metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataMetadata {
    /// Data source
    pub source: DataSource,

    /// When data was fetched/updated
    pub last_updated: DateTime<Utc>,

    /// When data should be refreshed
    pub expires_at: DateTime<Utc>,

    /// Source URL (if from internet)
    pub source_url: Option<String>,

    /// AI confidence score (if AI-sourced)
    pub ai_confidence: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataSource {
    /// From local JSON database
    LocalDatabase,

    /// Fetched from manufacturer website
    ManufacturerWebsite,

    /// Fetched from third-party database
    ThirdPartyDatabase,

    /// Retrieved via AI agent
    AiAgent,

    /// Cached from previous fetch
    Cache,
}
```

#### Local Database Schema

```json
// AppData/Roaming/Syslens/device_database.json
{
  "version": "1.0.0",
  "last_updated": "2025-12-28T10:00:00Z",
  "devices": {
    "cpu": [
      {
        "identifier": {
          "manufacturer": "Intel",
          "model": "Core i7-12700K",
          "partNumber": "BX8071512700K"
        },
        "specifications": {
          "specs": {
            "cores": "12",
            "threads": "20",
            "baseClock": "3.6 GHz",
            "boostClock": "5.0 GHz",
            "cache": "25 MB",
            "tdp": "125 W",
            "socket": "LGA1700",
            "lithography": "Intel 7",
            "memoryTypes": "DDR4-3200, DDR5-4800"
          },
          "categories": [
            {
              "name": "Performance",
              "specs": [
                {"label": "Cores", "value": "12", "unit": null},
                {"label": "Threads", "value": "20", "unit": null},
                {"label": "Base Clock", "value": "3.6", "unit": "GHz"},
                {"label": "Boost Clock", "value": "5.0", "unit": "GHz"}
              ]
            },
            {
              "name": "Cache & Memory",
              "specs": [
                {"label": "L3 Cache", "value": "25", "unit": "MB"},
                {"label": "Memory Support", "value": "DDR4-3200, DDR5-4800", "unit": null}
              ]
            },
            {
              "name": "Thermal & Power",
              "specs": [
                {"label": "TDP", "value": "125", "unit": "W"},
                {"label": "Max Temp", "value": "100", "unit": "°C"}
              ]
            }
          ],
          "description": "12th Gen Intel Core desktop processor with hybrid architecture",
          "releaseDate": "2021-11-04"
        },
        "drivers": null,
        "documentation": {
          "productPage": "https://ark.intel.com/content/www/us/en/ark/products/134594/intel-core-i712700k-processor-25m-cache-up-to-5-00-ghz.html",
          "supportPage": "https://www.intel.com/content/www/us/en/support/products/134594.html",
          "datasheets": [
            {
              "title": "Datasheet Volume 1",
              "url": "https://cdrdv2.intel.com/v1/dl/getContent/655258",
              "fileType": "PDF",
              "language": "en"
            }
          ]
        },
        "images": {
          "primaryImage": "https://example.com/images/i7-12700k.png",
          "thumbnail": "https://example.com/images/i7-12700k-thumb.png"
        }
      }
    ],
    "gpu": [...],
    "motherboard": [...],
    "memory": [...],
    "storage": [...]
  }
}
```

#### Cache Database Schema

```json
// AppData/Roaming/Syslens/device_cache.json
{
  "entries": [
    {
      "deviceId": "cpu-intel-i7-12700k",
      "deviceType": "Cpu",
      "data": { /* DeviceDeepInfo object */ },
      "cachedAt": "2025-12-28T10:00:00Z",
      "expiresAt": "2026-01-04T10:00:00Z"
    }
  ]
}
```

### API Design

#### Rust Tauri Commands

```rust
// projects/backend/src/commands/device_info.rs

use crate::models::{DeviceDeepInfo, DeviceType};

/// Get deep device information by device ID and type
#[tauri::command]
pub async fn get_device_deep_info(
    device_id: String,
    device_type: DeviceType,
    force_refresh: bool,
) -> Result<DeviceDeepInfo, String> {
    // 1. Check cache (unless force_refresh)
    // 2. If cached and not expired, return cached data
    // 3. If not cached or expired:
    //    a. Try internet fetch (manufacturer websites)
    //    b. Try local database lookup
    //    c. Try AI agent as last resort
    // 4. Cache successful result
    // 5. Return DeviceDeepInfo
}

/// Get driver update status for a device
#[tauri::command]
pub async fn check_driver_updates(
    device_id: String,
    device_type: DeviceType,
) -> Result<DriverInfo, String> {
    // Check driver versions with 1-day cache
}

/// Manually trigger AI lookup for a device
#[tauri::command]
pub async fn ai_lookup_device(
    device_id: String,
    device_type: DeviceType,
) -> Result<DeviceDeepInfo, String> {
    // Force AI agent lookup, bypass cache
}

/// Get all cached device information
#[tauri::command]
pub fn get_cached_devices() -> Result<Vec<DeviceDeepInfo>, String> {
    // Return all cached devices for offline viewing
}

/// Clear cache for a specific device or all devices
#[tauri::command]
pub fn clear_device_cache(device_id: Option<String>) -> Result<(), String> {
    // Clear cache entries
}

/// Export device information to JSON
#[tauri::command]
pub fn export_device_info(
    device_ids: Vec<String>,
    output_path: String,
) -> Result<(), String> {
    // Export selected device info to file
}
```

### Service Layer Architecture

```rust
// projects/backend/src/services/device_info_service.rs

pub struct DeviceInfoService {
    cache_manager: CacheManager,
    internet_fetcher: InternetFetcher,
    ai_agent: AiAgent,
    local_db: LocalDatabaseManager,
}

impl DeviceInfoService {
    /// Main entry point for getting device info
    pub async fn get_device_info(
        &self,
        device_id: &str,
        device_type: DeviceType,
        force_refresh: bool,
    ) -> Result<DeviceDeepInfo> {
        // Orchestrate data retrieval
    }

    /// Fetch from internet sources
    async fn fetch_from_internet(
        &self,
        identifier: &DeviceIdentifier,
        device_type: DeviceType,
    ) -> Result<DeviceDeepInfo> {
        // Internet fetching logic
    }

    /// Query local database
    fn query_local_database(
        &self,
        identifier: &DeviceIdentifier,
        device_type: DeviceType,
    ) -> Option<DeviceDeepInfo> {
        // Local DB lookup
    }

    /// Use AI agent for lookup
    async fn ai_lookup(
        &self,
        identifier: &DeviceIdentifier,
        device_type: DeviceType,
    ) -> Result<DeviceDeepInfo> {
        // AI-powered lookup
    }
}
```

### Internet Fetcher Implementation

```rust
// projects/backend/src/services/internet_fetcher.rs

use reqwest::Client;
use scraper::{Html, Selector};

pub struct InternetFetcher {
    client: Client,
}

impl InternetFetcher {
    /// Fetch device info from manufacturer website
    pub async fn fetch_manufacturer_info(
        &self,
        manufacturer: &str,
        model: &str,
        device_type: DeviceType,
    ) -> Result<DeviceDeepInfo> {
        match manufacturer.to_lowercase().as_str() {
            "intel" => self.fetch_intel_cpu(model).await,
            "amd" => self.fetch_amd_device(model, device_type).await,
            "nvidia" => self.fetch_nvidia_gpu(model).await,
            "asus" | "msi" | "gigabyte" | "asrock" => {
                self.fetch_motherboard_info(manufacturer, model).await
            }
            _ => Err("Manufacturer not supported".into()),
        }
    }

    /// Fetch Intel CPU from ARK database
    async fn fetch_intel_cpu(&self, model: &str) -> Result<DeviceDeepInfo> {
        // Scrape Intel ARK
        let url = format!("https://ark.intel.com/content/www/us/en/ark/search.html?q={}", model);
        // Parse HTML, extract specs
    }

    /// Fetch AMD device info
    async fn fetch_amd_device(
        &self,
        model: &str,
        device_type: DeviceType,
    ) -> Result<DeviceDeepInfo> {
        // AMD website scraping
    }

    /// Fetch NVIDIA GPU info
    async fn fetch_nvidia_gpu(&self, model: &str) -> Result<DeviceDeepInfo> {
        // NVIDIA website scraping
    }

    /// Fetch motherboard info from manufacturer
    async fn fetch_motherboard_info(
        &self,
        manufacturer: &str,
        model: &str,
    ) -> Result<DeviceDeepInfo> {
        // Manufacturer-specific scraping
    }

    /// Fallback: TechPowerUp database
    async fn fetch_from_techpowerup(
        &self,
        device_type: DeviceType,
        model: &str,
    ) -> Result<DeviceDeepInfo> {
        // TechPowerUp scraping as fallback
    }
}
```

### AI Agent Implementation

```rust
// projects/backend/src/services/ai_agent.rs

use serde_json::json;

pub struct AiAgent {
    api_key: String,
    client: reqwest::Client,
}

impl AiAgent {
    /// Use Claude API to search for device information
    pub async fn search_device_info(
        &self,
        identifier: &DeviceIdentifier,
        device_type: DeviceType,
    ) -> Result<DeviceDeepInfo> {
        let prompt = self.build_search_prompt(identifier, device_type);

        let response = self.call_claude_api(&prompt).await?;

        let deep_info = self.parse_ai_response(response, identifier)?;

        Ok(deep_info)
    }

    fn build_search_prompt(
        &self,
        identifier: &DeviceIdentifier,
        device_type: DeviceType,
    ) -> String {
        format!(
            r#"Search for detailed specifications and information about the following device:

Device Type: {:?}
Manufacturer: {}
Model: {}
Part Number: {}

Please find and provide:
1. Complete technical specifications
2. Driver download links (if applicable)
3. Product documentation links (manuals, datasheets)
4. Product image URLs
5. Release date and other metadata

Format the response as JSON matching this schema:
{{
    "specifications": {{
        "specs": {{}},
        "categories": [],
        "description": "",
        "releaseDate": ""
    }},
    "drivers": {{}},
    "documentation": {{}},
    "images": {{}}
}}

Search manufacturer websites first, then reputable tech databases like TechPowerUp.
Provide source URLs for all information found."#,
            device_type,
            identifier.manufacturer,
            identifier.model,
            identifier.part_number.as_deref().unwrap_or("N/A")
        )
    }

    async fn call_claude_api(&self, prompt: &str) -> Result<String> {
        let request_body = json!({
            "model": "claude-3-5-sonnet-20241022",
            "max_tokens": 4096,
            "messages": [
                {
                    "role": "user",
                    "content": prompt
                }
            ]
        });

        let response = self.client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&request_body)
            .send()
            .await?;

        let json: serde_json::Value = response.json().await?;
        let content = json["content"][0]["text"].as_str()
            .ok_or("Invalid API response")?;

        Ok(content.to_string())
    }

    fn parse_ai_response(
        &self,
        response: String,
        identifier: &DeviceIdentifier,
    ) -> Result<DeviceDeepInfo> {
        // Parse JSON response and construct DeviceDeepInfo
        // Include AI confidence score in metadata
    }
}
```

### Cache Manager Implementation

```rust
// projects/backend/src/services/cache_manager.rs

use std::path::PathBuf;
use chrono::{DateTime, Utc, Duration};

pub struct CacheManager {
    cache_file: PathBuf,
}

impl CacheManager {
    pub fn new() -> Result<Self> {
        let cache_dir = Self::get_app_data_dir()?;
        std::fs::create_dir_all(&cache_dir)?;

        Ok(Self {
            cache_file: cache_dir.join("device_cache.json"),
        })
    }

    fn get_app_data_dir() -> Result<PathBuf> {
        // Windows: %APPDATA%/Syslens
        // macOS: ~/Library/Application Support/Syslens
        // Linux: ~/.local/share/syslens
        #[cfg(target_os = "windows")]
        {
            let appdata = std::env::var("APPDATA")?;
            Ok(PathBuf::from(appdata).join("Syslens"))
        }
        #[cfg(target_os = "macos")]
        {
            let home = std::env::var("HOME")?;
            Ok(PathBuf::from(home).join("Library/Application Support/Syslens"))
        }
        #[cfg(target_os = "linux")]
        {
            let home = std::env::var("HOME")?;
            Ok(PathBuf::from(home).join(".local/share/syslens"))
        }
    }

    /// Get cached device info if not expired
    pub fn get(
        &self,
        device_id: &str,
        device_type: &DeviceType,
    ) -> Option<DeviceDeepInfo> {
        let cache = self.load_cache().ok()?;

        cache.entries.into_iter()
            .find(|e| e.device_id == device_id && &e.device_type == device_type)
            .filter(|e| e.expires_at > Utc::now())
            .map(|e| e.data)
    }

    /// Store device info in cache
    pub fn set(
        &self,
        device_id: String,
        device_type: DeviceType,
        data: DeviceDeepInfo,
        ttl: Duration,
    ) -> Result<()> {
        let mut cache = self.load_cache().unwrap_or_default();

        // Remove existing entry
        cache.entries.retain(|e| e.device_id != device_id || e.device_type != device_type);

        // Add new entry
        cache.entries.push(CacheEntry {
            device_id,
            device_type,
            data,
            cached_at: Utc::now(),
            expires_at: Utc::now() + ttl,
        });

        self.save_cache(&cache)
    }

    /// Clear all expired entries
    pub fn cleanup_expired(&self) -> Result<()> {
        let mut cache = self.load_cache()?;
        let now = Utc::now();
        cache.entries.retain(|e| e.expires_at > now);
        self.save_cache(&cache)
    }

    fn load_cache(&self) -> Result<DeviceCache> {
        if !self.cache_file.exists() {
            return Ok(DeviceCache::default());
        }

        let content = std::fs::read_to_string(&self.cache_file)?;
        let cache = serde_json::from_str(&content)?;
        Ok(cache)
    }

    fn save_cache(&self, cache: &DeviceCache) -> Result<()> {
        let content = serde_json::to_string_pretty(cache)?;
        std::fs::write(&self.cache_file, content)?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct DeviceCache {
    entries: Vec<CacheEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CacheEntry {
    device_id: String,
    device_type: DeviceType,
    data: DeviceDeepInfo,
    cached_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
}
```

### Local Database Manager

```rust
// projects/backend/src/services/local_database.rs

pub struct LocalDatabaseManager {
    db_file: PathBuf,
}

impl LocalDatabaseManager {
    pub fn new() -> Result<Self> {
        let db_dir = Self::get_app_data_dir()?;
        std::fs::create_dir_all(&db_dir)?;

        let db_file = db_dir.join("device_database.json");

        // Initialize with bundled database if doesn't exist
        if !db_file.exists() {
            Self::initialize_database(&db_file)?;
        }

        Ok(Self { db_file })
    }

    fn initialize_database(path: &PathBuf) -> Result<()> {
        // Copy bundled database from resources
        let bundled_db = include_str!("../../resources/device_database.json");
        std::fs::write(path, bundled_db)?;
        Ok(())
    }

    /// Search for device in local database
    pub fn find_device(
        &self,
        identifier: &DeviceIdentifier,
        device_type: &DeviceType,
    ) -> Option<DeviceDeepInfo> {
        let db = self.load_database().ok()?;

        let devices = match device_type {
            DeviceType::Cpu => &db.devices.cpu,
            DeviceType::Gpu => &db.devices.gpu,
            DeviceType::Motherboard => &db.devices.motherboard,
            DeviceType::Memory => &db.devices.memory,
            DeviceType::Storage => &db.devices.storage,
        };

        devices.iter()
            .find(|d| Self::matches_identifier(&d.identifier, identifier))
            .cloned()
    }

    fn matches_identifier(db_id: &DeviceIdentifier, search_id: &DeviceIdentifier) -> bool {
        // Fuzzy matching logic
        db_id.manufacturer.eq_ignore_ascii_case(&search_id.manufacturer) &&
        db_id.model.eq_ignore_ascii_case(&search_id.model)
    }

    /// Add or update device in local database
    pub fn upsert_device(&self, device_info: DeviceDeepInfo) -> Result<()> {
        let mut db = self.load_database()?;

        // Add to appropriate category
        // Update last_updated timestamp

        self.save_database(&db)
    }

    fn load_database(&self) -> Result<LocalDatabase> {
        let content = std::fs::read_to_string(&self.db_file)?;
        let db = serde_json::from_str(&content)?;
        Ok(db)
    }

    fn save_database(&self, db: &LocalDatabase) -> Result<()> {
        let content = serde_json::to_string_pretty(db)?;
        std::fs::write(&self.db_file, content)?;
        Ok(())
    }
}
```

### Frontend Service

```typescript
// projects/ui/src/app/core/services/device-info.service.ts

import { Injectable } from '@angular/core';
import { invoke } from '@tauri-apps/api/core';
import { Observable, from } from 'rxjs';

export interface DeviceDeepInfo {
  deviceId: string;
  deviceType: 'Cpu' | 'Gpu' | 'Motherboard' | 'Memory' | 'Storage';
  identifier: DeviceIdentifier;
  specifications?: DeviceSpecifications;
  drivers?: DriverInfo;
  documentation?: DocumentationLinks;
  images?: ProductImages;
  metadata: DataMetadata;
}

export interface DriverInfo {
  installedVersion?: string;
  latestVersion?: string;
  downloadUrl?: string;
  releaseDate?: string;
  releaseNotesUrl?: string;
  driverPageUrl?: string;
  updateAvailable: boolean;
}

export interface DataMetadata {
  source: 'LocalDatabase' | 'ManufacturerWebsite' | 'ThirdPartyDatabase' | 'AiAgent' | 'Cache';
  lastUpdated: string;
  expiresAt: string;
  sourceUrl?: string;
  aiConfidence?: number;
}

@Injectable({
  providedIn: 'root'
})
export class DeviceInfoService {

  getDeviceDeepInfo(
    deviceId: string,
    deviceType: string,
    forceRefresh = false
  ): Observable<DeviceDeepInfo> {
    return from(
      invoke<DeviceDeepInfo>('get_device_deep_info', {
        deviceId,
        deviceType,
        forceRefresh
      })
    );
  }

  checkDriverUpdates(
    deviceId: string,
    deviceType: string
  ): Observable<DriverInfo> {
    return from(
      invoke<DriverInfo>('check_driver_updates', {
        deviceId,
        deviceType
      })
    );
  }

  aiLookupDevice(
    deviceId: string,
    deviceType: string
  ): Observable<DeviceDeepInfo> {
    return from(
      invoke<DeviceDeepInfo>('ai_lookup_device', {
        deviceId,
        deviceType
      })
    );
  }

  getCachedDevices(): Observable<DeviceDeepInfo[]> {
    return from(invoke<DeviceDeepInfo[]>('get_cached_devices'));
  }

  clearCache(deviceId?: string): Observable<void> {
    return from(invoke<void>('clear_device_cache', { deviceId }));
  }

  exportDeviceInfo(deviceIds: string[], outputPath: string): Observable<void> {
    return from(
      invoke<void>('export_device_info', { deviceIds, outputPath })
    );
  }
}
```

### UI Components

#### Device Detail Modal

```typescript
// projects/ui/src/app/shared/components/device-detail-modal/device-detail-modal.component.ts

import { Component, Input, OnInit } from '@angular/core';
import { DeviceInfoService, DeviceDeepInfo } from '@/core/services/device-info.service';

@Component({
  selector: 'app-device-detail-modal',
  templateUrl: './device-detail-modal.component.html',
  styleUrls: ['./device-detail-modal.component.css']
})
export class DeviceDetailModalComponent implements OnInit {
  @Input() deviceId!: string;
  @Input() deviceType!: string;

  deviceInfo?: DeviceDeepInfo;
  loading = true;
  error?: string;

  activeTab: 'specs' | 'drivers' | 'docs' = 'specs';

  constructor(private deviceInfoService: DeviceInfoService) {}

  ngOnInit() {
    this.loadDeviceInfo();
  }

  loadDeviceInfo(forceRefresh = false) {
    this.loading = true;
    this.error = undefined;

    this.deviceInfoService
      .getDeviceDeepInfo(this.deviceId, this.deviceType, forceRefresh)
      .subscribe({
        next: (info) => {
          this.deviceInfo = info;
          this.loading = false;
        },
        error: (err) => {
          this.error = err.message || 'Failed to load device information';
          this.loading = false;
        }
      });
  }

  refresh() {
    this.loadDeviceInfo(true);
  }

  aiLookup() {
    this.loading = true;

    this.deviceInfoService
      .aiLookupDevice(this.deviceId, this.deviceType)
      .subscribe({
        next: (info) => {
          this.deviceInfo = info;
          this.loading = false;
        },
        error: (err) => {
          this.error = err.message || 'AI lookup failed';
          this.loading = false;
        }
      });
  }

  getTimeSinceUpdate(): string {
    if (!this.deviceInfo?.metadata.lastUpdated) return 'Unknown';

    const updated = new Date(this.deviceInfo.metadata.lastUpdated);
    const now = new Date();
    const diffMs = now.getTime() - updated.getTime();
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

    if (diffDays === 0) return 'Today';
    if (diffDays === 1) return 'Yesterday';
    return `${diffDays} days ago`;
  }

  getSourceBadgeColor(): string {
    switch (this.deviceInfo?.metadata.source) {
      case 'ManufacturerWebsite': return 'bg-green-500';
      case 'ThirdPartyDatabase': return 'bg-blue-500';
      case 'AiAgent': return 'bg-purple-500';
      case 'LocalDatabase': return 'bg-gray-500';
      case 'Cache': return 'bg-yellow-500';
      default: return 'bg-gray-500';
    }
  }
}
```

```html
<!-- projects/ui/src/app/shared/components/device-detail-modal/device-detail-modal.component.html -->

<div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
  <div class="bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-4xl w-full max-h-[90vh] overflow-hidden">
    <!-- Header -->
    <div class="p-6 border-b border-gray-200 dark:border-gray-700">
      <div class="flex items-start justify-between">
        <div class="flex-1">
          <h2 class="text-2xl font-bold text-gray-900 dark:text-white">
            {{ deviceInfo?.identifier.manufacturer }} {{ deviceInfo?.identifier.model }}
          </h2>
          <p class="text-sm text-gray-500 dark:text-gray-400 mt-1">
            {{ deviceType }}
            @if (deviceInfo?.identifier.partNumber) {
              <span class="ml-2">Part #: {{ deviceInfo.identifier.partNumber }}</span>
            }
          </p>
        </div>

        <div class="flex items-center gap-2">
          <!-- Source Badge -->
          @if (deviceInfo) {
            <span class="px-3 py-1 rounded-full text-xs font-medium text-white {{ getSourceBadgeColor() }}">
              {{ deviceInfo.metadata.source }}
            </span>
          }

          <!-- Last Updated -->
          @if (deviceInfo) {
            <span class="text-xs text-gray-500 dark:text-gray-400">
              Updated: {{ getTimeSinceUpdate() }}
            </span>
          }

          <!-- Actions -->
          <button
            (click)="refresh()"
            class="p-2 hover:bg-gray-100 dark:hover:bg-gray-700 rounded"
            title="Refresh"
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
            </svg>
          </button>

          <button class="p-2 hover:bg-gray-100 dark:hover:bg-gray-700 rounded">
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
      </div>
    </div>

    <!-- Loading State -->
    @if (loading) {
      <div class="p-12 text-center">
        <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500 mx-auto"></div>
        <p class="mt-4 text-gray-600 dark:text-gray-400">Loading device information...</p>
      </div>
    }

    <!-- Error State -->
    @if (error && !loading) {
      <div class="p-6">
        <div class="bg-red-100 dark:bg-red-900 border border-red-400 text-red-700 dark:text-red-200 px-4 py-3 rounded">
          <p class="font-bold">Error</p>
          <p>{{ error }}</p>
          <button
            (click)="aiLookup()"
            class="mt-3 px-4 py-2 bg-purple-500 text-white rounded hover:bg-purple-600"
          >
            Try AI Lookup
          </button>
        </div>
      </div>
    }

    <!-- Content -->
    @if (deviceInfo && !loading) {
      <div class="flex h-[calc(90vh-140px)]">
        <!-- Product Image Sidebar -->
        @if (deviceInfo.images?.primaryImage) {
          <div class="w-1/3 p-6 border-r border-gray-200 dark:border-gray-700 overflow-y-auto">
            <img
              [src]="deviceInfo.images.primaryImage"
              [alt]="deviceInfo.identifier.model"
              class="w-full rounded-lg shadow-md"
            />

            @if (deviceInfo.images.gallery?.length) {
              <div class="mt-4 grid grid-cols-3 gap-2">
                @for (image of deviceInfo.images.gallery; track image) {
                  <img
                    [src]="image"
                    class="w-full h-20 object-cover rounded cursor-pointer hover:opacity-80"
                  />
                }
              </div>
            }
          </div>
        }

        <!-- Main Content -->
        <div class="flex-1 flex flex-col">
          <!-- Tabs -->
          <div class="border-b border-gray-200 dark:border-gray-700">
            <nav class="flex -mb-px">
              <button
                (click)="activeTab = 'specs'"
                [class.border-blue-500]="activeTab === 'specs'"
                [class.text-blue-600]="activeTab === 'specs'"
                class="px-6 py-3 border-b-2 font-medium text-sm hover:text-blue-600"
              >
                Specifications
              </button>

              @if (deviceInfo.drivers) {
                <button
                  (click)="activeTab = 'drivers'"
                  [class.border-blue-500]="activeTab === 'drivers'"
                  [class.text-blue-600]="activeTab === 'drivers'"
                  class="px-6 py-3 border-b-2 font-medium text-sm hover:text-blue-600 relative"
                >
                  Drivers
                  @if (deviceInfo.drivers.updateAvailable) {
                    <span class="absolute top-2 right-2 w-2 h-2 bg-red-500 rounded-full"></span>
                  }
                </button>
              }

              @if (deviceInfo.documentation) {
                <button
                  (click)="activeTab = 'docs'"
                  [class.border-blue-500]="activeTab === 'docs'"
                  [class.text-blue-600]="activeTab === 'docs'"
                  class="px-6 py-3 border-b-2 font-medium text-sm hover:text-blue-600"
                >
                  Documentation
                </button>
              }
            </nav>
          </div>

          <!-- Tab Content -->
          <div class="flex-1 overflow-y-auto p-6">
            <!-- Specifications Tab -->
            @if (activeTab === 'specs' && deviceInfo.specifications) {
              <div class="space-y-6">
                @if (deviceInfo.specifications.description) {
                  <p class="text-gray-700 dark:text-gray-300">
                    {{ deviceInfo.specifications.description }}
                  </p>
                }

                @for (category of deviceInfo.specifications.categories; track category.name) {
                  <div>
                    <h3 class="text-lg font-semibold mb-3 text-gray-900 dark:text-white">
                      {{ category.name }}
                    </h3>
                    <div class="grid grid-cols-2 gap-4">
                      @for (spec of category.specs; track spec.label) {
                        <div class="flex justify-between py-2 border-b border-gray-200 dark:border-gray-700">
                          <span class="text-gray-600 dark:text-gray-400">{{ spec.label }}</span>
                          <span class="font-medium text-gray-900 dark:text-white">
                            {{ spec.value }}{{ spec.unit ? ' ' + spec.unit : '' }}
                          </span>
                        </div>
                      }
                    </div>
                  </div>
                }
              </div>
            }

            <!-- Drivers Tab -->
            @if (activeTab === 'drivers' && deviceInfo.drivers) {
              <div class="space-y-4">
                <!-- Update Available Alert -->
                @if (deviceInfo.drivers.updateAvailable) {
                  <div class="bg-yellow-100 dark:bg-yellow-900 border border-yellow-400 text-yellow-700 dark:text-yellow-200 px-4 py-3 rounded">
                    <p class="font-bold">Update Available</p>
                    <p>A newer driver version is available for download.</p>
                  </div>
                }

                <!-- Version Info -->
                <div class="grid grid-cols-2 gap-4">
                  @if (deviceInfo.drivers.installedVersion) {
                    <div class="bg-gray-50 dark:bg-gray-700 p-4 rounded">
                      <p class="text-sm text-gray-600 dark:text-gray-400">Installed Version</p>
                      <p class="text-xl font-semibold mt-1">{{ deviceInfo.drivers.installedVersion }}</p>
                    </div>
                  }

                  @if (deviceInfo.drivers.latestVersion) {
                    <div class="bg-gray-50 dark:bg-gray-700 p-4 rounded">
                      <p class="text-sm text-gray-600 dark:text-gray-400">Latest Version</p>
                      <p class="text-xl font-semibold mt-1">{{ deviceInfo.drivers.latestVersion }}</p>
                      @if (deviceInfo.drivers.releaseDate) {
                        <p class="text-xs text-gray-500 mt-1">{{ deviceInfo.drivers.releaseDate }}</p>
                      }
                    </div>
                  }
                </div>

                <!-- Download Links -->
                <div class="space-y-2">
                  @if (deviceInfo.drivers.downloadUrl) {
                    <a
                      [href]="deviceInfo.drivers.downloadUrl"
                      target="_blank"
                      class="block w-full px-4 py-3 bg-blue-500 text-white text-center rounded hover:bg-blue-600 font-medium"
                    >
                      Download Latest Driver
                    </a>
                  }

                  @if (deviceInfo.drivers.releaseNotesUrl) {
                    <a
                      [href]="deviceInfo.drivers.releaseNotesUrl"
                      target="_blank"
                      class="block w-full px-4 py-3 border border-blue-500 text-blue-500 text-center rounded hover:bg-blue-50 dark:hover:bg-gray-700"
                    >
                      View Release Notes
                    </a>
                  }

                  @if (deviceInfo.drivers.driverPageUrl) {
                    <a
                      [href]="deviceInfo.drivers.driverPageUrl"
                      target="_blank"
                      class="block w-full px-4 py-3 border border-gray-300 text-gray-700 dark:text-gray-300 text-center rounded hover:bg-gray-50 dark:hover:bg-gray-700"
                    >
                      View All Drivers
                    </a>
                  }
                </div>
              </div>
            }

            <!-- Documentation Tab -->
            @if (activeTab === 'docs' && deviceInfo.documentation) {
              <div class="space-y-6">
                <!-- Product Pages -->
                @if (deviceInfo.documentation.productPage || deviceInfo.documentation.supportPage) {
                  <div>
                    <h3 class="text-lg font-semibold mb-3">Product Pages</h3>
                    <div class="space-y-2">
                      @if (deviceInfo.documentation.productPage) {
                        <a
                          [href]="deviceInfo.documentation.productPage"
                          target="_blank"
                          class="flex items-center p-3 border rounded hover:bg-gray-50 dark:hover:bg-gray-700"
                        >
                          <svg class="w-5 h-5 mr-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                          </svg>
                          Product Information Page
                        </a>
                      }

                      @if (deviceInfo.documentation.supportPage) {
                        <a
                          [href]="deviceInfo.documentation.supportPage"
                          target="_blank"
                          class="flex items-center p-3 border rounded hover:bg-gray-50 dark:hover:bg-gray-700"
                        >
                          <svg class="w-5 h-5 mr-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M18.364 5.636l-3.536 3.536m0 5.656l3.536 3.536M9.172 9.172L5.636 5.636m3.536 9.192l-3.536 3.536M21 12a9 9 0 11-18 0 9 9 0 0118 0zm-5 0a4 4 0 11-8 0 4 4 0 018 0z" />
                          </svg>
                          Support & Downloads
                        </a>
                      }
                    </div>
                  </div>
                }

                <!-- Manuals -->
                @if (deviceInfo.documentation.manuals?.length) {
                  <div>
                    <h3 class="text-lg font-semibold mb-3">User Manuals</h3>
                    <div class="space-y-2">
                      @for (manual of deviceInfo.documentation.manuals; track manual.url) {
                        <a
                          [href]="manual.url"
                          target="_blank"
                          class="flex items-center justify-between p-3 border rounded hover:bg-gray-50 dark:hover:bg-gray-700"
                        >
                          <div class="flex items-center">
                            <svg class="w-5 h-5 mr-3 text-red-500" fill="currentColor" viewBox="0 0 20 20">
                              <path fill-rule="evenodd" d="M4 4a2 2 0 012-2h4.586A2 2 0 0112 2.586L15.414 6A2 2 0 0116 7.414V16a2 2 0 01-2 2H6a2 2 0 01-2-2V4z" clip-rule="evenodd" />
                            </svg>
                            <span>{{ manual.title }}</span>
                          </div>
                          <span class="text-xs text-gray-500">{{ manual.fileType }}</span>
                        </a>
                      }
                    </div>
                  </div>
                }

                <!-- Datasheets -->
                @if (deviceInfo.documentation.datasheets?.length) {
                  <div>
                    <h3 class="text-lg font-semibold mb-3">Technical Datasheets</h3>
                    <div class="space-y-2">
                      @for (datasheet of deviceInfo.documentation.datasheets; track datasheet.url) {
                        <a
                          [href]="datasheet.url"
                          target="_blank"
                          class="flex items-center justify-between p-3 border rounded hover:bg-gray-50 dark:hover:bg-gray-700"
                        >
                          <div class="flex items-center">
                            <svg class="w-5 h-5 mr-3 text-blue-500" fill="currentColor" viewBox="0 0 20 20">
                              <path fill-rule="evenodd" d="M4 4a2 2 0 012-2h4.586A2 2 0 0112 2.586L15.414 6A2 2 0 0116 7.414V16a2 2 0 01-2 2H6a2 2 0 01-2-2V4z" clip-rule="evenodd" />
                            </svg>
                            <span>{{ datasheet.title }}</span>
                          </div>
                          <span class="text-xs text-gray-500">{{ datasheet.fileType }}</span>
                        </a>
                      }
                    </div>
                  </div>
                }

                <!-- Firmware Updates -->
                @if (deviceInfo.documentation.firmwareUpdates?.length) {
                  <div>
                    <h3 class="text-lg font-semibold mb-3">Firmware Updates</h3>
                    <div class="space-y-2">
                      @for (firmware of deviceInfo.documentation.firmwareUpdates; track firmware.url) {
                        <a
                          [href]="firmware.url"
                          target="_blank"
                          class="flex items-center justify-between p-3 border rounded hover:bg-gray-50 dark:hover:bg-gray-700"
                        >
                          <div>
                            <p class="font-medium">{{ firmware.title }}</p>
                            <p class="text-sm text-gray-600 dark:text-gray-400">
                              Version {{ firmware.version }}
                              @if (firmware.releaseDate) {
                                <span class="ml-2">• {{ firmware.releaseDate }}</span>
                              }
                            </p>
                          </div>
                          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
                          </svg>
                        </a>
                      }
                    </div>
                  </div>
                }
              </div>
            }
          </div>
        </div>
      </div>

      <!-- Footer with AI Confidence (if AI-sourced) -->
      @if (deviceInfo.metadata.source === 'AiAgent' && deviceInfo.metadata.aiConfidence) {
        <div class="px-6 py-3 bg-purple-50 dark:bg-purple-900 border-t border-purple-200 dark:border-purple-700">
          <div class="flex items-center justify-between">
            <div class="flex items-center">
              <svg class="w-5 h-5 mr-2 text-purple-600" fill="currentColor" viewBox="0 0 20 20">
                <path d="M13 7H7v6h6V7z" />
                <path fill-rule="evenodd" d="M7 2a1 1 0 012 0v1h2V2a1 1 0 112 0v1h2a2 2 0 012 2v2h1a1 1 0 110 2h-1v2h1a1 1 0 110 2h-1v2a2 2 0 01-2 2h-2v1a1 1 0 11-2 0v-1H9v1a1 1 0 11-2 0v-1H5a2 2 0 01-2-2v-2H2a1 1 0 110-2h1V9H2a1 1 0 010-2h1V5a2 2 0 012-2h2V2zM5 5h10v10H5V5z" clip-rule="evenodd" />
              </svg>
              <span class="text-sm text-purple-900 dark:text-purple-100">
                Information retrieved by AI agent
              </span>
            </div>
            <div class="flex items-center">
              <span class="text-xs text-purple-700 dark:text-purple-300 mr-2">Confidence:</span>
              <div class="w-32 h-2 bg-purple-200 dark:bg-purple-700 rounded-full overflow-hidden">
                <div
                  class="h-full bg-purple-600"
                  [style.width.%]="deviceInfo.metadata.aiConfidence * 100"
                ></div>
              </div>
              <span class="ml-2 text-sm font-medium text-purple-900 dark:text-purple-100">
                {{ (deviceInfo.metadata.aiConfidence * 100).toFixed(0) }}%
              </span>
            </div>
          </div>
        </div>
      }
    }
  </div>
</div>
```

### Integration with Existing Hardware Pages

```typescript
// Update existing hardware components to add "View Details" button

// Example: projects/ui/src/app/features/hardware/hardware.component.html
<div class="hardware-card">
  <h3>{{ cpu.name }}</h3>
  <p>{{ cpu.manufacturer }}</p>

  <button
    (click)="showDeviceDetails(cpu.id, 'Cpu')"
    class="mt-4 px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600"
  >
    View Details
  </button>
</div>

<!-- Add modal component -->
<app-device-detail-modal
  *ngIf="selectedDevice"
  [deviceId]="selectedDevice.id"
  [deviceType]="selectedDevice.type"
  (close)="selectedDevice = null"
></app-device-detail-modal>
```

## Implementation Plan

### Phase 1: Core Infrastructure (Week 1-2)
- [ ] Create data models (Rust structs, TypeScript interfaces)
- [ ] Implement CacheManager with AppData storage
- [ ] Implement LocalDatabaseManager
- [ ] Create bundled device_database.json with ~50 common devices
- [ ] Implement basic Tauri commands (get_device_deep_info, etc.)
- [ ] Create DeviceInfoService orchestration layer

### Phase 2: Internet Fetching (Week 3-4)
- [ ] Implement InternetFetcher with reqwest + scraper
- [ ] Add Intel ARK scraper for CPUs
- [ ] Add AMD scraper for CPUs/GPUs
- [ ] Add NVIDIA scraper for GPUs
- [ ] Add motherboard manufacturer scrapers (ASUS, MSI, Gigabyte, ASRock)
- [ ] Implement TechPowerUp fallback scraper
- [ ] Add driver version checking logic
- [ ] Implement cache TTL (7-day specs, 1-day drivers)

### Phase 3: AI Agent Integration (Week 5)
- [ ] Implement AiAgent with Claude API integration
- [ ] Create prompt templates for device lookup
- [ ] Add AI response parsing and validation
- [ ] Implement AI confidence scoring
- [ ] Add error handling and retry logic
- [ ] Store AI results in local database for future use

### Phase 4: Frontend UI (Week 6-7)
- [ ] Create DeviceDetailModalComponent
- [ ] Build specifications tab with categorized display
- [ ] Build drivers tab with update detection
- [ ] Build documentation tab with links
- [ ] Add product image viewer
- [ ] Implement data source badges and timestamps
- [ ] Add refresh and AI lookup buttons
- [ ] Integrate with existing hardware pages

### Phase 5: Polish & Testing (Week 8)
- [ ] Add loading states and error handling
- [ ] Implement offline mode detection
- [ ] Add cache cleanup on app startup
- [ ] Write unit tests for core services
- [ ] Write integration tests for scrapers
- [ ] Performance testing with large caches
- [ ] Documentation and code comments
- [ ] User acceptance testing

## Testing Strategy

### Unit Tests

**Rust Backend:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_manager_get_set() {
        let cache = CacheManager::new().unwrap();
        let device_info = create_test_device_info();

        cache.set("test-id".to_string(), DeviceType::Cpu, device_info.clone(), Duration::days(7)).unwrap();

        let retrieved = cache.get("test-id", &DeviceType::Cpu).unwrap();
        assert_eq!(retrieved.device_id, device_info.device_id);
    }

    #[test]
    fn test_cache_expiration() {
        // Test that expired entries are not returned
    }

    #[test]
    fn test_local_database_search() {
        let db = LocalDatabaseManager::new().unwrap();
        let identifier = DeviceIdentifier {
            manufacturer: "Intel".to_string(),
            model: "Core i7-12700K".to_string(),
            // ...
        };

        let result = db.find_device(&identifier, &DeviceType::Cpu);
        assert!(result.is_some());
    }

    #[tokio::test]
    async fn test_internet_fetcher_intel() {
        let fetcher = InternetFetcher::new();
        let result = fetcher.fetch_intel_cpu("i7-12700K").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_ai_agent_search() {
        let agent = AiAgent::new("test-key");
        // Mock API response
        // Test parsing logic
    }
}
```

**Frontend:**
```typescript
describe('DeviceInfoService', () => {
  it('should fetch device deep info', (done) => {
    service.getDeviceDeepInfo('test-id', 'Cpu').subscribe(info => {
      expect(info).toBeDefined();
      expect(info.deviceType).toBe('Cpu');
      done();
    });
  });

  it('should detect driver updates', (done) => {
    service.checkDriverUpdates('gpu-id', 'Gpu').subscribe(driverInfo => {
      expect(driverInfo.updateAvailable).toBeDefined();
      done();
    });
  });
});
```

### Integration Tests

- Test full data flow: Hardware collector → DeviceInfoService → Cache → Internet → AI
- Test offline operation (disconnect network, verify cached data)
- Test AI fallback when internet fetcher fails
- Test cache expiration and refresh logic

### Manual QA

- [ ] Verify device detail modal displays correctly for all device types
- [ ] Test with various hardware configurations
- [ ] Verify driver update detection for NVIDIA/AMD GPUs
- [ ] Test AI lookup with obscure/OEM hardware
- [ ] Verify offline operation with no internet
- [ ] Check data freshness timestamps
- [ ] Test cache cleanup on app restart

## Dependencies

### Rust Crates

```toml
# projects/backend/Cargo.toml

[dependencies]
# Existing dependencies
tauri = "2.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
sysinfo = "0.30"

# New dependencies for device info
reqwest = { version = "0.11", features = ["json"] }
scraper = "0.18"  # HTML parsing
chrono = { version = "0.4", features = ["serde"] }
tokio = { version = "1", features = ["full"] }
lazy_static = "1.4"
regex = "1.10"
```

### API Keys

- Anthropic Claude API key (stored in environment variable or config file)
- No API keys needed for web scraping (respect robots.txt)

## Configuration

```json
// AppData/Roaming/Syslens/config.json
{
  "deviceInfo": {
    "enableInternetLookup": true,
    "enableAiAgent": true,
    "cacheSpecsTtlDays": 7,
    "cacheDriversTtlDays": 1,
    "aiApiKey": "${CLAUDE_API_KEY}",  // From env var or user input
    "preferredSources": [
      "ManufacturerWebsite",
      "ThirdPartyDatabase",
      "LocalDatabase",
      "AiAgent"
    ]
  }
}
```

## Open Questions

- [ ] Should we bundle a larger initial database or keep it minimal and rely on internet/AI?
  - **Recommendation:** Start with ~50 popular devices, expand based on AI lookups

- [ ] How to handle rate limiting on manufacturer websites?
  - **Recommendation:** Implement exponential backoff, respect robots.txt, cache aggressively

- [ ] Should AI API key be user-provided or bundled (with usage limits)?
  - **Recommendation:** User-provided (via settings UI) with optional built-in key with daily limits

- [ ] How to verify scraped data accuracy?
  - **Recommendation:** Compare multiple sources, display source URL, allow user reporting

- [ ] Should we support custom/community device databases?
  - **Phase 2 feature:** Allow importing community-contributed device JSON files

## Success Metrics

- [ ] 95%+ of common devices found in local database or internet lookup
- [ ] Driver update detection accuracy >90% for major GPU vendors
- [ ] AI lookup success rate >70% for obscure hardware
- [ ] Cache hit rate >80% for repeat lookups
- [ ] Page load time <2s for cached data, <10s for fresh internet lookup
- [ ] Zero internet dependency for previously viewed devices

## Future Enhancements (Phase 2)

- Natural language query interface ("Find BIOS update for my motherboard")
- Process history database with usage pattern analysis
- AI-powered bottleneck detection and upgrade recommendations
- Benchmark score integration (PassMark, 3DMark, Geekbench)
- Community-contributed device database
- Automatic driver download and installation
- Hardware compatibility checker for upgrades
- Price tracking for replacement/upgrade parts

---

**End of Specification**
