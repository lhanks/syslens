//! Tauri commands for deep device information.

use crate::models::{DataSource, DeviceDeepInfo, DeviceIdentifier, DeviceType};
use crate::services::device_sources::{fetch_from_all_sources, merge_results, DeviceSource, WikipediaSource};
use crate::services::{AiAgent, CacheManager, ClaudeClient, ImageCache, InternetFetcher, KnowledgeStore, LocalDatabaseManager};
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

/// Global Claude client instance
static CLAUDE_CLIENT: OnceLock<ClaudeClient> = OnceLock::new();

/// Global knowledge store instance
static KNOWLEDGE_STORE: OnceLock<KnowledgeStore> = OnceLock::new();

/// Global image cache instance
static IMAGE_CACHE: OnceLock<ImageCache> = OnceLock::new();

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

/// Get or initialize the Claude client.
fn get_claude_client() -> &'static ClaudeClient {
    CLAUDE_CLIENT.get_or_init(|| {
        ClaudeClient::new().expect("Failed to initialize ClaudeClient")
    })
}

/// Get or initialize the knowledge store.
fn get_knowledge_store() -> &'static KnowledgeStore {
    KNOWLEDGE_STORE.get_or_init(|| {
        KnowledgeStore::new().expect("Failed to initialize KnowledgeStore")
    })
}

/// Get or initialize the image cache.
fn get_image_cache() -> &'static ImageCache {
    IMAGE_CACHE.get_or_init(|| {
        ImageCache::new().expect("Failed to initialize ImageCache")
    })
}

/// Build the list of device sources.
fn build_device_sources() -> Vec<Box<dyn DeviceSource>> {
    let mut sources: Vec<Box<dyn DeviceSource>> = Vec::new();

    // Add Wikipedia source
    if let Ok(wiki) = WikipediaSource::new() {
        sources.push(Box::new(wiki));
    }

    sources
}

/// Get deep device information by device ID and type.
///
/// Data retrieval order:
/// 1. Check cache (unless force_refresh)
/// 2. Check knowledge store (learned devices)
/// 3. Check local database (bundled)
/// 4. Multi-source fetch (Wikipedia, etc.)
/// 5. Manufacturer websites fallback
/// 6. Claude AI lookup (if ANTHROPIC_API_KEY is set)
/// 7. Web scraping AI agent lookup (fallback)
#[tauri::command]
pub async fn get_device_deep_info(
    device_id: String,
    device_type: DeviceType,
    force_refresh: bool,
) -> Result<DeviceDeepInfo, String> {
    log::info!(
        "get_device_deep_info called: device_id={}, device_type={:?}, force_refresh={}",
        device_id, device_type, force_refresh
    );

    let cache = get_cache_manager();
    let knowledge_store = get_knowledge_store();
    let local_db = get_local_db_manager();
    let fetcher = get_internet_fetcher();
    let ai_agent = get_ai_agent();

    // Parse device identifier from ID
    let identifier = parse_device_identifier(&device_id, &device_type);
    log::debug!("Parsed identifier: manufacturer={}, model={}", identifier.manufacturer, identifier.model);

    // 1. Check cache first (unless force_refresh)
    if !force_refresh {
        if let Some(cached) = cache.get(&device_id, &device_type) {
            log::info!("Cache hit for device: {}", device_id);
            return Ok(cached);
        }
    }

    // 2. Check knowledge store (learned devices)
    if !force_refresh {
        if let Some(learned) = knowledge_store.get(&device_id, &device_type) {
            log::info!("Knowledge store hit for device: {}", device_id);
            // Cache the result
            let cache_ttl = get_cache_ttl(&device_type);
            if let Err(e) = cache.set(device_id.clone(), device_type.clone(), learned.clone(), cache_ttl) {
                log::warn!("Failed to cache learned device info: {}", e);
            }
            return Ok(learned);
        }
    }

    // 3. Search local database (bundled)
    if let Some(mut db_info) = local_db.find_device(&identifier, &device_type) {
        log::info!("Local DB hit for device: {}", device_id);
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

    // 4. Multi-source fetch (Wikipedia, etc.)
    log::info!("Trying multi-source fetch for: {}", device_id);
    let sources = build_device_sources();
    if !sources.is_empty() {
        let results = fetch_from_all_sources(&sources, &device_type, &identifier).await;
        let successful_count = results.iter().filter(|r| r.partial_info.is_some()).count();
        log::info!("Multi-source fetch: {} sources returned data", successful_count);

        if let Some(merged) = merge_results(results) {
            log::info!("Successfully merged data from sources: {}", merged.source_name);

            // Store in knowledge store for future lookups
            if let Err(e) = knowledge_store.store_or_merge(
                device_id.clone(),
                device_type.clone(),
                identifier.clone(),
                merged,
            ) {
                log::warn!("Failed to store in knowledge store: {}", e);
            }

            // Retrieve the stored info (now with proper formatting)
            if let Some(learned_info) = knowledge_store.get(&device_id, &device_type) {
                let cache_ttl = get_cache_ttl(&device_type);
                if let Err(e) = cache.set(device_id.clone(), device_type.clone(), learned_info.clone(), cache_ttl) {
                    log::warn!("Failed to cache multi-source device info: {}", e);
                }
                return Ok(learned_info);
            }
        }
    }

    // 5. Fetch from manufacturer websites
    log::info!("Trying manufacturer websites for: {}", device_id);
    match fetcher.fetch_device_info(&identifier, &device_type).await {
        Ok(mut web_info) => {
            log::info!("Successfully fetched device info from web for: {}", device_id);
            web_info.device_id = device_id.clone();

            let cache_ttl = get_cache_ttl(&device_type);
            if let Err(e) = cache.set(device_id.clone(), device_type.clone(), web_info.clone(), cache_ttl) {
                log::warn!("Failed to cache web-fetched device info: {}", e);
            }

            return Ok(web_info);
        }
        Err(e) => {
            log::warn!("Manufacturer website fetch failed for {}: {}", device_id, e);
        }
    }

    // 6. Claude AI lookup (if API key is configured)
    let claude_client = get_claude_client();
    if claude_client.is_available() {
        log::info!("Trying Claude AI lookup for: {}", device_id);
        match claude_client.lookup_device(&identifier, &device_type).await {
            Ok(mut claude_info) => {
                log::info!(
                    "Claude AI found device info for: {} (confidence: {:.2})",
                    device_id,
                    claude_info.metadata.ai_confidence.unwrap_or(0.0)
                );

                claude_info.device_id = device_id.clone();

                let cache_ttl = 3; // 3 days for AI-generated results
                if let Err(e) = cache.set(device_id.clone(), device_type.clone(), claude_info.clone(), cache_ttl) {
                    log::warn!("Failed to cache Claude AI device info: {}", e);
                }

                return Ok(claude_info);
            }
            Err(e) => {
                log::warn!("Claude AI lookup failed for {}: {}", device_id, e);
            }
        }
    } else {
        log::debug!("Claude AI not available (API key not configured)");
    }

    // 7. Web scraping AI agent lookup (fallback)
    log::info!("Trying web scraping AI agent lookup for: {}", device_id);
    match ai_agent.search_device(&identifier, &device_type).await {
        Ok(mut ai_info) => {
            log::info!(
                "AI Agent found device info for: {} (confidence: {:.2})",
                device_id,
                ai_info.metadata.ai_confidence.unwrap_or(0.0)
            );

            ai_info.device_id = device_id.clone();

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

// =============================================================================
// Image Cache Commands
// =============================================================================

/// Response for image fetch operations.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageCacheResponse {
    pub cache_key: String,
    pub file_path: String,
    pub is_cached: bool,
    pub thumbnail_path: Option<String>,
}

/// Response for image cache statistics.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageCacheStatsResponse {
    pub hits: u64,
    pub misses: u64,
    pub downloads: u64,
    pub download_failures: u64,
    pub total_bytes_cached: u64,
    pub cached_count: usize,
    pub total_size_bytes: u64,
    pub cache_dir: String,
}

/// Fetch and cache a device image from URL.
///
/// Returns the local file path to the cached image.
#[tauri::command]
pub async fn fetch_device_image(url: String) -> Result<ImageCacheResponse, String> {
    let image_cache = get_image_cache();

    match image_cache.fetch_and_cache(&url).await {
        Ok(result) => {
            log::info!("Image cached: {} -> {:?}", url, result.file_path);
            Ok(ImageCacheResponse {
                cache_key: result.cache_key,
                file_path: result.file_path.to_string_lossy().to_string(),
                is_cached: result.is_cached,
                thumbnail_path: result.thumbnail_path.map(|p| p.to_string_lossy().to_string()),
            })
        }
        Err(e) => {
            log::warn!("Failed to fetch image {}: {}", url, e);
            Err(e.to_string())
        }
    }
}

/// Fetch and cache a device image with a custom cache key.
///
/// Useful for associating images with specific device identifiers.
#[tauri::command]
pub async fn fetch_device_image_with_key(
    url: String,
    cache_key: String,
) -> Result<ImageCacheResponse, String> {
    let image_cache = get_image_cache();

    match image_cache.fetch_and_cache_with_key(&url, &cache_key).await {
        Ok(result) => {
            log::info!("Image cached with key {}: {} -> {:?}", cache_key, url, result.file_path);
            Ok(ImageCacheResponse {
                cache_key: result.cache_key,
                file_path: result.file_path.to_string_lossy().to_string(),
                is_cached: result.is_cached,
                thumbnail_path: result.thumbnail_path.map(|p| p.to_string_lossy().to_string()),
            })
        }
        Err(e) => {
            log::warn!("Failed to fetch image {} with key {}: {}", url, cache_key, e);
            Err(e.to_string())
        }
    }
}

/// Get the cached path for an image by cache key.
///
/// Returns None if the image is not cached.
#[tauri::command]
pub async fn get_cached_image_path(cache_key: String) -> Result<Option<String>, String> {
    let image_cache = get_image_cache();

    Ok(image_cache
        .get_cached_path(&cache_key)
        .await
        .map(|p| p.to_string_lossy().to_string()))
}

/// Check if an image is cached by cache key.
#[tauri::command]
pub async fn is_image_cached(cache_key: String) -> Result<bool, String> {
    let image_cache = get_image_cache();
    Ok(image_cache.is_cached(&cache_key).await)
}

/// Generate a cache key for a device image.
#[tauri::command]
pub fn generate_device_image_cache_key(
    device_type: String,
    manufacturer: String,
    model: String,
) -> String {
    ImageCache::generate_device_cache_key(&device_type, &manufacturer, &model)
}

/// Get image cache statistics.
#[tauri::command]
pub async fn get_image_cache_stats() -> Result<ImageCacheStatsResponse, String> {
    let image_cache = get_image_cache();
    let (hits, misses, downloads, download_failures, total_bytes) = image_cache.get_stats();

    Ok(ImageCacheStatsResponse {
        hits,
        misses,
        downloads,
        download_failures,
        total_bytes_cached: total_bytes,
        cached_count: image_cache.cached_count().await,
        total_size_bytes: image_cache.total_size().await,
        cache_dir: image_cache.cache_dir().to_string_lossy().to_string(),
    })
}

/// Cleanup old cached images.
///
/// Removes images older than the specified number of days.
#[tauri::command]
pub async fn cleanup_image_cache(max_age_days: u64) -> Result<usize, String> {
    let image_cache = get_image_cache();
    let max_age = std::time::Duration::from_secs(max_age_days * 24 * 60 * 60);

    image_cache
        .cleanup_older_than(max_age)
        .await
        .map_err(|e| e.to_string())
}

// =============================================================================
// Device Enrichment Commands
// =============================================================================

use crate::services::{DeviceEnrichmentService, EnrichedDeviceInfo};
use std::sync::Arc;

/// Global enrichment service instance
static ENRICHMENT_SERVICE: OnceLock<DeviceEnrichmentService> = OnceLock::new();

/// Get or initialize the device enrichment service.
fn get_enrichment_service() -> &'static DeviceEnrichmentService {
    ENRICHMENT_SERVICE.get_or_init(|| {
        // Create dedicated instances for the enrichment service
        let knowledge_store = Arc::new(
            KnowledgeStore::new().expect("Failed to create KnowledgeStore for enrichment")
        );
        let image_cache = Arc::new(
            ImageCache::new().expect("Failed to create ImageCache for enrichment")
        );
        DeviceEnrichmentService::new(knowledge_store, image_cache)
            .expect("Failed to initialize DeviceEnrichmentService")
    })
}

/// Enrich a device with comprehensive information from multiple sources.
///
/// This fetches product images, specifications, documentation, and driver info.
#[tauri::command]
pub async fn enrich_device(
    device_type: DeviceType,
    manufacturer: String,
    model: String,
    force_refresh: bool,
) -> Result<EnrichedDeviceInfo, String> {
    let enrichment_service = get_enrichment_service();

    let identifier = DeviceIdentifier {
        manufacturer,
        model,
        part_number: None,
        serial_number: None,
        hardware_ids: vec![],
    };

    enrichment_service
        .enrich_device(device_type, identifier, force_refresh)
        .await
        .map_err(|e| e.to_string())
}

/// Response for enrichment source listing.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnrichmentSourceResponse {
    pub name: String,
    pub priority: u8,
}

/// List available enrichment sources.
#[tauri::command]
pub fn list_enrichment_sources() -> Vec<EnrichmentSourceResponse> {
    let enrichment_service = get_enrichment_service();

    enrichment_service
        .list_sources()
        .into_iter()
        .map(|s| EnrichmentSourceResponse {
            name: s.name,
            priority: s.priority,
        })
        .collect()
}

/// Cleanup all cached device data.
///
/// Removes images and metadata older than the specified number of days.
#[tauri::command]
pub async fn cleanup_enrichment_cache(max_age_days: u64) -> Result<CleanupResponse, String> {
    let enrichment_service = get_enrichment_service();

    let result = enrichment_service
        .cleanup(max_age_days)
        .await
        .map_err(|e| e.to_string())?;

    Ok(CleanupResponse {
        images_removed: result.images_removed,
    })
}

/// Response for cleanup operation.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupResponse {
    pub images_removed: usize,
}
