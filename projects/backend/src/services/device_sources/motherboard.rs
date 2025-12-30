//! Motherboard device source for device information.
//!
//! Fetches motherboard specifications from manufacturer websites
//! and aggregator databases.

use crate::models::{DeviceIdentifier, DeviceType, SpecCategory, SpecItem};
use crate::services::device_sources::DeviceSource;
use crate::services::PartialDeviceInfo;
use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use std::collections::HashMap;

/// Motherboard device source.
pub struct MotherboardSource {
    #[allow(dead_code)] // Reserved for future web scraping
    client: Client,
}

impl MotherboardSource {
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

        if combined.contains("asus") || combined.contains("rog") || combined.contains("tuf") || combined.contains("prime") {
            Some(Manufacturer::Asus)
        } else if combined.contains("gigabyte") || combined.contains("aorus") {
            Some(Manufacturer::Gigabyte)
        } else if combined.contains("msi") || combined.contains("meg") || combined.contains("mpg") || combined.contains("mag") {
            Some(Manufacturer::Msi)
        } else if combined.contains("asrock") {
            Some(Manufacturer::AsRock)
        } else if combined.contains("evga") {
            Some(Manufacturer::Evga)
        } else if combined.contains("biostar") {
            Some(Manufacturer::Biostar)
        } else {
            None
        }
    }

    /// Normalize model name for search.
    fn normalize_model(model: &str) -> String {
        model
            .to_lowercase()
            .replace("motherboard", "")
            .replace("mainboard", "")
            .trim()
            .to_string()
    }

    /// Try to fetch from ASUS.
    async fn fetch_asus(&self, model: &str) -> Result<Option<MotherboardData>> {
        let normalized = Self::normalize_model(model);
        // ASUS product search
        let search_url = format!(
            "https://www.asus.com/search/?SearchKey={}",
            urlencoding::encode(&normalized)
        );

        log::debug!("ASUS search URL: {}", search_url);

        let response = self.client.get(&search_url).send().await;
        if response.is_err() {
            return Ok(None);
        }

        // ASUS uses JavaScript rendering, so we'll extract what we can
        // For now, return structured data if we find a match
        Ok(None)
    }

    /// Try to fetch from Gigabyte.
    async fn fetch_gigabyte(&self, model: &str) -> Result<Option<MotherboardData>> {
        let normalized = Self::normalize_model(model);
        let search_url = format!(
            "https://www.gigabyte.com/Search?kw={}",
            urlencoding::encode(&normalized)
        );

        log::debug!("Gigabyte search URL: {}", search_url);

        let response = self.client.get(&search_url).send().await;
        if response.is_err() {
            return Ok(None);
        }

        Ok(None)
    }

    /// Try to fetch from MSI.
    async fn fetch_msi(&self, model: &str) -> Result<Option<MotherboardData>> {
        let normalized = Self::normalize_model(model);
        let search_url = format!(
            "https://www.msi.com/search/{}",
            urlencoding::encode(&normalized)
        );

        log::debug!("MSI search URL: {}", search_url);

        let response = self.client.get(&search_url).send().await;
        if response.is_err() {
            return Ok(None);
        }

        Ok(None)
    }

    /// Try to fetch from ASRock.
    async fn fetch_asrock(&self, model: &str) -> Result<Option<MotherboardData>> {
        let normalized = Self::normalize_model(model);
        let search_url = format!(
            "https://www.asrock.com/mb/search.asp?sModel={}",
            urlencoding::encode(&normalized)
        );

        log::debug!("ASRock search URL: {}", search_url);

        let response = self.client.get(&search_url).send().await;
        if response.is_err() {
            return Ok(None);
        }

        Ok(None)
    }

    /// Parse chipset from model name.
    fn extract_chipset(model: &str) -> Option<String> {
        let model_upper = model.to_uppercase();

        // Intel chipsets
        let intel_chipsets = [
            "Z890", "B860", "H810",
            "Z790", "B760", "H770", "H710",
            "Z690", "B660", "H670", "H610",
            "Z590", "B560", "H570", "H510",
            "Z490", "B460", "H470", "H410",
            "Z390", "B365", "B360", "H370", "H310",
        ];

        // AMD chipsets
        let amd_chipsets = [
            "X870E", "X870", "B850", "B840",
            "X670E", "X670", "B650E", "B650", "A620",
            "X570", "B550", "A520",
            "X470", "B450", "A320",
            "TRX50", "TRX40", "WRX80",
        ];

        for chipset in intel_chipsets.iter().chain(amd_chipsets.iter()) {
            if model_upper.contains(chipset) {
                return Some(chipset.to_string());
            }
        }

        None
    }

    /// Parse form factor from model name.
    fn extract_form_factor(model: &str) -> Option<String> {
        let model_lower = model.to_lowercase();

        if model_lower.contains("e-atx") || model_lower.contains("eatx") || model_lower.contains("extended") {
            Some("E-ATX".to_string())
        } else if model_lower.contains("micro") || model_lower.contains("m-atx") || model_lower.contains("matx") {
            Some("Micro-ATX".to_string())
        } else if model_lower.contains("mini") || model_lower.contains("m-itx") || model_lower.contains("itx") {
            Some("Mini-ITX".to_string())
        } else if model_lower.contains("atx") {
            Some("ATX".to_string())
        } else {
            None
        }
    }

    /// Build basic specs from model name parsing.
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

        // Extract chipset
        if let Some(chipset) = Self::extract_chipset(model) {
            let platform = if chipset.starts_with('X') || chipset.starts_with('B') && chipset.chars().nth(1).is_some_and(|c| c.is_ascii_digit() && c >= '4') || chipset.starts_with('A') {
                if chipset.contains("TRX") || chipset.contains("WRX") {
                    "AMD HEDT"
                } else {
                    "AMD AM5/AM4"
                }
            } else {
                "Intel LGA1700/1200"
            };

            specs.insert("chipset".to_string(), chipset.clone());
            specs.insert("platform".to_string(), platform.to_string());

            general_specs.push(SpecItem {
                label: "Chipset".to_string(),
                value: chipset,
                unit: None,
            });
            general_specs.push(SpecItem {
                label: "Platform".to_string(),
                value: platform.to_string(),
                unit: None,
            });
        }

        // Extract form factor
        if let Some(form_factor) = Self::extract_form_factor(model) {
            specs.insert("form_factor".to_string(), form_factor.clone());
            general_specs.push(SpecItem {
                label: "Form Factor".to_string(),
                value: form_factor,
                unit: None,
            });
        }

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
enum Manufacturer {
    Asus,
    Gigabyte,
    Msi,
    AsRock,
    Evga,
    Biostar,
}

/// Data extracted from motherboard product pages.
#[derive(Debug, Default)]
#[allow(dead_code)] // Reserved for future web scraping
struct MotherboardData {
    name: Option<String>,
    image_url: Option<String>,
    page_url: Option<String>,
    specs: HashMap<String, String>,
    chipset: Option<String>,
    socket: Option<String>,
    form_factor: Option<String>,
}

#[async_trait]
impl DeviceSource for MotherboardSource {
    fn name(&self) -> &str {
        "Motherboard Source"
    }

    fn priority(&self) -> u8 {
        20 // Medium priority
    }

    fn supports(&self, device_type: &DeviceType, _identifier: &DeviceIdentifier) -> bool {
        matches!(device_type, DeviceType::Motherboard)
    }

    async fn fetch(
        &self,
        _device_type: &DeviceType,
        identifier: &DeviceIdentifier,
    ) -> Result<PartialDeviceInfo> {
        let manufacturer = &identifier.manufacturer;
        let model = &identifier.model;

        log::info!("Fetching motherboard info for {} {}", manufacturer, model);

        // Try manufacturer-specific fetch
        let mfr = Self::detect_manufacturer(manufacturer, model);
        let mut data: Option<MotherboardData> = None;

        if let Some(mfr) = mfr {
            data = match mfr {
                Manufacturer::Asus => self.fetch_asus(model).await.ok().flatten(),
                Manufacturer::Gigabyte => self.fetch_gigabyte(model).await.ok().flatten(),
                Manufacturer::Msi => self.fetch_msi(model).await.ok().flatten(),
                Manufacturer::AsRock => self.fetch_asrock(model).await.ok().flatten(),
                _ => None,
            };
        }

        // Build basic specs from model name parsing
        let (mut specs, categories) = Self::build_basic_specs(manufacturer, model);

        // Merge with fetched data if available
        if let Some(fetched) = data {
            for (key, value) in fetched.specs {
                specs.insert(key, value);
            }
        }

        // Build product page URL based on manufacturer
        let product_page = match mfr {
            Some(Manufacturer::Asus) => Some(format!(
                "https://www.asus.com/search/?SearchKey={}",
                urlencoding::encode(&Self::normalize_model(model))
            )),
            Some(Manufacturer::Gigabyte) => Some(format!(
                "https://www.gigabyte.com/Search?kw={}",
                urlencoding::encode(&Self::normalize_model(model))
            )),
            Some(Manufacturer::Msi) => Some(format!(
                "https://www.msi.com/search/{}",
                urlencoding::encode(&Self::normalize_model(model))
            )),
            Some(Manufacturer::AsRock) => Some(format!(
                "https://www.asrock.com/mb/search.asp?sModel={}",
                urlencoding::encode(&Self::normalize_model(model))
            )),
            _ => None,
        };

        // Build support page URL
        let support_page = match mfr {
            Some(Manufacturer::Asus) => Some("https://www.asus.com/support/".to_string()),
            Some(Manufacturer::Gigabyte) => Some("https://www.gigabyte.com/Support".to_string()),
            Some(Manufacturer::Msi) => Some("https://www.msi.com/support".to_string()),
            Some(Manufacturer::AsRock) => Some("https://www.asrock.com/support/".to_string()),
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
            source_name: "Motherboard Source".to_string(),
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
            MotherboardSource::detect_manufacturer("ASUS", "ROG STRIX Z790-E"),
            Some(Manufacturer::Asus)
        ));
        assert!(matches!(
            MotherboardSource::detect_manufacturer("Gigabyte", "B650 AORUS ELITE"),
            Some(Manufacturer::Gigabyte)
        ));
        assert!(matches!(
            MotherboardSource::detect_manufacturer("MSI", "MEG X670E ACE"),
            Some(Manufacturer::Msi)
        ));
    }

    #[test]
    fn test_extract_chipset() {
        assert_eq!(MotherboardSource::extract_chipset("ROG STRIX Z790-E"), Some("Z790".to_string()));
        assert_eq!(MotherboardSource::extract_chipset("B650 AORUS ELITE"), Some("B650".to_string()));
        assert_eq!(MotherboardSource::extract_chipset("MEG X670E ACE"), Some("X670E".to_string()));
    }

    #[test]
    fn test_extract_form_factor() {
        assert_eq!(MotherboardSource::extract_form_factor("B650M AORUS ELITE"), Some("Micro-ATX".to_string()));
        assert_eq!(MotherboardSource::extract_form_factor("ROG STRIX B650-I"), Some("Mini-ITX".to_string()));
        assert_eq!(MotherboardSource::extract_form_factor("Z790 AORUS XTREME"), Some("ATX".to_string()));
    }
}
