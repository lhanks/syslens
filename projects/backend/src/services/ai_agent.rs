//! AI-powered device information agent.
//!
//! Uses intelligent web search and parsing to find device specifications
//! when traditional manufacturer lookups fail.

use crate::models::{
    DataMetadata, DataSource, DeviceDeepInfo, DeviceIdentifier, DeviceSpecifications,
    DeviceType, DocumentationLinks, DriverInfo, ProductImages, SpecCategory, SpecItem,
};
use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use reqwest::Client;
use scraper::{Html, Selector};
use std::collections::HashMap;

/// User agent for web requests
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

/// Request timeout in seconds
const REQUEST_TIMEOUT: u64 = 15;

/// Minimum confidence threshold for accepting results
const MIN_CONFIDENCE: f32 = 0.3;

/// AI-powered device lookup agent.
///
/// Uses multiple search strategies and intelligent parsing to find
/// device information from various web sources.
pub struct AiAgent {
    client: Client,
}

/// Search result from web search.
#[derive(Debug, Clone)]
struct SearchResult {
    #[allow(dead_code)]
    title: String,
    url: String,
    #[allow(dead_code)]
    snippet: String,
}

/// Extracted specification from a web page.
#[derive(Debug, Clone)]
struct ExtractedSpec {
    label: String,
    value: String,
    confidence: f32,
}

impl AiAgent {
    /// Create a new AI agent with configured HTTP client.
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(REQUEST_TIMEOUT))
            .user_agent(USER_AGENT)
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self { client })
    }

    /// Search for device information using AI-powered techniques.
    ///
    /// This is the main entry point that orchestrates the search process:
    /// 1. Generate search queries based on device type
    /// 2. Search multiple sources
    /// 3. Parse and extract specifications
    /// 4. Merge and validate results
    pub async fn search_device(
        &self,
        identifier: &DeviceIdentifier,
        device_type: &DeviceType,
    ) -> Result<DeviceDeepInfo> {
        log::info!(
            "AI Agent searching for: {} {} ({:?})",
            identifier.manufacturer,
            identifier.model,
            device_type
        );

        // Generate search queries
        let queries = self.generate_search_queries(identifier, device_type);

        // Try each query until we get good results
        let mut best_result: Option<DeviceDeepInfo> = None;
        let mut best_confidence: f32 = 0.0;

        for query in queries.iter().take(3) {
            match self.search_and_extract(query, identifier, device_type).await {
                Ok(result) => {
                    let confidence = result.metadata.ai_confidence.unwrap_or(0.0);
                    log::debug!("Query '{}' returned confidence: {:.2}", query, confidence);

                    if confidence > best_confidence {
                        best_confidence = confidence;
                        best_result = Some(result);
                    }

                    if confidence >= 0.7 {
                        // High confidence result, stop searching
                        break;
                    }
                }
                Err(e) => {
                    log::debug!("Query '{}' failed: {}", query, e);
                }
            }
        }

        best_result
            .filter(|r| r.metadata.ai_confidence.unwrap_or(0.0) >= MIN_CONFIDENCE)
            .ok_or_else(|| anyhow::anyhow!("AI Agent could not find reliable information"))
    }

    /// Normalize manufacturer name for search queries.
    fn normalize_manufacturer(raw: &str) -> String {
        let lower = raw.to_lowercase();

        // Map common vendor IDs and variations to clean names
        if lower.contains("authenticamd") || lower.contains("advanced micro") {
            return "AMD".to_string();
        }
        if lower.contains("genuineintel") || lower.contains("intel") {
            return "Intel".to_string();
        }
        if lower.contains("nvidia") {
            return "NVIDIA".to_string();
        }

        // Return original if no match, but capitalize first letter
        let mut result = raw.to_string();
        if let Some(first) = result.get_mut(0..1) {
            first.make_ascii_uppercase();
        }
        result
    }

    /// Clean model name for better search results.
    fn clean_model_name(raw: &str) -> String {
        let mut cleaned = raw.to_string();

        // Remove common suffixes
        let suffixes_to_remove = [
            "-12-core-processor-",
            "-8-core-processor-",
            "-6-core-processor-",
            "-4-core-processor-",
            "-core-processor-",
            "-processor-",
            " 12-Core Processor",
            " 8-Core Processor",
            " with Radeon Graphics",
        ];

        for suffix in suffixes_to_remove {
            if cleaned.to_lowercase().contains(&suffix.to_lowercase()) {
                cleaned = cleaned.to_lowercase().replace(&suffix.to_lowercase(), "");
            }
        }

        // Remove manufacturer prefix from model if present
        let prefixes = ["amd-", "intel-", "nvidia-", "amd ", "intel ", "nvidia "];
        for prefix in prefixes {
            if cleaned.to_lowercase().starts_with(prefix) {
                cleaned = cleaned[prefix.len()..].to_string();
            }
        }

        // Replace hyphens with spaces and clean up
        cleaned = cleaned.replace('-', " ");
        cleaned = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");

        // Capitalize properly (e.g., "ryzen 9 9900x" -> "Ryzen 9 9900X")
        cleaned
            .split_whitespace()
            .map(|word| {
                let mut c = word.chars();
                match c.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Generate search queries based on device type and identifier.
    fn generate_search_queries(
        &self,
        identifier: &DeviceIdentifier,
        device_type: &DeviceType,
    ) -> Vec<String> {
        // Clean up manufacturer and model for better search results
        let manufacturer = Self::normalize_manufacturer(&identifier.manufacturer);
        let model = Self::clean_model_name(&identifier.model);

        log::debug!("Cleaned search terms - manufacturer: {}, model: {}", manufacturer, model);

        let type_keyword = match device_type {
            DeviceType::Cpu => "CPU processor",
            DeviceType::Gpu => "GPU graphics card",
            DeviceType::Motherboard => "motherboard",
            DeviceType::Memory => "RAM memory",
            DeviceType::Storage => "SSD HDD storage",
        };

        vec![
            format!("{} {} specifications", manufacturer, model),
            format!("{} {} {} specs", manufacturer, model, type_keyword),
            format!("{} {} site:techpowerup.com", manufacturer, model),
            format!("{} {} site:anandtech.com", manufacturer, model),
            format!("{} {} review specifications", manufacturer, model),
        ]
    }

    /// Perform search and extract specifications from results.
    async fn search_and_extract(
        &self,
        query: &str,
        identifier: &DeviceIdentifier,
        device_type: &DeviceType,
    ) -> Result<DeviceDeepInfo> {
        // Get search results
        let search_results = self.web_search(query).await?;

        if search_results.is_empty() {
            return Err(anyhow::anyhow!("No search results found"));
        }

        // Try to extract specs from top results
        let mut all_specs: Vec<ExtractedSpec> = Vec::new();
        let mut source_url: Option<String> = None;
        let mut images: Vec<String> = Vec::new();

        for result in search_results.iter().take(3) {
            match self.extract_specs_from_url(&result.url, identifier, device_type).await {
                Ok((specs, page_images)) => {
                    if !specs.is_empty() && source_url.is_none() {
                        source_url = Some(result.url.clone());
                    }
                    all_specs.extend(specs);
                    images.extend(page_images);
                }
                Err(e) => {
                    log::debug!("Failed to extract from {}: {}", result.url, e);
                }
            }
        }

        if all_specs.is_empty() {
            return Err(anyhow::anyhow!("Could not extract specifications from any source"));
        }

        // Merge and deduplicate specs
        let merged_specs = self.merge_specifications(all_specs);

        // Calculate overall confidence
        let avg_confidence: f32 = if merged_specs.is_empty() {
            0.0
        } else {
            merged_specs.iter().map(|s| s.confidence).sum::<f32>() / merged_specs.len() as f32
        };

        // Build the device info
        let device_id = self.generate_device_id(identifier, device_type);

        Ok(DeviceDeepInfo {
            device_id,
            device_type: device_type.clone(),
            identifier: identifier.clone(),
            specifications: Some(self.build_specifications(&merged_specs, device_type)),
            drivers: self.build_driver_info(identifier, device_type),
            documentation: Some(DocumentationLinks {
                product_page: source_url.clone(),
                support_page: None,
                manuals: vec![],
                datasheets: vec![],
                firmware_updates: vec![],
            }),
            images: if images.is_empty() {
                None
            } else {
                Some(ProductImages {
                    primary_image: images.first().cloned(),
                    primary_image_cached: None,
                    gallery: vec![], // Images converted on-demand
                    thumbnail: None,
                    thumbnail_cached: None,
                    metadata: None,
                })
            },
            metadata: DataMetadata {
                source: DataSource::AiAgent,
                last_updated: Utc::now(),
                expires_at: Utc::now() + Duration::days(3), // Shorter TTL for AI results
                source_url,
                ai_confidence: Some(avg_confidence),
            },
        })
    }

    /// Perform web search using DuckDuckGo HTML.
    async fn web_search(&self, query: &str) -> Result<Vec<SearchResult>> {
        let search_url = format!(
            "https://html.duckduckgo.com/html/?q={}",
            urlencoding::encode(query)
        );

        let response = self.client.get(&search_url).send().await?;
        let html = response.text().await?;

        // Parse in blocking task
        let results = tokio::task::spawn_blocking(move || {
            Self::parse_duckduckgo_results(&html)
        })
        .await
        .context("Spawn blocking failed")??;

        Ok(results)
    }

    /// Parse DuckDuckGo HTML search results.
    fn parse_duckduckgo_results(html: &str) -> Result<Vec<SearchResult>> {
        let document = Html::parse_document(html);
        let result_selector = Selector::parse(".result").unwrap();
        let title_selector = Selector::parse(".result__a").unwrap();
        let snippet_selector = Selector::parse(".result__snippet").unwrap();

        let mut results = Vec::new();

        for result in document.select(&result_selector) {
            let title = result
                .select(&title_selector)
                .next()
                .map(|e| e.text().collect::<String>().trim().to_string());

            let url = result
                .select(&title_selector)
                .next()
                .and_then(|e| e.value().attr("href"))
                .map(|s| s.to_string());

            let snippet = result
                .select(&snippet_selector)
                .next()
                .map(|e| e.text().collect::<String>().trim().to_string())
                .unwrap_or_default();

            if let (Some(title), Some(url)) = (title, url) {
                // Extract actual URL from DuckDuckGo redirect
                let actual_url = Self::extract_ddg_url(&url).unwrap_or(url);
                results.push(SearchResult {
                    title,
                    url: actual_url,
                    snippet,
                });
            }
        }

        Ok(results)
    }

    /// Extract actual URL from DuckDuckGo redirect URL.
    fn extract_ddg_url(ddg_url: &str) -> Option<String> {
        // DuckDuckGo uses format: //duckduckgo.com/l/?uddg=https%3A%2F%2F...
        if ddg_url.contains("uddg=") {
            let parts: Vec<&str> = ddg_url.split("uddg=").collect();
            if parts.len() > 1 {
                let encoded_url = parts[1].split('&').next().unwrap_or("");
                return urlencoding::decode(encoded_url)
                    .map(|s| s.into_owned())
                    .ok();
            }
        }
        // If URL starts with //, prepend https:
        if ddg_url.starts_with("//") {
            return Some(format!("https:{}", ddg_url));
        }
        None
    }

    /// Extract specifications from a URL.
    async fn extract_specs_from_url(
        &self,
        url: &str,
        identifier: &DeviceIdentifier,
        device_type: &DeviceType,
    ) -> Result<(Vec<ExtractedSpec>, Vec<String>)> {
        let response = self.client.get(url).send().await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("HTTP error: {}", response.status()));
        }

        let html = response.text().await?;

        // Parse in blocking task
        let identifier = identifier.clone();
        let device_type = device_type.clone();
        let url = url.to_string();

        tokio::task::spawn_blocking(move || {
            Self::parse_page_for_specs(&html, &identifier, &device_type, &url)
        })
        .await
        .context("Spawn blocking failed")?
    }

    /// Parse a web page for device specifications.
    fn parse_page_for_specs(
        html: &str,
        identifier: &DeviceIdentifier,
        device_type: &DeviceType,
        url: &str,
    ) -> Result<(Vec<ExtractedSpec>, Vec<String>)> {
        let document = Html::parse_document(html);
        let mut specs = Vec::new();
        let mut images = Vec::new();

        // Check if page is relevant to our device
        let page_text = document.root_element().text().collect::<String>().to_lowercase();
        let model_lower = identifier.model.to_lowercase();

        if !page_text.contains(&model_lower) {
            return Ok((vec![], vec![]));
        }

        // Extract specs from tables
        specs.extend(Self::extract_table_specs(&document, device_type));

        // Extract specs from definition lists
        specs.extend(Self::extract_dl_specs(&document, device_type));

        // Extract specs from key-value patterns
        specs.extend(Self::extract_kv_specs(&document, device_type));

        // Extract images
        images.extend(Self::extract_product_images(&document, url));

        Ok((specs, images))
    }

    /// Extract specifications from HTML tables.
    fn extract_table_specs(document: &Html, device_type: &DeviceType) -> Vec<ExtractedSpec> {
        let mut specs = Vec::new();
        let table_selector = Selector::parse("table").unwrap();
        let row_selector = Selector::parse("tr").unwrap();
        let cell_selector = Selector::parse("td, th").unwrap();

        let keywords = Self::get_spec_keywords(device_type);

        for table in document.select(&table_selector) {
            for row in table.select(&row_selector) {
                let cells: Vec<_> = row.select(&cell_selector).collect();

                if cells.len() >= 2 {
                    let label = cells[0].text().collect::<String>().trim().to_string();
                    let value = cells[1].text().collect::<String>().trim().to_string();

                    if !label.is_empty() && !value.is_empty() && label != value {
                        let confidence = Self::calculate_spec_confidence(&label, &value, &keywords);
                        if confidence > 0.1 {
                            specs.push(ExtractedSpec {
                                label,
                                value,
                                confidence,
                            });
                        }
                    }
                }
            }
        }

        specs
    }

    /// Extract specifications from definition lists.
    fn extract_dl_specs(document: &Html, device_type: &DeviceType) -> Vec<ExtractedSpec> {
        let mut specs = Vec::new();
        let dl_selector = Selector::parse("dl").unwrap();
        let dt_selector = Selector::parse("dt").unwrap();
        let dd_selector = Selector::parse("dd").unwrap();

        let keywords = Self::get_spec_keywords(device_type);

        for dl in document.select(&dl_selector) {
            let dts: Vec<_> = dl.select(&dt_selector).collect();
            let dds: Vec<_> = dl.select(&dd_selector).collect();

            for (dt, dd) in dts.into_iter().zip(dds.into_iter()) {
                let label = dt.text().collect::<String>().trim().to_string();
                let value = dd.text().collect::<String>().trim().to_string();

                if !label.is_empty() && !value.is_empty() {
                    let confidence = Self::calculate_spec_confidence(&label, &value, &keywords);
                    if confidence > 0.1 {
                        specs.push(ExtractedSpec {
                            label,
                            value,
                            confidence,
                        });
                    }
                }
            }
        }

        specs
    }

    /// Extract specifications from labeled spans/divs.
    fn extract_kv_specs(document: &Html, device_type: &DeviceType) -> Vec<ExtractedSpec> {
        let mut specs = Vec::new();

        // Look for common spec patterns: label + value in adjacent elements
        let spec_selectors = [
            ".spec-item",
            ".specification",
            "[class*='spec']",
            ".info-row",
            ".detail-row",
        ];

        let keywords = Self::get_spec_keywords(device_type);

        for selector_str in spec_selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                for element in document.select(&selector) {
                    let text = element.text().collect::<String>();
                    // Try to split on common separators
                    for sep in [":", "|", "-", "–"] {
                        if let Some(pos) = text.find(sep) {
                            let label = text[..pos].trim().to_string();
                            let value = text[pos + sep.len()..].trim().to_string();

                            if !label.is_empty() && !value.is_empty() && label.len() < 50 {
                                let confidence =
                                    Self::calculate_spec_confidence(&label, &value, &keywords);
                                if confidence > 0.1 {
                                    specs.push(ExtractedSpec {
                                        label,
                                        value,
                                        confidence,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        specs
    }

    /// Extract product images from the page.
    fn extract_product_images(document: &Html, base_url: &str) -> Vec<String> {
        let mut images = Vec::new();
        let img_selector = Selector::parse("img[src*='product'], img[src*='gpu'], img[src*='cpu'], img.product-image, .gallery img").unwrap();

        for img in document.select(&img_selector) {
            if let Some(src) = img.value().attr("src").or_else(|| img.value().attr("data-src")) {
                let full_url = if src.starts_with("http") {
                    src.to_string()
                } else if src.starts_with("//") {
                    format!("https:{}", src)
                } else if src.starts_with('/') {
                    // Get base domain from URL
                    if let Ok(url) = url::Url::parse(base_url) {
                        format!("{}://{}{}", url.scheme(), url.host_str().unwrap_or(""), src)
                    } else {
                        continue;
                    }
                } else {
                    continue;
                };

                if !images.contains(&full_url) {
                    images.push(full_url);
                }
            }
        }

        images
    }

    /// Get specification keywords for a device type.
    fn get_spec_keywords(device_type: &DeviceType) -> Vec<&'static str> {
        match device_type {
            DeviceType::Cpu => vec![
                "cores", "threads", "frequency", "clock", "ghz", "mhz", "cache",
                "tdp", "watt", "socket", "lithography", "nm", "architecture",
                "launch", "release", "sse", "avx", "turbo", "boost", "base",
            ],
            DeviceType::Gpu => vec![
                "cuda", "cores", "shaders", "vram", "memory", "bandwidth", "bus",
                "clock", "boost", "tdp", "watt", "architecture", "directx", "opengl",
                "vulkan", "ray tracing", "tensor", "rtx", "gddr", "pcie",
            ],
            DeviceType::Motherboard => vec![
                "socket", "chipset", "memory", "slots", "pcie", "sata", "usb",
                "m.2", "nvme", "form factor", "atx", "ddr", "audio", "lan",
            ],
            DeviceType::Memory => vec![
                "ddr", "speed", "mhz", "cas", "latency", "timing", "voltage",
                "capacity", "dimm", "ecc", "registered", "unbuffered",
            ],
            DeviceType::Storage => vec![
                "capacity", "interface", "read", "write", "iops", "tbw",
                "nand", "controller", "cache", "form factor", "nvme", "sata",
            ],
        }
    }

    /// Calculate confidence score for an extracted spec.
    fn calculate_spec_confidence(label: &str, value: &str, keywords: &[&str]) -> f32 {
        let label_lower = label.to_lowercase();
        let value_lower = value.to_lowercase();

        let mut score: f32 = 0.0;

        // Check if label contains relevant keywords
        for keyword in keywords {
            if label_lower.contains(keyword) {
                score += 0.3;
                break;
            }
        }

        // Check if value looks like a specification
        if value.chars().any(|c| c.is_numeric()) {
            score += 0.2;
        }

        // Check for common unit patterns
        let unit_patterns = ["ghz", "mhz", "gb", "mb", "tb", "watt", "nm", "mm", "%"];
        for unit in unit_patterns {
            if value_lower.contains(unit) {
                score += 0.2;
                break;
            }
        }

        // Penalize overly long values (likely not specs)
        if value.len() > 100 {
            score *= 0.5;
        }

        // Penalize if label looks like navigation
        let nav_words = ["home", "menu", "click", "more", "next", "previous"];
        for nav in nav_words {
            if label_lower.contains(nav) {
                score *= 0.1;
                break;
            }
        }

        score.min(1.0)
    }

    /// Merge and deduplicate extracted specifications.
    fn merge_specifications(&self, specs: Vec<ExtractedSpec>) -> Vec<ExtractedSpec> {
        let mut merged: HashMap<String, ExtractedSpec> = HashMap::new();

        for spec in specs {
            let normalized_label = spec.label.to_lowercase().trim().to_string();

            merged
                .entry(normalized_label)
                .and_modify(|existing| {
                    // Keep the one with higher confidence
                    if spec.confidence > existing.confidence {
                        *existing = spec.clone();
                    }
                })
                .or_insert(spec);
        }

        let mut result: Vec<_> = merged.into_values().collect();
        result.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        result
    }

    /// Build DeviceSpecifications from extracted specs.
    fn build_specifications(
        &self,
        specs: &[ExtractedSpec],
        device_type: &DeviceType,
    ) -> DeviceSpecifications {
        let category_name = match device_type {
            DeviceType::Cpu => "CPU Specifications",
            DeviceType::Gpu => "GPU Specifications",
            DeviceType::Motherboard => "Motherboard Specifications",
            DeviceType::Memory => "Memory Specifications",
            DeviceType::Storage => "Storage Specifications",
        };

        let spec_items: Vec<SpecItem> = specs
            .iter()
            .map(|s| SpecItem {
                label: s.label.clone(),
                value: s.value.clone(),
                unit: None,
            })
            .collect();

        let specs_map: HashMap<String, String> = specs
            .iter()
            .map(|s| (s.label.clone(), s.value.clone()))
            .collect();

        DeviceSpecifications {
            specs: specs_map.clone(),
            categories: vec![SpecCategory {
                name: category_name.to_string(),
                specs: spec_items,
            }],
            description: None,
            release_date: specs_map
                .iter()
                .find(|(k, _)| {
                    let kl = k.to_lowercase();
                    kl.contains("launch") || kl.contains("release")
                })
                .map(|(_, v)| v.clone()),
            eol_date: None,
        }
    }

    /// Build driver info based on manufacturer.
    fn build_driver_info(
        &self,
        identifier: &DeviceIdentifier,
        device_type: &DeviceType,
    ) -> Option<DriverInfo> {
        let mfr = identifier.manufacturer.to_lowercase();

        let driver_url = match device_type {
            DeviceType::Gpu => match mfr.as_str() {
                "nvidia" => Some("https://www.nvidia.com/Download/index.aspx"),
                "amd" => Some("https://www.amd.com/en/support/download/drivers.html"),
                "intel" => Some("https://www.intel.com/content/www/us/en/download-center/home.html"),
                _ => None,
            },
            DeviceType::Motherboard => match mfr.as_str() {
                "asus" => Some("https://www.asus.com/support/download-center/"),
                "msi" => Some("https://www.msi.com/support/download/"),
                "gigabyte" => Some("https://www.gigabyte.com/Support/"),
                "asrock" => Some("https://www.asrock.com/support/index.asp"),
                _ => None,
            },
            _ => None,
        };

        driver_url.map(|url| DriverInfo {
            installed_version: None,
            latest_version: None,
            download_url: None,
            release_date: None,
            release_notes_url: None,
            driver_page_url: Some(url.to_string()),
            update_available: false,
        })
    }

    /// Generate a device ID from identifier and type.
    fn generate_device_id(&self, identifier: &DeviceIdentifier, device_type: &DeviceType) -> String {
        let prefix = match device_type {
            DeviceType::Cpu => "cpu",
            DeviceType::Gpu => "gpu",
            DeviceType::Motherboard => "mb",
            DeviceType::Memory => "mem",
            DeviceType::Storage => "stor",
        };

        let mfr = identifier.manufacturer.to_lowercase().replace(' ', "-");
        let model = identifier.model.to_lowercase().replace(' ', "-");

        format!("{}-{}-{}", prefix, mfr, model)
    }
}

impl Default for AiAgent {
    fn default() -> Self {
        Self::new().expect("Failed to create default AiAgent")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spec_confidence_cpu() {
        let keywords = AiAgent::get_spec_keywords(&DeviceType::Cpu);

        // High confidence: keyword match + numeric value + unit
        let score1 = AiAgent::calculate_spec_confidence("Base Clock", "3.6 GHz", &keywords);
        assert!(score1 > 0.5);

        // Medium confidence: numeric but no keyword
        let score2 = AiAgent::calculate_spec_confidence("Price", "$299", &keywords);
        assert!(score2 < score1);

        // Low confidence: navigation element
        let score3 = AiAgent::calculate_spec_confidence("Click here", "for more info", &keywords);
        assert!(score3 < 0.2);
    }

    #[test]
    fn test_generate_device_id() {
        let agent = AiAgent::new().unwrap();
        let identifier = DeviceIdentifier {
            manufacturer: "Intel".to_string(),
            model: "Core i9 14900K".to_string(),
            part_number: None,
            serial_number: None,
            hardware_ids: vec![],
        };

        let device_id = agent.generate_device_id(&identifier, &DeviceType::Cpu);
        assert_eq!(device_id, "cpu-intel-core-i9-14900k");
    }
}
