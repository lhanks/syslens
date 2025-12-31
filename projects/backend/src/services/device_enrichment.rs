//! Device Enrichment Service.
//!
//! This service orchestrates multi-source fetching of device information,
//! manages caching, and provides a unified API for the frontend.

use crate::models::{
    DeviceIdentifier, DeviceType, ImageEntry, ImageMetadata, ImageType, ProductImages,
};
use crate::services::device_sources::{
    fetch_from_all_sources, merge_results, AMDProductSource, DeviceSource, IntelArkSource,
    ManufacturerSource, MemorySource, MonitorSource, MotherboardSource, StorageSource,
    TechPowerUpSource, WikiChipSource, WikipediaSource,
};
use crate::services::{ImageCache, KnowledgeStore, PartialDeviceInfo};
use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;

/// Default cache expiration for device info (30 days).
const DEFAULT_CACHE_EXPIRATION_DAYS: u64 = 30;

/// Enriched device information with all available data.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnrichedDeviceInfo {
    /// Device identifier
    pub identifier: DeviceIdentifier,

    /// Device type
    pub device_type: DeviceType,

    /// Product images
    pub images: Option<ProductImages>,

    /// Merged specifications
    pub specs: std::collections::HashMap<String, String>,

    /// Categorized specifications
    pub categories: Vec<crate::models::SpecCategory>,

    /// Device description
    pub description: Option<String>,

    /// Release date
    pub release_date: Option<String>,

    /// Product page URL
    pub product_page: Option<String>,

    /// Support page URL
    pub support_page: Option<String>,

    /// Documentation links
    pub documentation: Option<crate::models::DocumentationLinks>,

    /// Driver information
    pub drivers: Option<crate::models::DriverInfo>,

    /// Data sources used
    pub sources: Vec<String>,

    /// Overall confidence score (0.0-1.0)
    pub confidence: f32,

    /// When the data was last fetched
    pub fetched_at: Option<chrono::DateTime<chrono::Utc>>,

    /// Whether data is from cache
    pub from_cache: bool,
}

/// Device enrichment service that orchestrates multi-source fetching.
pub struct DeviceEnrichmentService {
    sources: Vec<Box<dyn DeviceSource>>,
    knowledge_store: Arc<KnowledgeStore>,
    image_cache: Arc<ImageCache>,
}

impl DeviceEnrichmentService {
    /// Create a new DeviceEnrichmentService.
    pub fn new(knowledge_store: Arc<KnowledgeStore>, image_cache: Arc<ImageCache>) -> Result<Self> {
        // Initialize all sources
        let mut sources: Vec<Box<dyn DeviceSource>> = Vec::new();

        // Add CPU sources (Intel/AMD official sources have highest priority)
        if let Ok(source) = IntelArkSource::new() {
            log::info!(
                "Registered Intel ARK source (priority {})",
                source.priority()
            );
            sources.push(Box::new(source));
        }

        if let Ok(source) = AMDProductSource::new() {
            log::info!(
                "Registered AMD Product source (priority {})",
                source.priority()
            );
            sources.push(Box::new(source));
        }

        // Add GPU source
        if let Ok(source) = TechPowerUpSource::new() {
            log::info!(
                "Registered TechPowerUp source (priority {})",
                source.priority()
            );
            sources.push(Box::new(source));
        }

        // Add WikiChip for detailed CPU architecture info
        if let Ok(source) = WikiChipSource::new() {
            log::info!(
                "Registered WikiChip source (priority {})",
                source.priority()
            );
            sources.push(Box::new(source));
        }

        // Add generic manufacturer source
        if let Ok(source) = ManufacturerSource::new() {
            log::info!(
                "Registered Manufacturer source (priority {})",
                source.priority()
            );
            sources.push(Box::new(source));
        }

        // Add Wikipedia as fallback
        if let Ok(source) = WikipediaSource::new() {
            log::info!(
                "Registered Wikipedia source (priority {})",
                source.priority()
            );
            sources.push(Box::new(source));
        }

        // Add motherboard source
        if let Ok(source) = MotherboardSource::new() {
            log::info!(
                "Registered Motherboard source (priority {})",
                source.priority()
            );
            sources.push(Box::new(source));
        }

        // Add storage source
        if let Ok(source) = StorageSource::new() {
            log::info!("Registered Storage source (priority {})", source.priority());
            sources.push(Box::new(source));
        }

        // Add memory source
        if let Ok(source) = MemorySource::new() {
            log::info!("Registered Memory source (priority {})", source.priority());
            sources.push(Box::new(source));
        }

        // Add monitor source
        if let Ok(source) = MonitorSource::new() {
            log::info!("Registered Monitor source (priority {})", source.priority());
            sources.push(Box::new(source));
        }

        // Sort by priority (lower = higher priority)
        sources.sort_by_key(|s| s.priority());

        log::info!(
            "DeviceEnrichmentService initialized with {} sources",
            sources.len()
        );

        Ok(Self {
            sources,
            knowledge_store,
            image_cache,
        })
    }

    /// Enrich a device with all available information.
    pub async fn enrich_device(
        &self,
        device_type: DeviceType,
        identifier: DeviceIdentifier,
        force_refresh: bool,
    ) -> Result<EnrichedDeviceInfo> {
        let device_id = self.generate_device_id(&device_type, &identifier);

        // Check cache first (unless force refresh)
        if !force_refresh {
            if let Some(cached) = self
                .check_cache(&device_id, &device_type, &identifier)
                .await?
            {
                if !self.is_stale(&cached) {
                    log::debug!("Using cached device info for {}", device_id);
                    return Ok(cached);
                }
            }
        }

        // Fetch from all sources in parallel
        log::info!(
            "Fetching device info from {} sources for {}",
            self.sources.len(),
            device_id
        );
        let results = fetch_from_all_sources(&self.sources, &device_type, &identifier).await;

        // Log results
        for result in &results {
            if result.error.is_some() {
                log::debug!("Source {} failed: {:?}", result.source_name, result.error);
            } else {
                log::debug!("Source {} succeeded", result.source_name);
            }
        }

        // Merge results
        let merged = merge_results(results);

        if merged.is_none() {
            return Err(anyhow::anyhow!(
                "No device information found from any source"
            ));
        }

        let partial = merged.unwrap();

        // Fetch and cache images
        let images = self
            .process_images(&partial, &device_type, &identifier)
            .await?;

        // Build enriched info
        let enriched = EnrichedDeviceInfo {
            identifier: identifier.clone(),
            device_type: device_type.clone(),
            images,
            specs: partial.specs.clone(),
            categories: partial.categories.clone(),
            description: partial.description.clone(),
            release_date: partial.release_date.clone(),
            product_page: partial.product_page.clone(),
            support_page: partial.support_page.clone(),
            documentation: partial.documentation.clone(),
            drivers: partial.driver_info.clone(),
            sources: partial.source_name.split(", ").map(String::from).collect(),
            confidence: partial.confidence,
            fetched_at: Some(chrono::Utc::now()),
            from_cache: false,
        };

        // Store in knowledge store
        self.store_in_cache(&device_id, &device_type, &identifier, &partial)
            .await?;

        Ok(enriched)
    }

    /// Generate a unique device ID.
    fn generate_device_id(
        &self,
        device_type: &DeviceType,
        identifier: &DeviceIdentifier,
    ) -> String {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(format!("{:?}", device_type).as_bytes());
        hasher.update(identifier.manufacturer.as_bytes());
        hasher.update(identifier.model.as_bytes());

        format!("{:x}", hasher.finalize())[..16].to_string()
    }

    /// Check knowledge store cache.
    async fn check_cache(
        &self,
        device_id: &str,
        device_type: &DeviceType,
        identifier: &DeviceIdentifier,
    ) -> Result<Option<EnrichedDeviceInfo>> {
        // Get from knowledge store - returns DeviceDeepInfo
        if let Some(device) = self.knowledge_store.get(device_id, device_type) {
            // Convert DeviceDeepInfo to EnrichedDeviceInfo
            let specs = device
                .specifications
                .as_ref()
                .map(|s| s.specs.clone())
                .unwrap_or_default();

            let categories = device
                .specifications
                .as_ref()
                .map(|s| s.categories.clone())
                .unwrap_or_default();

            let description = device
                .specifications
                .as_ref()
                .and_then(|s| s.description.clone());

            let release_date = device
                .specifications
                .as_ref()
                .and_then(|s| s.release_date.clone());

            let product_page = device
                .documentation
                .as_ref()
                .and_then(|d| d.product_page.clone());

            let support_page = device
                .documentation
                .as_ref()
                .and_then(|d| d.support_page.clone());

            let enriched = EnrichedDeviceInfo {
                identifier: identifier.clone(),
                device_type: device_type.clone(),
                images: device.images,
                specs,
                categories,
                description,
                release_date,
                product_page,
                support_page,
                documentation: device.documentation,
                drivers: device.drivers,
                sources: vec![device.metadata.source.to_string()],
                confidence: device.metadata.ai_confidence.unwrap_or(0.8),
                fetched_at: Some(device.metadata.last_updated),
                from_cache: true,
            };

            return Ok(Some(enriched));
        }

        Ok(None)
    }

    /// Check if cached data is stale.
    fn is_stale(&self, cached: &EnrichedDeviceInfo) -> bool {
        if let Some(fetched_at) = cached.fetched_at {
            let age = chrono::Utc::now() - fetched_at;
            age > chrono::Duration::days(DEFAULT_CACHE_EXPIRATION_DAYS as i64)
        } else {
            true // No timestamp means stale
        }
    }

    /// Process and cache images from partial device info.
    async fn process_images(
        &self,
        partial: &PartialDeviceInfo,
        device_type: &DeviceType,
        identifier: &DeviceIdentifier,
    ) -> Result<Option<ProductImages>> {
        let mut images = ProductImages::default();
        let cache_key_base = self.generate_device_id(device_type, identifier);

        // Cache primary image
        if let Some(ref url) = partial.image_url {
            let cache_key = format!("{}_primary", cache_key_base);
            match self
                .image_cache
                .fetch_and_cache_with_key(url, &cache_key)
                .await
            {
                Ok(result) => {
                    images.primary_image = Some(url.clone());
                    images.primary_image_cached =
                        Some(result.file_path.to_string_lossy().to_string());

                    // Generate thumbnail
                    if let Some(ref cached_path) = images.primary_image_cached {
                        match self
                            .image_cache
                            .generate_thumbnail(std::path::Path::new(cached_path), 128)
                            .await
                        {
                            Ok(thumb_path) => {
                                images.thumbnail_cached =
                                    Some(thumb_path.to_string_lossy().to_string());
                            }
                            Err(e) => {
                                log::warn!("Failed to generate thumbnail: {}", e);
                            }
                        }
                    }

                    images.metadata = Some(ImageMetadata {
                        fetched_at: chrono::Utc::now(),
                        source: partial.source_name.clone(),
                        ai_generated: false,
                        cache_key: cache_key.clone(),
                        file_size: None,
                    });
                }
                Err(e) => {
                    log::warn!("Failed to cache primary image: {}", e);
                    images.primary_image = Some(url.clone());
                }
            }
        }

        // Cache gallery images
        for (i, (url, _)) in partial.image_gallery.iter().enumerate().take(5) {
            let cache_key = format!("{}_gallery_{}", cache_key_base, i);
            match self
                .image_cache
                .fetch_and_cache_with_key(url, &cache_key)
                .await
            {
                Ok(result) => {
                    images.gallery.push(ImageEntry {
                        url: url.clone(),
                        cached_path: Some(result.file_path.to_string_lossy().to_string()),
                        image_type: ImageType::Product,
                        description: None,
                        width: None,
                        height: None,
                    });
                }
                Err(e) => {
                    log::warn!("Failed to cache gallery image: {}", e);
                    images.gallery.push(ImageEntry {
                        url: url.clone(),
                        cached_path: None,
                        image_type: ImageType::Product,
                        description: None,
                        width: None,
                        height: None,
                    });
                }
            }
        }

        // Return None if no images were found
        if images.primary_image.is_none() && images.gallery.is_empty() {
            Ok(None)
        } else {
            Ok(Some(images))
        }
    }

    /// Store enriched info in knowledge store.
    async fn store_in_cache(
        &self,
        device_id: &str,
        device_type: &DeviceType,
        identifier: &DeviceIdentifier,
        partial: &PartialDeviceInfo,
    ) -> Result<()> {
        self.knowledge_store.store_or_merge(
            device_id.to_string(),
            device_type.clone(),
            identifier.clone(),
            partial.clone(),
        )?;

        Ok(())
    }

    /// Get a list of available sources.
    pub fn list_sources(&self) -> Vec<SourceInfo> {
        self.sources
            .iter()
            .map(|s| SourceInfo {
                name: s.name().to_string(),
                priority: s.priority(),
            })
            .collect()
    }

    /// Cleanup old cached data.
    pub async fn cleanup(&self, max_age_days: u64) -> Result<CleanupResult> {
        let images_cleaned = self
            .image_cache
            .cleanup_older_than(Duration::from_secs(max_age_days * 24 * 60 * 60))
            .await?;

        Ok(CleanupResult {
            images_removed: images_cleaned,
        })
    }
}

/// Information about a data source.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceInfo {
    pub name: String,
    pub priority: u8,
}

/// Result of cleanup operation.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupResult {
    pub images_removed: usize,
}
