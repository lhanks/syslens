//! Tauri commands for deep device information.

use crate::models::{DataSource, DeviceDeepInfo, DeviceIdentifier, DeviceType};
use crate::services::{AiAgent, CacheManager, InternetFetcher, LocalDatabaseManager};
use chrono::{Duration, Utc};
use std::sync::OnceLock;

/// Global cache manager instance
static CACHE_MANAGER: OnceLock<CacheManager> = OnceLock::new();

/// Global local database manager instance
static LOCAL_DB_MANAGER: OnceLock<LocalDatabaseManager> = OnceLock::new();

/// Global internet fetcher instance
static INTERNET_FETCHER: OnceLock<InternetFetcher> = OnceLock::new();

/// Global AI agent instance
static AI_AGENT: OnceLock<AiAgent> = OnceLock::new();

/// Get or initialize the cache manager.
fn get_cache_manager() -> &'static CacheManager {
    CACHE_MANAGER.get_or_init(|| {
        CacheManager::new().expect("Failed to initialize CacheManager")
    })
}

/// Get or initialize the local database manager.
fn get_local_db_manager() -> &'static LocalDatabaseManager {
    LOCAL_DB_MANAGER.get_or_init(|| {
        LocalDatabaseManager::new().expect("Failed to initialize LocalDatabaseManager")
    })
}

/// Get or initialize the internet fetcher.
fn get_internet_fetcher() -> &'static InternetFetcher {
    INTERNET_FETCHER.get_or_init(|| {
        InternetFetcher::new().expect("Failed to initialize InternetFetcher")
    })
}

/// Get or initialize the AI agent.
fn get_ai_agent() -> &'static AiAgent {
    AI_AGENT.get_or_init(|| {
        AiAgent::new().expect("Failed to initialize AiAgent")
    })
}

/// Get deep device information by device ID and type.
///
/// Data retrieval order:
/// 1. Check cache (unless force_refresh)
/// 2. Check local database
/// 3. Fetch from internet (manufacturer websites)
/// 4. AI agent lookup (intelligent web search)
#[tauri::command]
pub async fn get_device_deep_info(
    device_id: String,
    device_type: DeviceType,
    force_refresh: bool,
) -> Result<DeviceDeepInfo, String> {
    let cache = get_cache_manager();
    let local_db = get_local_db_manager();
    let fetcher = get_internet_fetcher();
    let ai_agent = get_ai_agent();

    // 1. Check cache first (unless force_refresh)
    if !force_refresh {
        if let Some(cached) = cache.get(&device_id, &device_type) {
            log::debug!("Cache hit for device: {}", device_id);
            return Ok(cached);
        }
    }

    // 2. Search local database
    // Create identifier from device_id for lookup
    let identifier = parse_device_identifier(&device_id, &device_type);
    if let Some(mut db_info) = local_db.find_device(&identifier, &device_type) {
        // Update metadata
        db_info.metadata.source = DataSource::LocalDatabase;
        db_info.metadata.last_updated = Utc::now();
        db_info.metadata.expires_at = Utc::now() + Duration::days(7);

        // Cache the result
        let cache_ttl = get_cache_ttl(&device_type);
        if let Err(e) = cache.set(device_id.clone(), device_type.clone(), db_info.clone(), cache_ttl) {
            log::warn!("Failed to cache device info: {}", e);
        }

        return Ok(db_info);
    }

    // 3. Fetch from internet (manufacturer websites)
    log::info!("Local DB miss, attempting internet fetch for: {}", device_id);
    match fetcher.fetch_device_info(&identifier, &device_type).await {
        Ok(mut web_info) => {
            log::info!("Successfully fetched device info from web for: {}", device_id);

            // Ensure device_id is set correctly
            web_info.device_id = device_id.clone();

            // Cache the result
            let cache_ttl = get_cache_ttl(&device_type);
            if let Err(e) = cache.set(device_id.clone(), device_type.clone(), web_info.clone(), cache_ttl) {
                log::warn!("Failed to cache web-fetched device info: {}", e);
            }

            return Ok(web_info);
        }
        Err(e) => {
            log::warn!("Internet fetch failed for {}: {}", device_id, e);
        }
    }

    // 4. AI agent lookup (intelligent web search)
    log::info!("Internet fetch failed, attempting AI agent lookup for: {}", device_id);
    match ai_agent.search_device(&identifier, &device_type).await {
        Ok(mut ai_info) => {
            log::info!(
                "AI Agent found device info for: {} (confidence: {:.2})",
                device_id,
                ai_info.metadata.ai_confidence.unwrap_or(0.0)
            );

            // Ensure device_id is set correctly
            ai_info.device_id = device_id.clone();

            // Cache with shorter TTL for AI results
            let cache_ttl = 3; // 3 days for AI-generated results
            if let Err(e) = cache.set(device_id.clone(), device_type.clone(), ai_info.clone(), cache_ttl) {
                log::warn!("Failed to cache AI-generated device info: {}", e);
            }

            return Ok(ai_info);
        }
        Err(e) => {
            log::warn!("AI agent lookup failed for {}: {}", device_id, e);
        }
    }

    // Return not found error
    Err(format!(
        "Device information not found for {} (type: {:?})",
        device_id, device_type
    ))
}

/// Parse device identifier from device_id string.
fn parse_device_identifier(device_id: &str, _device_type: &DeviceType) -> DeviceIdentifier {
    // Device ID format: "type-manufacturer-model" or just "manufacturer-model"
    let parts: Vec<&str> = device_id.split('-').collect();

    let (manufacturer, model) = if parts.len() >= 3 {
        // Format: "type-manufacturer-model-..."
        (
            parts[1].to_string(),
            parts[2..].join("-"),
        )
    } else if parts.len() == 2 {
        (parts[0].to_string(), parts[1].to_string())
    } else {
        (String::new(), device_id.to_string())
    };

    DeviceIdentifier {
        manufacturer,
        model,
        part_number: None,
        serial_number: None,
        hardware_ids: vec![],
    }
}

/// Get cache TTL in days based on device type.
fn get_cache_ttl(device_type: &DeviceType) -> i64 {
    match device_type {
        DeviceType::Gpu => 1, // Driver info changes frequently
        _ => 7,              // Specs rarely change
    }
}

/// Search for device information by manufacturer and model.
#[tauri::command]
pub async fn search_device_info(
    manufacturer: String,
    model: String,
    device_type: DeviceType,
) -> Result<Option<DeviceDeepInfo>, String> {
    let local_db = get_local_db_manager();

    let identifier = DeviceIdentifier {
        manufacturer,
        model,
        part_number: None,
        serial_number: None,
        hardware_ids: vec![],
    };

    Ok(local_db.find_device(&identifier, &device_type))
}

/// Get all cached device information.
#[tauri::command]
pub fn get_cached_devices() -> Result<Vec<DeviceDeepInfo>, String> {
    let cache = get_cache_manager();
    Ok(cache.get_all())
}

/// Clear cache for a specific device or all devices.
#[tauri::command]
pub fn clear_device_cache(device_id: Option<String>, device_type: Option<DeviceType>) -> Result<(), String> {
    let cache = get_cache_manager();

    match (device_id, device_type) {
        (Some(id), Some(dt)) => {
            cache.remove(&id, &dt).map_err(|e| e.to_string())
        }
        _ => {
            cache.clear().map_err(|e| e.to_string())
        }
    }
}

/// Cleanup expired cache entries.
#[tauri::command]
pub fn cleanup_device_cache() -> Result<usize, String> {
    let cache = get_cache_manager();
    cache.cleanup_expired().map_err(|e| e.to_string())
}

/// Get database statistics.
#[tauri::command]
pub fn get_device_database_stats() -> Result<DatabaseStatsResponse, String> {
    let cache = get_cache_manager();
    let local_db = get_local_db_manager();

    let cache_stats = cache.stats();
    let db_stats = local_db.stats();

    Ok(DatabaseStatsResponse {
        database_version: db_stats.version,
        database_last_updated: db_stats.last_updated,
        cpu_count: db_stats.cpu_count,
        gpu_count: db_stats.gpu_count,
        motherboard_count: db_stats.motherboard_count,
        memory_count: db_stats.memory_count,
        storage_count: db_stats.storage_count,
        cache_total_entries: cache_stats.total_entries,
        cache_valid_entries: cache_stats.valid_entries,
        cache_expired_entries: cache_stats.expired_entries,
    })
}

/// Response for database statistics.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseStatsResponse {
    pub database_version: String,
    pub database_last_updated: String,
    pub cpu_count: usize,
    pub gpu_count: usize,
    pub motherboard_count: usize,
    pub memory_count: usize,
    pub storage_count: usize,
    pub cache_total_entries: usize,
    pub cache_valid_entries: usize,
    pub cache_expired_entries: usize,
}
