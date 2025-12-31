//! Monitor device source for display specifications.
//!
//! Fetches monitor specifications from manufacturer websites
//! and aggregator databases.

use crate::models::{DeviceIdentifier, DeviceType, SpecCategory, SpecItem};
use crate::services::device_sources::DeviceSource;
use crate::services::PartialDeviceInfo;
use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use std::collections::HashMap;

/// Monitor device source.
pub struct MonitorSource {
    #[allow(dead_code)] // Reserved for future web scraping
    client: Client,
}

impl MonitorSource {
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self { client })
    }

    /// Detect manufacturer from model name.
    fn detect_manufacturer(manufacturer: &str, model: &str) -> Option<Manufacturer> {
        let combined = format!("{} {}", manufacturer, model).to_lowercase();

        if combined.contains("dell") || combined.contains("alienware") {
            Some(Manufacturer::Dell)
        } else if combined.contains("samsung") || combined.contains("odyssey") {
            Some(Manufacturer::Samsung)
        } else if combined.contains("lg")
            || combined.contains("ultragear")
            || combined.contains("ultrawide")
        {
            Some(Manufacturer::Lg)
        } else if combined.contains("asus")
            || combined.contains("rog")
            || combined.contains("proart")
        {
            Some(Manufacturer::Asus)
        } else if combined.contains("acer")
            || combined.contains("predator")
            || combined.contains("nitro")
        {
            Some(Manufacturer::Acer)
        } else if combined.contains("benq") || combined.contains("zowie") {
            Some(Manufacturer::BenQ)
        } else if combined.contains("msi") || combined.contains("optix") {
            Some(Manufacturer::Msi)
        } else if combined.contains("viewsonic") {
            Some(Manufacturer::ViewSonic)
        } else if combined.contains("aoc") || combined.contains("agon") {
            Some(Manufacturer::Aoc)
        } else if combined.contains("hp") || combined.contains("omen") {
            Some(Manufacturer::Hp)
        } else if combined.contains("lenovo") || combined.contains("legion") {
            Some(Manufacturer::Lenovo)
        } else if combined.contains("gigabyte") || combined.contains("aorus") {
            Some(Manufacturer::Gigabyte)
        } else {
            None
        }
    }

    /// Extract resolution from model name.
    fn extract_resolution(model: &str) -> Option<(String, String)> {
        let model_upper = model.to_uppercase();

        // Common resolution patterns
        let resolutions = [
            ("8K", "7680x4320", "8K UHD"),
            ("5K", "5120x2880", "5K"),
            ("4K", "3840x2160", "4K UHD"),
            ("UHD", "3840x2160", "4K UHD"),
            ("2160P", "3840x2160", "4K UHD"),
            ("WQHD", "3440x1440", "WQHD Ultrawide"),
            ("1440P", "2560x1440", "QHD"),
            ("QHD", "2560x1440", "QHD"),
            ("2K", "2560x1440", "QHD"),
            ("1080P", "1920x1080", "FHD"),
            ("FHD", "1920x1080", "FHD"),
            ("FULLHD", "1920x1080", "FHD"),
        ];

        for (pattern, resolution, name) in resolutions {
            if model_upper.contains(pattern) {
                return Some((resolution.to_string(), name.to_string()));
            }
        }

        None
    }

    /// Extract refresh rate from model name.
    fn extract_refresh_rate(model: &str) -> Option<String> {
        let model_lower = model.to_lowercase();

        // Common refresh rates
        let rates = [
            ("500hz", "500 Hz"),
            ("480hz", "480 Hz"),
            ("390hz", "390 Hz"),
            ("360hz", "360 Hz"),
            ("280hz", "280 Hz"),
            ("270hz", "270 Hz"),
            ("240hz", "240 Hz"),
            ("200hz", "200 Hz"),
            ("180hz", "180 Hz"),
            ("175hz", "175 Hz"),
            ("170hz", "170 Hz"),
            ("165hz", "165 Hz"),
            ("160hz", "160 Hz"),
            ("144hz", "144 Hz"),
            ("120hz", "120 Hz"),
            ("100hz", "100 Hz"),
            ("75hz", "75 Hz"),
            ("60hz", "60 Hz"),
        ];

        for (pattern, formatted) in rates {
            if model_lower.contains(pattern) {
                return Some(formatted.to_string());
            }
        }

        None
    }

    /// Extract screen size from model name.
    fn extract_screen_size(model: &str) -> Option<String> {
        let model_lower = model.to_lowercase();

        // Common screen sizes (in inches)
        let sizes = [
            ("49\"", "49\""),
            ("49in", "49\""),
            ("45\"", "45\""),
            ("45in", "45\""),
            ("43\"", "43\""),
            ("43in", "43\""),
            ("42\"", "42\""),
            ("42in", "42\""),
            ("38\"", "38\""),
            ("38in", "38\""),
            ("34\"", "34\""),
            ("34in", "34\""),
            ("32\"", "32\""),
            ("32in", "32\""),
            ("31.5", "32\""),
            ("27\"", "27\""),
            ("27in", "27\""),
            ("27-", "27\""),
            ("26.5", "27\""),
            ("25\"", "25\""),
            ("25in", "25\""),
            ("24.5", "25\""),
            ("24\"", "24\""),
            ("24in", "24\""),
            ("23.8", "24\""),
            ("22\"", "22\""),
            ("22in", "22\""),
            ("21.5", "22\""),
        ];

        for (pattern, formatted) in sizes {
            if model_lower.contains(pattern) {
                return Some(formatted.to_string());
            }
        }

        // Try to extract number followed by inch indicators
        let re_patterns = ["27", "32", "24", "34", "38", "43", "49"];
        for size in re_patterns {
            if model.contains(size) {
                // Check if it's likely a screen size (not a model number suffix)
                let idx = model.find(size);
                if let Some(idx) = idx {
                    // Check if followed by inch indicator or end of segment
                    let after = &model[idx + size.len()..];
                    if after.starts_with('"')
                        || after.starts_with("in")
                        || after.starts_with('-')
                        || after.starts_with(' ')
                    {
                        return Some(format!("{}\"", size));
                    }
                }
            }
        }

        None
    }

    /// Detect panel type from model name.
    fn extract_panel_type(model: &str) -> Option<String> {
        let model_upper = model.to_uppercase();

        if model_upper.contains("OLED")
            || model_upper.contains("QD-OLED")
            || model_upper.contains("WOLED")
        {
            Some("OLED".to_string())
        } else if model_upper.contains("MINI-LED") || model_upper.contains("MINILED") {
            Some("Mini-LED".to_string())
        } else if model_upper.contains("NANO IPS") || model_upper.contains("NANO-IPS") {
            Some("Nano IPS".to_string())
        } else if model_upper.contains("IPS") {
            Some("IPS".to_string())
        } else if model_upper.contains("VA") {
            Some("VA".to_string())
        } else if model_upper.contains("TN") {
            Some("TN".to_string())
        } else {
            None
        }
    }

    /// Detect adaptive sync technology.
    fn extract_adaptive_sync(model: &str) -> Option<String> {
        let model_upper = model.to_uppercase();

        if model_upper.contains("G-SYNC ULTIMATE") {
            Some("G-Sync Ultimate".to_string())
        } else if model_upper.contains("G-SYNC") || model_upper.contains("GSYNC") {
            Some("G-Sync".to_string())
        } else if model_upper.contains("FREESYNC PREMIUM PRO") {
            Some("FreeSync Premium Pro".to_string())
        } else if model_upper.contains("FREESYNC PREMIUM") {
            Some("FreeSync Premium".to_string())
        } else if model_upper.contains("FREESYNC") {
            Some("FreeSync".to_string())
        } else {
            None
        }
    }

    /// Build specs from model name parsing.
    fn build_basic_specs(
        manufacturer: &str,
        model: &str,
    ) -> (HashMap<String, String>, Vec<SpecCategory>) {
        let mut specs = HashMap::new();
        let mut general_specs = Vec::new();

        // Add manufacturer
        specs.insert("manufacturer".to_string(), manufacturer.to_string());
        general_specs.push(SpecItem {
            label: "Manufacturer".to_string(),
            value: manufacturer.to_string(),
            unit: None,
        });

        // Extract screen size
        if let Some(size) = Self::extract_screen_size(model) {
            specs.insert("screen_size".to_string(), size.clone());
            general_specs.push(SpecItem {
                label: "Screen Size".to_string(),
                value: size,
                unit: None,
            });
        }

        // Extract resolution
        if let Some((resolution, name)) = Self::extract_resolution(model) {
            specs.insert("resolution".to_string(), resolution.clone());
            specs.insert("resolution_name".to_string(), name.clone());
            general_specs.push(SpecItem {
                label: "Resolution".to_string(),
                value: format!("{} ({})", name, resolution),
                unit: None,
            });
        }

        // Extract refresh rate
        if let Some(rate) = Self::extract_refresh_rate(model) {
            specs.insert("refresh_rate".to_string(), rate.clone());
            general_specs.push(SpecItem {
                label: "Refresh Rate".to_string(),
                value: rate,
                unit: None,
            });
        }

        // Extract panel type
        if let Some(panel) = Self::extract_panel_type(model) {
            specs.insert("panel_type".to_string(), panel.clone());
            general_specs.push(SpecItem {
                label: "Panel Type".to_string(),
                value: panel,
                unit: None,
            });
        }

        // Extract adaptive sync
        if let Some(sync) = Self::extract_adaptive_sync(model) {
            specs.insert("adaptive_sync".to_string(), sync.clone());
            general_specs.push(SpecItem {
                label: "Adaptive Sync".to_string(),
                value: sync,
                unit: None,
            });
        }

        let categories = if general_specs.is_empty() {
            vec![]
        } else {
            vec![SpecCategory {
                name: "Display".to_string(),
                specs: general_specs,
            }]
        };

        (specs, categories)
    }
}

#[derive(Debug, Clone, Copy)]
enum Manufacturer {
    Dell,
    Samsung,
    Lg,
    Asus,
    Acer,
    BenQ,
    Msi,
    ViewSonic,
    Aoc,
    Hp,
    Lenovo,
    Gigabyte,
}

#[async_trait]
impl DeviceSource for MonitorSource {
    fn name(&self) -> &str {
        "Monitor Source"
    }

    fn priority(&self) -> u8 {
        20 // Medium priority
    }

    fn supports(&self, device_type: &DeviceType, _identifier: &DeviceIdentifier) -> bool {
        matches!(device_type, DeviceType::Monitor)
    }

    async fn fetch(
        &self,
        _device_type: &DeviceType,
        identifier: &DeviceIdentifier,
    ) -> Result<PartialDeviceInfo> {
        let manufacturer = &identifier.manufacturer;
        let model = &identifier.model;

        log::info!("Fetching monitor info for {} {}", manufacturer, model);

        // Build basic specs from model name parsing
        let (specs, categories) = Self::build_basic_specs(manufacturer, model);

        // Determine manufacturer-specific URLs
        let mfr = Self::detect_manufacturer(manufacturer, model);

        let product_page = match mfr {
            Some(Manufacturer::Dell) => Some(format!(
                "https://www.dell.com/search?q={}",
                urlencoding::encode(model)
            )),
            Some(Manufacturer::Samsung) => Some(format!(
                "https://www.samsung.com/search/?searchvalue={}",
                urlencoding::encode(model)
            )),
            Some(Manufacturer::Lg) => Some(format!(
                "https://www.lg.com/search?q={}",
                urlencoding::encode(model)
            )),
            Some(Manufacturer::Asus) => Some(format!(
                "https://www.asus.com/search/?SearchKey={}",
                urlencoding::encode(model)
            )),
            Some(Manufacturer::Acer) => Some(format!(
                "https://www.acer.com/search?q={}",
                urlencoding::encode(model)
            )),
            Some(Manufacturer::BenQ) => Some(format!(
                "https://www.benq.com/search?q={}",
                urlencoding::encode(model)
            )),
            _ => None,
        };

        let support_page = match mfr {
            Some(Manufacturer::Dell) => Some("https://www.dell.com/support".to_string()),
            Some(Manufacturer::Samsung) => Some("https://www.samsung.com/support".to_string()),
            Some(Manufacturer::Lg) => Some("https://www.lg.com/support".to_string()),
            Some(Manufacturer::Asus) => Some("https://www.asus.com/support".to_string()),
            Some(Manufacturer::Acer) => Some("https://www.acer.com/support".to_string()),
            Some(Manufacturer::BenQ) => Some("https://www.benq.com/support".to_string()),
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
            source_name: "Monitor Source".to_string(),
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
    fn test_detect_manufacturer() {
        assert!(matches!(
            MonitorSource::detect_manufacturer("Dell", "S2722DGM"),
            Some(Manufacturer::Dell)
        ));
        assert!(matches!(
            MonitorSource::detect_manufacturer("LG", "27GP850-B"),
            Some(Manufacturer::Lg)
        ));
        assert!(matches!(
            MonitorSource::detect_manufacturer("ASUS", "ROG Swift PG27AQDM"),
            Some(Manufacturer::Asus)
        ));
    }

    #[test]
    fn test_extract_resolution() {
        assert_eq!(
            MonitorSource::extract_resolution("4K Gaming Monitor"),
            Some(("3840x2160".to_string(), "4K UHD".to_string()))
        );
        assert_eq!(
            MonitorSource::extract_resolution("QHD 165Hz"),
            Some(("2560x1440".to_string(), "QHD".to_string()))
        );
    }

    #[test]
    fn test_extract_refresh_rate() {
        assert_eq!(
            MonitorSource::extract_refresh_rate("240Hz Gaming"),
            Some("240 Hz".to_string())
        );
        assert_eq!(
            MonitorSource::extract_refresh_rate("165hz IPS"),
            Some("165 Hz".to_string())
        );
    }

    #[test]
    fn test_extract_panel_type() {
        assert_eq!(
            MonitorSource::extract_panel_type("OLED Gaming Monitor"),
            Some("OLED".to_string())
        );
        assert_eq!(
            MonitorSource::extract_panel_type("IPS Panel 144Hz"),
            Some("IPS".to_string())
        );
        assert_eq!(
            MonitorSource::extract_panel_type("Nano IPS Display"),
            Some("Nano IPS".to_string())
        );
    }

    #[test]
    fn test_extract_screen_size() {
        assert_eq!(
            MonitorSource::extract_screen_size("27\" Gaming Monitor"),
            Some("27\"".to_string())
        );
        assert_eq!(
            MonitorSource::extract_screen_size("32in 4K Display"),
            Some("32\"".to_string())
        );
    }
}
