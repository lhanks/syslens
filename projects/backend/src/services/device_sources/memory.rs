//! Memory device source for RAM specifications.
//!
//! Fetches memory specifications from manufacturer websites
//! and aggregator databases.

use crate::models::{DeviceIdentifier, DeviceType, SpecCategory, SpecItem};
use crate::services::device_sources::DeviceSource;
use crate::services::PartialDeviceInfo;
use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use std::collections::HashMap;

/// Memory device source.
pub struct MemorySource {
    #[allow(dead_code)] // Reserved for future web scraping
    client: Client,
}

impl MemorySource {
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self { client })
    }

    /// Detect memory generation from model name.
    fn detect_memory_type(model: &str) -> MemoryType {
        let model_upper = model.to_uppercase();

        if model_upper.contains("DDR5") {
            MemoryType::Ddr5
        } else if model_upper.contains("DDR4") {
            MemoryType::Ddr4
        } else if model_upper.contains("DDR3") {
            MemoryType::Ddr3
        } else if model_upper.contains("DDR2") {
            MemoryType::Ddr2
        } else {
            // Default to DDR4 for modern systems
            MemoryType::Ddr4
        }
    }

    /// Detect manufacturer from model name.
    fn detect_manufacturer(manufacturer: &str, model: &str) -> Option<Manufacturer> {
        let combined = format!("{} {}", manufacturer, model).to_lowercase();

        if combined.contains("corsair") || combined.contains("vengeance") || combined.contains("dominator") {
            Some(Manufacturer::Corsair)
        } else if combined.contains("g.skill") || combined.contains("gskill") || combined.contains("trident") || combined.contains("ripjaws") {
            Some(Manufacturer::GSkill)
        } else if combined.contains("kingston") || combined.contains("fury") || combined.contains("hyperx") {
            Some(Manufacturer::Kingston)
        } else if combined.contains("crucial") || combined.contains("ballistix") {
            Some(Manufacturer::Crucial)
        } else if combined.contains("teamgroup") || combined.contains("team") || combined.contains("t-force") {
            Some(Manufacturer::TeamGroup)
        } else if combined.contains("samsung") {
            Some(Manufacturer::Samsung)
        } else if combined.contains("sk hynix") || combined.contains("hynix") {
            Some(Manufacturer::SKHynix)
        } else if combined.contains("micron") {
            Some(Manufacturer::Micron)
        } else if combined.contains("adata") || combined.contains("xpg") {
            Some(Manufacturer::Adata)
        } else if combined.contains("patriot") || combined.contains("viper") {
            Some(Manufacturer::Patriot)
        } else {
            None
        }
    }

    /// Extract capacity from model name.
    fn extract_capacity(model: &str) -> Option<String> {
        let model_upper = model.to_uppercase();

        // Common capacity patterns
        let patterns = [
            ("128GB", "128 GB"),
            ("64GB", "64 GB"),
            ("48GB", "48 GB"),
            ("32GB", "32 GB"),
            ("16GB", "16 GB"),
            ("8GB", "8 GB"),
            ("4GB", "4 GB"),
            ("2GB", "2 GB"),
        ];

        for (pattern, formatted) in patterns {
            if model_upper.contains(pattern) || model_upper.contains(&pattern.replace("GB", " GB")) {
                return Some(formatted.to_string());
            }
        }

        None
    }

    /// Extract speed from model name.
    fn extract_speed(model: &str) -> Option<String> {
        let model_upper = model.to_uppercase();

        // DDR5 speeds
        let ddr5_speeds = [
            ("8400", "DDR5-8400"),
            ("8000", "DDR5-8000"),
            ("7600", "DDR5-7600"),
            ("7200", "DDR5-7200"),
            ("6800", "DDR5-6800"),
            ("6400", "DDR5-6400"),
            ("6000", "DDR5-6000"),
            ("5600", "DDR5-5600"),
            ("5200", "DDR5-5200"),
            ("4800", "DDR5-4800"),
        ];

        // DDR4 speeds
        let ddr4_speeds = [
            ("4800", "DDR4-4800"),
            ("4600", "DDR4-4600"),
            ("4400", "DDR4-4400"),
            ("4133", "DDR4-4133"),
            ("4000", "DDR4-4000"),
            ("3600", "DDR4-3600"),
            ("3200", "DDR4-3200"),
            ("3000", "DDR4-3000"),
            ("2666", "DDR4-2666"),
            ("2400", "DDR4-2400"),
            ("2133", "DDR4-2133"),
        ];

        // Check DDR5 speeds first
        if model_upper.contains("DDR5") {
            for (pattern, formatted) in ddr5_speeds {
                if model_upper.contains(pattern) {
                    return Some(formatted.to_string());
                }
            }
        }

        // Check DDR4 speeds
        for (pattern, formatted) in ddr4_speeds {
            if model_upper.contains(pattern) {
                return Some(formatted.to_string());
            }
        }

        // Also check DDR5 speeds without explicit DDR5 marker
        for (pattern, formatted) in ddr5_speeds {
            if model_upper.contains(pattern) {
                return Some(formatted.to_string());
            }
        }

        None
    }

    /// Extract kit configuration (e.g., 2x16GB, 4x8GB).
    fn extract_kit_config(model: &str) -> Option<String> {
        let model_lower = model.to_lowercase();

        // Common kit patterns
        let patterns = [
            ("4x32", "4x32GB"),
            ("2x64", "2x64GB"),
            ("4x16", "4x16GB"),
            ("2x32", "2x32GB"),
            ("4x8", "4x8GB"),
            ("2x16", "2x16GB"),
            ("2x8", "2x8GB"),
            ("2x4", "2x4GB"),
            ("1x32", "1x32GB"),
            ("1x16", "1x16GB"),
            ("1x8", "1x8GB"),
        ];

        for (pattern, formatted) in patterns {
            if model_lower.contains(pattern) {
                return Some(formatted.to_string());
            }
        }

        None
    }

    /// Build specs from model name parsing.
    fn build_basic_specs(manufacturer: &str, model: &str) -> (HashMap<String, String>, Vec<SpecCategory>) {
        let mut specs = HashMap::new();
        let mut general_specs = Vec::new();

        // Add manufacturer
        specs.insert("manufacturer".to_string(), manufacturer.to_string());
        general_specs.push(SpecItem {
            label: "Manufacturer".to_string(),
            value: manufacturer.to_string(),
            unit: None,
        });

        // Detect memory type
        let mem_type = Self::detect_memory_type(model);
        let type_str = match mem_type {
            MemoryType::Ddr5 => "DDR5",
            MemoryType::Ddr4 => "DDR4",
            MemoryType::Ddr3 => "DDR3",
            MemoryType::Ddr2 => "DDR2",
        };
        specs.insert("memory_type".to_string(), type_str.to_string());
        general_specs.push(SpecItem {
            label: "Memory Type".to_string(),
            value: type_str.to_string(),
            unit: None,
        });

        // Extract capacity
        if let Some(capacity) = Self::extract_capacity(model) {
            specs.insert("capacity".to_string(), capacity.clone());
            general_specs.push(SpecItem {
                label: "Capacity".to_string(),
                value: capacity,
                unit: None,
            });
        }

        // Extract speed
        if let Some(speed) = Self::extract_speed(model) {
            specs.insert("speed".to_string(), speed.clone());
            general_specs.push(SpecItem {
                label: "Speed".to_string(),
                value: speed,
                unit: None,
            });
        }

        // Extract kit configuration
        if let Some(kit) = Self::extract_kit_config(model) {
            specs.insert("kit_config".to_string(), kit.clone());
            general_specs.push(SpecItem {
                label: "Kit Configuration".to_string(),
                value: kit,
                unit: None,
            });
        }

        // Form factor based on memory type
        let form_factor = match mem_type {
            MemoryType::Ddr5 | MemoryType::Ddr4 => "DIMM",
            MemoryType::Ddr3 | MemoryType::Ddr2 => "DIMM",
        };
        specs.insert("form_factor".to_string(), form_factor.to_string());
        general_specs.push(SpecItem {
            label: "Form Factor".to_string(),
            value: form_factor.to_string(),
            unit: None,
        });

        let categories = if general_specs.is_empty() {
            vec![]
        } else {
            vec![SpecCategory {
                name: "General".to_string(),
                specs: general_specs,
            }]
        };

        (specs, categories)
    }
}

#[derive(Debug, Clone, Copy)]
enum MemoryType {
    Ddr5,
    Ddr4,
    Ddr3,
    Ddr2,
}

#[derive(Debug, Clone, Copy)]
enum Manufacturer {
    Corsair,
    GSkill,
    Kingston,
    Crucial,
    TeamGroup,
    Samsung,
    SKHynix,
    Micron,
    Adata,
    Patriot,
}

#[async_trait]
impl DeviceSource for MemorySource {
    fn name(&self) -> &str {
        "Memory Source"
    }

    fn priority(&self) -> u8 {
        20 // Medium priority
    }

    fn supports(&self, device_type: &DeviceType, _identifier: &DeviceIdentifier) -> bool {
        matches!(device_type, DeviceType::Memory)
    }

    async fn fetch(
        &self,
        _device_type: &DeviceType,
        identifier: &DeviceIdentifier,
    ) -> Result<PartialDeviceInfo> {
        let manufacturer = &identifier.manufacturer;
        let model = &identifier.model;

        log::info!("Fetching memory info for {} {}", manufacturer, model);

        // Build basic specs from model name parsing
        let (specs, categories) = Self::build_basic_specs(manufacturer, model);

        // Determine manufacturer-specific URLs
        let mfr = Self::detect_manufacturer(manufacturer, model);

        let product_page = match mfr {
            Some(Manufacturer::Corsair) => Some(format!(
                "https://www.corsair.com/search/?text={}",
                urlencoding::encode(model)
            )),
            Some(Manufacturer::GSkill) => Some(format!(
                "https://www.gskill.com/search?q={}",
                urlencoding::encode(model)
            )),
            Some(Manufacturer::Kingston) => Some(format!(
                "https://www.kingston.com/search?q={}",
                urlencoding::encode(model)
            )),
            Some(Manufacturer::Crucial) => Some(format!(
                "https://www.crucial.com/search?q={}",
                urlencoding::encode(model)
            )),
            _ => None,
        };

        let support_page = match mfr {
            Some(Manufacturer::Corsair) => Some("https://www.corsair.com/support".to_string()),
            Some(Manufacturer::GSkill) => Some("https://www.gskill.com/support".to_string()),
            Some(Manufacturer::Kingston) => Some("https://www.kingston.com/support".to_string()),
            Some(Manufacturer::Crucial) => Some("https://www.crucial.com/support".to_string()),
            _ => None,
        };

        Ok(PartialDeviceInfo {
            specs,
            categories,
            description: Some(format!("{} {}", manufacturer, model)),
            release_date: None,
            product_page,
            support_page,
            image_url: None,
            source_name: "Memory Source".to_string(),
            source_url: None,
            confidence: 0.6, // Medium confidence - parsed from model name
            image_cached_path: None,
            thumbnail_url: None,
            thumbnail_cached_path: None,
            image_gallery: vec![],
            documentation: None,
            driver_info: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_memory_type() {
        assert!(matches!(
            MemorySource::detect_memory_type("Vengeance DDR5-6000"),
            MemoryType::Ddr5
        ));
        assert!(matches!(
            MemorySource::detect_memory_type("Trident Z DDR4-3600"),
            MemoryType::Ddr4
        ));
    }

    #[test]
    fn test_detect_manufacturer() {
        assert!(matches!(
            MemorySource::detect_manufacturer("Corsair", "Vengeance RGB"),
            Some(Manufacturer::Corsair)
        ));
        assert!(matches!(
            MemorySource::detect_manufacturer("G.Skill", "Trident Z5"),
            Some(Manufacturer::GSkill)
        ));
    }

    #[test]
    fn test_extract_capacity() {
        assert_eq!(MemorySource::extract_capacity("Vengeance 32GB DDR5"), Some("32 GB".to_string()));
        assert_eq!(MemorySource::extract_capacity("16GB DDR4-3600"), Some("16 GB".to_string()));
    }

    #[test]
    fn test_extract_speed() {
        assert_eq!(MemorySource::extract_speed("DDR5-6000 RGB"), Some("DDR5-6000".to_string()));
        assert_eq!(MemorySource::extract_speed("DDR4-3600 CL16"), Some("DDR4-3600".to_string()));
    }

    #[test]
    fn test_extract_kit_config() {
        assert_eq!(MemorySource::extract_kit_config("2x16GB DDR5"), Some("2x16GB".to_string()));
        assert_eq!(MemorySource::extract_kit_config("4x8GB Kit"), Some("4x8GB".to_string()));
    }
}
