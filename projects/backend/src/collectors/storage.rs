//! Storage information collector

use crate::models::{
    DiskHealth, DiskPerformance, HealthStatus, InterfaceType, MediaType, NetworkDrive,
    NetworkDriveStatus, Partition, PartitionStyle, PhysicalDisk, Volume,
};
use sysinfo::{Disk, DiskKind, Disks};

/// Collector for storage information
pub struct StorageCollector;

/// Helper struct to hold MSFT_PhysicalDisk data
#[cfg(target_os = "windows")]
#[derive(Debug, Clone)]
struct MsftPhysicalDiskData {
    firmware_version: Option<String>,
    media_type: Option<u16>,
    bus_type: Option<u16>,
    health_status: Option<u16>,
}

impl StorageCollector {
    /// Get physical disk information
    pub fn get_physical_disks() -> Vec<PhysicalDisk> {
        #[cfg(target_os = "windows")]
        {
            Self::get_physical_disks_windows()
        }

        #[cfg(not(target_os = "windows"))]
        {
            Vec::new()
        }
    }

    #[cfg(target_os = "windows")]
    fn get_physical_disks_windows() -> Vec<PhysicalDisk> {
        use serde::Deserialize;
        use wmi::{COMLibrary, WMIConnection};

        #[derive(Deserialize, Debug)]
        #[serde(rename_all = "PascalCase")]
        #[allow(dead_code)]
        struct Win32DiskDrive {
            device_id: Option<String>,
            index: Option<u32>,
            model: Option<String>,
            manufacturer: Option<String>,
            serial_number: Option<String>,
            media_type: Option<String>,
            interface_type: Option<String>,
            size: Option<u64>,
            status: Option<String>,
            firmware_revision: Option<String>,
        }

        #[derive(Deserialize, Debug, Clone)]
        #[serde(rename_all = "PascalCase")]
        #[allow(dead_code)]
        struct MsftPhysicalDiskInfo {
            device_id: Option<String>,
            friendly_name: Option<String>,
            firmware_version: Option<String>,
            media_type: Option<u16>,        // 0=Unspecified, 3=HDD, 4=SSD, 5=SCM
            bus_type: Option<u16>,          // 17=NVMe, 11=SATA, 7=USB
            health_status: Option<u16>,
            operational_status: Option<Vec<u16>>,
        }

        let mut physical_disks = Vec::new();

        let com = COMLibrary::new()
            .or_else(|_| COMLibrary::without_security())
            .unwrap_or_else(|_| unsafe { COMLibrary::assume_initialized() });

        // Get MSFT_PhysicalDisk data first (more reliable for NVMe)
        let msft_disks = Self::get_msft_physical_disks();

        if let Ok(wmi_con) = WMIConnection::new(com) {
            // Query Win32_DiskDrive for physical disk info
            let query = "SELECT DeviceID, Index, Model, Manufacturer, SerialNumber, MediaType, InterfaceType, Size, Status, FirmwareRevision FROM Win32_DiskDrive";
            if let Ok(results) = wmi_con.raw_query::<Win32DiskDrive>(query) {
                // Try to get partition style from MSFT_Disk (Storage namespace)
                let partition_styles = Self::get_partition_styles();

                for disk in results {
                    let device_id = disk.index.unwrap_or(0);
                    let model = disk.model.clone().unwrap_or_default();

                    // Try to find matching MSFT_PhysicalDisk data
                    let msft_data = msft_disks.get(&device_id);

                    // Detect media type from model name, interface, and MSFT data
                    let interface_str = disk.interface_type.clone().unwrap_or_default();
                    let mut media_type = Self::detect_media_type_from_info(
                        &model,
                        &interface_str,
                        disk.media_type.as_deref(),
                    );

                    // Override with MSFT media type if available (more reliable)
                    if let Some(msft) = msft_data {
                        if let Some(mt) = msft.media_type {
                            media_type = match mt {
                                3 => MediaType::HDD,
                                4 => MediaType::SSD,
                                _ => media_type,
                            };
                        }
                        // Check bus type for NVMe
                        if let Some(bus) = msft.bus_type {
                            if bus == 17 {
                                media_type = MediaType::NVMe;
                            }
                        }
                    }

                    // Map interface type
                    let mut interface_type = match interface_str.to_uppercase().as_str() {
                        s if s.contains("NVME") => InterfaceType::NVMe,
                        "SATA" | "IDE" => InterfaceType::SATA,
                        "USB" => InterfaceType::USB,
                        "SCSI" => {
                            if model.to_uppercase().contains("NVME") || media_type == MediaType::NVMe {
                                InterfaceType::NVMe
                            } else {
                                InterfaceType::SCSI
                            }
                        }
                        _ => InterfaceType::Unknown,
                    };

                    // Override with MSFT bus type if more accurate
                    if let Some(msft) = msft_data {
                        if let Some(bus) = msft.bus_type {
                            interface_type = match bus {
                                17 => InterfaceType::NVMe,
                                11 => InterfaceType::SATA,
                                7 => InterfaceType::USB,
                                _ => interface_type,
                            };
                        }
                    }

                    // Get partition style for this disk
                    let partition_style = partition_styles
                        .get(&device_id)
                        .cloned()
                        .unwrap_or(PartitionStyle::GPT);

                    // Extract manufacturer from model if not provided
                    let manufacturer = disk.manufacturer.clone()
                        .filter(|m| !m.is_empty() && m != "(Standard disk drives)")
                        .unwrap_or_else(|| Self::extract_manufacturer(&model));

                    // Get firmware - prefer MSFT data over Win32
                    let firmware = msft_data
                        .and_then(|m| m.firmware_version.clone())
                        .filter(|s| !s.trim().is_empty())
                        .or_else(|| disk.firmware_revision.clone().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()))
                        .unwrap_or_else(|| "N/A".to_string());

                    // Get status - check MSFT health status
                    let status = if let Some(msft) = msft_data {
                        match msft.health_status {
                            Some(0) => "Healthy".to_string(),
                            Some(1) => "Warning".to_string(),
                            Some(2) => "Unhealthy".to_string(),
                            _ => disk.status.clone()
                                .map(|s| s.trim().to_string())
                                .filter(|s| !s.is_empty())
                                .unwrap_or_else(|| "OK".to_string()),
                        }
                    } else {
                        disk.status.clone()
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .unwrap_or_else(|| "OK".to_string())
                    };

                    physical_disks.push(PhysicalDisk {
                        device_id,
                        model: model.trim().to_string(),
                        manufacturer,
                        serial_number: disk.serial_number.clone()
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .unwrap_or_default(),
                        media_type,
                        interface_type,
                        size_bytes: disk.size.unwrap_or(0),
                        partition_style,
                        status,
                        firmware,
                    });
                }
            }
        }

        physical_disks
    }

    #[cfg(target_os = "windows")]
    fn get_msft_physical_disks() -> std::collections::HashMap<u32, MsftPhysicalDiskData> {
        use serde::Deserialize;
        use wmi::{COMLibrary, WMIConnection};

        #[derive(Deserialize, Debug)]
        #[serde(rename_all = "PascalCase")]
        #[allow(dead_code)]
        struct MsftPhysicalDisk {
            device_id: Option<String>,
            friendly_name: Option<String>,
            firmware_version: Option<String>,
            media_type: Option<u16>,
            bus_type: Option<u16>,
            health_status: Option<u16>,
        }

        let mut disks = std::collections::HashMap::new();

        let com = COMLibrary::new()
            .or_else(|_| COMLibrary::without_security())
            .unwrap_or_else(|_| unsafe { COMLibrary::assume_initialized() });

        if let Ok(wmi_con) = WMIConnection::with_namespace_path("ROOT\\Microsoft\\Windows\\Storage", com) {
            let query = "SELECT DeviceId, FriendlyName, FirmwareVersion, MediaType, BusType, HealthStatus FROM MSFT_PhysicalDisk";
            if let Ok(results) = wmi_con.raw_query::<MsftPhysicalDisk>(query) {
                for disk in results {
                    if let Some(id_str) = &disk.device_id {
                        if let Ok(id) = id_str.parse::<u32>() {
                            disks.insert(id, MsftPhysicalDiskData {
                                firmware_version: disk.firmware_version,
                                media_type: disk.media_type,
                                bus_type: disk.bus_type,
                                health_status: disk.health_status,
                            });
                        }
                    }
                }
            }
        }

        disks
    }

    #[cfg(target_os = "windows")]
    fn get_partition_styles() -> std::collections::HashMap<u32, PartitionStyle> {
        use serde::Deserialize;
        use wmi::{COMLibrary, WMIConnection};

        #[derive(Deserialize, Debug)]
        #[serde(rename_all = "PascalCase")]
        struct MsftDisk {
            number: Option<u32>,
            partition_style: Option<u32>,
        }

        let mut styles = std::collections::HashMap::new();

        let com = COMLibrary::new()
            .or_else(|_| COMLibrary::without_security())
            .unwrap_or_else(|_| unsafe { COMLibrary::assume_initialized() });

        // Connect to Storage namespace for MSFT_Disk
        if let Ok(wmi_con) = WMIConnection::with_namespace_path("ROOT\\Microsoft\\Windows\\Storage", com) {
            let query = "SELECT Number, PartitionStyle FROM MSFT_Disk";
            if let Ok(results) = wmi_con.raw_query::<MsftDisk>(query) {
                for disk in results {
                    if let Some(number) = disk.number {
                        let style = match disk.partition_style {
                            Some(1) => PartitionStyle::MBR,
                            Some(2) => PartitionStyle::GPT,
                            _ => PartitionStyle::RAW,
                        };
                        styles.insert(number, style);
                    }
                }
            }
        }

        styles
    }

    #[cfg(target_os = "windows")]
    fn detect_media_type_from_info(model: &str, interface: &str, media_type_str: Option<&str>) -> MediaType {
        let model_upper = model.to_uppercase();
        let interface_upper = interface.to_uppercase();

        // Check for NVMe first
        if model_upper.contains("NVME") || interface_upper.contains("NVME") {
            return MediaType::NVMe;
        }

        // Check for SSD indicators in model name
        if model_upper.contains("SSD") || model_upper.contains("SOLID STATE") {
            return MediaType::SSD;
        }

        // Check media type string from WMI
        if let Some(mt) = media_type_str {
            let mt_upper = mt.to_uppercase();
            if mt_upper.contains("REMOVABLE") {
                return MediaType::Removable;
            }
            if mt_upper.contains("FIXED") {
                // Could be HDD or SSD - check for known SSD brands
                let ssd_brands = ["SAMSUNG", "CRUCIAL", "SANDISK", "WD_BLACK", "KINGSTON", "INTEL", "MICRON", "SK HYNIX"];
                for brand in ssd_brands {
                    if model_upper.contains(brand) {
                        return MediaType::SSD;
                    }
                }
            }
        }

        // Default to Unknown if we can't determine
        MediaType::Unknown
    }

    #[cfg(target_os = "windows")]
    fn extract_manufacturer(model: &str) -> String {
        let model_upper = model.to_uppercase();
        let known_manufacturers = [
            ("SAMSUNG", "Samsung"),
            ("WD", "Western Digital"),
            ("WESTERN DIGITAL", "Western Digital"),
            ("SEAGATE", "Seagate"),
            ("TOSHIBA", "Toshiba"),
            ("CRUCIAL", "Crucial"),
            ("SANDISK", "SanDisk"),
            ("KINGSTON", "Kingston"),
            ("INTEL", "Intel"),
            ("MICRON", "Micron"),
            ("SK HYNIX", "SK Hynix"),
            ("HYNIX", "SK Hynix"),
            ("PHISON", "Phison"),
            ("ADATA", "ADATA"),
            ("CORSAIR", "Corsair"),
            ("PNY", "PNY"),
            ("TRANSCEND", "Transcend"),
            ("PATRIOT", "Patriot"),
            ("TEAM", "Team Group"),
            ("SILICON POWER", "Silicon Power"),
        ];

        for (pattern, name) in known_manufacturers {
            if model_upper.contains(pattern) {
                return name.to_string();
            }
        }

        String::new()
    }

    /// Get partitions for a disk
    pub fn get_partitions(_disk_id: u32) -> Vec<Partition> {
        // Would need platform-specific implementation
        Vec::new()
    }

    /// Get all volumes (logical drives)
    pub fn get_volumes() -> Vec<Volume> {
        let disks = Disks::new_with_refreshed_list();
        let mut volumes = Vec::new();

        for disk in disks.list() {
            let total = disk.total_space();
            let available = disk.available_space();
            let used = total.saturating_sub(available);
            let percent_used = if total > 0 {
                (used as f32 / total as f32) * 100.0
            } else {
                0.0
            };

            // Extract drive letter from mount point
            let mount_point = disk.mount_point().to_string_lossy();
            let drive_letter = mount_point.chars().next().filter(|c| c.is_ascii_alphabetic());

            let volume = Volume {
                drive_letter,
                label: disk.name().to_string_lossy().to_string(),
                file_system: disk.file_system().to_string_lossy().to_string(),
                total_bytes: total,
                free_bytes: available,
                used_bytes: used,
                percent_used,
                volume_serial: String::new(),
                is_compressed: false,
                is_encrypted: false,
                is_system: Self::is_system_drive(disk),
                is_boot: Self::is_boot_drive(disk),
            };
            volumes.push(volume);
        }

        volumes
    }

    /// Get disk health information (S.M.A.R.T.)
    pub fn get_disk_health(disk_id: u32) -> DiskHealth {
        #[cfg(target_os = "windows")]
        {
            Self::get_disk_health_windows(disk_id)
        }

        #[cfg(not(target_os = "windows"))]
        {
            DiskHealth {
                device_id: disk_id,
                status: HealthStatus::Unknown,
                temperature_celsius: None,
                power_on_hours: None,
                power_cycles: None,
                wear_level_percent: None,
                smart_attributes: Vec::new(),
            }
        }
    }

    #[cfg(target_os = "windows")]
    fn get_disk_health_windows(disk_id: u32) -> DiskHealth {
        use serde::Deserialize;
        use wmi::{COMLibrary, WMIConnection};

        #[derive(Deserialize, Debug)]
        #[serde(rename_all = "PascalCase")]
        #[allow(dead_code)]
        struct MsftPhysicalDisk {
            device_id: Option<String>,
            health_status: Option<u16>,       // 0=Healthy, 1=Warning, 2=Unhealthy
            operational_status: Option<Vec<u16>>,
        }

        #[derive(Deserialize, Debug)]
        #[serde(rename_all = "PascalCase")]
        #[allow(dead_code)]
        struct MsftStorageReliabilityCounter {
            device_id: Option<String>,
            temperature: Option<u16>,         // Kelvin
            wear: Option<u8>,                 // Percentage used (0-100)
            power_on_hours: Option<u32>,
            start_stop_cycle_count: Option<u32>,
        }

        let mut health = DiskHealth {
            device_id: disk_id,
            status: HealthStatus::Unknown,
            temperature_celsius: None,
            power_on_hours: None,
            power_cycles: None,
            wear_level_percent: None,
            smart_attributes: Vec::new(),
        };

        let com = COMLibrary::new()
            .or_else(|_| COMLibrary::without_security())
            .unwrap_or_else(|_| unsafe { COMLibrary::assume_initialized() });

        // Try to get health from MSFT_PhysicalDisk
        if let Ok(wmi_con) = WMIConnection::with_namespace_path("ROOT\\Microsoft\\Windows\\Storage", com) {
            // Get health status
            let query = format!(
                "SELECT DeviceId, HealthStatus, OperationalStatus FROM MSFT_PhysicalDisk WHERE DeviceId = '{}'",
                disk_id
            );
            if let Ok(results) = wmi_con.raw_query::<MsftPhysicalDisk>(&query) {
                if let Some(disk) = results.into_iter().next() {
                    health.status = match disk.health_status {
                        Some(0) => HealthStatus::Good,
                        Some(1) => HealthStatus::Warning,
                        Some(2) => HealthStatus::Critical,
                        _ => HealthStatus::Unknown,
                    };
                }
            }

            // Try to get reliability counters (requires admin)
            let query = format!(
                "SELECT DeviceId, Temperature, Wear, PowerOnHours, StartStopCycleCount FROM MSFT_StorageReliabilityCounter WHERE DeviceId = '{}'",
                disk_id
            );
            if let Ok(results) = wmi_con.raw_query::<MsftStorageReliabilityCounter>(&query) {
                if let Some(counter) = results.into_iter().next() {
                    // Temperature is in Kelvin, convert to Celsius
                    if let Some(temp_k) = counter.temperature {
                        if temp_k > 273 {
                            health.temperature_celsius = Some((temp_k as i32) - 273);
                        }
                    }
                    if let Some(wear) = counter.wear {
                        health.wear_level_percent = Some(wear);
                    }
                    if let Some(hours) = counter.power_on_hours {
                        health.power_on_hours = Some(hours as u64);
                    }
                    if let Some(cycles) = counter.start_stop_cycle_count {
                        health.power_cycles = Some(cycles as u64);
                    }
                }
            }
        }

        health
    }

    /// Get disk performance metrics
    pub fn get_disk_performance() -> Vec<DiskPerformance> {
        // Would need platform-specific implementation (performance counters)
        Vec::new()
    }

    /// Get mapped network drives
    pub fn get_network_drives() -> Vec<NetworkDrive> {
        #[cfg(target_os = "windows")]
        {
            Self::get_network_drives_windows()
        }

        #[cfg(not(target_os = "windows"))]
        {
            Vec::new()
        }
    }

    #[cfg(target_os = "windows")]
    fn get_network_drives_windows() -> Vec<NetworkDrive> {
        use serde::Deserialize;
        use wmi::{COMLibrary, WMIConnection};

        #[derive(Deserialize, Debug)]
        #[serde(rename_all = "PascalCase")]
        #[allow(dead_code)]
        struct Win32LogicalDisk {
            device_id: Option<String>,       // Drive letter (e.g., "Z:")
            drive_type: Option<u32>,         // 4 = Network Drive (used in WHERE clause)
            provider_name: Option<String>,   // UNC path (e.g., "\\\\server\\share")
            volume_name: Option<String>,     // Volume label
            size: Option<u64>,               // Total size in bytes
            free_space: Option<u64>,         // Free space in bytes
        }

        let mut network_drives = Vec::new();

        let com = COMLibrary::new()
            .or_else(|_| COMLibrary::without_security())
            .unwrap_or_else(|_| unsafe { COMLibrary::assume_initialized() });

        if let Ok(wmi_con) = WMIConnection::new(com) {
            // Query for network drives (DriveType = 4)
            let query = "SELECT DeviceID, DriveType, ProviderName, VolumeName, Size, FreeSpace FROM Win32_LogicalDisk WHERE DriveType = 4";
            if let Ok(results) = wmi_con.raw_query::<Win32LogicalDisk>(query) {
                for disk in results {
                    let drive_letter = disk.device_id
                        .map(|d| d.trim_end_matches(':').to_string())
                        .unwrap_or_default();

                    let unc_path = disk.provider_name.unwrap_or_default();

                    // Parse server and share from UNC path (\\server\share)
                    let (server, share_name) = if unc_path.starts_with("\\\\") {
                        let parts: Vec<&str> = unc_path
                            .trim_start_matches("\\\\")
                            .splitn(2, '\\')
                            .collect();
                        (
                            parts.first().unwrap_or(&"").to_string(),
                            parts.get(1).unwrap_or(&"").to_string(),
                        )
                    } else {
                        (String::new(), disk.volume_name.unwrap_or_default())
                    };

                    // Check if drive is accessible
                    let status = if std::path::Path::new(&format!("{}:\\", drive_letter)).exists() {
                        NetworkDriveStatus::Connected
                    } else {
                        NetworkDriveStatus::Disconnected
                    };

                    // Calculate used bytes if both size and free_space are available
                    let used_bytes = match (disk.size, disk.free_space) {
                        (Some(total), Some(free)) => Some(total.saturating_sub(free)),
                        _ => None,
                    };

                    network_drives.push(NetworkDrive {
                        drive_letter,
                        unc_path,
                        server,
                        share_name,
                        status,
                        total_bytes: disk.size,
                        free_bytes: disk.free_space,
                        used_bytes,
                    });
                }
            }
        }

        network_drives
    }

    /// Detect media type from disk kind
    #[allow(dead_code)]
    fn detect_media_type(kind: DiskKind) -> MediaType {
        match kind {
            DiskKind::SSD => MediaType::SSD,
            DiskKind::HDD => MediaType::HDD,
            _ => MediaType::Unknown,
        }
    }

    /// Check if disk is the system drive
    fn is_system_drive(disk: &Disk) -> bool {
        let mount = disk.mount_point().to_string_lossy().to_lowercase();

        #[cfg(target_os = "windows")]
        {
            mount.starts_with("c:")
        }

        #[cfg(not(target_os = "windows"))]
        {
            mount == "/"
        }
    }

    /// Check if disk is the boot drive
    fn is_boot_drive(disk: &Disk) -> bool {
        // On most systems, system drive is also boot drive
        Self::is_system_drive(disk)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_volumes() {
        let volumes = StorageCollector::get_volumes();
        // Should have at least one volume (system drive)
        assert!(!volumes.is_empty());
    }

    #[test]
    fn test_volume_properties() {
        let volumes = StorageCollector::get_volumes();
        if let Some(vol) = volumes.first() {
            assert!(vol.total_bytes > 0);
            assert!(vol.percent_used >= 0.0 && vol.percent_used <= 100.0);
        }
    }
}
