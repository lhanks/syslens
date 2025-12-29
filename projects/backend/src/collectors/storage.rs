//! Storage information collector

use crate::models::{
    DiskHealth, DiskPerformance, HealthStatus, MediaType, NetworkDrive,
    NetworkDriveStatus, Partition, PhysicalDisk, Volume,
};
use sysinfo::{Disk, DiskKind, Disks};

/// Collector for storage information
pub struct StorageCollector;

impl StorageCollector {
    /// Get physical disk information
    pub fn get_physical_disks() -> Vec<PhysicalDisk> {
        // sysinfo provides logical disks, not physical
        // For physical disk info, we'd need WMI on Windows
        #[cfg(target_os = "windows")]
        {
            // Would use WMI: Win32_DiskDrive
            Vec::new()
        }

        #[cfg(not(target_os = "windows"))]
        {
            Vec::new()
        }
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
        // S.M.A.R.T. data requires elevated permissions and platform-specific APIs
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

    /// Get disk performance metrics
    pub fn get_disk_performance() -> Vec<DiskPerformance> {
        // Would need platform-specific implementation (performance counters)
        Vec::new()
    }

    /// Get mapped network drives
    pub fn get_network_drives() -> Vec<NetworkDrive> {
        let disks = Disks::new_with_refreshed_list();
        let mut network_drives = Vec::new();

        for disk in disks.list() {
            if disk.is_removable() {
                // Check if it's a network drive
                let mount_point = disk.mount_point().to_string_lossy();
                if mount_point.starts_with("\\\\") {
                    let parts: Vec<&str> = mount_point.trim_start_matches("\\\\").splitn(2, '\\').collect();
                    let server = parts.first().unwrap_or(&"").to_string();
                    let share_name = parts.get(1).unwrap_or(&"").to_string();

                    network_drives.push(NetworkDrive {
                        drive_letter: mount_point.chars().next().map(|c| c.to_string()).unwrap_or_default(),
                        unc_path: mount_point.to_string(),
                        server,
                        share_name,
                        status: NetworkDriveStatus::Connected,
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
