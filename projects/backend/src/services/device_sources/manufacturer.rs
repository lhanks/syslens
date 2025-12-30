//! Manufacturer source for device information.
//!
//! Fetches device information directly from manufacturer websites
//! (NVIDIA, AMD, Intel, etc.) for official product pages and images.

use crate::models::{DeviceIdentifier, DeviceType, DocumentationLinks, DriverInfo};
use crate::services::device_sources::DeviceSource;
use crate::services::PartialDeviceInfo;
use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use scraper::{Html, Selector};
use std::collections::HashMap;

/// Manufacturer-based device information source.
/// Fetches from official manufacturer websites.
pub struct ManufacturerSource {
    client: Client,
}

impl ManufacturerSource {
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self { client })
    }

    /// Determine the manufacturer from identifier.
    fn detect_manufacturer(identifier: &DeviceIdentifier) -> Option<Manufacturer> {
        let mfr_lower = identifier.manufacturer.to_lowercase();
        let model_lower = identifier.model.to_lowercase();

        if mfr_lower.contains("nvidia") || model_lower.contains("geforce") || model_lower.contains("quadro") {
            Some(Manufacturer::Nvidia)
        } else if mfr_lower.contains("amd") || mfr_lower.contains("ati") || model_lower.contains("radeon") || model_lower.contains("ryzen") {
            Some(Manufacturer::Amd)
        } else if mfr_lower.contains("intel") || model_lower.contains("core i") || model_lower.contains("xeon") || model_lower.contains("arc") {
            Some(Manufacturer::Intel)
        } else {
            None
        }
    }

    /// Fetch NVIDIA GPU information.
    async fn fetch_nvidia(&self, identifier: &DeviceIdentifier) -> Result<PartialDeviceInfo> {
        // Construct search URL for NVIDIA
        let model = &identifier.model;
        let search_term = Self::extract_nvidia_model(model);

        log::debug!("Searching NVIDIA for: {}", search_term);

        // NVIDIA product pages follow patterns like:
        // https://www.nvidia.com/en-us/geforce/graphics-cards/40-series/rtx-4090/
        let product_url = Self::construct_nvidia_url(&search_term);

        // Try to fetch the product page
        let response = self
            .client
            .get(&product_url)
            .send()
            .await
            .context("Failed to fetch NVIDIA product page")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("NVIDIA product page not found"));
        }

        let html = response.text().await?;
        let document = Html::parse_document(&html);

        // Extract product image
        let image_url = self.extract_nvidia_image(&document);

        // Build documentation links
        let documentation = Some(DocumentationLinks {
            product_page: Some(product_url.clone()),
            support_page: Some("https://www.nvidia.com/en-us/geforce/drivers/".to_string()),
            manuals: vec![],
            datasheets: vec![],
            firmware_updates: vec![],
        });

        // Build driver info
        let driver_info = Some(DriverInfo {
            installed_version: None,
            latest_version: None,
            download_url: Some("https://www.nvidia.com/Download/index.aspx".to_string()),
            release_date: None,
            release_notes_url: None,
            driver_page_url: Some("https://www.nvidia.com/en-us/geforce/drivers/".to_string()),
            update_available: false,
        });

        Ok(PartialDeviceInfo {
            specs: HashMap::new(),
            categories: vec![],
            description: Some(format!("NVIDIA {}", model)),
            release_date: None,
            product_page: Some(product_url.clone()),
            support_page: Some("https://www.nvidia.com/en-us/geforce/drivers/".to_string()),
            image_url,
            source_name: "NVIDIA Official".to_string(),
            source_url: Some(product_url),
            confidence: 0.95, // Very high - official source
            image_cached_path: None,
            thumbnail_url: None,
            thumbnail_cached_path: None,
            image_gallery: vec![],
            documentation,
            driver_info,
        })
    }

    /// Extract NVIDIA model number from full model string.
    fn extract_nvidia_model(model: &str) -> String {
        let model_lower = model.to_lowercase();

        // Extract RTX/GTX model numbers
        if let Some(pos) = model_lower.find("rtx") {
            let after_rtx = &model[pos + 3..].trim_start();
            // Get model number (e.g., "4090", "5070 Ti")
            let parts: Vec<&str> = after_rtx.split_whitespace().take(2).collect();
            return format!("rtx-{}", parts.join("-").to_lowercase());
        }

        if let Some(pos) = model_lower.find("gtx") {
            let after_gtx = &model[pos + 3..].trim_start();
            let parts: Vec<&str> = after_gtx.split_whitespace().take(2).collect();
            return format!("gtx-{}", parts.join("-").to_lowercase());
        }

        model.to_lowercase().replace(' ', "-")
    }

    /// Construct NVIDIA product page URL.
    fn construct_nvidia_url(model: &str) -> String {
        // Determine series from model number
        let series = if model.contains("5090") || model.contains("5080") || model.contains("5070") {
            "50-series"
        } else if model.contains("4090") || model.contains("4080") || model.contains("4070") || model.contains("4060") {
            "40-series"
        } else if model.contains("3090") || model.contains("3080") || model.contains("3070") || model.contains("3060") {
            "30-series"
        } else {
            "40-series" // Default
        };

        format!(
            "https://www.nvidia.com/en-us/geforce/graphics-cards/{}/{}/",
            series, model
        )
    }

    /// Extract product image from NVIDIA page.
    fn extract_nvidia_image(&self, document: &Html) -> Option<String> {
        // Try multiple selectors for NVIDIA product images
        let selectors = [
            "picture.product-image img",
            ".hero-image img",
            ".product-hero img",
            "meta[property='og:image']",
        ];

        for selector_str in selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(element) = document.select(&selector).next() {
                    // Check for src, data-src, or content attribute
                    if let Some(src) = element.value().attr("src")
                        .or_else(|| element.value().attr("data-src"))
                        .or_else(|| element.value().attr("content"))
                    {
                        if src.contains("http") {
                            return Some(src.to_string());
                        }
                    }
                }
            }
        }

        None
    }

    /// Fetch AMD GPU/CPU information.
    async fn fetch_amd(&self, identifier: &DeviceIdentifier, device_type: &DeviceType) -> Result<PartialDeviceInfo> {
        let model = &identifier.model;
        log::debug!("Searching AMD for: {}", model);

        // Construct AMD product URL
        let product_url = match device_type {
            DeviceType::Gpu => Self::construct_amd_gpu_url(model),
            DeviceType::Cpu => Self::construct_amd_cpu_url(model),
            _ => return Err(anyhow::anyhow!("Unsupported device type for AMD")),
        };

        // Try to fetch the product page
        let response = self
            .client
            .get(&product_url)
            .send()
            .await
            .context("Failed to fetch AMD product page")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("AMD product page not found"));
        }

        let html = response.text().await?;
        let document = Html::parse_document(&html);

        // Extract product image
        let image_url = self.extract_amd_image(&document);

        // Build documentation links
        let documentation = Some(DocumentationLinks {
            product_page: Some(product_url.clone()),
            support_page: Some("https://www.amd.com/en/support".to_string()),
            manuals: vec![],
            datasheets: vec![],
            firmware_updates: vec![],
        });

        // Build driver info
        let driver_info = Some(DriverInfo {
            installed_version: None,
            latest_version: None,
            download_url: Some("https://www.amd.com/en/support".to_string()),
            release_date: None,
            release_notes_url: None,
            driver_page_url: Some("https://www.amd.com/en/support".to_string()),
            update_available: false,
        });

        Ok(PartialDeviceInfo {
            specs: HashMap::new(),
            categories: vec![],
            description: Some(format!("AMD {}", model)),
            release_date: None,
            product_page: Some(product_url.clone()),
            support_page: Some("https://www.amd.com/en/support".to_string()),
            image_url,
            source_name: "AMD Official".to_string(),
            source_url: Some(product_url),
            confidence: 0.95, // Very high - official source
            image_cached_path: None,
            thumbnail_url: None,
            thumbnail_cached_path: None,
            image_gallery: vec![],
            documentation,
            driver_info,
        })
    }

    /// Construct AMD GPU product page URL.
    fn construct_amd_gpu_url(model: &str) -> String {
        let model_slug = model
            .to_lowercase()
            .replace("radeon ", "")
            .replace("rx ", "rx-")
            .replace(' ', "-");

        format!(
            "https://www.amd.com/en/products/graphics/amd-radeon-{}",
            model_slug
        )
    }

    /// Construct AMD CPU product page URL.
    fn construct_amd_cpu_url(model: &str) -> String {
        let model_slug = model
            .to_lowercase()
            .replace("amd ", "")
            .replace("ryzen ", "ryzen-")
            .replace(' ', "-");

        format!(
            "https://www.amd.com/en/products/processors/desktops/{}",
            model_slug
        )
    }

    /// Extract product image from AMD page.
    fn extract_amd_image(&self, document: &Html) -> Option<String> {
        let selectors = [
            ".product-image img",
            ".hero-image img",
            "meta[property='og:image']",
            ".product-hero-image img",
        ];

        for selector_str in selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(element) = document.select(&selector).next() {
                    if let Some(src) = element.value().attr("src")
                        .or_else(|| element.value().attr("data-src"))
                        .or_else(|| element.value().attr("content"))
                    {
                        if src.contains("http") {
                            return Some(src.to_string());
                        } else if src.starts_with('/') {
                            return Some(format!("https://www.amd.com{}", src));
                        }
                    }
                }
            }
        }

        None
    }

    /// Fetch Intel CPU information.
    async fn fetch_intel(&self, identifier: &DeviceIdentifier) -> Result<PartialDeviceInfo> {
        let model = &identifier.model;
        log::debug!("Searching Intel ARK for: {}", model);

        // Construct Intel ARK search URL
        let search_url = format!(
            "https://ark.intel.com/content/www/us/en/ark/search.html?q={}",
            urlencoding::encode(model)
        );

        // Try to fetch the search page
        let response = self
            .client
            .get(&search_url)
            .send()
            .await
            .context("Failed to fetch Intel ARK")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Intel ARK search failed"));
        }

        let html = response.text().await?;
        let document = Html::parse_document(&html);

        // Try to find product link
        let product_url = self.extract_intel_product_link(&document)
            .unwrap_or(search_url.clone());

        // Extract product image
        let image_url = self.extract_intel_image(&document);

        // Build documentation links
        let documentation = Some(DocumentationLinks {
            product_page: Some(product_url.clone()),
            support_page: Some("https://www.intel.com/content/www/us/en/support.html".to_string()),
            manuals: vec![],
            datasheets: vec![],
            firmware_updates: vec![],
        });

        Ok(PartialDeviceInfo {
            specs: HashMap::new(),
            categories: vec![],
            description: Some(format!("Intel {}", model)),
            release_date: None,
            product_page: Some(product_url.clone()),
            support_page: Some("https://www.intel.com/content/www/us/en/support.html".to_string()),
            image_url,
            source_name: "Intel ARK".to_string(),
            source_url: Some(product_url),
            confidence: 0.95, // Very high - official source
            image_cached_path: None,
            thumbnail_url: None,
            thumbnail_cached_path: None,
            image_gallery: vec![],
            documentation,
            driver_info: None,
        })
    }

    /// Extract product link from Intel ARK search results.
    fn extract_intel_product_link(&self, document: &Html) -> Option<String> {
        let selectors = [
            "a.search-result",
            ".resultList a",
            "a[href*='/ark/products/']",
        ];

        for selector_str in selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(element) = document.select(&selector).next() {
                    if let Some(href) = element.value().attr("href") {
                        if href.starts_with("http") {
                            return Some(href.to_string());
                        } else {
                            return Some(format!("https://ark.intel.com{}", href));
                        }
                    }
                }
            }
        }

        None
    }

    /// Extract product image from Intel ARK page.
    fn extract_intel_image(&self, document: &Html) -> Option<String> {
        let selectors = [
            ".product-image img",
            "meta[property='og:image']",
            ".badge-image img",
        ];

        for selector_str in selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(element) = document.select(&selector).next() {
                    if let Some(src) = element.value().attr("src")
                        .or_else(|| element.value().attr("content"))
                    {
                        if src.contains("http") {
                            return Some(src.to_string());
                        }
                    }
                }
            }
        }

        None
    }
}

/// Supported manufacturers.
#[derive(Debug, Clone, Copy)]
enum Manufacturer {
    Nvidia,
    Amd,
    Intel,
}

#[async_trait]
impl DeviceSource for ManufacturerSource {
    fn name(&self) -> &str {
        "Manufacturer Official"
    }

    fn priority(&self) -> u8 {
        5 // Highest priority - official sources
    }

    fn supports(&self, device_type: &DeviceType, identifier: &DeviceIdentifier) -> bool {
        // Only support GPU and CPU for now
        if !matches!(device_type, DeviceType::Gpu | DeviceType::Cpu) {
            return false;
        }

        // Check if we recognize the manufacturer
        Self::detect_manufacturer(identifier).is_some()
    }

    async fn fetch(
        &self,
        device_type: &DeviceType,
        identifier: &DeviceIdentifier,
    ) -> Result<PartialDeviceInfo> {
        let manufacturer = Self::detect_manufacturer(identifier)
            .context("Unknown manufacturer")?;

        match manufacturer {
            Manufacturer::Nvidia => self.fetch_nvidia(identifier).await,
            Manufacturer::Amd => self.fetch_amd(identifier, device_type).await,
            Manufacturer::Intel => self.fetch_intel(identifier).await,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_manufacturer() {
        let nvidia_id = DeviceIdentifier {
            manufacturer: "NVIDIA".to_string(),
            model: "GeForce RTX 4090".to_string(),
            part_number: None,
            serial_number: None,
            hardware_ids: vec![],
        };
        assert!(matches!(ManufacturerSource::detect_manufacturer(&nvidia_id), Some(Manufacturer::Nvidia)));

        let amd_id = DeviceIdentifier {
            manufacturer: "AMD".to_string(),
            model: "Radeon RX 7900 XTX".to_string(),
            part_number: None,
            serial_number: None,
            hardware_ids: vec![],
        };
        assert!(matches!(ManufacturerSource::detect_manufacturer(&amd_id), Some(Manufacturer::Amd)));

        let intel_id = DeviceIdentifier {
            manufacturer: "Intel".to_string(),
            model: "Core i9-13900K".to_string(),
            part_number: None,
            serial_number: None,
            hardware_ids: vec![],
        };
        assert!(matches!(ManufacturerSource::detect_manufacturer(&intel_id), Some(Manufacturer::Intel)));
    }

    #[test]
    fn test_extract_nvidia_model() {
        assert_eq!(ManufacturerSource::extract_nvidia_model("GeForce RTX 4090"), "rtx-4090");
        assert_eq!(ManufacturerSource::extract_nvidia_model("NVIDIA GeForce RTX 5070 Ti"), "rtx-5070-ti");
        assert_eq!(ManufacturerSource::extract_nvidia_model("GTX 1080"), "gtx-1080");
    }

    #[test]
    fn test_construct_nvidia_url() {
        assert!(ManufacturerSource::construct_nvidia_url("rtx-4090").contains("40-series"));
        assert!(ManufacturerSource::construct_nvidia_url("rtx-5070").contains("50-series"));
        assert!(ManufacturerSource::construct_nvidia_url("rtx-3080").contains("30-series"));
    }
}
