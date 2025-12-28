//! Hardware configuration data models

use serde::{Deserialize, Serialize};

/// CPU static information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CpuInfo {
    pub name: String,
    pub manufacturer: String,
    pub architecture: String,
    pub family: String,
    pub model: String,
    pub stepping: String,
    pub physical_cores: u32,
    pub logical_processors: u32,
    pub base_clock_mhz: u32,
    pub max_clock_mhz: u32,
    pub cache: CacheInfo,
    pub socket: String,
    pub tdp_watts: Option<u32>,
}

/// CPU cache information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheInfo {
    pub l1_data_kb: u32,
    pub l1_instruction_kb: u32,
    pub l2_kb: u32,
    pub l3_kb: u32,
}

/// Real-time CPU metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CpuMetrics {
    pub total_usage: f32,
    pub per_core_usage: Vec<f32>,
    pub current_clock_mhz: u32,
    pub temperature: Option<f32>,
    pub power_draw: Option<f32>,
}

/// Memory static information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryInfo {
    pub total_bytes: u64,
    pub usable_bytes: u64,
    pub memory_type: String,
    pub speed_mhz: u32,
    pub slots_used: u32,
    pub slots_total: u32,
    pub max_capacity_bytes: u64,
    pub modules: Vec<MemoryModule>,
}

/// Individual memory module
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryModule {
    pub slot: String,
    pub capacity_bytes: u64,
    pub manufacturer: String,
    pub part_number: String,
    pub serial_number: String,
    pub speed_mhz: u32,
    pub configured_speed_mhz: u32,
}

/// Real-time memory metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryMetrics {
    pub in_use_bytes: u64,
    pub available_bytes: u64,
    pub committed_bytes: u64,
    pub cached_bytes: u64,
    pub paged_pool_bytes: u64,
    pub non_paged_pool_bytes: u64,
}

/// GPU static information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GpuInfo {
    pub id: String,
    pub name: String,
    pub manufacturer: String,
    pub driver_version: String,
    pub driver_date: String,
    pub vram_bytes: u64,
    pub current_resolution: String,
    pub refresh_rate_hz: u32,
    pub adapter_type: GpuAdapterType,
}

/// GPU adapter type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GpuAdapterType {
    Discrete,
    Integrated,
}

/// Real-time GPU metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GpuMetrics {
    pub gpu_id: String,
    pub usage_percent: f32,
    pub vram_used_bytes: u64,
    pub temperature: Option<f32>,
    pub clock_mhz: Option<u32>,
    pub fan_speed_percent: Option<f32>,
    pub power_draw: Option<f32>,
}

/// Motherboard information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MotherboardInfo {
    pub manufacturer: String,
    pub product: String,
    pub version: String,
    pub serial_number: String,
    pub chipset: Option<String>,
}

/// USB device information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsbDevice {
    pub name: String,
    pub manufacturer: Option<String>,
    pub vid: String,
    pub pid: String,
    pub port: String,
    pub speed: UsbSpeed,
    pub is_bus_powered: bool,
}

/// USB speed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UsbSpeed {
    Low,
    Full,
    High,
    Super,
    SuperPlus,
    Unknown,
}

/// Audio device information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioDevice {
    pub id: String,
    pub name: String,
    pub device_type: AudioDeviceType,
    pub is_default: bool,
    pub status: AudioDeviceStatus,
}

/// Audio device type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AudioDeviceType {
    Playback,
    Recording,
}

/// Audio device status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AudioDeviceStatus {
    Active,
    Disabled,
    NotPresent,
}

/// Monitor information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Monitor {
    pub id: String,
    pub name: String,
    pub manufacturer: Option<String>,
    pub resolution: String,
    pub size_inches: Option<f32>,
    pub connection: String,
    pub hdr_support: bool,
    pub refresh_rate_hz: u32,
}
