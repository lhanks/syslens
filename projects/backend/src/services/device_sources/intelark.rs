//! Intel ARK database source for Intel CPU information.
//!
//! Fetches CPU specifications and images from Intel's official ARK database.
//! This is the authoritative source for Intel processor specifications.

use crate::models::{DeviceIdentifier, DeviceType, SpecCategory, SpecItem};
use crate::services::device_sources::DeviceSource;
use crate::services::PartialDeviceInfo;
use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use scraper::{Html, Selector};
use std::collections::HashMap;

const INTEL_ARK_SEARCH: &str = "https://ark.intel.com/content/www/us/en/ark/search.html";
const INTEL_ARK_BASE: &str = "https://ark.intel.com";

/// Intel ARK-based device information source for Intel CPUs.
pub struct IntelArkSource {
    client: Client,
}

impl IntelArkSource {
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self { client })
    }

    /// Normalize CPU model name for search.
    fn normalize_model(model: &str) -> String {
        let model = model
            .to_lowercase()
            .replace("intel ", "")
            .replace("core ", "")
            .replace("(r)", "")
            .replace("(tm)", "")
            .replace("processor", "")
            .trim()
            .to_string();

        // Extract the model number (e.g., "i9-14900K" from various formats)
        model
    }

    /// Extract model number from CPU name (e.g., "i9-14900K" from "Intel Core i9-14900K Processor")
    fn extract_model_number(model: &str) -> Option<String> {
        let patterns = [
            // Core series patterns
            r"(i[3579]-\d{4,5}[A-Z]*)",
            r"(i[3579] \d{4,5}[A-Z]*)",
            // Xeon patterns
            r"([EWX]\d?-\d{4,5}[A-Z]*)",
            r"(Gold \d{4}[A-Z]*)",
            r"(Silver \d{4}[A-Z]*)",
            r"(Platinum \d{4}[A-Z]*)",
            // Pentium/Celeron
            r"(G\d{4}[A-Z]*)",
            r"(N\d{4}[A-Z]*)",
        ];

        let model_upper = model.to_uppercase();
        for pattern in patterns {
            if let Ok(re) = regex::Regex::new(&pattern.to_uppercase()) {
                if let Some(captures) = re.captures(&model_upper) {
                    if let Some(m) = captures.get(1) {
                        return Some(m.as_str().to_string());
                    }
                }
            }
        }
        None
    }

    /// Build search URL for Intel ARK.
    fn build_search_url(model: &str) -> String {
        let normalized = Self::normalize_model(model);
        let encoded = urlencoding::encode(&normalized);
        format!("{}?q={}", INTEL_ARK_SEARCH, encoded)
    }

    /// Search for CPU and get the product page URL.
    async fn search_cpu(&self, model: &str) -> Result<Option<String>> {
        let search_url = Self::build_search_url(model);
        log::debug!("Intel ARK search URL: {}", search_url);

        let response = self
            .client
            .get(&search_url)
            .send()
            .await
            .context("Failed to search Intel ARK")?;

        let html = response.text().await?;
        let document = Html::parse_document(&html);

        // Look for search results
        let result_selector = Selector::parse("div.search-result a, a.result-title, div.results a").unwrap();
        let normalized_model = Self::normalize_model(model).to_lowercase();
        let model_number = Self::extract_model_number(model);

        for link in document.select(&result_selector) {
            let text = link.text().collect::<String>().to_lowercase();
            let href = link.value().attr("href");

            // Check for match
            let is_match = if let Some(ref num) = model_number {
                text.contains(&num.to_lowercase())
            } else {
                text.contains(&normalized_model)
            };

            if is_match {
                if let Some(href) = href {
                    let full_url = if href.starts_with('/') {
                        format!("{}{}", INTEL_ARK_BASE, href)
                    } else if href.starts_with("http") {
                        href.to_string()
                    } else {
                        continue;
                    };

                    // Only match product pages
                    if full_url.contains("/products/") {
                        log::debug!("Found Intel ARK match: {} -> {}", text.trim(), full_url);
                        return Ok(Some(full_url));
                    }
                }
            }
        }

        // Try alternative selector for newer ARK layout
        let alt_selector = Selector::parse("table.results tbody tr td a, .ark-product-link").unwrap();
        for link in document.select(&alt_selector) {
            let text = link.text().collect::<String>().to_lowercase();
            let href = link.value().attr("href");

            let is_match = if let Some(ref num) = model_number {
                text.contains(&num.to_lowercase())
            } else {
                text.contains(&normalized_model)
            };

            if is_match {
                if let Some(href) = href {
                    let full_url = if href.starts_with('/') {
                        format!("{}{}", INTEL_ARK_BASE, href)
                    } else if href.starts_with("http") {
                        href.to_string()
                    } else {
                        continue;
                    };

                    if full_url.contains("/products/") {
                        log::debug!("Found Intel ARK match (alt): {} -> {}", text.trim(), full_url);
                        return Ok(Some(full_url));
                    }
                }
            }
        }

        Ok(None)
    }

    /// Fetch and parse CPU product page.
    async fn fetch_product_page(&self, url: &str) -> Result<CpuPageData> {
        let response = self
            .client
            .get(url)
            .send()
            .await
            .context("Failed to fetch Intel ARK product page")?;

        let html = response.text().await?;
        let document = Html::parse_document(&html);

        let mut data = CpuPageData {
            page_url: url.to_string(),
            ..Default::default()
        };

        // Extract product image
        let img_selectors = [
            "div.product-image img",
            "img.product-image",
            "div.ark-product-image img",
            ".badge-image img",
        ];

        for selector_str in img_selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(img) = document.select(&selector).next() {
                    if let Some(src) = img.value().attr("src").or_else(|| img.value().attr("data-src")) {
                        data.image_url = Some(if src.starts_with("//") {
                            format!("https:{}", src)
                        } else if src.starts_with('/') {
                            format!("{}{}", INTEL_ARK_BASE, src)
                        } else {
                            src.to_string()
                        });
                        break;
                    }
                }
            }
        }

        // Extract product title
        let title_selectors = ["h1.product-title", "h1", ".product-name"];
        for selector_str in title_selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(title) = document.select(&selector).next() {
                    let text = title.text().collect::<String>().trim().to_string();
                    if !text.is_empty() && text.len() < 200 {
                        data.name = Some(text);
                        break;
                    }
                }
            }
        }

        // Extract specs from ARK specification tables
        // ARK uses a format with section headers and key-value pairs
        let section_selector = Selector::parse("div.specs-section, section.specs, div.blade-inside").unwrap();
        let row_selector = Selector::parse("li.ark-data-param, tr.ark-row, div.spec-row").unwrap();
        let label_selector = Selector::parse("span.label, td.label, .spec-label").unwrap();
        let value_selector = Selector::parse("span.value, td.value, .spec-value").unwrap();

        for section in document.select(&section_selector) {
            for row in section.select(&row_selector) {
                let label = row.select(&label_selector).next()
                    .map(|e| e.text().collect::<String>().trim().to_string());
                let value = row.select(&value_selector).next()
                    .map(|e| e.text().collect::<String>().trim().to_string());

                if let (Some(label), Some(value)) = (label, value) {
                    if !label.is_empty() && !value.is_empty() {
                        data.specs.insert(label, value);
                    }
                }
            }
        }

        // Alternative spec extraction using data attributes
        let alt_spec_selector = Selector::parse("[data-key]").unwrap();
        for element in document.select(&alt_spec_selector) {
            if let Some(key) = element.value().attr("data-key") {
                let value = element.text().collect::<String>().trim().to_string();
                if !value.is_empty() {
                    data.specs.insert(key.to_string(), value);
                }
            }
        }

        // Extract key specs using common ARK field IDs
        let spec_fields = [
            ("CoreCount", "Cores"),
            ("ThreadCount", "Threads"),
            ("MaxTDP", "TDP"),
            ("ClockSpeed", "Base Frequency"),
            ("ClockSpeedMax", "Max Turbo Frequency"),
            ("Cache", "Cache"),
            ("SocketsSupported", "Socket"),
            ("ProcessorNumber", "Processor Number"),
            ("Lithography", "Lithography"),
            ("LaunchDate", "Launch Date"),
        ];

        for (id, label) in spec_fields {
            // Use static selectors to avoid lifetime issues
            let selector_str: String = format!("span[data-key=\"{}\"], [id=\"{}\"]", id, id);
            // Parse selector and use it within this scope
            let parsed = Selector::parse(&selector_str);
            if let Ok(id_selector) = parsed {
                if let Some(element) = document.select(&id_selector).next() {
                    let value = element.text().collect::<String>().trim().to_string();
                    if !value.is_empty() {
                        data.specs.insert(label.to_string(), value);
                    }
                }
            }
        }

        // Extract launch date
        if let Some(date) = data.specs.get("Launch Date").or(data.specs.get("LaunchDate")) {
            data.release_date = Some(date.clone());
        }

        // Support page URL
        data.support_url = Some(format!("{}/support.html", url.trim_end_matches(".html")));

        Ok(data)
    }

    /// Convert raw specs to categorized specs.
    fn categorize_specs(raw_specs: &HashMap<String, String>) -> (HashMap<String, String>, Vec<SpecCategory>) {
        let mut specs = HashMap::new();
        let mut essentials = Vec::new();
        let mut performance = Vec::new();
        let mut memory = Vec::new();
        let mut expansion = Vec::new();
        let mut package = Vec::new();
        let mut graphics = Vec::new();

        let spec_mappings = [
            // Essentials
            ("Processor Number", "processor_number", "Essentials"),
            ("Code Name", "code_name", "Essentials"),
            ("Lithography", "lithography", "Essentials"),
            ("Launch Date", "launch_date", "Essentials"),
            ("MSRP", "msrp", "Essentials"),
            ("Status", "status", "Essentials"),

            // Performance
            ("Cores", "cores", "Performance"),
            ("# of Cores", "cores", "Performance"),
            ("Total Cores", "cores", "Performance"),
            ("CoreCount", "cores", "Performance"),
            ("Threads", "threads", "Performance"),
            ("# of Threads", "threads", "Performance"),
            ("ThreadCount", "threads", "Performance"),
            ("Base Frequency", "base_frequency", "Performance"),
            ("ClockSpeed", "base_frequency", "Performance"),
            ("Max Turbo Frequency", "turbo_frequency", "Performance"),
            ("ClockSpeedMax", "turbo_frequency", "Performance"),
            ("TDP", "tdp", "Performance"),
            ("MaxTDP", "tdp", "Performance"),
            ("Processor Base Power", "tdp", "Performance"),
            ("Cache", "cache", "Performance"),
            ("L3 Cache", "l3_cache", "Performance"),
            ("L2 Cache", "l2_cache", "Performance"),

            // Memory
            ("Max Memory Size", "max_memory", "Memory"),
            ("Memory Types", "memory_types", "Memory"),
            ("Max Memory Speed", "max_memory_speed", "Memory"),
            ("Max Memory Bandwidth", "max_memory_bandwidth", "Memory"),
            ("Max # of Memory Channels", "memory_channels", "Memory"),
            ("ECC Memory Supported", "ecc_support", "Memory"),

            // Expansion
            ("Max # of PCI Express Lanes", "pcie_lanes", "Expansion"),
            ("PCI Express Revision", "pcie_revision", "Expansion"),
            ("PCI Express Configurations", "pcie_configs", "Expansion"),
            ("Scalability", "scalability", "Expansion"),

            // Package
            ("Socket", "socket", "Package"),
            ("Sockets Supported", "socket", "Package"),
            ("SocketsSupported", "socket", "Package"),
            ("Package Size", "package_size", "Package"),
            ("Max CPU Configuration", "max_cpu_config", "Package"),

            // Graphics (for CPUs with integrated graphics)
            ("Processor Graphics", "processor_graphics", "Graphics"),
            ("Graphics Base Frequency", "graphics_base_freq", "Graphics"),
            ("Graphics Max Dynamic Frequency", "graphics_max_freq", "Graphics"),
            ("Graphics Output", "graphics_output", "Graphics"),
            ("Max Resolution", "max_resolution", "Graphics"),
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
                    "Essentials" => essentials.push(item),
                    "Performance" => performance.push(item),
                    "Memory" => memory.push(item),
                    "Expansion" => expansion.push(item),
                    "Package" => package.push(item),
                    "Graphics" => graphics.push(item),
                    _ => {}
                }
            }
        }

        let mut categories = Vec::new();
        if !essentials.is_empty() {
            categories.push(SpecCategory {
                name: "Essentials".to_string(),
                specs: essentials,
            });
        }
        if !performance.is_empty() {
            categories.push(SpecCategory {
                name: "CPU Specifications".to_string(),
                specs: performance,
            });
        }
        if !memory.is_empty() {
            categories.push(SpecCategory {
                name: "Memory Specifications".to_string(),
                specs: memory,
            });
        }
        if !expansion.is_empty() {
            categories.push(SpecCategory {
                name: "Expansion Options".to_string(),
                specs: expansion,
            });
        }
        if !package.is_empty() {
            categories.push(SpecCategory {
                name: "Package Specifications".to_string(),
                specs: package,
            });
        }
        if !graphics.is_empty() {
            categories.push(SpecCategory {
                name: "Processor Graphics".to_string(),
                specs: graphics,
            });
        }

        (specs, categories)
    }

    /// Extract unit from a value string.
    fn extract_unit(value: &str) -> Option<String> {
        let units = ["GHz", "MHz", "MB", "GB", "TB", "W", "nm", "GT/s", "GB/s"];
        for unit in units {
            if value.contains(unit) {
                return Some(unit.to_string());
            }
        }
        None
    }
}

/// Data extracted from an Intel ARK CPU page.
#[derive(Debug, Default)]
struct CpuPageData {
    name: Option<String>,
    image_url: Option<String>,
    page_url: String,
    support_url: Option<String>,
    specs: HashMap<String, String>,
    release_date: Option<String>,
}

#[async_trait]
impl DeviceSource for IntelArkSource {
    fn name(&self) -> &str {
        "Intel ARK"
    }

    fn priority(&self) -> u8 {
        5 // Highest priority for Intel CPUs
    }

    fn supports(&self, device_type: &DeviceType, identifier: &DeviceIdentifier) -> bool {
        // Only supports Intel CPUs
        if !matches!(device_type, DeviceType::Cpu) {
            return false;
        }

        // Check if manufacturer is Intel
        let manufacturer = identifier.manufacturer.to_lowercase();
        manufacturer.contains("intel") || manufacturer.contains("genuineintel")
    }

    async fn fetch(
        &self,
        _device_type: &DeviceType,
        identifier: &DeviceIdentifier,
    ) -> Result<PartialDeviceInfo> {
        // Search for the CPU
        let product_url = self
            .search_cpu(&identifier.model)
            .await?
            .context("CPU not found on Intel ARK")?;

        // Fetch product page data
        let page_data = self.fetch_product_page(&product_url).await?;

        // Categorize specs
        let (specs, categories) = Self::categorize_specs(&page_data.specs);

        Ok(PartialDeviceInfo {
            specs,
            categories,
            description: page_data.name,
            release_date: page_data.release_date,
            product_page: Some(page_data.page_url.clone()),
            support_page: page_data.support_url,
            image_url: page_data.image_url.clone(),
            source_name: "Intel ARK".to_string(),
            source_url: Some(page_data.page_url),
            confidence: 0.95, // Very high confidence - official Intel source
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
    fn test_normalize_model() {
        assert_eq!(IntelArkSource::normalize_model("Intel Core i9-14900K Processor"), "i9-14900k");
        assert_eq!(IntelArkSource::normalize_model("Intel(R) Core(TM) i7-13700K"), "i7-13700k");
    }

    #[test]
    fn test_extract_model_number() {
        assert_eq!(IntelArkSource::extract_model_number("Intel Core i9-14900K"), Some("I9-14900K".to_string()));
        assert_eq!(IntelArkSource::extract_model_number("Intel Core i7-13700K Processor"), Some("I7-13700K".to_string()));
        assert_eq!(IntelArkSource::extract_model_number("Intel Xeon Gold 6348"), Some("GOLD 6348".to_string()));
    }

    #[test]
    fn test_build_search_url() {
        let url = IntelArkSource::build_search_url("Intel Core i9-14900K");
        assert!(url.contains("ark.intel.com"));
        assert!(url.contains("i9-14900k"));
    }
}
