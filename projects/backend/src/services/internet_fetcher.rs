//! Internet fetcher for device information from manufacturer websites.

use crate::models::{
    DataMetadata, DataSource, DeviceDeepInfo, DeviceIdentifier, DeviceSpecifications, DeviceType,
    DocumentationLinks, DriverInfo, ProductImages, SpecCategory, SpecItem,
};
use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use reqwest::Client;
use scraper::{Html, Selector};
use std::collections::HashMap;

/// HTTP client timeout in seconds
const REQUEST_TIMEOUT: u64 = 30;

/// User agent for web requests
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

/// Fetches device information from manufacturer websites.
pub struct InternetFetcher {
    client: Client,
}

impl InternetFetcher {
    /// Create a new InternetFetcher with configured HTTP client.
    pub fn new() -> Result<Self> {
        use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, ACCEPT_LANGUAGE, CACHE_CONTROL};

        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8"));
        headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("en-US,en;q=0.9"));
        headers.insert(CACHE_CONTROL, HeaderValue::from_static("no-cache"));
        headers.insert("Sec-Fetch-Dest", HeaderValue::from_static("document"));
        headers.insert("Sec-Fetch-Mode", HeaderValue::from_static("navigate"));
        headers.insert("Sec-Fetch-Site", HeaderValue::from_static("none"));
        headers.insert("Sec-Fetch-User", HeaderValue::from_static("?1"));
        headers.insert("Upgrade-Insecure-Requests", HeaderValue::from_static("1"));

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(REQUEST_TIMEOUT))
            .user_agent(USER_AGENT)
            .default_headers(headers)
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self { client })
    }

    /// Normalize manufacturer name to a standard form.
    /// Handles variations like "Advanced Micro Devices, Inc." -> "amd"
    fn normalize_manufacturer(raw_manufacturer: &str, model: &str) -> String {
        let lower = raw_manufacturer.to_lowercase();
        let model_lower = model.to_lowercase();

        // Check for AMD variations (including CPU vendor ID "AuthenticAMD")
        if lower.contains("advanced") && lower.contains("micro") {
            return "amd".to_string();
        }
        if lower == "amd" || lower.starts_with("amd ") || lower == "authenticamd" {
            return "amd".to_string();
        }

        // Check for NVIDIA variations
        if lower.contains("nvidia") {
            return "nvidia".to_string();
        }

        // Check for Intel variations (including CPU vendor ID "GenuineIntel")
        if lower.contains("intel") || lower == "genuineintel" {
            return "intel".to_string();
        }

        // Check model name for hints
        if model_lower.contains("radeon") || model_lower.contains("ryzen") {
            return "amd".to_string();
        }
        if model_lower.contains("geforce")
            || model_lower.contains("rtx")
            || model_lower.contains("gtx")
        {
            return "nvidia".to_string();
        }
        if model_lower.contains("core i")
            || model_lower.contains("xeon")
            || model_lower.contains("celeron")
            || model_lower.contains("pentium")
        {
            return "intel".to_string();
        }

        // Return cleaned lowercase version
        lower.replace([',', '.'], "").trim().to_string()
    }

    /// Fetch device info from the appropriate manufacturer website.
    pub async fn fetch_device_info(
        &self,
        identifier: &DeviceIdentifier,
        device_type: &DeviceType,
    ) -> Result<DeviceDeepInfo> {
        let manufacturer =
            Self::normalize_manufacturer(&identifier.manufacturer, &identifier.model);
        log::debug!(
            "Normalized manufacturer: {} -> {}",
            identifier.manufacturer,
            manufacturer
        );

        match device_type {
            DeviceType::Cpu => match manufacturer.as_str() {
                "intel" => self.fetch_intel_cpu(&identifier.model).await,
                "amd" => self.fetch_amd_cpu(&identifier.model).await,
                _ => Err(anyhow::anyhow!(
                    "Unsupported CPU manufacturer: {}",
                    manufacturer
                )),
            },
            DeviceType::Gpu => match manufacturer.as_str() {
                "nvidia" => self.fetch_nvidia_gpu(&identifier.model).await,
                "amd" => self.fetch_amd_gpu(&identifier.model).await,
                _ => Err(anyhow::anyhow!(
                    "Unsupported GPU manufacturer: {}",
                    manufacturer
                )),
            },
            DeviceType::Motherboard => {
                self.fetch_motherboard_info(&identifier.manufacturer, &identifier.model)
                    .await
            }
            _ => Err(anyhow::anyhow!(
                "Internet fetch not yet implemented for {:?}",
                device_type
            )),
        }
    }

    /// Fetch Intel CPU information from Intel ARK.
    async fn fetch_intel_cpu(&self, model: &str) -> Result<DeviceDeepInfo> {
        log::info!("Fetching Intel CPU info for: {}", model);
        let model = model.to_string();

        // Search Intel ARK for the CPU
        let search_url = format!(
            "https://ark.intel.com/content/www/us/en/ark/search.html?_charset_=UTF-8&q={}",
            urlencoding::encode(&model)
        );

        let response = self.client.get(&search_url).send().await?;
        let html = response.text().await?;

        // Parse search results in blocking task (Html is not Send)
        let model_clone = model.clone();
        let product_url = tokio::task::spawn_blocking(move || {
            let document = Html::parse_document(&html);
            Self::extract_intel_product_url_static(&document, &model_clone)
        })
        .await
        .context("Spawn blocking failed")??;

        // Fetch the product page
        let product_response = self.client.get(&product_url).send().await?;
        let product_html = product_response.text().await?;

        // Parse product page in blocking task
        let model_final = model.clone();
        let url_final = product_url.clone();
        tokio::task::spawn_blocking(move || {
            let document = Html::parse_document(&product_html);
            Self::parse_intel_product_page_static(&document, &model_final, &url_final)
        })
        .await
        .context("Spawn blocking failed")?
    }

    /// Extract the product URL from Intel ARK search results (static version for spawn_blocking).
    fn extract_intel_product_url_static(document: &Html, model: &str) -> Result<String> {
        // Look for product links in search results
        let link_selector = Selector::parse("a.result-title, a[href*='/ark/products/']").unwrap();

        for element in document.select(&link_selector) {
            if let Some(href) = element.value().attr("href") {
                let text = element.text().collect::<String>().to_lowercase();
                let model_lower = model.to_lowercase();

                // Check if this link matches our model
                if text.contains(&model_lower)
                    || href.to_lowercase().contains(&model_lower.replace(" ", "-"))
                {
                    let full_url = if href.starts_with("http") {
                        href.to_string()
                    } else {
                        format!("https://ark.intel.com{}", href)
                    };
                    return Ok(full_url);
                }
            }
        }

        Err(anyhow::anyhow!(
            "Could not find Intel product page for: {}",
            model
        ))
    }

    /// Parse Intel product page for specifications (static version for spawn_blocking).
    fn parse_intel_product_page_static(
        document: &Html,
        model: &str,
        source_url: &str,
    ) -> Result<DeviceDeepInfo> {
        let mut specs = HashMap::new();
        let mut categories = Vec::new();

        // Parse specification sections
        let section_selector = Selector::parse(".specs-section, .blade-inside").unwrap();
        let row_selector = Selector::parse(".specs-row, tr").unwrap();
        let label_selector = Selector::parse(".specs-label, .label, th").unwrap();
        let value_selector = Selector::parse(".specs-value, .value, td").unwrap();

        for section in document.select(&section_selector) {
            let mut section_specs = Vec::new();

            for row in section.select(&row_selector) {
                let label = row
                    .select(&label_selector)
                    .next()
                    .map(|e| e.text().collect::<String>().trim().to_string());
                let value = row
                    .select(&value_selector)
                    .next()
                    .map(|e| e.text().collect::<String>().trim().to_string());

                if let (Some(label), Some(value)) = (label, value) {
                    if !label.is_empty() && !value.is_empty() {
                        specs.insert(label.clone(), value.clone());
                        section_specs.push(SpecItem {
                            label,
                            value,
                            unit: None,
                        });
                    }
                }
            }

            if !section_specs.is_empty() {
                // Try to get section title
                let title_selector = Selector::parse("h2, .section-title").unwrap();
                let section_name = section
                    .select(&title_selector)
                    .next()
                    .map(|e| e.text().collect::<String>().trim().to_string())
                    .unwrap_or_else(|| "Specifications".to_string());

                categories.push(SpecCategory {
                    name: section_name,
                    specs: section_specs,
                });
            }
        }

        // Extract product image
        let image_selector = Selector::parse(".product-image img, .blade-media img").unwrap();
        let primary_image = document
            .select(&image_selector)
            .next()
            .and_then(|e| e.value().attr("src"))
            .map(|s| {
                if s.starts_with("http") {
                    s.to_string()
                } else {
                    format!("https://ark.intel.com{}", s)
                }
            });

        let device_id = format!("cpu-intel-{}", model.to_lowercase().replace(" ", "-"));

        Ok(DeviceDeepInfo {
            device_id,
            device_type: DeviceType::Cpu,
            identifier: DeviceIdentifier {
                manufacturer: "Intel".to_string(),
                model: model.to_string(),
                part_number: specs.get("Processor Number").cloned(),
                serial_number: None,
                hardware_ids: vec![],
            },
            specifications: Some(DeviceSpecifications {
                specs: specs.clone(),
                categories,
                description: None,
                release_date: specs.get("Launch Date").map(|s| s.to_string()),
                eol_date: None,
            }),
            drivers: None,
            documentation: Some(DocumentationLinks {
                product_page: Some(source_url.to_string()),
                support_page: None,
                manuals: vec![],
                datasheets: vec![],
                firmware_updates: vec![],
            }),
            images: Some(ProductImages {
                primary_image,
                primary_image_cached: None,
                gallery: vec![],
                thumbnail: None,
                thumbnail_cached: None,
                metadata: None,
            }),
            metadata: DataMetadata {
                source: DataSource::ManufacturerWebsite,
                last_updated: Utc::now(),
                expires_at: Utc::now() + Duration::days(7),
                source_url: Some(source_url.to_string()),
                ai_confidence: None,
            },
        })
    }

    /// Clean up CPU model name by removing common suffixes and prefixes.
    /// "amd-ryzen-9-9900x-12-core-processor-" -> "ryzen-9-9900x"
    fn clean_cpu_model(model: &str) -> String {
        let mut cleaned = model.to_lowercase();

        // Remove manufacturer prefixes
        cleaned = cleaned.trim_start_matches("amd-").to_string();
        cleaned = cleaned.trim_start_matches("intel-").to_string();

        // Remove common suffixes like "-12-core-processor", "-with-radeon-graphics", etc.
        if let Some(idx) = cleaned.find("-core-processor") {
            // Find where the core count starts (e.g., "-12-core-processor")
            if let Some(count_idx) = cleaned[..idx].rfind('-') {
                // Check if it's a number before -core-processor
                let potential_count = &cleaned[count_idx + 1..idx];
                if potential_count.parse::<u32>().is_ok() {
                    cleaned = cleaned[..count_idx].to_string();
                }
            }
        }

        // Remove other common suffixes
        for suffix in &["-with-radeon-graphics", "-processor", "-cpu"] {
            if cleaned.ends_with(suffix) {
                cleaned = cleaned[..cleaned.len() - suffix.len()].to_string();
            }
        }

        // Remove trailing hyphens
        cleaned = cleaned.trim_end_matches('-').to_string();

        log::debug!("Cleaned CPU model: {} -> {}", model, cleaned);
        cleaned
    }

    /// Detect AMD Ryzen series from model name (e.g., "ryzen-9-9900x" -> "9000")
    fn detect_amd_series(model: &str) -> &'static str {
        let model_lower = model.to_lowercase();

        // Extract the first digit after "ryzen-X-" to determine series
        // e.g., "ryzen-9-9900x" -> 9900 -> 9000 series
        // "ryzen-7-7800x3d" -> 7800 -> 7000 series
        if let Some(idx) = model_lower.find("ryzen-") {
            let after_ryzen = &model_lower[idx + 6..]; // Skip "ryzen-"
                                                       // Skip the tier (5, 7, 9) and the dash
            if let Some(dash_idx) = after_ryzen.find('-') {
                let model_number = &after_ryzen[dash_idx + 1..];
                // Get first digit of model number
                if let Some(first_char) = model_number.chars().next() {
                    match first_char {
                        '9' => return "9000",
                        '8' => return "8000",
                        '7' => return "7000",
                        '5' => return "5000",
                        '3' => return "3000",
                        _ => {}
                    }
                }
            }
        }
        "7000" // Default fallback
    }

    /// Fetch AMD CPU information.
    async fn fetch_amd_cpu(&self, model: &str) -> Result<DeviceDeepInfo> {
        log::info!("Fetching AMD CPU info for: {}", model);

        // Clean the model name
        let cleaned_model = Self::clean_cpu_model(model);
        log::info!("Cleaned AMD CPU model: {}", cleaned_model);

        // Detect the series from the model name
        let series = Self::detect_amd_series(&cleaned_model);
        log::info!("Detected AMD series: {}", series);

        // AMD product page URL pattern
        let model_slug = cleaned_model.replace(" ", "-");

        let product_url = format!(
            "https://www.amd.com/en/products/processors/desktops/ryzen/{}-series/amd-{}.html",
            series, model_slug
        );
        log::info!("Trying AMD product URL: {}", product_url);

        let response = self.client.get(&product_url).send().await;

        match response {
            Ok(resp) => {
                let status = resp.status();
                if status.is_success() {
                    log::info!("AMD product page returned success");
                    let html = resp.text().await?;
                    let model_final = cleaned_model.clone();
                    let url_final = product_url.clone();
                    tokio::task::spawn_blocking(move || {
                        let document = Html::parse_document(&html);
                        Self::parse_amd_product_page_static(&document, &model_final, &url_final)
                    })
                    .await
                    .context("Spawn blocking failed")?
                } else {
                    log::warn!(
                        "AMD product page returned status {}, falling back to TechPowerUp",
                        status
                    );
                    self.fetch_from_techpowerup(&cleaned_model, &DeviceType::Cpu)
                        .await
                }
            }
            Err(e) => {
                log::warn!("AMD request failed: {}, falling back to TechPowerUp", e);
                // Fallback to TechPowerUp with cleaned model name
                self.fetch_from_techpowerup(&cleaned_model, &DeviceType::Cpu)
                    .await
            }
        }
    }

    /// Parse AMD product page for specifications (static version for spawn_blocking).
    fn parse_amd_product_page_static(
        document: &Html,
        model: &str,
        source_url: &str,
    ) -> Result<DeviceDeepInfo> {
        let mut specs = HashMap::new();
        let mut categories = Vec::new();

        // Parse specification tables
        let table_selector = Selector::parse(".specs-table, table").unwrap();
        let row_selector = Selector::parse("tr").unwrap();
        let cell_selector = Selector::parse("td, th").unwrap();

        for table in document.select(&table_selector) {
            let mut section_specs = Vec::new();

            for row in table.select(&row_selector) {
                let cells: Vec<_> = row.select(&cell_selector).collect();
                if cells.len() >= 2 {
                    let label = cells[0].text().collect::<String>().trim().to_string();
                    let value = cells[1].text().collect::<String>().trim().to_string();

                    if !label.is_empty() && !value.is_empty() {
                        specs.insert(label.clone(), value.clone());
                        section_specs.push(SpecItem {
                            label,
                            value,
                            unit: None,
                        });
                    }
                }
            }

            if !section_specs.is_empty() {
                categories.push(SpecCategory {
                    name: "Specifications".to_string(),
                    specs: section_specs,
                });
            }
        }

        let device_id = format!("cpu-amd-{}", model.to_lowercase().replace(" ", "-"));

        // Extract product image - first try og:image meta tag (most reliable)
        let mut image_url: Option<String> = None;
        let og_image_selector = Selector::parse("meta[property='og:image']").unwrap();
        if let Some(og_img) = document.select(&og_image_selector).next() {
            if let Some(content) = og_img.value().attr("content") {
                if !content.is_empty() {
                    image_url = Some(if content.starts_with("//") {
                        format!("https:{}", content)
                    } else if content.starts_with('/') {
                        format!("https://www.amd.com{}", content)
                    } else {
                        content.to_string()
                    });
                }
            }
        }

        // Fall back to various image selectors if og:image not found
        if image_url.is_none() {
            let img_selectors = [
                "img.cmp-image__image",
                "div.cmp-image img",
                ".cmp-imagethumbnailcarousel__mainimage img",
                "img.product-image",
                "div.product-hero img",
                ".product-media img",
                "img[alt*='AMD']",
                "img[alt*='Ryzen']",
            ];

            for selector_str in img_selectors {
                if let Ok(selector) = Selector::parse(selector_str) {
                    if let Some(img) = document.select(&selector).next() {
                        let src = img
                            .value()
                            .attr("src")
                            .or_else(|| img.value().attr("srcset"))
                            .or_else(|| img.value().attr("data-src"));

                        if let Some(src) = src {
                            // Get the first URL from srcset if present
                            let src = src
                                .split(',')
                                .next()
                                .unwrap_or(src)
                                .split(' ')
                                .next()
                                .unwrap_or(src);
                            image_url = Some(if src.starts_with("//") {
                                format!("https:{}", src)
                            } else if src.starts_with('/') {
                                format!("https://www.amd.com{}", src)
                            } else {
                                src.to_string()
                            });
                            break;
                        }
                    }
                }
            }
        }

        // Build ProductImages if we found an image
        let images = image_url.map(|url| ProductImages {
            primary_image: Some(url),
            ..Default::default()
        });

        Ok(DeviceDeepInfo {
            device_id,
            device_type: DeviceType::Cpu,
            identifier: DeviceIdentifier {
                manufacturer: "AMD".to_string(),
                model: model.to_string(),
                part_number: None,
                serial_number: None,
                hardware_ids: vec![],
            },
            specifications: Some(DeviceSpecifications {
                specs,
                categories,
                description: None,
                release_date: None,
                eol_date: None,
            }),
            drivers: Some(DriverInfo {
                installed_version: None,
                latest_version: None,
                download_url: None,
                release_date: None,
                release_notes_url: None,
                driver_page_url: Some(
                    "https://www.amd.com/en/support/downloads/drivers.html".to_string(),
                ),
                update_available: false,
            }),
            documentation: Some(DocumentationLinks {
                product_page: Some(source_url.to_string()),
                support_page: Some("https://www.amd.com/en/support".to_string()),
                manuals: vec![],
                datasheets: vec![],
                firmware_updates: vec![],
            }),
            images,
            metadata: DataMetadata {
                source: DataSource::ManufacturerWebsite,
                last_updated: Utc::now(),
                expires_at: Utc::now() + Duration::days(7),
                source_url: Some(source_url.to_string()),
                ai_confidence: None,
            },
        })
    }

    /// Clean up GPU model name by removing common prefixes.
    /// "nvidia-geforce-rtx-5070" -> "rtx-5070"
    fn clean_gpu_model(model: &str) -> String {
        let mut cleaned = model.to_lowercase();

        // Remove manufacturer prefixes
        cleaned = cleaned.trim_start_matches("nvidia-").to_string();
        cleaned = cleaned.trim_start_matches("amd-").to_string();

        // Remove "geforce-" prefix
        cleaned = cleaned.trim_start_matches("geforce-").to_string();

        // Also handle space-separated versions
        cleaned = cleaned.replace("nvidia ", "");
        cleaned = cleaned.replace("geforce ", "");

        // Remove trailing hyphens
        cleaned = cleaned.trim_end_matches('-').to_string();

        log::debug!("Cleaned GPU model: {} -> {}", model, cleaned);
        cleaned
    }

    /// Detect NVIDIA GPU series from model name (e.g., "rtx-5070" -> "50")
    fn detect_nvidia_series(model: &str) -> &'static str {
        let model_lower = model.to_lowercase();

        // Check for RTX series - the first digit after "rtx-" indicates the series
        if model_lower.contains("rtx") {
            // Extract the model number (e.g., "5070", "4090", "3080")
            if let Some(idx) = model_lower.find("rtx") {
                let after_rtx = &model_lower[idx + 3..];
                // Skip any separator (space or dash)
                let trimmed = after_rtx.trim_start_matches(['-', ' ']);
                if let Some(first_char) = trimmed.chars().next() {
                    match first_char {
                        '5' => return "50-series",
                        '4' => return "40-series",
                        '3' => return "30-series",
                        '2' => return "20-series",
                        _ => {}
                    }
                }
            }
        }

        // Check for GTX series
        if model_lower.contains("gtx") {
            if model_lower.contains("1660")
                || model_lower.contains("1650")
                || model_lower.contains("1630")
            {
                return "16-series";
            }
            if model_lower.contains("1080")
                || model_lower.contains("1070")
                || model_lower.contains("1060")
                || model_lower.contains("1050")
            {
                return "10-series";
            }
        }

        "40-series" // Default fallback
    }

    /// Fetch NVIDIA GPU information.
    async fn fetch_nvidia_gpu(&self, model: &str) -> Result<DeviceDeepInfo> {
        log::info!("Fetching NVIDIA GPU info for: {}", model);

        // Clean the model name
        let cleaned_model = Self::clean_gpu_model(model);
        log::info!("Cleaned NVIDIA GPU model: {}", cleaned_model);

        // Detect the series from the model name
        let series = Self::detect_nvidia_series(&cleaned_model);
        log::info!("Detected NVIDIA series: {}", series);

        // NVIDIA product page URL pattern
        let model_slug = cleaned_model.replace(" ", "-");

        let product_url = format!(
            "https://www.nvidia.com/en-us/geforce/graphics-cards/{}/{}/",
            series, model_slug
        );
        log::info!("Trying NVIDIA product URL: {}", product_url);

        let response = self.client.get(&product_url).send().await;

        match response {
            Ok(resp) => {
                let status = resp.status();
                if status.is_success() {
                    log::info!("NVIDIA product page returned success");
                    let html = resp.text().await?;
                    let model_final = cleaned_model.clone();
                    let url_final = product_url.clone();
                    tokio::task::spawn_blocking(move || {
                        let document = Html::parse_document(&html);
                        Self::parse_nvidia_product_page_static(&document, &model_final, &url_final)
                    })
                    .await
                    .context("Spawn blocking failed")?
                } else {
                    log::warn!(
                        "NVIDIA product page returned status {}, falling back to TechPowerUp",
                        status
                    );
                    // Fallback to TechPowerUp with cleaned model name
                    self.fetch_from_techpowerup(&cleaned_model, &DeviceType::Gpu)
                        .await
                }
            }
            Err(e) => {
                log::warn!("NVIDIA request failed: {}, falling back to TechPowerUp", e);
                // Fallback to TechPowerUp with cleaned model name
                self.fetch_from_techpowerup(&cleaned_model, &DeviceType::Gpu)
                    .await
            }
        }
    }

    /// Parse NVIDIA product page for specifications (static version for spawn_blocking).
    fn parse_nvidia_product_page_static(
        document: &Html,
        model: &str,
        source_url: &str,
    ) -> Result<DeviceDeepInfo> {
        let mut specs = HashMap::new();

        // NVIDIA pages often have specs in structured data or tables
        let spec_selector = Selector::parse(".specs-table tr, .spec-row").unwrap();
        let label_selector = Selector::parse(".spec-label, td:first-child").unwrap();
        let value_selector = Selector::parse(".spec-value, td:last-child").unwrap();

        for row in document.select(&spec_selector) {
            let label = row
                .select(&label_selector)
                .next()
                .map(|e| e.text().collect::<String>().trim().to_string());
            let value = row
                .select(&value_selector)
                .next()
                .map(|e| e.text().collect::<String>().trim().to_string());

            if let (Some(label), Some(value)) = (label, value) {
                if !label.is_empty() && !value.is_empty() && label != value {
                    specs.insert(label, value);
                }
            }
        }

        let device_id = format!("gpu-nvidia-{}", model.to_lowercase().replace(" ", "-"));

        Ok(DeviceDeepInfo {
            device_id,
            device_type: DeviceType::Gpu,
            identifier: DeviceIdentifier {
                manufacturer: "NVIDIA".to_string(),
                model: model.to_string(),
                part_number: None,
                serial_number: None,
                hardware_ids: vec![],
            },
            specifications: Some(DeviceSpecifications {
                specs: specs.clone(),
                categories: vec![SpecCategory {
                    name: "GPU Specifications".to_string(),
                    specs: specs
                        .iter()
                        .map(|(k, v)| SpecItem {
                            label: k.clone(),
                            value: v.clone(),
                            unit: None,
                        })
                        .collect(),
                }],
                description: None,
                release_date: None,
                eol_date: None,
            }),
            drivers: Some(DriverInfo {
                installed_version: None,
                latest_version: None,
                download_url: None,
                release_date: None,
                release_notes_url: None,
                driver_page_url: Some("https://www.nvidia.com/Download/index.aspx".to_string()),
                update_available: false,
            }),
            documentation: Some(DocumentationLinks {
                product_page: Some(source_url.to_string()),
                support_page: Some("https://www.nvidia.com/en-us/geforce/drivers/".to_string()),
                manuals: vec![],
                datasheets: vec![],
                firmware_updates: vec![],
            }),
            images: None,
            metadata: DataMetadata {
                source: DataSource::ManufacturerWebsite,
                last_updated: Utc::now(),
                expires_at: Utc::now() + Duration::days(7),
                source_url: Some(source_url.to_string()),
                ai_confidence: None,
            },
        })
    }

    /// Fetch AMD GPU information.
    async fn fetch_amd_gpu(&self, model: &str) -> Result<DeviceDeepInfo> {
        log::info!("Fetching AMD GPU info for: {}", model);
        let model = model.to_string();

        // Fallback to TechPowerUp for AMD GPUs
        self.fetch_from_techpowerup(&model, &DeviceType::Gpu).await
    }

    /// Fetch motherboard information from manufacturer website.
    async fn fetch_motherboard_info(
        &self,
        manufacturer: &str,
        model: &str,
    ) -> Result<DeviceDeepInfo> {
        log::info!("Fetching motherboard info for: {} {}", manufacturer, model);

        let mfr_lower = manufacturer.to_lowercase();

        match mfr_lower.as_str() {
            "asus" => self.fetch_asus_motherboard(model).await,
            "msi" => self.fetch_msi_motherboard(model).await,
            "gigabyte" => self.fetch_gigabyte_motherboard(model).await,
            "asrock" => self.fetch_asrock_motherboard(model).await,
            _ => Err(anyhow::anyhow!(
                "Unsupported motherboard manufacturer: {}",
                manufacturer
            )),
        }
    }

    /// Fetch ASUS motherboard information.
    async fn fetch_asus_motherboard(&self, model: &str) -> Result<DeviceDeepInfo> {
        let model_slug = model.to_uppercase().replace(" ", "-");
        let product_url = format!(
            "https://www.asus.com/motherboards-components/motherboards/all-series/{}/",
            model_slug
        );

        let device_id = format!("mb-asus-{}", model.to_lowercase().replace(" ", "-"));

        // Return basic info with product URL
        Ok(DeviceDeepInfo {
            device_id,
            device_type: DeviceType::Motherboard,
            identifier: DeviceIdentifier {
                manufacturer: "ASUS".to_string(),
                model: model.to_string(),
                part_number: None,
                serial_number: None,
                hardware_ids: vec![],
            },
            specifications: None,
            drivers: Some(DriverInfo {
                installed_version: None,
                latest_version: None,
                download_url: None,
                release_date: None,
                release_notes_url: None,
                driver_page_url: Some(format!("{}/helpdesk_download/", product_url)),
                update_available: false,
            }),
            documentation: Some(DocumentationLinks {
                product_page: Some(product_url.clone()),
                support_page: Some(format!("{}/helpdesk/", product_url)),
                manuals: vec![],
                datasheets: vec![],
                firmware_updates: vec![],
            }),
            images: None,
            metadata: DataMetadata {
                source: DataSource::ManufacturerWebsite,
                last_updated: Utc::now(),
                expires_at: Utc::now() + Duration::days(7),
                source_url: Some(product_url),
                ai_confidence: None,
            },
        })
    }

    /// Fetch MSI motherboard information.
    async fn fetch_msi_motherboard(&self, model: &str) -> Result<DeviceDeepInfo> {
        let model_slug = model.to_uppercase().replace(" ", "-");
        let product_url = format!("https://www.msi.com/Motherboard/{}", model_slug);

        let device_id = format!("mb-msi-{}", model.to_lowercase().replace(" ", "-"));

        Ok(DeviceDeepInfo {
            device_id,
            device_type: DeviceType::Motherboard,
            identifier: DeviceIdentifier {
                manufacturer: "MSI".to_string(),
                model: model.to_string(),
                part_number: None,
                serial_number: None,
                hardware_ids: vec![],
            },
            specifications: None,
            drivers: Some(DriverInfo {
                installed_version: None,
                latest_version: None,
                download_url: None,
                release_date: None,
                release_notes_url: None,
                driver_page_url: Some(format!("{}/support", product_url)),
                update_available: false,
            }),
            documentation: Some(DocumentationLinks {
                product_page: Some(product_url.clone()),
                support_page: Some(format!("{}/support", product_url)),
                manuals: vec![],
                datasheets: vec![],
                firmware_updates: vec![],
            }),
            images: None,
            metadata: DataMetadata {
                source: DataSource::ManufacturerWebsite,
                last_updated: Utc::now(),
                expires_at: Utc::now() + Duration::days(7),
                source_url: Some(product_url),
                ai_confidence: None,
            },
        })
    }

    /// Fetch Gigabyte motherboard information.
    async fn fetch_gigabyte_motherboard(&self, model: &str) -> Result<DeviceDeepInfo> {
        let model_slug = model.to_uppercase().replace(" ", "-");
        let product_url = format!("https://www.gigabyte.com/Motherboard/{}", model_slug);

        let device_id = format!("mb-gigabyte-{}", model.to_lowercase().replace(" ", "-"));

        Ok(DeviceDeepInfo {
            device_id,
            device_type: DeviceType::Motherboard,
            identifier: DeviceIdentifier {
                manufacturer: "Gigabyte".to_string(),
                model: model.to_string(),
                part_number: None,
                serial_number: None,
                hardware_ids: vec![],
            },
            specifications: None,
            drivers: Some(DriverInfo {
                installed_version: None,
                latest_version: None,
                download_url: None,
                release_date: None,
                release_notes_url: None,
                driver_page_url: Some(format!("{}#kf", product_url)),
                update_available: false,
            }),
            documentation: Some(DocumentationLinks {
                product_page: Some(product_url.clone()),
                support_page: Some(format!("{}#kf", product_url)),
                manuals: vec![],
                datasheets: vec![],
                firmware_updates: vec![],
            }),
            images: None,
            metadata: DataMetadata {
                source: DataSource::ManufacturerWebsite,
                last_updated: Utc::now(),
                expires_at: Utc::now() + Duration::days(7),
                source_url: Some(product_url),
                ai_confidence: None,
            },
        })
    }

    /// Fetch ASRock motherboard information.
    async fn fetch_asrock_motherboard(&self, model: &str) -> Result<DeviceDeepInfo> {
        let model_slug = model.replace(" ", "%20");
        let product_url = format!("https://www.asrock.com/mb/{}", model_slug);

        let device_id = format!("mb-asrock-{}", model.to_lowercase().replace(" ", "-"));

        Ok(DeviceDeepInfo {
            device_id,
            device_type: DeviceType::Motherboard,
            identifier: DeviceIdentifier {
                manufacturer: "ASRock".to_string(),
                model: model.to_string(),
                part_number: None,
                serial_number: None,
                hardware_ids: vec![],
            },
            specifications: None,
            drivers: Some(DriverInfo {
                installed_version: None,
                latest_version: None,
                download_url: None,
                release_date: None,
                release_notes_url: None,
                driver_page_url: Some(format!("{}/index.asp", product_url)),
                update_available: false,
            }),
            documentation: Some(DocumentationLinks {
                product_page: Some(product_url.clone()),
                support_page: Some(format!("{}/index.asp", product_url)),
                manuals: vec![],
                datasheets: vec![],
                firmware_updates: vec![],
            }),
            images: None,
            metadata: DataMetadata {
                source: DataSource::ManufacturerWebsite,
                last_updated: Utc::now(),
                expires_at: Utc::now() + Duration::days(7),
                source_url: Some(product_url),
                ai_confidence: None,
            },
        })
    }

    /// Generate TechPowerUp URL slugs to try for a given model.
    /// TechPowerUp uses slugs like "geforce-rtx-5070" for GPUs.
    fn generate_techpowerup_slugs(model: &str, device_type: &DeviceType) -> Vec<String> {
        let mut slugs = Vec::new();
        let model_lower = model.to_lowercase().replace(' ', "-");

        match device_type {
            DeviceType::Gpu => {
                // For NVIDIA GPUs, add "geforce-" prefix if not present
                if model_lower.contains("rtx") || model_lower.contains("gtx") {
                    // Try with geforce prefix
                    if !model_lower.starts_with("geforce-") {
                        slugs.push(format!("geforce-{}", model_lower));
                    }
                    slugs.push(model_lower.clone());
                    // Try with nvidia-geforce prefix
                    if !model_lower.starts_with("nvidia-") {
                        slugs.push(format!(
                            "nvidia-geforce-{}",
                            model_lower.trim_start_matches("geforce-")
                        ));
                    }
                } else if model_lower.contains("radeon") || model_lower.contains("rx-") {
                    // For AMD GPUs
                    if !model_lower.starts_with("radeon-") {
                        slugs.push(format!("radeon-{}", model_lower));
                    }
                    slugs.push(model_lower.clone());
                } else {
                    slugs.push(model_lower.clone());
                }
            }
            DeviceType::Cpu => {
                // For CPUs, try variations
                if model_lower.contains("ryzen") {
                    // AMD Ryzen - add "amd-" prefix if not present
                    if !model_lower.starts_with("amd-") {
                        slugs.push(format!("amd-{}", model_lower));
                    }
                    slugs.push(model_lower.clone());
                } else if model_lower.contains("core") || model_lower.contains("xeon") {
                    // Intel - add "intel-" prefix if not present
                    if !model_lower.starts_with("intel-") {
                        slugs.push(format!("intel-{}", model_lower));
                    }
                    slugs.push(model_lower.clone());
                } else {
                    slugs.push(model_lower.clone());
                }
            }
            _ => {
                slugs.push(model_lower);
            }
        }

        log::debug!("Generated TechPowerUp slugs: {:?}", slugs);
        slugs
    }

    /// Fallback: Fetch from TechPowerUp database.
    pub async fn fetch_from_techpowerup(
        &self,
        model: &str,
        device_type: &DeviceType,
    ) -> Result<DeviceDeepInfo> {
        log::info!(
            "Fetching from TechPowerUp for: {} ({:?})",
            model,
            device_type
        );
        let model = model.to_string();
        let device_type = device_type.clone();

        let (db_type, _prefix) = match device_type {
            DeviceType::Cpu => ("cpu-specs", "cpu"),
            DeviceType::Gpu => ("gpu-specs", "gpu"),
            _ => {
                return Err(anyhow::anyhow!(
                    "TechPowerUp only supports CPU and GPU lookups"
                ))
            }
        };

        // Try direct URL patterns first (TechPowerUp uses predictable slugs)
        let direct_slugs = Self::generate_techpowerup_slugs(&model, &device_type);
        for slug in &direct_slugs {
            let direct_url = format!("https://www.techpowerup.com/{}/{}", db_type, slug);
            log::debug!("Trying TechPowerUp direct URL: {}", direct_url);

            if let Ok(resp) = self.client.get(&direct_url).send().await {
                if resp.status().is_success() {
                    let product_html = resp.text().await?;
                    let model_final = model.clone();
                    let device_type_final = device_type.clone();
                    let url_final = direct_url.clone();
                    return tokio::task::spawn_blocking(move || {
                        let document = Html::parse_document(&product_html);
                        Self::parse_techpowerup_page_static(
                            &document,
                            &model_final,
                            &device_type_final,
                            &url_final,
                        )
                    })
                    .await
                    .context("Spawn blocking failed")?;
                }
            }
        }

        // Fallback: Fetch TechPowerUp main database page and search
        let database_url = format!("https://www.techpowerup.com/{}/", db_type);
        log::debug!("TechPowerUp database URL: {}", database_url);

        let response = self.client.get(&database_url).send().await;

        match response {
            Ok(resp) if resp.status().is_success() => {
                let html = resp.text().await?;
                let model_clone = model.clone();
                let db_type_str = db_type.to_string();

                // First, extract the product page URL from search results
                let product_url = tokio::task::spawn_blocking(move || {
                    let document = Html::parse_document(&html);
                    Self::extract_techpowerup_product_url(&document, &model_clone, &db_type_str)
                })
                .await
                .context("Spawn blocking failed")??;

                log::debug!("TechPowerUp product URL: {}", product_url);

                // Fetch the actual product page
                let product_response = self.client.get(&product_url).send().await?;
                if !product_response.status().is_success() {
                    return Err(anyhow::anyhow!(
                        "TechPowerUp product page returned {}",
                        product_response.status()
                    ));
                }

                let product_html = product_response.text().await?;
                let model_final = model.clone();
                let device_type_final = device_type.clone();
                let url_final = product_url.clone();
                tokio::task::spawn_blocking(move || {
                    let document = Html::parse_document(&product_html);
                    Self::parse_techpowerup_page_static(
                        &document,
                        &model_final,
                        &device_type_final,
                        &url_final,
                    )
                })
                .await
                .context("Spawn blocking failed")?
            }
            _ => Err(anyhow::anyhow!("Could not find {} on TechPowerUp", model)),
        }
    }

    /// Extract the product URL from TechPowerUp search results.
    fn extract_techpowerup_product_url(
        document: &Html,
        model: &str,
        db_type: &str,
    ) -> Result<String> {
        // TechPowerUp search results have links in a table
        // Look for links that match the model name
        let link_selector =
            Selector::parse("table.processors a, table a[href*='specs/'], a[href*='-specs/']")
                .unwrap();
        let model_lower = model.to_lowercase();
        let model_parts: Vec<&str> = model_lower.split('-').collect();

        log::debug!(
            "Looking for TechPowerUp product link matching: {} (parts: {:?})",
            model,
            model_parts
        );

        for element in document.select(&link_selector) {
            if let Some(href) = element.value().attr("href") {
                let text = element.text().collect::<String>().to_lowercase();
                let href_lower = href.to_lowercase();

                // Check if the link text or href contains the key model parts
                // For "ryzen-9-9900x", check for "ryzen", "9900x"
                let key_parts: Vec<&str> = model_parts
                    .iter()
                    .filter(|p| {
                        p.len() > 1 && !["amd", "intel", "nvidia", "9", "7", "5", "3"].contains(p)
                    })
                    .cloned()
                    .collect();

                let matches = key_parts
                    .iter()
                    .all(|part| text.contains(part) || href_lower.contains(part));

                if matches && !key_parts.is_empty() {
                    let full_url = if href.starts_with("http") {
                        href.to_string()
                    } else if href.starts_with("/") {
                        format!("https://www.techpowerup.com{}", href)
                    } else {
                        format!("https://www.techpowerup.com/{}/{}", db_type, href)
                    };
                    log::debug!(
                        "Found TechPowerUp product URL: {} (matched text: {})",
                        full_url,
                        text.trim()
                    );
                    return Ok(full_url);
                }
            }
        }

        Err(anyhow::anyhow!(
            "Could not find product link for {} on TechPowerUp search results",
            model
        ))
    }

    /// Parse TechPowerUp specification page (static version for spawn_blocking).
    fn parse_techpowerup_page_static(
        document: &Html,
        model: &str,
        device_type: &DeviceType,
        source_url: &str,
    ) -> Result<DeviceDeepInfo> {
        // Check for bot verification page indicators
        let body_text = document
            .root_element()
            .text()
            .collect::<String>()
            .to_lowercase();

        if body_text.contains("checking your browser")
            || body_text.contains("verify you are human")
            || body_text.contains("please wait")
            || body_text.contains("ddos protection")
            || body_text.contains("cloudflare")
        {
            return Err(anyhow::anyhow!(
                "TechPowerUp returned bot verification page"
            ));
        }

        let mut specs = HashMap::new();

        // TechPowerUp uses definition lists for specs
        let dl_selector = Selector::parse("dl.clearfix").unwrap();
        let dt_selector = Selector::parse("dt").unwrap();
        let dd_selector = Selector::parse("dd").unwrap();

        for dl in document.select(&dl_selector) {
            let dts: Vec<_> = dl.select(&dt_selector).collect();
            let dds: Vec<_> = dl.select(&dd_selector).collect();

            for (dt, dd) in dts.into_iter().zip(dds.into_iter()) {
                let label = dt.text().collect::<String>().trim().to_string();
                let value = dd.text().collect::<String>().trim().to_string();

                if !label.is_empty() && !value.is_empty() {
                    specs.insert(label, value);
                }
            }
        }

        // Reject empty results - means page didn't have expected content
        if specs.is_empty() {
            return Err(anyhow::anyhow!(
                "TechPowerUp page had no specifications (possible bot check or invalid page)"
            ));
        }

        // Extract manufacturer from specs or model name
        let manufacturer = specs
            .get("Manufacturer")
            .cloned()
            .or_else(|| {
                if model.to_lowercase().contains("intel") {
                    Some("Intel".to_string())
                } else if model.to_lowercase().contains("amd") {
                    Some("AMD".to_string())
                } else if model.to_lowercase().contains("nvidia") {
                    Some("NVIDIA".to_string())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "Unknown".to_string());

        let prefix = match device_type {
            DeviceType::Cpu => "cpu",
            DeviceType::Gpu => "gpu",
            _ => "device",
        };

        let device_id = format!("{}-{}", prefix, model.to_lowercase().replace(" ", "-"));

        Ok(DeviceDeepInfo {
            device_id,
            device_type: device_type.clone(),
            identifier: DeviceIdentifier {
                manufacturer,
                model: model.to_string(),
                part_number: None,
                serial_number: None,
                hardware_ids: vec![],
            },
            specifications: Some(DeviceSpecifications {
                specs: specs.clone(),
                categories: vec![SpecCategory {
                    name: "Specifications".to_string(),
                    specs: specs
                        .iter()
                        .map(|(k, v)| SpecItem {
                            label: k.clone(),
                            value: v.clone(),
                            unit: None,
                        })
                        .collect(),
                }],
                description: None,
                release_date: specs.get("Release Date").cloned(),
                eol_date: None,
            }),
            drivers: None,
            documentation: Some(DocumentationLinks {
                product_page: Some(source_url.to_string()),
                support_page: None,
                manuals: vec![],
                datasheets: vec![],
                firmware_updates: vec![],
            }),
            images: None,
            metadata: DataMetadata {
                source: DataSource::ThirdPartyDatabase,
                last_updated: Utc::now(),
                expires_at: Utc::now() + Duration::days(7),
                source_url: Some(source_url.to_string()),
                ai_confidence: None,
            },
        })
    }
}

impl Default for InternetFetcher {
    fn default() -> Self {
        Self::new().expect("Failed to create default InternetFetcher")
    }
}
