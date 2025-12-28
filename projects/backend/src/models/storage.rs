//! Storage configuration data models

use serde::{Deserialize, Serialize};

/// Physical disk information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhysicalDisk {
    pub device_id: u32,
    pub model: String,
    pub manufacturer: String,
    pub serial_number: String,
    pub media_type: MediaType,
    pub interface_type: InterfaceType,
    pub size_bytes: u64,
    pub partition_style: PartitionStyle,
    pub status: String,
    pub firmware: String,
}

/// Storage media type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MediaType {
    HDD,
    SSD,
    NVMe,
    Removable,
    Unknown,
}

/// Storage interface type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InterfaceType {
    SATA,
    NVMe,
    USB,
    SCSI,
    Unknown,
}

/// Partition style
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PartitionStyle {
    GPT,
    MBR,
    RAW,
}

/// Disk partition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Partition {
    pub partition_number: u32,
    pub disk_id: u32,
    pub partition_type: String,
    pub size_bytes: u64,
    pub offset_bytes: u64,
    pub is_bootable: bool,
    pub is_active: bool,
}

/// Logical volume
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Volume {
    pub drive_letter: Option<char>,
    pub label: String,
    pub file_system: String,
    pub total_bytes: u64,
    pub free_bytes: u64,
    pub used_bytes: u64,
    pub percent_used: f32,
    pub volume_serial: String,
    pub is_compressed: bool,
    pub is_encrypted: bool,
    pub is_system: bool,
    pub is_boot: bool,
}

/// Disk health status (S.M.A.R.T.)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskHealth {
    pub device_id: u32,
    pub status: HealthStatus,
    pub temperature_celsius: Option<i32>,
    pub power_on_hours: Option<u64>,
    pub power_cycles: Option<u64>,
    pub wear_level_percent: Option<u8>,
    pub smart_attributes: Vec<SmartAttribute>,
}

/// Health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Good,
    Warning,
    Critical,
    Unknown,
}

/// S.M.A.R.T. attribute
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SmartAttribute {
    pub id: u8,
    pub name: String,
    pub current: u8,
    pub worst: u8,
    pub threshold: u8,
    pub raw_value: String,
}

/// Real-time disk performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskPerformance {
    pub device_id: u32,
    pub read_bytes_per_sec: u64,
    pub write_bytes_per_sec: u64,
    pub read_iops: u64,
    pub write_iops: u64,
    pub queue_depth: u32,
    pub active_time_percent: f32,
}

/// Mapped network drive
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkDrive {
    pub drive_letter: String,
    pub unc_path: String,
    pub server: String,
    pub share_name: String,
    pub status: NetworkDriveStatus,
}

/// Network drive connection status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkDriveStatus {
    Connected,
    Disconnected,
    Unknown,
}
