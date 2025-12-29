//! Internet fetcher for device information from manufacturer websites.

use crate::models::{
    DataMetadata, DataSource, DeviceDeepInfo, DeviceIdentifier, DeviceSpecifications,
    DeviceType, DocumentationLinks, DriverInfo, ProductImages, SpecCategory,
    SpecItem,
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
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(REQUEST_TIMEOUT))
            .user_agent(USER_AGENT)
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self { client })
    }

    /// Fetch device info from the appropriate manufacturer website.
    pub async fn fetch_device_info(
        &self,
        identifier: &DeviceIdentifier,
        device_type: &DeviceType,
    ) -> Result<DeviceDeepInfo> {
        let manufacturer = identifier.manufacturer.to_lowercase();

        match device_type {
            DeviceType::Cpu => match manufacturer.as_str() {
                "intel" => self.fetch_intel_cpu(&identifier.model).await,
                "amd" => self.fetch_amd_cpu(&identifier.model).await,
                _ => Err(anyhow::anyhow!("Unsupported CPU manufacturer: {}", manufacturer)),
            },
            DeviceType::Gpu => match manufacturer.as_str() {
                "nvidia" => self.fetch_nvidia_gpu(&identifier.model).await,
                "amd" => self.fetch_amd_gpu(&identifier.model).await,
                _ => Err(anyhow::anyhow!("Unsupported GPU manufacturer: {}", manufacturer)),
            },
            DeviceType::Motherboard => {
                self.fetch_motherboard_info(&identifier.manufacturer, &identifier.model).await
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
                if text.contains(&model_lower) || href.to_lowercase().contains(&model_lower.replace(" ", "-")) {
                    let full_url = if href.starts_with("http") {
                        href.to_string()
                    } else {
                        format!("https://ark.intel.com{}", href)
                    };
                    return Ok(full_url);
                }
            }
        }

        Err(anyhow::anyhow!("Could not find Intel product page for: {}", model))
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
                let label = row.select(&label_selector).next()
                    .map(|e| e.text().collect::<String>().trim().to_string());
                let value = row.select(&value_selector).next()
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
                let section_name = section.select(&title_selector).next()
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
        let primary_image = document.select(&image_selector).next()
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
                gallery: vec![],
                thumbnail: None,
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

    /// Fetch AMD CPU information.
    async fn fetch_amd_cpu(&self, model: &str) -> Result<DeviceDeepInfo> {
        log::info!("Fetching AMD CPU info for: {}", model);
        let model = model.to_string();

        // AMD product page URL pattern
        let model_slug = model.to_lowercase()
            .replace(" ", "-");

        let product_url = format!(
            "https://www.amd.com/en/products/processors/desktops/ryzen/7000-series/amd-{}.html",
            model_slug
        );

        let response = self.client.get(&product_url).send().await;

        match response {
            Ok(resp) if resp.status().is_success() => {
                let html = resp.text().await?;
                let model_final = model.clone();
                let url_final = product_url.clone();
                tokio::task::spawn_blocking(move || {
                    let document = Html::parse_document(&html);
                    Self::parse_amd_product_page_static(&document, &model_final, &url_final)
                })
                .await
                .context("Spawn blocking failed")?
            }
            _ => {
                // Fallback to TechPowerUp
                self.fetch_from_techpowerup(&model, &DeviceType::Cpu).await
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
                driver_page_url: Some("https://www.amd.com/en/support/downloads/drivers.html".to_string()),
                update_available: false,
            }),
            documentation: Some(DocumentationLinks {
                product_page: Some(source_url.to_string()),
                support_page: Some("https://www.amd.com/en/support".to_string()),
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

    /// Fetch NVIDIA GPU information.
    async fn fetch_nvidia_gpu(&self, model: &str) -> Result<DeviceDeepInfo> {
        log::info!("Fetching NVIDIA GPU info for: {}", model);
        let model = model.to_string();

        // NVIDIA product page URL pattern
        let model_slug = model.to_lowercase()
            .replace("geforce ", "")
            .replace(" ", "-");

        let product_url = format!(
            "https://www.nvidia.com/en-us/geforce/graphics-cards/40-series/{}/",
            model_slug
        );

        let response = self.client.get(&product_url).send().await;

        match response {
            Ok(resp) if resp.status().is_success() => {
                let html = resp.text().await?;
                let model_final = model.clone();
                let url_final = product_url.clone();
                tokio::task::spawn_blocking(move || {
                    let document = Html::parse_document(&html);
                    Self::parse_nvidia_product_page_static(&document, &model_final, &url_final)
                })
                .await
                .context("Spawn blocking failed")?
            }
            _ => {
                // Fallback to TechPowerUp
                self.fetch_from_techpowerup(&model, &DeviceType::Gpu).await
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
            let label = row.select(&label_selector).next()
                .map(|e| e.text().collect::<String>().trim().to_string());
            let value = row.select(&value_selector).next()
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
                    specs: specs.iter().map(|(k, v)| SpecItem {
                        label: k.clone(),
                        value: v.clone(),
                        unit: None,
                    }).collect(),
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
    async fn fetch_motherboard_info(&self, manufacturer: &str, model: &str) -> Result<DeviceDeepInfo> {
        log::info!("Fetching motherboard info for: {} {}", manufacturer, model);

        let mfr_lower = manufacturer.to_lowercase();

        match mfr_lower.as_str() {
            "asus" => self.fetch_asus_motherboard(model).await,
            "msi" => self.fetch_msi_motherboard(model).await,
            "gigabyte" => self.fetch_gigabyte_motherboard(model).await,
            "asrock" => self.fetch_asrock_motherboard(model).await,
            _ => Err(anyhow::anyhow!("Unsupported motherboard manufacturer: {}", manufacturer)),
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

    /// Fallback: Fetch from TechPowerUp database.
    pub async fn fetch_from_techpowerup(&self, model: &str, device_type: &DeviceType) -> Result<DeviceDeepInfo> {
        log::info!("Fetching from TechPowerUp for: {} ({:?})", model, device_type);
        let model = model.to_string();
        let device_type = device_type.clone();

        let (db_type, _prefix) = match device_type {
            DeviceType::Cpu => ("cpu-specs", "cpu"),
            DeviceType::Gpu => ("gpu-specs", "gpu"),
            _ => return Err(anyhow::anyhow!("TechPowerUp only supports CPU and GPU lookups")),
        };

        // Search TechPowerUp
        let search_url = format!(
            "https://www.techpowerup.com/{}/specs/{}.html",
            db_type,
            urlencoding::encode(&model)
        );

        let response = self.client.get(&search_url).send().await;

        match response {
            Ok(resp) if resp.status().is_success() => {
                let html = resp.text().await?;
                let model_final = model.clone();
                let device_type_final = device_type.clone();
                let url_final = search_url.clone();
                tokio::task::spawn_blocking(move || {
                    let document = Html::parse_document(&html);
                    Self::parse_techpowerup_page_static(&document, &model_final, &device_type_final, &url_final)
                })
                .await
                .context("Spawn blocking failed")?
            }
            _ => Err(anyhow::anyhow!("Could not find {} on TechPowerUp", model)),
        }
    }

    /// Parse TechPowerUp specification page (static version for spawn_blocking).
    fn parse_techpowerup_page_static(
        document: &Html,
        model: &str,
        device_type: &DeviceType,
        source_url: &str,
    ) -> Result<DeviceDeepInfo> {
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

        // Extract manufacturer from specs or model name
        let manufacturer = specs.get("Manufacturer")
            .cloned()
            .or_else(|| {
                if model.to_lowercase().contains("intel") { Some("Intel".to_string()) }
                else if model.to_lowercase().contains("amd") { Some("AMD".to_string()) }
                else if model.to_lowercase().contains("nvidia") { Some("NVIDIA".to_string()) }
                else { None }
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
                    specs: specs.iter().map(|(k, v)| SpecItem {
                        label: k.clone(),
                        value: v.clone(),
                        unit: None,
                    }).collect(),
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
