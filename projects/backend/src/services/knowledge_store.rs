//! Knowledge store for persisting learned device information.
//!
//! Unlike the cache which expires, the knowledge store persists device info
//! learned from various internet sources with confidence scoring.

use crate::models::{
    DataMetadata, DataSource, DeviceDeepInfo, DeviceIdentifier, DeviceSpecifications,
    DeviceType, SpecCategory, SpecItem,
};
use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::RwLock;

/// A specification with confidence and source tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LearnedSpec {
    pub value: String,
    pub confidence: f32,
    pub sources: Vec<String>,
    pub last_updated: DateTime<Utc>,
}

/// Source information for a learned device entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceInfo {
    pub name: String,
    pub url: Option<String>,
    pub confidence: f32,
    pub fetched_at: DateTime<Utc>,
}

/// A learned device entry with merged information from multiple sources.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LearnedDevice {
    pub device_id: String,
    pub device_type: DeviceType,
    pub identifier: DeviceIdentifier,
    /// Merged specifications with confidence
    pub specs: HashMap<String, LearnedSpec>,
    /// Categorized specs for display
    pub categories: Vec<SpecCategory>,
    /// Description from best source
    pub description: Option<String>,
    /// Release date if known
    pub release_date: Option<String>,
    /// All sources that contributed to this entry
    pub sources: Vec<SourceInfo>,
    /// When this entry was first created
    pub created_at: DateTime<Utc>,
    /// When any source last verified/updated this entry
    pub last_verified: DateTime<Utc>,
}

/// The knowledge database structure.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeDatabase {
    pub version: String,
    pub devices: Vec<LearnedDevice>,
}

/// Partial device info returned by a source (may have missing fields).
#[derive(Debug, Clone, Default)]
pub struct PartialDeviceInfo {
    pub specs: HashMap<String, String>,
    pub categories: Vec<SpecCategory>,
    pub description: Option<String>,
    pub release_date: Option<String>,
    pub product_page: Option<String>,
    pub support_page: Option<String>,
    pub image_url: Option<String>,
    pub source_name: String,
    pub source_url: Option<String>,
    pub confidence: f32,
}

/// Manages learned device knowledge with persistence.
pub struct KnowledgeStore {
    db_file: PathBuf,
    database: RwLock<KnowledgeDatabase>,
}

impl KnowledgeStore {
    /// Create a new KnowledgeStore, loading existing data from disk.
    pub fn new() -> Result<Self> {
        let db_dir = Self::get_app_data_dir()?;
        std::fs::create_dir_all(&db_dir).context("Failed to create knowledge store directory")?;

        let db_file = db_dir.join("learned_devices.json");
        let database = Self::load_database(&db_file).unwrap_or_else(|_| KnowledgeDatabase {
            version: "1.0.0".to_string(),
            devices: Vec::new(),
        });

        Ok(Self {
            db_file,
            database: RwLock::new(database),
        })
    }

    /// Get the app data directory for Syslens.
    fn get_app_data_dir() -> Result<PathBuf> {
        dirs::data_dir()
            .map(|p| p.join("Syslens"))
            .context("Failed to get app data directory")
    }

    /// Load database from disk.
    fn load_database(path: &PathBuf) -> Result<KnowledgeDatabase> {
        if !path.exists() {
            return Ok(KnowledgeDatabase::default());
        }

        let content = std::fs::read_to_string(path).context("Failed to read knowledge database")?;
        let db =
            serde_json::from_str(&content).context("Failed to parse knowledge database")?;
        Ok(db)
    }

    /// Save database to disk.
    fn save_database(&self) -> Result<()> {
        let db = self
            .database
            .read()
            .map_err(|_| anyhow::anyhow!("Failed to acquire read lock"))?;
        let content =
            serde_json::to_string_pretty(&*db).context("Failed to serialize knowledge database")?;
        std::fs::write(&self.db_file, content).context("Failed to write knowledge database")?;
        Ok(())
    }

    /// Look up a device by ID and type.
    pub fn get(&self, device_id: &str, device_type: &DeviceType) -> Option<DeviceDeepInfo> {
        let db = self.database.read().ok()?;

        db.devices
            .iter()
            .find(|d| d.device_id == device_id && &d.device_type == device_type)
            .map(|learned| self.convert_to_device_deep_info(learned))
    }

    /// Look up a device by model name (fuzzy match).
    pub fn find_by_model(
        &self,
        model: &str,
        device_type: &DeviceType,
    ) -> Option<DeviceDeepInfo> {
        let db = self.database.read().ok()?;
        let model_lower = model.to_lowercase();

        // Try exact match first
        if let Some(learned) = db.devices.iter().find(|d| {
            &d.device_type == device_type
                && d.identifier.model.to_lowercase() == model_lower
        }) {
            return Some(self.convert_to_device_deep_info(learned));
        }

        // Try fuzzy match
        db.devices
            .iter()
            .find(|d| {
                &d.device_type == device_type
                    && (d.identifier.model.to_lowercase().contains(&model_lower)
                        || model_lower.contains(&d.identifier.model.to_lowercase()))
            })
            .map(|learned| self.convert_to_device_deep_info(learned))
    }

    /// Store or merge device info from a source.
    pub fn store_or_merge(
        &self,
        device_id: String,
        device_type: DeviceType,
        identifier: DeviceIdentifier,
        partial: PartialDeviceInfo,
    ) -> Result<()> {
        let mut db = self
            .database
            .write()
            .map_err(|_| anyhow::anyhow!("Failed to acquire write lock"))?;

        let now = Utc::now();
        let source_info = SourceInfo {
            name: partial.source_name.clone(),
            url: partial.source_url.clone(),
            confidence: partial.confidence,
            fetched_at: now,
        };

        // Find existing entry or create new one
        if let Some(existing) = db
            .devices
            .iter_mut()
            .find(|d| d.device_id == device_id && d.device_type == device_type)
        {
            // Merge specs
            for (key, value) in partial.specs {
                let normalized_key = Self::normalize_spec_key(&key);
                if let Some(existing_spec) = existing.specs.get_mut(&normalized_key) {
                    // If new source has higher confidence, update value
                    if partial.confidence > existing_spec.confidence {
                        existing_spec.value = value;
                        existing_spec.confidence = partial.confidence;
                    }
                    // Add source if not already present
                    if !existing_spec.sources.contains(&partial.source_name) {
                        existing_spec.sources.push(partial.source_name.clone());
                    }
                    existing_spec.last_updated = now;
                } else {
                    existing.specs.insert(
                        normalized_key,
                        LearnedSpec {
                            value,
                            confidence: partial.confidence,
                            sources: vec![partial.source_name.clone()],
                            last_updated: now,
                        },
                    );
                }
            }

            // Update categories if new source has more
            if partial.categories.len() > existing.categories.len() {
                existing.categories = partial.categories;
            }

            // Update description if better confidence
            if partial.description.is_some() && partial.confidence > 0.5 {
                existing.description = partial.description;
            }

            // Update release date if not set
            if existing.release_date.is_none() && partial.release_date.is_some() {
                existing.release_date = partial.release_date;
            }

            // Add source
            if !existing.sources.iter().any(|s| s.name == partial.source_name) {
                existing.sources.push(source_info);
            }

            existing.last_verified = now;
        } else {
            // Create new entry
            let mut specs = HashMap::new();
            for (key, value) in partial.specs {
                let normalized_key = Self::normalize_spec_key(&key);
                specs.insert(
                    normalized_key,
                    LearnedSpec {
                        value,
                        confidence: partial.confidence,
                        sources: vec![partial.source_name.clone()],
                        last_updated: now,
                    },
                );
            }

            db.devices.push(LearnedDevice {
                device_id,
                device_type,
                identifier,
                specs,
                categories: partial.categories,
                description: partial.description,
                release_date: partial.release_date,
                sources: vec![source_info],
                created_at: now,
                last_verified: now,
            });
        }

        drop(db);
        self.save_database()
    }

    /// Normalize specification key for consistent matching.
    fn normalize_spec_key(key: &str) -> String {
        key.to_lowercase()
            .replace([' ', '-', '_'], "")
            .replace("clock", "clk")
    }

    /// Convert learned device to DeviceDeepInfo.
    fn convert_to_device_deep_info(&self, learned: &LearnedDevice) -> DeviceDeepInfo {
        // Convert specs HashMap to raw specs
        let raw_specs: HashMap<String, String> = learned
            .specs
            .iter()
            .map(|(k, v)| (k.clone(), v.value.clone()))
            .collect();

        // Use stored categories or generate from specs
        let categories = if learned.categories.is_empty() {
            self.generate_categories_from_specs(&learned.specs, &learned.device_type)
        } else {
            learned.categories.clone()
        };

        // Calculate average confidence
        let avg_confidence = if learned.specs.is_empty() {
            0.5
        } else {
            learned.specs.values().map(|s| s.confidence).sum::<f32>()
                / learned.specs.len() as f32
        };

        // Get best source URL
        let best_source = learned.sources.iter().max_by(|a, b| {
            a.confidence
                .partial_cmp(&b.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        DeviceDeepInfo {
            device_id: learned.device_id.clone(),
            device_type: learned.device_type.clone(),
            identifier: learned.identifier.clone(),
            specifications: Some(DeviceSpecifications {
                specs: raw_specs,
                categories,
                description: learned.description.clone(),
                release_date: learned.release_date.clone(),
                eol_date: None,
            }),
            drivers: None,
            documentation: None,
            images: None,
            metadata: DataMetadata {
                source: DataSource::ThirdPartyDatabase,
                last_updated: learned.last_verified,
                expires_at: learned.last_verified + Duration::days(30),
                source_url: best_source.and_then(|s| s.url.clone()),
                ai_confidence: Some(avg_confidence),
            },
        }
    }

    /// Generate spec categories from raw specs based on device type.
    fn generate_categories_from_specs(
        &self,
        specs: &HashMap<String, LearnedSpec>,
        device_type: &DeviceType,
    ) -> Vec<SpecCategory> {
        let mut categories = Vec::new();

        match device_type {
            DeviceType::Cpu => {
                let mut core_specs = Vec::new();
                let mut cache_specs = Vec::new();
                let mut power_specs = Vec::new();

                for (key, spec) in specs {
                    let item = SpecItem {
                        label: Self::format_spec_label(key),
                        value: spec.value.clone(),
                        unit: Self::extract_unit(&spec.value),
                    };

                    if key.contains("core")
                        || key.contains("thread")
                        || key.contains("clk")
                        || key.contains("frequency")
                    {
                        core_specs.push(item);
                    } else if key.contains("cache") || key.contains("l1") || key.contains("l2") || key.contains("l3") {
                        cache_specs.push(item);
                    } else if key.contains("tdp") || key.contains("power") || key.contains("watt") {
                        power_specs.push(item);
                    } else {
                        core_specs.push(item);
                    }
                }

                if !core_specs.is_empty() {
                    categories.push(SpecCategory {
                        name: "Processor".to_string(),
                        specs: core_specs,
                    });
                }
                if !cache_specs.is_empty() {
                    categories.push(SpecCategory {
                        name: "Cache".to_string(),
                        specs: cache_specs,
                    });
                }
                if !power_specs.is_empty() {
                    categories.push(SpecCategory {
                        name: "Power".to_string(),
                        specs: power_specs,
                    });
                }
            }
            DeviceType::Gpu => {
                let mut engine_specs = Vec::new();
                let mut memory_specs = Vec::new();
                let mut power_specs = Vec::new();

                for (key, spec) in specs {
                    let item = SpecItem {
                        label: Self::format_spec_label(key),
                        value: spec.value.clone(),
                        unit: Self::extract_unit(&spec.value),
                    };

                    if key.contains("memory")
                        || key.contains("vram")
                        || key.contains("bandwidth")
                        || key.contains("bus")
                    {
                        memory_specs.push(item);
                    } else if key.contains("tdp") || key.contains("power") || key.contains("watt") {
                        power_specs.push(item);
                    } else {
                        engine_specs.push(item);
                    }
                }

                if !engine_specs.is_empty() {
                    categories.push(SpecCategory {
                        name: "GPU Engine".to_string(),
                        specs: engine_specs,
                    });
                }
                if !memory_specs.is_empty() {
                    categories.push(SpecCategory {
                        name: "Memory".to_string(),
                        specs: memory_specs,
                    });
                }
                if !power_specs.is_empty() {
                    categories.push(SpecCategory {
                        name: "Power".to_string(),
                        specs: power_specs,
                    });
                }
            }
            _ => {
                // Generic categorization
                let items: Vec<SpecItem> = specs
                    .iter()
                    .map(|(key, spec)| SpecItem {
                        label: Self::format_spec_label(key),
                        value: spec.value.clone(),
                        unit: Self::extract_unit(&spec.value),
                    })
                    .collect();

                if !items.is_empty() {
                    categories.push(SpecCategory {
                        name: "Specifications".to_string(),
                        specs: items,
                    });
                }
            }
        }

        categories
    }

    /// Format a normalized spec key back to readable label.
    fn format_spec_label(key: &str) -> String {
        let formatted = key
            .replace("clk", " Clock")
            .replace("mem", "Memory ")
            .replace("tdp", "TDP");

        // Capitalize first letter of each word
        formatted
            .split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                    None => String::new(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Extract unit from a value string.
    fn extract_unit(value: &str) -> Option<String> {
        let units = ["MHz", "GHz", "GB", "MB", "W", "nm", "bit", "GB/s"];
        for unit in units {
            if value.contains(unit) {
                return Some(unit.to_string());
            }
        }
        None
    }

    /// Get all learned devices.
    pub fn get_all(&self) -> Vec<DeviceDeepInfo> {
        let db = match self.database.read() {
            Ok(d) => d,
            Err(_) => return Vec::new(),
        };

        db.devices
            .iter()
            .map(|d| self.convert_to_device_deep_info(d))
            .collect()
    }

    /// Get statistics about the knowledge store.
    pub fn stats(&self) -> KnowledgeStats {
        let db = match self.database.read() {
            Ok(d) => d,
            Err(_) => return KnowledgeStats::default(),
        };

        let total = db.devices.len();
        let mut by_type: HashMap<String, usize> = HashMap::new();
        let mut total_sources = 0;

        for device in &db.devices {
            *by_type.entry(device.device_type.to_string()).or_insert(0) += 1;
            total_sources += device.sources.len();
        }

        KnowledgeStats {
            total_devices: total,
            devices_by_type: by_type,
            total_sources,
            avg_sources_per_device: if total > 0 {
                total_sources as f32 / total as f32
            } else {
                0.0
            },
        }
    }
}

/// Statistics about the knowledge store.
#[derive(Debug, Default)]
pub struct KnowledgeStats {
    pub total_devices: usize,
    pub devices_by_type: HashMap<String, usize>,
    pub total_sources: usize,
    pub avg_sources_per_device: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_spec_key() {
        assert_eq!(KnowledgeStore::normalize_spec_key("Base Clock"), "baseclk");
        assert_eq!(KnowledgeStore::normalize_spec_key("CUDA Cores"), "cudacores");
        assert_eq!(
            KnowledgeStore::normalize_spec_key("Memory Bandwidth"),
            "memorybandwidth"
        );
    }

    #[test]
    fn test_format_spec_label() {
        assert_eq!(KnowledgeStore::format_spec_label("baseclk"), "Base Clock");
        assert_eq!(KnowledgeStore::format_spec_label("tdp"), "TDP");
    }

    #[test]
    fn test_extract_unit() {
        assert_eq!(
            KnowledgeStore::extract_unit("2100 MHz"),
            Some("MHz".to_string())
        );
        assert_eq!(
            KnowledgeStore::extract_unit("12 GB"),
            Some("GB".to_string())
        );
        assert_eq!(KnowledgeStore::extract_unit("8 cores"), None);
    }
}
