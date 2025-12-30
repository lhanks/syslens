//! WikiChip source for detailed CPU architecture information.
//!
//! Fetches detailed technical specifications, architecture info, die shots,
//! and block diagrams from WikiChip's comprehensive processor database.

use crate::models::{DeviceIdentifier, DeviceType, SpecCategory, SpecItem};
use crate::services::device_sources::DeviceSource;
use crate::services::PartialDeviceInfo;
use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use scraper::{Html, Selector};
use std::collections::HashMap;

const WIKICHIP_BASE: &str = "https://en.wikichip.org";
const WIKICHIP_API: &str = "https://en.wikichip.org/w/api.php";

/// WikiChip source for detailed CPU architecture information.
pub struct WikiChipSource {
    client: Client,
}

impl WikiChipSource {
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self { client })
    }

    /// Build WikiChip page title for Intel CPUs.
    fn build_intel_title(model: &str) -> String {
        let model_lower = model.to_lowercase();

        // Extract series and model number
        if let Some(captures) = regex::Regex::new(r"(?i)(i[3579])[\s-]?(\d{4,5}[a-z]*)").ok()
            .and_then(|re| re.captures(&model_lower))
        {
            let series = captures.get(1).map(|m| m.as_str()).unwrap_or("");
            let number = captures.get(2).map(|m| m.as_str()).unwrap_or("");
            return format!("intel/core_{}/{}_{}", series, series, number);
        }

        // Xeon patterns
        if model_lower.contains("xeon") {
            if let Some(captures) = regex::Regex::new(r"(?i)([egw]\d?)-?(\d{4,5}[a-z]*)").ok()
                .and_then(|re| re.captures(&model_lower))
            {
                let series = captures.get(1).map(|m| m.as_str()).unwrap_or("");
                let number = captures.get(2).map(|m| m.as_str()).unwrap_or("");
                return format!("intel/xeon_{}/{}-{}", series.to_lowercase(), series.to_lowercase(), number);
            }
        }

        // Default fallback
        format!("intel/{}", model_lower.replace(" ", "_"))
    }

    /// Build WikiChip page title for AMD CPUs.
    fn build_amd_title(model: &str) -> String {
        let model_lower = model.to_lowercase();

        // Ryzen patterns
        if let Some(captures) = regex::Regex::new(r"(?i)ryzen\s*(\d)\s*(\d{4}[a-z0-9]*)").ok()
            .and_then(|re| re.captures(&model_lower))
        {
            let series = captures.get(1).map(|m| m.as_str()).unwrap_or("");
            let number = captures.get(2).map(|m| m.as_str()).unwrap_or("");
            return format!("amd/ryzen_{}/{}", series, number);
        }

        // Threadripper patterns
        if let Some(captures) = regex::Regex::new(r"(?i)threadripper\s*(pro\s*)?(\d{4}[a-z]*)").ok()
            .and_then(|re| re.captures(&model_lower))
        {
            let number = captures.get(2).map(|m| m.as_str()).unwrap_or("");
            if model_lower.contains("pro") {
                return format!("amd/ryzen_threadripper_pro/{}", number);
            }
            return format!("amd/ryzen_threadripper/{}", number);
        }

        // EPYC patterns
        if let Some(captures) = regex::Regex::new(r"(?i)epyc\s*(\d{4}[a-z]*)").ok()
            .and_then(|re| re.captures(&model_lower))
        {
            let number = captures.get(1).map(|m| m.as_str()).unwrap_or("");
            return format!("amd/epyc/{}", number);
        }

        // Default fallback
        format!("amd/{}", model_lower.replace(" ", "_"))
    }

    /// Search WikiChip using its API.
    async fn search_wikichip(&self, query: &str) -> Result<Option<String>> {
        let search_url = format!(
            "{}?action=opensearch&search={}&limit=5&namespace=0&format=json",
            WIKICHIP_API,
            urlencoding::encode(query)
        );

        log::debug!("WikiChip search URL: {}", search_url);

        let response = self.client.get(&search_url).send().await?;
        let json: serde_json::Value = response.json().await?;

        // OpenSearch returns [query, [titles], [descriptions], [urls]]
        if let Some(urls) = json.get(3).and_then(|v| v.as_array()) {
            for url in urls {
                if let Some(url_str) = url.as_str() {
                    // Filter to only processor pages
                    if url_str.contains("/intel/") || url_str.contains("/amd/") {
                        return Ok(Some(url_str.to_string()));
                    }
                }
            }
        }

        Ok(None)
    }

    /// Get page URL for CPU.
    async fn get_page_url(&self, identifier: &DeviceIdentifier) -> Result<Option<String>> {
        let manufacturer = identifier.manufacturer.to_lowercase();
        let model = &identifier.model;

        // Try direct URL construction first
        let title = if manufacturer.contains("intel") {
            Self::build_intel_title(model)
        } else if manufacturer.contains("amd") || manufacturer.contains("authenticamd") {
            Self::build_amd_title(model)
        } else {
            return Ok(None);
        };

        let direct_url = format!("{}/wiki/{}", WIKICHIP_BASE, title);
        log::debug!("Trying direct WikiChip URL: {}", direct_url);

        // Check if page exists
        let response = self.client.head(&direct_url).send().await?;
        if response.status().is_success() {
            return Ok(Some(direct_url));
        }

        // Fall back to search
        let search_query = format!("{} {}",
            if manufacturer.contains("intel") { "Intel" } else { "AMD" },
            model
        );

        self.search_wikichip(&search_query).await
    }

    /// Fetch and parse WikiChip page.
    async fn fetch_page(&self, url: &str) -> Result<WikiChipPageData> {
        let response = self
            .client
            .get(url)
            .send()
            .await
            .context("Failed to fetch WikiChip page")?;

        let html = response.text().await?;
        let document = Html::parse_document(&html);

        let mut data = WikiChipPageData {
            page_url: url.to_string(),
            ..Default::default()
        };

        // Extract product title
        let title_selector = Selector::parse("h1#firstHeading, h1.firstHeading").unwrap();
        if let Some(title) = document.select(&title_selector).next() {
            data.name = Some(title.text().collect::<String>().trim().to_string());
        }

        // Extract infobox image
        let img_selectors = [
            "table.infobox img",
            "div.infobox img",
            ".infobox-image img",
            "table.wikitable img",
        ];

        for selector_str in img_selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(img) = document.select(&selector).next() {
                    if let Some(src) = img.value().attr("src") {
                        // WikiChip images may be relative
                        data.image_url = Some(if src.starts_with("//") {
                            format!("https:{}", src)
                        } else if src.starts_with('/') {
                            format!("{}{}", WIKICHIP_BASE, src)
                        } else {
                            src.to_string()
                        });
                        break;
                    }
                }
            }
        }

        // Extract die shot if available
        let dieshot_selector = Selector::parse("a[href*='die'], img[alt*='die'], a[title*='die shot']").unwrap();
        if let Some(dieshot) = document.select(&dieshot_selector).next() {
            if let Some(href) = dieshot.value().attr("href") {
                if href.contains("/wiki/File:") {
                    data.gallery_images.push(format!("{}{}", WIKICHIP_BASE, href));
                }
            }
        }

        // Extract specs from infobox
        let infobox_selector = Selector::parse("table.infobox tr, div.infobox-row").unwrap();
        let th_selector = Selector::parse("th, .infobox-label").unwrap();
        let td_selector = Selector::parse("td, .infobox-data").unwrap();

        for row in document.select(&infobox_selector) {
            let label = row.select(&th_selector).next()
                .map(|e| e.text().collect::<String>().trim().to_string());
            let value = row.select(&td_selector).next()
                .map(|e| e.text().collect::<String>().trim().to_string());

            if let (Some(label), Some(value)) = (label, value) {
                if !label.is_empty() && !value.is_empty() && label.len() < 100 {
                    data.specs.insert(label, value);
                }
            }
        }

        // Extract specs from specification section tables
        let spec_table_selector = Selector::parse("table.wikitable tr").unwrap();
        for row in document.select(&spec_table_selector) {
            let th = row.select(&th_selector).next()
                .map(|e| e.text().collect::<String>().trim().to_string());
            let td = row.select(&td_selector).next()
                .map(|e| e.text().collect::<String>().trim().to_string());

            if let (Some(label), Some(value)) = (th, td) {
                if !label.is_empty() && !value.is_empty() && label.len() < 100 {
                    data.specs.entry(label).or_insert(value);
                }
            }
        }

        // Extract architecture information from text
        let content_selector = Selector::parse("div#mw-content-text p").unwrap();
        let mut description_parts = Vec::new();

        for para in document.select(&content_selector).take(3) {
            let text = para.text().collect::<String>().trim().to_string();
            if !text.is_empty() && text.len() > 50 && !text.starts_with('[') {
                description_parts.push(text);
            }
        }

        if !description_parts.is_empty() {
            data.description = Some(description_parts.join("\n\n"));
        }

        Ok(data)
    }

    /// Convert raw specs to categorized specs.
    fn categorize_specs(raw_specs: &HashMap<String, String>) -> (HashMap<String, String>, Vec<SpecCategory>) {
        let mut specs = HashMap::new();
        let mut general = Vec::new();
        let mut microarch = Vec::new();
        let mut cores = Vec::new();
        let mut cache = Vec::new();
        let mut memory = Vec::new();
        let mut features = Vec::new();

        let spec_mappings = [
            // General
            ("Model Number", "model_number", "General"),
            ("Part Number", "part_number", "General"),
            ("Market", "market", "General"),
            ("Release Date", "release_date", "General"),
            ("Launch Date", "launch_date", "General"),
            ("Released", "release_date", "General"),
            ("MSRP", "msrp", "General"),
            ("Launch Price", "msrp", "General"),

            // Microarchitecture
            ("Architecture", "architecture", "Microarchitecture"),
            ("Microarchitecture", "microarchitecture", "Microarchitecture"),
            ("Core Name", "core_name", "Microarchitecture"),
            ("Codename", "codename", "Microarchitecture"),
            ("Process", "process", "Microarchitecture"),
            ("Process Technology", "process", "Microarchitecture"),
            ("Lithography", "lithography", "Microarchitecture"),
            ("Die Size", "die_size", "Microarchitecture"),
            ("Transistors", "transistors", "Microarchitecture"),
            ("Transistor Count", "transistors", "Microarchitecture"),
            ("Pipeline Stages", "pipeline_stages", "Microarchitecture"),
            ("I/O", "io", "Microarchitecture"),

            // Cores
            ("Core Count", "cores", "Cores"),
            ("Cores", "cores", "Cores"),
            ("# of Cores", "cores", "Cores"),
            ("Thread Count", "threads", "Cores"),
            ("Threads", "threads", "Cores"),
            ("# of Threads", "threads", "Cores"),
            ("Base Frequency", "base_frequency", "Cores"),
            ("Base Clock", "base_frequency", "Cores"),
            ("Boost Frequency", "boost_frequency", "Cores"),
            ("Boost Clock", "boost_frequency", "Cores"),
            ("Turbo Frequency", "turbo_frequency", "Cores"),
            ("Max Turbo Frequency", "turbo_frequency", "Cores"),
            ("TDP", "tdp", "Cores"),
            ("Thermal Design Power", "tdp", "Cores"),

            // Cache
            ("L1 Cache", "l1_cache", "Cache"),
            ("L1 Data Cache", "l1d_cache", "Cache"),
            ("L1 Instruction Cache", "l1i_cache", "Cache"),
            ("L2 Cache", "l2_cache", "Cache"),
            ("L3 Cache", "l3_cache", "Cache"),
            ("Total Cache", "total_cache", "Cache"),

            // Memory
            ("Memory Support", "memory_support", "Memory"),
            ("Max Memory", "max_memory", "Memory"),
            ("Memory Channels", "memory_channels", "Memory"),
            ("Memory Bandwidth", "memory_bandwidth", "Memory"),
            ("Memory Types", "memory_types", "Memory"),

            // Features
            ("Socket", "socket", "Features"),
            ("CPU Socket", "socket", "Features"),
            ("PCIe", "pcie", "Features"),
            ("PCIe Lanes", "pcie_lanes", "Features"),
            ("AVX", "avx", "Features"),
            ("AVX-512", "avx512", "Features"),
            ("SSE", "sse", "Features"),
            ("Virtualization", "virtualization", "Features"),
            ("VT-x", "vtx", "Features"),
            ("AMD-V", "amdv", "Features"),
            ("Security", "security", "Features"),
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
                    "Microarchitecture" => microarch.push(item),
                    "Cores" => cores.push(item),
                    "Cache" => cache.push(item),
                    "Memory" => memory.push(item),
                    "Features" => features.push(item),
                    _ => {}
                }
            }
        }

        let mut categories = Vec::new();
        if !general.is_empty() {
            categories.push(SpecCategory { name: "General".to_string(), specs: general });
        }
        if !microarch.is_empty() {
            categories.push(SpecCategory { name: "Microarchitecture".to_string(), specs: microarch });
        }
        if !cores.is_empty() {
            categories.push(SpecCategory { name: "Core Configuration".to_string(), specs: cores });
        }
        if !cache.is_empty() {
            categories.push(SpecCategory { name: "Cache Hierarchy".to_string(), specs: cache });
        }
        if !memory.is_empty() {
            categories.push(SpecCategory { name: "Memory".to_string(), specs: memory });
        }
        if !features.is_empty() {
            categories.push(SpecCategory { name: "Features".to_string(), specs: features });
        }

        (specs, categories)
    }

    /// Extract unit from a value string.
    fn extract_unit(value: &str) -> Option<String> {
        let units = ["GHz", "MHz", "MB", "KB", "GB", "TB", "W", "nm", "mm²", "billion", "million"];
        for unit in units {
            if value.contains(unit) {
                return Some(unit.to_string());
            }
        }
        None
    }
}

/// Data extracted from a WikiChip page.
#[derive(Debug, Default)]
struct WikiChipPageData {
    name: Option<String>,
    image_url: Option<String>,
    gallery_images: Vec<String>,
    page_url: String,
    specs: HashMap<String, String>,
    description: Option<String>,
}

#[async_trait]
impl DeviceSource for WikiChipSource {
    fn name(&self) -> &str {
        "WikiChip"
    }

    fn priority(&self) -> u8 {
        15 // Lower priority than official sources, higher than Wikipedia
    }

    fn supports(&self, device_type: &DeviceType, identifier: &DeviceIdentifier) -> bool {
        // WikiChip primarily covers CPUs
        if !matches!(device_type, DeviceType::Cpu) {
            return false;
        }

        // Support Intel and AMD processors
        let manufacturer = identifier.manufacturer.to_lowercase();
        manufacturer.contains("intel")
            || manufacturer.contains("genuineintel")
            || manufacturer.contains("amd")
            || manufacturer.contains("authenticamd")
    }

    async fn fetch(
        &self,
        _device_type: &DeviceType,
        identifier: &DeviceIdentifier,
    ) -> Result<PartialDeviceInfo> {
        // Get page URL
        let page_url = self
            .get_page_url(identifier)
            .await?
            .context("CPU not found on WikiChip")?;

        // Fetch page data
        let page_data = self.fetch_page(&page_url).await?;

        // Categorize specs
        let (specs, categories) = Self::categorize_specs(&page_data.specs);

        // Get release date
        let release_date = page_data.specs.get("Release Date")
            .or_else(|| page_data.specs.get("Launch Date"))
            .or_else(|| page_data.specs.get("Released"))
            .cloned();

        Ok(PartialDeviceInfo {
            specs,
            categories,
            description: page_data.description.or(page_data.name.clone()),
            release_date,
            product_page: Some(page_data.page_url.clone()),
            support_page: None,
            image_url: page_data.image_url.clone(),
            source_name: "WikiChip".to_string(),
            source_url: Some(page_data.page_url),
            confidence: 0.75, // Good confidence for technical details
            image_cached_path: None,
            thumbnail_url: None,
            thumbnail_cached_path: None,
            image_gallery: vec![], // Could add die shots here
            documentation: None,
            driver_info: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_intel_title() {
        assert_eq!(WikiChipSource::build_intel_title("i9-14900K"), "intel/core_i9/i9_14900k");
        assert_eq!(WikiChipSource::build_intel_title("Intel Core i7-13700K"), "intel/core_i7/i7_13700k");
        assert_eq!(WikiChipSource::build_intel_title("i5-12400"), "intel/core_i5/i5_12400");
    }

    #[test]
    fn test_build_amd_title() {
        assert_eq!(WikiChipSource::build_amd_title("Ryzen 9 7950X"), "amd/ryzen_9/7950x");
        assert_eq!(WikiChipSource::build_amd_title("AMD Ryzen 7 5800X"), "amd/ryzen_7/5800x");
        assert_eq!(WikiChipSource::build_amd_title("Threadripper Pro 5995WX"), "amd/ryzen_threadripper_pro/5995wx");
        assert_eq!(WikiChipSource::build_amd_title("EPYC 9654"), "amd/epyc/9654");
    }
}
