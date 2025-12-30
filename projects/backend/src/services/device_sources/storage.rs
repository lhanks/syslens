//! Storage device source for SSD/HDD specifications.
//!
//! Fetches storage device specifications from manufacturer websites
//! and product databases.

use crate::models::{DeviceIdentifier, DeviceType, SpecCategory, SpecItem};
use crate::services::device_sources::DeviceSource;
use crate::services::PartialDeviceInfo;
use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use std::collections::HashMap;

/// Storage device source.
pub struct StorageSource {
    #[allow(dead_code)] // Reserved for future web scraping
    client: Client,
}

impl StorageSource {
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self { client })
    }

    /// Detect storage manufacturer from model name.
    fn detect_manufacturer(manufacturer: &str, model: &str) -> Option<StorageManufacturer> {
        let combined = format!("{} {}", manufacturer, model).to_lowercase();

        if combined.contains("samsung") {
            Some(StorageManufacturer::Samsung)
        } else if combined.contains("western digital") || combined.contains("wd") || combined.contains("sandisk") {
            Some(StorageManufacturer::WesternDigital)
        } else if combined.contains("seagate") {
            Some(StorageManufacturer::Seagate)
        } else if combined.contains("crucial") || combined.contains("micron") {
            Some(StorageManufacturer::Crucial)
        } else if combined.contains("kingston") {
            Some(StorageManufacturer::Kingston)
        } else if combined.contains("sk hynix") || combined.contains("hynix") {
            Some(StorageManufacturer::SKHynix)
        } else if combined.contains("intel") || combined.contains("solidigm") {
            Some(StorageManufacturer::Intel)
        } else if combined.contains("toshiba") || combined.contains("kioxia") {
            Some(StorageManufacturer::Toshiba)
        } else if combined.contains("corsair") {
            Some(StorageManufacturer::Corsair)
        } else if combined.contains("sabrent") {
            Some(StorageManufacturer::Sabrent)
        } else {
            None
        }
    }

    /// Detect storage type from model name.
    fn detect_storage_type(model: &str) -> StorageType {
        let model_lower = model.to_lowercase();

        if model_lower.contains("nvme") || model_lower.contains("m.2") || model_lower.contains("pcie") {
            StorageType::NVMe
        } else if model_lower.contains("sata") || model_lower.contains("ssd") {
            StorageType::SataSsd
        } else if model_lower.contains("hdd") || model_lower.contains("barracuda") || model_lower.contains("ironwolf") || model_lower.contains("wd red") || model_lower.contains("wd blue") || model_lower.contains("wd black") {
            StorageType::Hdd
        } else {
            // Default to NVMe for modern drives
            StorageType::NVMe
        }
    }

    /// Extract capacity from model name.
    fn extract_capacity(model: &str) -> Option<String> {
        let model_upper = model.to_uppercase();

        // Look for TB patterns
        let tb_patterns = ["16TB", "14TB", "12TB", "10TB", "8TB", "6TB", "4TB", "2TB", "1TB"];
        for pattern in tb_patterns {
            if model_upper.contains(pattern) {
                return Some(pattern.to_string());
            }
        }

        // Look for GB patterns
        let gb_patterns = ["960GB", "512GB", "500GB", "480GB", "256GB", "250GB", "240GB", "128GB", "120GB"];
        for pattern in gb_patterns {
            if model_upper.contains(pattern) {
                return Some(pattern.to_string());
            }
        }

        None
    }

    /// Extract interface from model name.
    fn extract_interface(model: &str, storage_type: &StorageType) -> String {
        let model_lower = model.to_lowercase();

        match storage_type {
            StorageType::NVMe => {
                if model_lower.contains("pcie 5") || model_lower.contains("gen5") || model_lower.contains("gen 5") {
                    "PCIe 5.0 x4 NVMe".to_string()
                } else if model_lower.contains("pcie 4") || model_lower.contains("gen4") || model_lower.contains("gen 4") {
                    "PCIe 4.0 x4 NVMe".to_string()
                } else if model_lower.contains("pcie 3") || model_lower.contains("gen3") || model_lower.contains("gen 3") {
                    "PCIe 3.0 x4 NVMe".to_string()
                } else {
                    "NVMe".to_string()
                }
            }
            StorageType::SataSsd => "SATA III (6 Gb/s)".to_string(),
            StorageType::Hdd => {
                if model_lower.contains("sas") {
                    "SAS".to_string()
                } else {
                    "SATA III (6 Gb/s)".to_string()
                }
            }
        }
    }

    /// Build specs from model name parsing.
    fn build_specs(manufacturer: &str, model: &str) -> (HashMap<String, String>, Vec<SpecCategory>) {
        let mut specs = HashMap::new();
        let mut general_specs = Vec::new();
        let mut performance_specs = Vec::new();

        let storage_type = Self::detect_storage_type(model);
        let mfr = Self::detect_manufacturer(manufacturer, model);

        // Manufacturer
        let mfr_name = match mfr {
            Some(StorageManufacturer::Samsung) => "Samsung",
            Some(StorageManufacturer::WesternDigital) => "Western Digital",
            Some(StorageManufacturer::Seagate) => "Seagate",
            Some(StorageManufacturer::Crucial) => "Crucial",
            Some(StorageManufacturer::Kingston) => "Kingston",
            Some(StorageManufacturer::SKHynix) => "SK Hynix",
            Some(StorageManufacturer::Intel) => "Intel/Solidigm",
            Some(StorageManufacturer::Toshiba) => "Toshiba/Kioxia",
            Some(StorageManufacturer::Corsair) => "Corsair",
            Some(StorageManufacturer::Sabrent) => "Sabrent",
            None => manufacturer,
        };

        specs.insert("manufacturer".to_string(), mfr_name.to_string());
        general_specs.push(SpecItem {
            label: "Manufacturer".to_string(),
            value: mfr_name.to_string(),
            unit: None,
        });

        // Storage type
        let type_str = match storage_type {
            StorageType::NVMe => "NVMe SSD",
            StorageType::SataSsd => "SATA SSD",
            StorageType::Hdd => "HDD",
        };
        specs.insert("type".to_string(), type_str.to_string());
        general_specs.push(SpecItem {
            label: "Type".to_string(),
            value: type_str.to_string(),
            unit: None,
        });

        // Interface
        let interface = Self::extract_interface(model, &storage_type);
        specs.insert("interface".to_string(), interface.clone());
        general_specs.push(SpecItem {
            label: "Interface".to_string(),
            value: interface,
            unit: None,
        });

        // Capacity
        if let Some(capacity) = Self::extract_capacity(model) {
            specs.insert("capacity".to_string(), capacity.clone());
            general_specs.push(SpecItem {
                label: "Capacity".to_string(),
                value: capacity,
                unit: None,
            });
        }

        // Form factor
        let form_factor = match storage_type {
            StorageType::NVMe => "M.2 2280",
            StorageType::SataSsd => "2.5\"",
            StorageType::Hdd => "3.5\"",
        };
        specs.insert("form_factor".to_string(), form_factor.to_string());
        general_specs.push(SpecItem {
            label: "Form Factor".to_string(),
            value: form_factor.to_string(),
            unit: None,
        });

        // Add estimated performance specs for known product lines
        Self::add_performance_estimates(model, &storage_type, &mut specs, &mut performance_specs);

        let mut categories = vec![SpecCategory {
            name: "General".to_string(),
            specs: general_specs,
        }];

        if !performance_specs.is_empty() {
            categories.push(SpecCategory {
                name: "Performance".to_string(),
                specs: performance_specs,
            });
        }

        (specs, categories)
    }

    /// Add estimated performance specs for known product lines.
    fn add_performance_estimates(
        model: &str,
        storage_type: &StorageType,
        specs: &mut HashMap<String, String>,
        performance_specs: &mut Vec<SpecItem>,
    ) {
        let model_lower = model.to_lowercase();

        // Samsung 990 Pro / 980 Pro
        if model_lower.contains("990 pro") {
            specs.insert("seq_read".to_string(), "7,450 MB/s".to_string());
            specs.insert("seq_write".to_string(), "6,900 MB/s".to_string());
            performance_specs.push(SpecItem { label: "Sequential Read".to_string(), value: "7,450".to_string(), unit: Some("MB/s".to_string()) });
            performance_specs.push(SpecItem { label: "Sequential Write".to_string(), value: "6,900".to_string(), unit: Some("MB/s".to_string()) });
        } else if model_lower.contains("980 pro") {
            specs.insert("seq_read".to_string(), "7,000 MB/s".to_string());
            specs.insert("seq_write".to_string(), "5,100 MB/s".to_string());
            performance_specs.push(SpecItem { label: "Sequential Read".to_string(), value: "7,000".to_string(), unit: Some("MB/s".to_string()) });
            performance_specs.push(SpecItem { label: "Sequential Write".to_string(), value: "5,100".to_string(), unit: Some("MB/s".to_string()) });
        } else if model_lower.contains("970 evo") {
            specs.insert("seq_read".to_string(), "3,500 MB/s".to_string());
            specs.insert("seq_write".to_string(), "3,300 MB/s".to_string());
            performance_specs.push(SpecItem { label: "Sequential Read".to_string(), value: "3,500".to_string(), unit: Some("MB/s".to_string()) });
            performance_specs.push(SpecItem { label: "Sequential Write".to_string(), value: "3,300".to_string(), unit: Some("MB/s".to_string()) });
        }
        // WD Black SN850X / SN770
        else if model_lower.contains("sn850x") {
            specs.insert("seq_read".to_string(), "7,300 MB/s".to_string());
            specs.insert("seq_write".to_string(), "6,600 MB/s".to_string());
            performance_specs.push(SpecItem { label: "Sequential Read".to_string(), value: "7,300".to_string(), unit: Some("MB/s".to_string()) });
            performance_specs.push(SpecItem { label: "Sequential Write".to_string(), value: "6,600".to_string(), unit: Some("MB/s".to_string()) });
        } else if model_lower.contains("sn770") {
            specs.insert("seq_read".to_string(), "5,150 MB/s".to_string());
            specs.insert("seq_write".to_string(), "4,900 MB/s".to_string());
            performance_specs.push(SpecItem { label: "Sequential Read".to_string(), value: "5,150".to_string(), unit: Some("MB/s".to_string()) });
            performance_specs.push(SpecItem { label: "Sequential Write".to_string(), value: "4,900".to_string(), unit: Some("MB/s".to_string()) });
        }
        // Generic estimates by type
        else {
            match storage_type {
                StorageType::NVMe => {
                    if model_lower.contains("gen5") || model_lower.contains("pcie 5") {
                        specs.insert("seq_read".to_string(), "~10,000 MB/s".to_string());
                        performance_specs.push(SpecItem { label: "Sequential Read (est.)".to_string(), value: "~10,000".to_string(), unit: Some("MB/s".to_string()) });
                    } else if model_lower.contains("gen4") || model_lower.contains("pcie 4") {
                        specs.insert("seq_read".to_string(), "~7,000 MB/s".to_string());
                        performance_specs.push(SpecItem { label: "Sequential Read (est.)".to_string(), value: "~7,000".to_string(), unit: Some("MB/s".to_string()) });
                    }
                }
                StorageType::SataSsd => {
                    specs.insert("seq_read".to_string(), "~550 MB/s".to_string());
                    specs.insert("seq_write".to_string(), "~520 MB/s".to_string());
                    performance_specs.push(SpecItem { label: "Sequential Read (est.)".to_string(), value: "~550".to_string(), unit: Some("MB/s".to_string()) });
                    performance_specs.push(SpecItem { label: "Sequential Write (est.)".to_string(), value: "~520".to_string(), unit: Some("MB/s".to_string()) });
                }
                StorageType::Hdd => {
                    specs.insert("seq_read".to_string(), "~200 MB/s".to_string());
                    performance_specs.push(SpecItem { label: "Sequential Read (est.)".to_string(), value: "~200".to_string(), unit: Some("MB/s".to_string()) });
                }
            }
        }
    }

    /// Get manufacturer support URL.
    fn get_support_url(mfr: Option<StorageManufacturer>) -> Option<String> {
        match mfr {
            Some(StorageManufacturer::Samsung) => Some("https://www.samsung.com/us/support/".to_string()),
            Some(StorageManufacturer::WesternDigital) => Some("https://support-en.wd.com/".to_string()),
            Some(StorageManufacturer::Seagate) => Some("https://www.seagate.com/support/".to_string()),
            Some(StorageManufacturer::Crucial) => Some("https://www.crucial.com/support".to_string()),
            Some(StorageManufacturer::Kingston) => Some("https://www.kingston.com/support".to_string()),
            Some(StorageManufacturer::SKHynix) => Some("https://ssd.skhynix.com/".to_string()),
            Some(StorageManufacturer::Intel) => Some("https://www.solidigm.com/support.html".to_string()),
            Some(StorageManufacturer::Toshiba) => Some("https://personal.kioxia.com/support/".to_string()),
            Some(StorageManufacturer::Corsair) => Some("https://www.corsair.com/support".to_string()),
            Some(StorageManufacturer::Sabrent) => Some("https://www.sabrent.com/support".to_string()),
            None => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum StorageManufacturer {
    Samsung,
    WesternDigital,
    Seagate,
    Crucial,
    Kingston,
    SKHynix,
    Intel,
    Toshiba,
    Corsair,
    Sabrent,
}

#[derive(Debug, Clone, Copy)]
enum StorageType {
    NVMe,
    SataSsd,
    Hdd,
}

#[async_trait]
impl DeviceSource for StorageSource {
    fn name(&self) -> &str {
        "Storage Source"
    }

    fn priority(&self) -> u8 {
        20 // Medium priority
    }

    fn supports(&self, device_type: &DeviceType, _identifier: &DeviceIdentifier) -> bool {
        matches!(device_type, DeviceType::Storage)
    }

    async fn fetch(
        &self,
        _device_type: &DeviceType,
        identifier: &DeviceIdentifier,
    ) -> Result<PartialDeviceInfo> {
        let manufacturer = &identifier.manufacturer;
        let model = &identifier.model;

        log::info!("Fetching storage info for {} {}", manufacturer, model);

        let mfr = Self::detect_manufacturer(manufacturer, model);
        let (specs, categories) = Self::build_specs(manufacturer, model);

        Ok(PartialDeviceInfo {
            specs,
            categories,
            description: Some(format!("{} {}", manufacturer, model)),
            release_date: None,
            product_page: None,
            support_page: Self::get_support_url(mfr),
            image_url: None,
            source_name: "Storage Source".to_string(),
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
    fn test_detect_storage_type() {
        assert!(matches!(StorageSource::detect_storage_type("Samsung 990 Pro NVMe"), StorageType::NVMe));
        assert!(matches!(StorageSource::detect_storage_type("Crucial MX500 SATA SSD"), StorageType::SataSsd));
        assert!(matches!(StorageSource::detect_storage_type("Seagate Barracuda 2TB"), StorageType::Hdd));
    }

    #[test]
    fn test_extract_capacity() {
        assert_eq!(StorageSource::extract_capacity("Samsung 990 Pro 2TB"), Some("2TB".to_string()));
        assert_eq!(StorageSource::extract_capacity("WD Black SN850X 1TB"), Some("1TB".to_string()));
        assert_eq!(StorageSource::extract_capacity("Crucial MX500 500GB"), Some("500GB".to_string()));
    }
}
