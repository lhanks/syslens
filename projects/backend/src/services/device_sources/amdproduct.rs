//! AMD Product database source for AMD CPU and GPU information.
//!
//! Fetches specifications from AMD's official product database.
//! This is the authoritative source for AMD processor and graphics specifications.

use crate::models::{DeviceIdentifier, DeviceType, SpecCategory, SpecItem};
use crate::services::device_sources::DeviceSource;
use crate::services::PartialDeviceInfo;
use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use scraper::{Html, Selector};
use std::collections::HashMap;

const AMD_BASE: &str = "https://www.amd.com";
const AMD_SEARCH: &str = "https://www.amd.com/en/search.html";

/// AMD Product database source for AMD CPUs and GPUs.
pub struct AMDProductSource {
    client: Client,
}

impl AMDProductSource {
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self { client })
    }

    /// Normalize AMD model name for URL slug generation.
    fn normalize_for_slug(model: &str) -> String {
        model
            .to_lowercase()
            .replace("amd ", "")
            .replace("radeon ", "")
            .replace("ryzen ", "ryzen-")
            .replace("threadripper ", "threadripper-")
            .replace("epyc ", "epyc-")
            .replace("  ", " ")
            .replace(" ", "-")
            .replace("--", "-")
            .trim_matches('-')
            .to_string()
    }

    /// Normalize model for search.
    fn normalize_model(model: &str) -> String {
        model
            .to_lowercase()
            .replace("amd ", "")
            .replace("(r)", "")
            .replace("(tm)", "")
            .trim()
            .to_string()
    }

    /// Extract the core model identifier (e.g., "7950X" from "Ryzen 9 7950X")
    fn extract_model_number(model: &str) -> Option<String> {
        let patterns = [
            // Ryzen patterns - capture the series and model
            r"(?i)ryzen\s*(\d)\s*(\d{4}[A-Z0-9]*)",
            // EPYC patterns
            r"(?i)epyc\s*(\d{4}[A-Z]*)",
            // Threadripper patterns
            r"(?i)threadripper\s*(pro\s*)?(\d{4}[A-Z]*)",
            // Radeon patterns
            r"(?i)r[xX]\s*(\d{4}\s*[A-Z]*)",
        ];

        for pattern in patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                if let Some(captures) = re.captures(model) {
                    // Return the full match after the product line
                    let mut result = String::new();
                    for i in 1..captures.len() {
                        if let Some(m) = captures.get(i) {
                            if !result.is_empty() {
                                result.push(' ');
                            }
                            result.push_str(m.as_str());
                        }
                    }
                    if !result.is_empty() {
                        return Some(result.to_uppercase());
                    }
                }
            }
        }
        None
    }

    /// Try to construct direct product URL.
    fn build_direct_url(model: &str, device_type: &DeviceType) -> String {
        let slug = Self::normalize_for_slug(model);
        let product_type = match device_type {
            DeviceType::Cpu => "cpu",
            DeviceType::Gpu => "graphics",
            _ => "processors",
        };
        format!("{}/en/products/{}/amd-{}", AMD_BASE, product_type, slug)
    }

    /// Build search URL.
    fn build_search_url(model: &str) -> String {
        let normalized = Self::normalize_model(model);
        let encoded = urlencoding::encode(&normalized);
        format!("{}?q={}", AMD_SEARCH, encoded)
    }

    /// Search for product and get the product page URL.
    async fn search_product(&self, model: &str, device_type: &DeviceType) -> Result<Option<String>> {
        // First try direct URL construction
        let direct_url = Self::build_direct_url(model, device_type);
        log::debug!("Trying direct AMD URL: {}", direct_url);

        let response = self.client.get(&direct_url).send().await;
        if let Ok(resp) = response {
            if resp.status().is_success() {
                let html = resp.text().await?;
                // Verify it's a real product page
                if html.contains("product-specs") || html.contains("amd-product") {
                    return Ok(Some(direct_url));
                }
            }
        }

        // Fall back to search
        let search_url = Self::build_search_url(model);
        log::debug!("AMD search URL: {}", search_url);

        let response = self
            .client
            .get(&search_url)
            .send()
            .await
            .context("Failed to search AMD")?;

        let html = response.text().await?;
        let document = Html::parse_document(&html);

        // Look for search results
        let result_selector = Selector::parse("a.search-result, div.search-result a, .product-link").unwrap();
        let normalized_model = Self::normalize_model(model).to_lowercase();
        let model_number = Self::extract_model_number(model);

        let product_path = match device_type {
            DeviceType::Cpu => "/products/cpu/",
            DeviceType::Gpu => "/products/graphics/",
            _ => "/products/",
        };

        for link in document.select(&result_selector) {
            let text = link.text().collect::<String>().to_lowercase();
            let href = link.value().attr("href");

            // Check for match
            let is_match = if let Some(ref num) = model_number {
                text.contains(&num.to_lowercase())
            } else {
                text.contains(&normalized_model) || normalized_model.split('-').all(|part| text.contains(part))
            };

            if is_match {
                if let Some(href) = href {
                    // Check if it's a product page URL
                    if href.contains(product_path) || href.contains("/products/") {
                        let full_url = if href.starts_with('/') {
                            format!("{}{}", AMD_BASE, href)
                        } else if href.starts_with("http") {
                            href.to_string()
                        } else {
                            continue;
                        };
                        log::debug!("Found AMD match: {} -> {}", text.trim(), full_url);
                        return Ok(Some(full_url));
                    }
                }
            }
        }

        Ok(None)
    }

    /// Fetch and parse product page.
    async fn fetch_product_page(&self, url: &str, _device_type: &DeviceType) -> Result<ProductPageData> {
        let response = self
            .client
            .get(url)
            .send()
            .await
            .context("Failed to fetch AMD product page")?;

        let html = response.text().await?;
        let document = Html::parse_document(&html);

        let mut data = ProductPageData {
            page_url: url.to_string(),
            ..Default::default()
        };

        // Extract product image
        let img_selectors = [
            "img.product-image",
            "div.product-hero img",
            ".product-media img",
            "picture.product-image source",
            "img[alt*='AMD']",
        ];

        for selector_str in img_selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(img) = document.select(&selector).next() {
                    let src = img.value().attr("src")
                        .or_else(|| img.value().attr("srcset"))
                        .or_else(|| img.value().attr("data-src"));

                    if let Some(src) = src {
                        // Get the first URL from srcset if present
                        let src = src.split(',').next().unwrap_or(src).split(' ').next().unwrap_or(src);
                        data.image_url = Some(if src.starts_with("//") {
                            format!("https:{}", src)
                        } else if src.starts_with('/') {
                            format!("{}{}", AMD_BASE, src)
                        } else {
                            src.to_string()
                        });
                        break;
                    }
                }
            }
        }

        // Extract product title
        let title_selectors = ["h1.product-name", "h1.title", "h1", ".product-title"];
        for selector_str in title_selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(title) = document.select(&selector).next() {
                    let text = title.text().collect::<String>().trim().to_string();
                    if !text.is_empty() && text.len() < 200 && text.to_lowercase().contains("amd") {
                        data.name = Some(text);
                        break;
                    }
                }
            }
        }

        // Extract specs from AMD specification tables/sections
        // AMD uses various formats for specs
        let spec_row_selectors = [
            "div.specs-row",
            "tr.spec-row",
            "li.spec-item",
            "div.product-spec",
            "table.specs tbody tr",
        ];

        for selector_str in spec_row_selectors {
            if let Ok(row_selector) = Selector::parse(selector_str) {
                for row in document.select(&row_selector) {
                    let text: String = row.text().collect();
                    let parts: Vec<&str> = text.split(':').collect();
                    if parts.len() >= 2 {
                        let label = parts[0].trim().to_string();
                        let value = parts[1..].join(":").trim().to_string();
                        if !label.is_empty() && !value.is_empty() {
                            data.specs.insert(label, value);
                        }
                    }
                }
            }
        }

        // Try label/value pair extraction
        let label_value_selector = Selector::parse("div.spec-label, span.spec-label, dt").unwrap();
        let value_selector = Selector::parse("div.spec-value, span.spec-value, dd").unwrap();

        let labels: Vec<_> = document.select(&label_value_selector).collect();
        let values: Vec<_> = document.select(&value_selector).collect();

        for (label_el, value_el) in labels.iter().zip(values.iter()) {
            let label = label_el.text().collect::<String>().trim().to_string();
            let value = value_el.text().collect::<String>().trim().to_string();
            if !label.is_empty() && !value.is_empty() {
                data.specs.insert(label, value);
            }
        }

        // Extract from JSON-LD if present
        let script_selector = Selector::parse("script[type='application/ld+json']").unwrap();
        for script in document.select(&script_selector) {
            let json_text = script.text().collect::<String>();
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&json_text) {
                if let Some(name) = json.get("name").and_then(|v| v.as_str()) {
                    if data.name.is_none() {
                        data.name = Some(name.to_string());
                    }
                }
                if let Some(image) = json.get("image").and_then(|v| v.as_str()) {
                    if data.image_url.is_none() {
                        data.image_url = Some(image.to_string());
                    }
                }
            }
        }

        // Extract support/driver page URL
        data.support_url = Some(format!("{}/en/support", AMD_BASE));
        data.driver_url = Some(format!("{}/en/support/download/drivers.html", AMD_BASE));

        Ok(data)
    }

    /// Convert raw specs to categorized specs for CPUs.
    fn categorize_cpu_specs(raw_specs: &HashMap<String, String>) -> (HashMap<String, String>, Vec<SpecCategory>) {
        let mut specs = HashMap::new();
        let mut general = Vec::new();
        let mut performance = Vec::new();
        let mut memory = Vec::new();
        let mut connectivity = Vec::new();
        let mut thermal = Vec::new();

        let spec_mappings = [
            // General
            ("Platform", "platform", "General"),
            ("Product Family", "product_family", "General"),
            ("Product Line", "product_line", "General"),
            ("Launch Date", "launch_date", "General"),
            ("OS Support", "os_support", "General"),

            // Performance
            ("# of CPU Cores", "cores", "Performance"),
            ("Cores", "cores", "Performance"),
            ("# of Threads", "threads", "Performance"),
            ("Threads", "threads", "Performance"),
            ("Base Clock", "base_clock", "Performance"),
            ("Max Boost Clock", "boost_clock", "Performance"),
            ("Max. Boost Clock", "boost_clock", "Performance"),
            ("L1 Cache", "l1_cache", "Performance"),
            ("L2 Cache", "l2_cache", "Performance"),
            ("L3 Cache", "l3_cache", "Performance"),
            ("Total L3 Cache", "l3_cache", "Performance"),
            ("Unlocked for Overclocking", "unlocked", "Performance"),
            ("CPU Socket", "socket", "Performance"),

            // Memory
            ("Max Memory Speed", "max_memory_speed", "Memory"),
            ("System Memory Type", "memory_type", "Memory"),
            ("Memory Channels", "memory_channels", "Memory"),
            ("Max Memory", "max_memory", "Memory"),

            // Connectivity
            ("PCI Express Version", "pcie_version", "Connectivity"),
            ("PCIe Lanes", "pcie_lanes", "Connectivity"),
            ("USB Ports", "usb_ports", "Connectivity"),

            // Thermal
            ("Default TDP", "tdp", "Thermal"),
            ("TDP", "tdp", "Thermal"),
            ("Max Temperature", "max_temp", "Thermal"),
            ("Processor Technology", "process_node", "Thermal"),
        ];

        for (raw_key, spec_key, category) in spec_mappings {
            if let Some(value) = raw_specs.get(raw_key) {
                specs.insert(spec_key.to_string(), value.clone());

                let item = SpecItem {
                    label: raw_key.to_string(),
                    value: value.clone(),
                    unit: Self::extract_unit(value),
                };

                match category {
                    "General" => general.push(item),
                    "Performance" => performance.push(item),
                    "Memory" => memory.push(item),
                    "Connectivity" => connectivity.push(item),
                    "Thermal" => thermal.push(item),
                    _ => {}
                }
            }
        }

        let mut categories = Vec::new();
        if !general.is_empty() {
            categories.push(SpecCategory { name: "General Specifications".to_string(), specs: general });
        }
        if !performance.is_empty() {
            categories.push(SpecCategory { name: "CPU Specifications".to_string(), specs: performance });
        }
        if !memory.is_empty() {
            categories.push(SpecCategory { name: "Memory Specifications".to_string(), specs: memory });
        }
        if !connectivity.is_empty() {
            categories.push(SpecCategory { name: "Connectivity".to_string(), specs: connectivity });
        }
        if !thermal.is_empty() {
            categories.push(SpecCategory { name: "Thermal".to_string(), specs: thermal });
        }

        (specs, categories)
    }

    /// Convert raw specs to categorized specs for GPUs.
    fn categorize_gpu_specs(raw_specs: &HashMap<String, String>) -> (HashMap<String, String>, Vec<SpecCategory>) {
        let mut specs = HashMap::new();
        let mut general = Vec::new();
        let mut gpu_engine = Vec::new();
        let mut memory = Vec::new();
        let mut display = Vec::new();
        let mut board = Vec::new();

        let spec_mappings = [
            // General
            ("Graphics Card Series", "series", "General"),
            ("Launch Date", "launch_date", "General"),
            ("Product Family", "product_family", "General"),

            // GPU Engine
            ("Compute Units", "compute_units", "GPU Engine"),
            ("Stream Processors", "stream_processors", "GPU Engine"),
            ("Ray Accelerators", "ray_accelerators", "GPU Engine"),
            ("AI Accelerators", "ai_accelerators", "GPU Engine"),
            ("Game Clock", "game_clock", "GPU Engine"),
            ("Game Frequency", "game_clock", "GPU Engine"),
            ("Boost Clock", "boost_clock", "GPU Engine"),
            ("Boost Frequency", "boost_clock", "GPU Engine"),
            ("Base Frequency", "base_clock", "GPU Engine"),
            ("Peak Half Precision", "peak_half", "GPU Engine"),
            ("Peak Single Precision", "peak_single", "GPU Engine"),
            ("Peak Texture Fill-Rate", "texture_fill", "GPU Engine"),

            // Memory
            ("Memory Size", "memory_size", "Memory"),
            ("Memory Type", "memory_type", "Memory"),
            ("Memory Interface", "memory_interface", "Memory"),
            ("Memory Bandwidth", "memory_bandwidth", "Memory"),
            ("Effective Memory Clock", "memory_clock", "Memory"),
            ("Infinity Cache", "infinity_cache", "Memory"),

            // Display
            ("Max Displays", "max_displays", "Display"),
            ("Max Resolution", "max_resolution", "Display"),
            ("DisplayPort", "displayport", "Display"),
            ("HDMI", "hdmi", "Display"),
            ("USB Type-C", "usb_c", "Display"),

            // Board
            ("Typical Board Power", "tdp", "Board"),
            ("TDP", "tdp", "Board"),
            ("Minimum PSU Recommendation", "min_psu", "Board"),
            ("Power Connectors", "power_connectors", "Board"),
            ("PCI Express", "pcie", "Board"),
            ("Form Factor", "form_factor", "Board"),
            ("Length", "length", "Board"),
        ];

        for (raw_key, spec_key, category) in spec_mappings {
            if let Some(value) = raw_specs.get(raw_key) {
                specs.insert(spec_key.to_string(), value.clone());

                let item = SpecItem {
                    label: raw_key.to_string(),
                    value: value.clone(),
                    unit: Self::extract_unit(value),
                };

                match category {
                    "General" => general.push(item),
                    "GPU Engine" => gpu_engine.push(item),
                    "Memory" => memory.push(item),
                    "Display" => display.push(item),
                    "Board" => board.push(item),
                    _ => {}
                }
            }
        }

        let mut categories = Vec::new();
        if !general.is_empty() {
            categories.push(SpecCategory { name: "General".to_string(), specs: general });
        }
        if !gpu_engine.is_empty() {
            categories.push(SpecCategory { name: "GPU Engine".to_string(), specs: gpu_engine });
        }
        if !memory.is_empty() {
            categories.push(SpecCategory { name: "Memory".to_string(), specs: memory });
        }
        if !display.is_empty() {
            categories.push(SpecCategory { name: "Display".to_string(), specs: display });
        }
        if !board.is_empty() {
            categories.push(SpecCategory { name: "Board Design".to_string(), specs: board });
        }

        (specs, categories)
    }

    /// Extract unit from a value string.
    fn extract_unit(value: &str) -> Option<String> {
        let units = ["GHz", "MHz", "MB", "GB", "TB", "W", "nm", "GT/s", "GB/s", "bit", "mm"];
        for unit in units {
            if value.contains(unit) {
                return Some(unit.to_string());
            }
        }
        None
    }
}

/// Data extracted from an AMD product page.
#[derive(Debug, Default)]
struct ProductPageData {
    name: Option<String>,
    image_url: Option<String>,
    page_url: String,
    support_url: Option<String>,
    driver_url: Option<String>,
    specs: HashMap<String, String>,
}

#[async_trait]
impl DeviceSource for AMDProductSource {
    fn name(&self) -> &str {
        "AMD Product Database"
    }

    fn priority(&self) -> u8 {
        5 // Highest priority for AMD products
    }

    fn supports(&self, device_type: &DeviceType, identifier: &DeviceIdentifier) -> bool {
        // Only supports AMD CPUs and GPUs
        if !matches!(device_type, DeviceType::Cpu | DeviceType::Gpu) {
            return false;
        }

        // Check if manufacturer is AMD
        let manufacturer = identifier.manufacturer.to_lowercase();
        let model = identifier.model.to_lowercase();

        manufacturer.contains("amd")
            || manufacturer.contains("authenticamd")
            || manufacturer.contains("advanced micro")
            || model.contains("ryzen")
            || model.contains("radeon")
            || model.contains("epyc")
            || model.contains("threadripper")
    }

    async fn fetch(
        &self,
        device_type: &DeviceType,
        identifier: &DeviceIdentifier,
    ) -> Result<PartialDeviceInfo> {
        // Search for the product
        let product_url = self
            .search_product(&identifier.model, device_type)
            .await?
            .context("Product not found on AMD")?;

        // Fetch product page data
        let page_data = self.fetch_product_page(&product_url, device_type).await?;

        // Categorize specs based on device type
        let (specs, categories) = match device_type {
            DeviceType::Cpu => Self::categorize_cpu_specs(&page_data.specs),
            DeviceType::Gpu => Self::categorize_gpu_specs(&page_data.specs),
            _ => (page_data.specs.clone(), vec![]),
        };

        Ok(PartialDeviceInfo {
            specs,
            categories,
            description: page_data.name,
            release_date: page_data.specs.get("Launch Date").cloned(),
            product_page: Some(page_data.page_url.clone()),
            support_page: page_data.support_url,
            image_url: page_data.image_url.clone(),
            source_name: "AMD".to_string(),
            source_url: Some(page_data.page_url),
            confidence: 0.95, // Very high confidence - official AMD source
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
    fn test_normalize_for_slug() {
        assert_eq!(AMDProductSource::normalize_for_slug("AMD Ryzen 9 7950X"), "ryzen-9-7950x");
        assert_eq!(AMDProductSource::normalize_for_slug("Radeon RX 7900 XTX"), "rx-7900-xtx");
    }

    #[test]
    fn test_extract_model_number() {
        assert_eq!(AMDProductSource::extract_model_number("AMD Ryzen 9 7950X"), Some("9 7950X".to_string()));
        assert_eq!(AMDProductSource::extract_model_number("Radeon RX 7900 XTX"), Some("7900 XTX".to_string()));
    }

    #[test]
    fn test_normalize_model() {
        assert_eq!(AMDProductSource::normalize_model("AMD Ryzen 9 7950X"), "ryzen 9 7950x");
        assert_eq!(AMDProductSource::normalize_model("AMD(R) Radeon(TM) RX 7900 XTX"), "radeon rx 7900 xtx");
    }
}
