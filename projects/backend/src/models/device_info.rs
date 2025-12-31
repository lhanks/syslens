//! Data models for deep device information with internet lookup support.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Device category enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "PascalCase")]
pub enum DeviceType {
    Cpu,
    Gpu,
    Motherboard,
    Memory,
    Storage,
    Monitor,
}

impl std::fmt::Display for DeviceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeviceType::Cpu => write!(f, "CPU"),
            DeviceType::Gpu => write!(f, "GPU"),
            DeviceType::Motherboard => write!(f, "Motherboard"),
            DeviceType::Memory => write!(f, "Memory"),
            DeviceType::Storage => write!(f, "Storage"),
            DeviceType::Monitor => write!(f, "Monitor"),
        }
    }
}

/// Deep device information with internet-sourced data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceDeepInfo {
    /// Device identifier (matches hardware collector ID)
    pub device_id: String,

    /// Device category
    pub device_type: DeviceType,

    /// Basic identifying information
    pub identifier: DeviceIdentifier,

    /// Detailed specifications
    pub specifications: Option<DeviceSpecifications>,

    /// Driver information
    pub drivers: Option<DriverInfo>,

    /// Documentation links
    pub documentation: Option<DocumentationLinks>,

    /// Product images
    pub images: Option<ProductImages>,

    /// Data source and freshness
    pub metadata: DataMetadata,
}

/// Device identifying information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceIdentifier {
    pub manufacturer: String,
    pub model: String,
    pub part_number: Option<String>,
    pub serial_number: Option<String>,
    /// Hardware IDs (PCI ID, USB VID/PID, etc.)
    #[serde(default)]
    pub hardware_ids: Vec<String>,
}

/// Detailed device specifications
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceSpecifications {
    /// Raw specification key-value pairs
    #[serde(default)]
    pub specs: HashMap<String, String>,

    /// Categorized specifications for display
    #[serde(default)]
    pub categories: Vec<SpecCategory>,

    /// Marketing/product description
    pub description: Option<String>,

    /// Release date
    pub release_date: Option<String>,

    /// End of life date (if applicable)
    pub eol_date: Option<String>,
}

/// Specification category for organized display
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpecCategory {
    pub name: String,
    pub specs: Vec<SpecItem>,
}

/// Individual specification item
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpecItem {
    pub label: String,
    pub value: String,
    pub unit: Option<String>,
}

/// Driver information and links
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriverInfo {
    /// Currently installed driver version
    pub installed_version: Option<String>,

    /// Latest available driver version
    pub latest_version: Option<String>,

    /// Direct download URL for latest driver
    pub download_url: Option<String>,

    /// Driver release date
    pub release_date: Option<String>,

    /// Release notes URL
    pub release_notes_url: Option<String>,

    /// Generic driver download page (fallback)
    pub driver_page_url: Option<String>,

    /// Update available flag
    #[serde(default)]
    pub update_available: bool,
}

/// Documentation and support links
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentationLinks {
    /// Product page URL
    pub product_page: Option<String>,

    /// Support/download page
    pub support_page: Option<String>,

    /// User manual PDF links
    #[serde(default)]
    pub manuals: Vec<DocumentLink>,

    /// Technical datasheets
    #[serde(default)]
    pub datasheets: Vec<DocumentLink>,

    /// BIOS/Firmware downloads (motherboards/storage)
    #[serde(default)]
    pub firmware_updates: Vec<FirmwareLink>,
}

/// Link to a document (manual, datasheet, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentLink {
    pub title: String,
    pub url: String,
    pub file_type: String,
    pub language: Option<String>,
}

/// Link to firmware/BIOS update
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FirmwareLink {
    pub title: String,
    pub version: String,
    pub url: String,
    pub release_date: Option<String>,
}

/// Image type for categorizing product images
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub enum ImageType {
    /// Main product photograph
    #[default]
    Product,
    /// Retail packaging
    Packaging,
    /// Hardware installed in system
    Installation,
    /// Technical diagram or schematic
    Diagram,
    /// Die shot or internal view
    DieShot,
    /// Other uncategorized image
    Other,
}

/// A single image entry with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageEntry {
    /// Original URL of the image
    pub url: String,

    /// Local cached path (if downloaded)
    pub cached_path: Option<String>,

    /// Type of image
    #[serde(default)]
    pub image_type: ImageType,

    /// Description or alt text
    pub description: Option<String>,

    /// Width in pixels (if known)
    pub width: Option<u32>,

    /// Height in pixels (if known)
    pub height: Option<u32>,
}

/// Image metadata for tracking source and freshness
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageMetadata {
    /// When image was fetched
    pub fetched_at: DateTime<Utc>,

    /// Source URL or service name
    pub source: String,

    /// Whether image is AI-generated
    #[serde(default)]
    pub ai_generated: bool,

    /// Cache key (hash of URL or identifier)
    pub cache_key: String,

    /// File size in bytes
    pub file_size: Option<u64>,
}

/// Product images with comprehensive metadata
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ProductImages {
    /// Primary product image URL (original source)
    pub primary_image: Option<String>,

    /// Local cached path to primary image
    pub primary_image_cached: Option<String>,

    /// Additional product images with metadata
    #[serde(default)]
    pub gallery: Vec<ImageEntry>,

    /// Thumbnail URL (256x256 or smaller)
    pub thumbnail: Option<String>,

    /// Cached thumbnail path
    pub thumbnail_cached: Option<String>,

    /// Image metadata for tracking
    pub metadata: Option<ImageMetadata>,
}

/// Data source enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DataSource {
    /// From local JSON database
    LocalDatabase,
    /// Fetched from manufacturer website
    ManufacturerWebsite,
    /// Fetched from third-party database
    ThirdPartyDatabase,
    /// Retrieved via AI agent
    AiAgent,
    /// Cached from previous fetch
    Cache,
}

impl std::fmt::Display for DataSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataSource::LocalDatabase => write!(f, "Local Database"),
            DataSource::ManufacturerWebsite => write!(f, "Manufacturer Website"),
            DataSource::ThirdPartyDatabase => write!(f, "Third-Party Database"),
            DataSource::AiAgent => write!(f, "AI Agent"),
            DataSource::Cache => write!(f, "Cache"),
        }
    }
}

/// Data source and freshness metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataMetadata {
    /// Data source
    pub source: DataSource,

    /// When data was fetched/updated
    pub last_updated: DateTime<Utc>,

    /// When data should be refreshed
    pub expires_at: DateTime<Utc>,

    /// Source URL (if from internet)
    pub source_url: Option<String>,

    /// AI confidence score (if AI-sourced, 0.0-1.0)
    pub ai_confidence: Option<f32>,
}

/// Cache entry for storing device information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheEntry {
    pub device_id: String,
    pub device_type: DeviceType,
    pub data: DeviceDeepInfo,
    pub cached_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

/// Cache database structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DeviceCache {
    pub entries: Vec<CacheEntry>,
}

/// Local database structure for bundled device information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalDatabase {
    pub version: String,
    pub last_updated: DateTime<Utc>,
    pub devices: DeviceCategories,
}

/// Device categories in local database
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DeviceCategories {
    #[serde(default)]
    pub cpu: Vec<DeviceDeepInfo>,
    #[serde(default)]
    pub gpu: Vec<DeviceDeepInfo>,
    #[serde(default)]
    pub motherboard: Vec<DeviceDeepInfo>,
    #[serde(default)]
    pub memory: Vec<DeviceDeepInfo>,
    #[serde(default)]
    pub storage: Vec<DeviceDeepInfo>,
    #[serde(default)]
    pub monitor: Vec<DeviceDeepInfo>,
}

impl DeviceCategories {
    /// Get devices by type
    pub fn get_by_type(&self, device_type: &DeviceType) -> &Vec<DeviceDeepInfo> {
        match device_type {
            DeviceType::Cpu => &self.cpu,
            DeviceType::Gpu => &self.gpu,
            DeviceType::Motherboard => &self.motherboard,
            DeviceType::Memory => &self.memory,
            DeviceType::Storage => &self.storage,
            DeviceType::Monitor => &self.monitor,
        }
    }

    /// Get mutable devices by type
    pub fn get_by_type_mut(&mut self, device_type: &DeviceType) -> &mut Vec<DeviceDeepInfo> {
        match device_type {
            DeviceType::Cpu => &mut self.cpu,
            DeviceType::Gpu => &mut self.gpu,
            DeviceType::Motherboard => &mut self.motherboard,
            DeviceType::Memory => &mut self.memory,
            DeviceType::Storage => &mut self.storage,
            DeviceType::Monitor => &mut self.monitor,
        }
    }
}

impl Default for LocalDatabase {
    fn default() -> Self {
        Self {
            version: "1.0.0".to_string(),
            last_updated: Utc::now(),
            devices: DeviceCategories::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_type_display() {
        assert_eq!(format!("{}", DeviceType::Cpu), "CPU");
        assert_eq!(format!("{}", DeviceType::Gpu), "GPU");
        assert_eq!(format!("{}", DeviceType::Motherboard), "Motherboard");
        assert_eq!(format!("{}", DeviceType::Memory), "Memory");
        assert_eq!(format!("{}", DeviceType::Storage), "Storage");
    }

    #[test]
    fn test_data_source_display() {
        assert_eq!(format!("{}", DataSource::LocalDatabase), "Local Database");
        assert_eq!(
            format!("{}", DataSource::ManufacturerWebsite),
            "Manufacturer Website"
        );
        assert_eq!(
            format!("{}", DataSource::ThirdPartyDatabase),
            "Third-Party Database"
        );
        assert_eq!(format!("{}", DataSource::AiAgent), "AI Agent");
        assert_eq!(format!("{}", DataSource::Cache), "Cache");
    }

    #[test]
    fn test_device_type_serialization() {
        let cpu = DeviceType::Cpu;
        let json = serde_json::to_string(&cpu).unwrap();
        assert_eq!(json, "\"Cpu\"");

        let gpu = DeviceType::Gpu;
        let json = serde_json::to_string(&gpu).unwrap();
        assert_eq!(json, "\"Gpu\"");
    }

    #[test]
    fn test_device_type_deserialization() {
        let cpu: DeviceType = serde_json::from_str("\"Cpu\"").unwrap();
        assert_eq!(cpu, DeviceType::Cpu);

        let gpu: DeviceType = serde_json::from_str("\"Gpu\"").unwrap();
        assert_eq!(gpu, DeviceType::Gpu);
    }

    #[test]
    fn test_device_identifier_serialization() {
        let identifier = DeviceIdentifier {
            manufacturer: "Intel".to_string(),
            model: "Core i7-12700K".to_string(),
            part_number: Some("BX8071512700K".to_string()),
            serial_number: None,
            hardware_ids: vec!["PCI\\VEN_8086".to_string()],
        };

        let json = serde_json::to_string(&identifier).unwrap();
        assert!(json.contains("\"manufacturer\":\"Intel\""));
        assert!(json.contains("\"model\":\"Core i7-12700K\""));
        assert!(json.contains("\"partNumber\":\"BX8071512700K\""));
        assert!(json.contains("\"serialNumber\":null"));
        assert!(json.contains("\"hardwareIds\""));
    }

    #[test]
    fn test_device_categories_get_by_type() {
        let mut categories = DeviceCategories::default();

        // Test get_by_type returns empty vectors for each type
        assert!(categories.get_by_type(&DeviceType::Cpu).is_empty());
        assert!(categories.get_by_type(&DeviceType::Gpu).is_empty());
        assert!(categories.get_by_type(&DeviceType::Motherboard).is_empty());
        assert!(categories.get_by_type(&DeviceType::Memory).is_empty());
        assert!(categories.get_by_type(&DeviceType::Storage).is_empty());

        // Test get_by_type_mut allows modification
        categories
            .get_by_type_mut(&DeviceType::Cpu)
            .push(DeviceDeepInfo {
                device_id: "test-cpu".to_string(),
                device_type: DeviceType::Cpu,
                identifier: DeviceIdentifier {
                    manufacturer: "Intel".to_string(),
                    model: "Test CPU".to_string(),
                    part_number: None,
                    serial_number: None,
                    hardware_ids: vec![],
                },
                specifications: None,
                drivers: None,
                documentation: None,
                images: None,
                metadata: DataMetadata {
                    source: DataSource::LocalDatabase,
                    last_updated: Utc::now(),
                    expires_at: Utc::now(),
                    source_url: None,
                    ai_confidence: None,
                },
            });

        assert_eq!(categories.get_by_type(&DeviceType::Cpu).len(), 1);
    }

    #[test]
    fn test_local_database_default() {
        let db = LocalDatabase::default();
        assert_eq!(db.version, "1.0.0");
        assert!(db.devices.cpu.is_empty());
        assert!(db.devices.gpu.is_empty());
    }

    #[test]
    fn test_spec_item_serialization() {
        let item = SpecItem {
            label: "Base Clock".to_string(),
            value: "3.6 GHz".to_string(),
            unit: Some("GHz".to_string()),
        };

        let json = serde_json::to_string(&item).unwrap();
        assert!(json.contains("\"label\":\"Base Clock\""));
        assert!(json.contains("\"value\":\"3.6 GHz\""));
        assert!(json.contains("\"unit\":\"GHz\""));
    }

    #[test]
    fn test_driver_info_defaults() {
        let driver_info = DriverInfo {
            installed_version: Some("1.0.0".to_string()),
            latest_version: None,
            download_url: None,
            release_date: None,
            release_notes_url: None,
            driver_page_url: None,
            update_available: false,
        };

        let json = serde_json::to_string(&driver_info).unwrap();
        assert!(json.contains("\"updateAvailable\":false"));
    }
}
