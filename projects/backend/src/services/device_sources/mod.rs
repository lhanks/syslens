//! Device information sources for multi-source fetching.
//!
//! This module provides a trait-based architecture for fetching device information
//! from multiple sources (Wikipedia, manufacturer sites, etc.) and merging results.

pub mod amdproduct;
pub mod intelark;
pub mod manufacturer;
pub mod memory;
pub mod monitor;
pub mod motherboard;
pub mod storage;
pub mod techpowerup;
pub mod wikichip;
pub mod wikipedia;

use crate::models::{DeviceIdentifier, DeviceType};
use crate::services::PartialDeviceInfo;
use anyhow::Result;
use async_trait::async_trait;

/// Result from a device source fetch operation.
#[derive(Debug, Clone)]
pub struct SourceResult {
    pub source_name: String,
    pub partial_info: Option<PartialDeviceInfo>,
    pub error: Option<String>,
}

impl SourceResult {
    pub fn success(source_name: String, info: PartialDeviceInfo) -> Self {
        Self {
            source_name,
            partial_info: Some(info),
            error: None,
        }
    }

    pub fn failure(source_name: String, error: String) -> Self {
        Self {
            source_name,
            partial_info: None,
            error: Some(error),
        }
    }
}

/// Trait for device information sources.
///
/// Implement this trait to add new data sources for device information.
/// Each source should fetch partial device info and report its confidence level.
#[async_trait]
pub trait DeviceSource: Send + Sync {
    /// Name of this source (e.g., "Wikipedia", "NVIDIA Official")
    fn name(&self) -> &str;

    /// Priority level (lower = higher priority, 0-255)
    /// Used to determine order when fetching and for tie-breaking confidence.
    fn priority(&self) -> u8;

    /// Check if this source supports the given device type and identifier.
    fn supports(&self, device_type: &DeviceType, identifier: &DeviceIdentifier) -> bool;

    /// Fetch device information from this source.
    ///
    /// Returns partial device info with whatever data could be found.
    /// The confidence field indicates how reliable the data is (0.0-1.0).
    async fn fetch(
        &self,
        device_type: &DeviceType,
        identifier: &DeviceIdentifier,
    ) -> Result<PartialDeviceInfo>;
}

/// Fetch from multiple sources in parallel and return all results.
pub async fn fetch_from_all_sources(
    sources: &[Box<dyn DeviceSource>],
    device_type: &DeviceType,
    identifier: &DeviceIdentifier,
) -> Vec<SourceResult> {
    use futures::future::join_all;

    let futures = sources
        .iter()
        .filter(|s| s.supports(device_type, identifier))
        .map(|source| async {
            let name = source.name().to_string();
            match source.fetch(device_type, identifier).await {
                Ok(info) => SourceResult::success(name, info),
                Err(e) => SourceResult::failure(name, e.to_string()),
            }
        });

    join_all(futures).await
}

/// Merge multiple partial results into a single PartialDeviceInfo.
///
/// Uses confidence-weighted merging:
/// - Higher confidence values win for conflicting specs
/// - All sources are recorded
/// - Categories from highest-confidence source are used
pub fn merge_results(results: Vec<SourceResult>) -> Option<PartialDeviceInfo> {
    let successful: Vec<_> = results
        .into_iter()
        .filter_map(|r| r.partial_info)
        .collect();

    if successful.is_empty() {
        return None;
    }

    // Sort by confidence (highest first)
    let mut sorted = successful;
    sorted.sort_by(|a, b| {
        b.confidence
            .partial_cmp(&a.confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Start with highest confidence result
    let mut merged = sorted.remove(0);
    let mut all_sources = vec![merged.source_name.clone()];

    // Merge in other results
    for partial in sorted {
        all_sources.push(partial.source_name.clone());

        // Add specs that don't exist yet
        for (key, value) in partial.specs {
            merged.specs.entry(key).or_insert(value);
        }

        // Use description if we don't have one
        if merged.description.is_none() && partial.description.is_some() {
            merged.description = partial.description;
        }

        // Use release date if we don't have one
        if merged.release_date.is_none() && partial.release_date.is_some() {
            merged.release_date = partial.release_date;
        }

        // Use URLs if we don't have them
        if merged.product_page.is_none() && partial.product_page.is_some() {
            merged.product_page = partial.product_page;
        }
        if merged.support_page.is_none() && partial.support_page.is_some() {
            merged.support_page = partial.support_page;
        }
        if merged.image_url.is_none() && partial.image_url.is_some() {
            merged.image_url = partial.image_url;
        }
    }

    // Update source name to reflect all sources
    merged.source_name = all_sources.join(", ");

    Some(merged)
}

// Re-export sources
pub use amdproduct::AMDProductSource;
pub use intelark::IntelArkSource;
pub use manufacturer::ManufacturerSource;
pub use memory::MemorySource;
pub use monitor::MonitorSource;
pub use motherboard::MotherboardSource;
pub use storage::StorageSource;
pub use techpowerup::TechPowerUpSource;
pub use wikichip::WikiChipSource;
pub use wikipedia::WikipediaSource;
