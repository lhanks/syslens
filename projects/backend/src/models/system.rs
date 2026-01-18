//! System configuration data models

use serde::{Deserialize, Serialize};

/// Device identification information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInfo {
    pub computer_name: String,
    pub device_name: String,
    pub manufacturer: String,
    pub model: String,
    pub system_type: String,
    pub serial_number: String,
    pub product_id: Option<String>,
    pub system_sku: Option<String>,
}

/// BIOS/UEFI firmware information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BiosInfo {
    pub vendor: String,
    pub version: String,
    pub firmware_version: String,
    pub release_date: String,
    pub uefi_version: Option<String>,
    pub secure_boot: bool,
    pub tpm_version: Option<String>,
    pub tpm_status: TpmStatus,
}

/// TPM status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TpmStatus {
    Enabled,
    Disabled,
    NotPresent,
    Unknown,
}

/// Boot configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BootConfig {
    pub boot_mode: BootMode,
    pub secure_boot_enabled: bool,
    pub boot_device: String,
    pub boot_order: Vec<String>,
    pub boot_priority: String,
    pub fast_startup: bool,
    pub hibernation: bool,
    pub last_boot_time: String,
    pub boot_duration_seconds: u32,
}

/// Boot mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BootMode {
    UEFI,
    Legacy,
}

/// Operating system information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OsInfo {
    pub name: String,
    pub version: String,
    pub build: String,
    pub architecture: String,
    pub install_date: String,
    pub last_update: Option<String>,
    pub activation_status: ActivationStatus,
    pub product_key: Option<String>,
}

/// Windows activation status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActivationStatus {
    Activated,
    NotActivated,
    GracePeriod,
    Unknown,
}

/// System uptime information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemUptime {
    pub uptime_seconds: u64,
    pub last_shutdown: Option<String>,
    pub restart_pending: bool,
    pub sleep_count: u32,
}

/// Domain/workgroup information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DomainInfo {
    pub domain: Option<String>,
    pub workgroup: Option<String>,
    pub domain_role: DomainRole,
    pub ad_site: Option<String>,
    pub logon_server: Option<String>,
}

/// Domain role
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DomainRole {
    Workstation,
    MemberWorkstation,
    StandaloneServer,
    MemberServer,
    BackupDomainController,
    PrimaryDomainController,
}

/// Current user information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserInfo {
    pub username: String,
    pub user_sid: String,
    pub user_profile: String,
    pub is_admin: bool,
    pub login_time: String,
}

/// System restore point information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestorePoint {
    pub sequence_number: u32,
    pub description: String,
    pub restore_point_type: RestorePointType,
    pub creation_time: String,
}

/// Type of restore point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RestorePointType {
    ApplicationInstall,
    ApplicationUninstall,
    ModifySettings,
    CancelledOperation,
    BackupRecovery,
    DeviceDriverInstall,
    ManualCheckpoint,
    WindowsUpdate,
    Unknown,
}

// Re-export hardware, network, storage models needed for SystemReport
use super::{
    AudioDevice, CpuInfo, GpuInfo, MemoryInfo, Monitor, MotherboardInfo, NetworkAdapter,
    PhysicalDisk, ServiceSummary, UsbDevice, Volume,
};

/// Complete system report containing all static system information
/// Used for exporting system information to a file
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemReport {
    /// Report metadata
    pub report_generated_at: String,
    pub syslens_version: String,

    /// System information
    pub device_info: DeviceInfo,
    pub bios_info: BiosInfo,
    pub boot_config: BootConfig,
    pub os_info: OsInfo,
    pub uptime: SystemUptime,
    pub domain_info: DomainInfo,
    pub user_info: UserInfo,

    /// Hardware information
    pub cpu_info: CpuInfo,
    pub memory_info: MemoryInfo,
    pub gpu_info: Vec<GpuInfo>,
    pub motherboard_info: MotherboardInfo,
    pub usb_devices: Vec<UsbDevice>,
    pub audio_devices: Vec<AudioDevice>,
    pub monitors: Vec<Monitor>,

    /// Storage information
    pub physical_disks: Vec<PhysicalDisk>,
    pub volumes: Vec<Volume>,

    /// Network information
    pub network_adapters: Vec<NetworkAdapter>,

    /// Services summary
    pub services_summary: ServiceSummary,
}
