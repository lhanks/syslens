//! TechPowerUp GPU database source for device information.
//!
//! Fetches GPU specifications and images from TechPowerUp's comprehensive
//! GPU database. This is one of the best sources for GPU specifications.

use crate::models::{DeviceIdentifier, DeviceType, SpecCategory, SpecItem};
use crate::services::device_sources::DeviceSource;
use crate::services::PartialDeviceInfo;
use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use scraper::{Html, Selector};
use std::collections::HashMap;

const TECHPOWERUP_GPU_SPECS: &str = "https://www.techpowerup.com/gpu-specs/";
const TECHPOWERUP_BASE: &str = "https://www.techpowerup.com";

/// TechPowerUp-based device information source for GPUs.
pub struct TechPowerUpSource {
    client: Client,
}

impl TechPowerUpSource {
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self { client })
    }

    /// Normalize GPU model name for search.
    fn normalize_model(model: &str) -> String {
        model
            .to_lowercase()
            .replace("nvidia ", "")
            .replace("amd ", "")
            .replace("geforce ", "")
            .replace("radeon ", "")
            .trim()
            .to_string()
    }

    /// Build search URL for TechPowerUp.
    fn build_search_url(model: &str) -> String {
        let normalized = Self::normalize_model(model);
        let encoded = urlencoding::encode(&normalized);
        format!("{}?q={}", TECHPOWERUP_GPU_SPECS, encoded)
    }

    /// Search for GPU and get the product page URL.
    async fn search_gpu(&self, model: &str) -> Result<Option<String>> {
        let search_url = Self::build_search_url(model);
        log::debug!("TechPowerUp search URL: {}", search_url);

        let response = self
            .client
            .get(&search_url)
            .send()
            .await
            .context("Failed to search TechPowerUp")?;

        let html = response.text().await?;
        let document = Html::parse_document(&html);

        // Look for GPU list table
        let table_selector = Selector::parse("table.processors tbody tr").unwrap();
        let link_selector = Selector::parse("td.name a").unwrap();

        let normalized_model = Self::normalize_model(model).to_lowercase();

        for row in document.select(&table_selector) {
            if let Some(link) = row.select(&link_selector).next() {
                let name = link.text().collect::<String>().to_lowercase();

                // Check for exact or close match
                if name.contains(&normalized_model) || self.is_close_match(&name, &normalized_model)
                {
                    if let Some(href) = link.value().attr("href") {
                        let full_url = if href.starts_with('/') {
                            format!("{}{}", TECHPOWERUP_BASE, href)
                        } else {
                            href.to_string()
                        };
                        log::debug!("Found TechPowerUp match: {} -> {}", name, full_url);
                        return Ok(Some(full_url));
                    }
                }
            }
        }

        Ok(None)
    }

    /// Check if two model names are a close match.
    fn is_close_match(&self, name: &str, model: &str) -> bool {
        // Extract model numbers and compare
        let name_numbers: Vec<&str> = name
            .split(|c: char| !c.is_alphanumeric())
            .filter(|s| !s.is_empty())
            .collect();
        let model_numbers: Vec<&str> = model
            .split(|c: char| !c.is_alphanumeric())
            .filter(|s| !s.is_empty())
            .collect();

        // Check if all model parts are in the name
        model_numbers.iter().all(|part| {
            name_numbers
                .iter()
                .any(|n| n.contains(part) || part.contains(n))
        })
    }

    /// Fetch and parse GPU product page.
    async fn fetch_product_page(&self, url: &str) -> Result<GpuPageData> {
        let response = self
            .client
            .get(url)
            .send()
            .await
            .context("Failed to fetch TechPowerUp product page")?;

        let html = response.text().await?;
        let document = Html::parse_document(&html);

        let mut data = GpuPageData {
            page_url: url.to_string(),
            ..Default::default()
        };

        // Extract product image
        let img_selector = Selector::parse("div.gpudb-large-image img").unwrap();
        if let Some(img) = document.select(&img_selector).next() {
            if let Some(src) = img.value().attr("src") {
                data.image_url = Some(if src.starts_with('/') {
                    format!("{}{}", TECHPOWERUP_BASE, src)
                } else if src.starts_with("//") {
                    format!("https:{}", src)
                } else {
                    src.to_string()
                });
            }
        }

        // Try alternative image selector
        if data.image_url.is_none() {
            let alt_img_selector =
                Selector::parse("div.gpu-image img, .gpudb-card-large img").unwrap();
            if let Some(img) = document.select(&alt_img_selector).next() {
                if let Some(src) = img.value().attr("src") {
                    data.image_url = Some(if src.starts_with('/') {
                        format!("{}{}", TECHPOWERUP_BASE, src)
                    } else if src.starts_with("//") {
                        format!("https:{}", src)
                    } else {
                        src.to_string()
                    });
                }
            }
        }

        // Extract specs from the specs table
        let spec_selector = Selector::parse("section.details dl.clearfix").unwrap();
        let dt_selector = Selector::parse("dt").unwrap();
        let dd_selector = Selector::parse("dd").unwrap();

        for section in document.select(&spec_selector) {
            let dts: Vec<_> = section.select(&dt_selector).collect();
            let dds: Vec<_> = section.select(&dd_selector).collect();

            for (dt, dd) in dts.iter().zip(dds.iter()) {
                let key = dt.text().collect::<String>().trim().to_string();
                let value = dd.text().collect::<String>().trim().to_string();

                if !key.is_empty() && !value.is_empty() {
                    data.specs.insert(key, value);
                }
            }
        }

        // Try alternative spec table format
        let alt_spec_selector = Selector::parse("table.specs tr").unwrap();
        let th_selector = Selector::parse("th").unwrap();
        let td_selector = Selector::parse("td").unwrap();

        for row in document.select(&alt_spec_selector) {
            if let Some(th) = row.select(&th_selector).next() {
                if let Some(td) = row.select(&td_selector).next() {
                    let key = th.text().collect::<String>().trim().to_string();
                    let value = td.text().collect::<String>().trim().to_string();

                    if !key.is_empty() && !value.is_empty() {
                        data.specs.insert(key, value);
                    }
                }
            }
        }

        // Extract GPU name/title
        let title_selector = Selector::parse("h1.gpudb-name, h1.title").unwrap();
        if let Some(title) = document.select(&title_selector).next() {
            data.name = Some(title.text().collect::<String>().trim().to_string());
        }

        // Extract release date
        if let Some(date) = data
            .specs
            .get("Release Date")
            .or(data.specs.get("Launch Date"))
        {
            data.release_date = Some(date.clone());
        }

        Ok(data)
    }

    /// Convert raw specs to categorized specs.
    fn categorize_specs(
        raw_specs: &HashMap<String, String>,
    ) -> (HashMap<String, String>, Vec<SpecCategory>) {
        let mut specs = HashMap::new();
        let mut gpu_engine = Vec::new();
        let mut memory = Vec::new();
        let mut board_design = Vec::new();
        let mut graphics_features = Vec::new();

        let spec_mappings = [
            // GPU Engine specs
            ("GPU Name", "gpu_name", "GPU Engine"),
            ("GPU Variant", "gpu_variant", "GPU Engine"),
            ("Architecture", "architecture", "GPU Engine"),
            ("Foundry", "foundry", "GPU Engine"),
            ("Process Size", "process", "GPU Engine"),
            ("Transistors", "transistors", "GPU Engine"),
            ("Die Size", "die_size", "GPU Engine"),
            ("Shaders", "shaders", "GPU Engine"),
            ("TMUs", "tmus", "GPU Engine"),
            ("ROPs", "rops", "GPU Engine"),
            ("SM Count", "sm_count", "GPU Engine"),
            ("Tensor Cores", "tensor_cores", "GPU Engine"),
            ("RT Cores", "rt_cores", "GPU Engine"),
            ("Base Clock", "base_clock", "GPU Engine"),
            ("Boost Clock", "boost_clock", "GPU Engine"),
            ("GPU Clock", "gpu_clock", "GPU Engine"),
            // Memory specs
            ("Memory Size", "memory_size", "Memory"),
            ("Memory Type", "memory_type", "Memory"),
            ("Memory Bus", "memory_bus", "Memory"),
            ("Bandwidth", "bandwidth", "Memory"),
            ("Memory Clock", "memory_clock", "Memory"),
            ("Effective Memory Clock", "effective_memory_clock", "Memory"),
            // Board Design specs
            ("TDP", "tdp", "Board Design"),
            ("Slot Width", "slot_width", "Board Design"),
            ("Length", "length", "Board Design"),
            ("Width", "width", "Board Design"),
            ("Height", "height", "Board Design"),
            ("Power Connectors", "power_connectors", "Board Design"),
            ("Outputs", "outputs", "Board Design"),
            ("Bus Interface", "bus_interface", "Board Design"),
            // Graphics Features
            ("DirectX", "directx", "Graphics Features"),
            ("OpenGL", "opengl", "Graphics Features"),
            ("OpenCL", "opencl", "Graphics Features"),
            ("Vulkan", "vulkan", "Graphics Features"),
            ("CUDA", "cuda", "Graphics Features"),
            ("Shader Model", "shader_model", "Graphics Features"),
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
                    "GPU Engine" => gpu_engine.push(item),
                    "Memory" => memory.push(item),
                    "Board Design" => board_design.push(item),
                    "Graphics Features" => graphics_features.push(item),
                    _ => {}
                }
            }
        }

        let mut categories = Vec::new();
        if !gpu_engine.is_empty() {
            categories.push(SpecCategory {
                name: "GPU Engine".to_string(),
                specs: gpu_engine,
            });
        }
        if !memory.is_empty() {
            categories.push(SpecCategory {
                name: "Memory".to_string(),
                specs: memory,
            });
        }
        if !board_design.is_empty() {
            categories.push(SpecCategory {
                name: "Board Design".to_string(),
                specs: board_design,
            });
        }
        if !graphics_features.is_empty() {
            categories.push(SpecCategory {
                name: "Graphics Features".to_string(),
                specs: graphics_features,
            });
        }

        (specs, categories)
    }

    /// Extract unit from a value string.
    fn extract_unit(value: &str) -> Option<String> {
        let units = [
            "MHz", "GHz", "GB", "MB", "W", "nm", "bit", "GB/s", "mm²", "mm", "billion",
        ];
        for unit in units {
            if value.contains(unit) {
                return Some(unit.to_string());
            }
        }
        None
    }
}

/// Data extracted from a TechPowerUp GPU page.
#[derive(Debug, Default)]
struct GpuPageData {
    name: Option<String>,
    image_url: Option<String>,
    page_url: String,
    specs: HashMap<String, String>,
    release_date: Option<String>,
}

#[async_trait]
impl DeviceSource for TechPowerUpSource {
    fn name(&self) -> &str {
        "TechPowerUp"
    }

    fn priority(&self) -> u8 {
        10 // High priority for GPUs
    }

    fn supports(&self, device_type: &DeviceType, _identifier: &DeviceIdentifier) -> bool {
        // TechPowerUp is primarily for GPUs
        matches!(device_type, DeviceType::Gpu)
    }

    async fn fetch(
        &self,
        _device_type: &DeviceType,
        identifier: &DeviceIdentifier,
    ) -> Result<PartialDeviceInfo> {
        // Search for the GPU
        let product_url = self
            .search_gpu(&identifier.model)
            .await?
            .context("GPU not found on TechPowerUp")?;

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
            support_page: None,
            image_url: page_data.image_url.clone(),
            source_name: "TechPowerUp".to_string(),
            source_url: Some(page_data.page_url),
            confidence: 0.9, // High confidence - structured database
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
        assert_eq!(
            TechPowerUpSource::normalize_model("NVIDIA GeForce RTX 4090"),
            "rtx 4090"
        );
        assert_eq!(
            TechPowerUpSource::normalize_model("AMD Radeon RX 7900 XTX"),
            "rx 7900 xtx"
        );
        assert_eq!(TechPowerUpSource::normalize_model("RTX 5070"), "rtx 5070");
    }

    #[test]
    fn test_build_search_url() {
        let url = TechPowerUpSource::build_search_url("GeForce RTX 4090");
        assert!(url.contains("techpowerup.com/gpu-specs"));
        assert!(url.contains("rtx%204090"));
    }
}
