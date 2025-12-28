//! Storage-related Tauri commands

use crate::collectors::StorageCollector;
use crate::models::{
    DiskHealth, DiskPerformance, NetworkDrive, Partition, PhysicalDisk, Volume,
};

/// Get all physical disks
#[tauri::command]
pub fn get_physical_disks() -> Vec<PhysicalDisk> {
    log::debug!("Command: get_physical_disks");
    StorageCollector::get_physical_disks()
}

/// Get partitions for a specific disk
#[tauri::command]
pub fn get_partitions(disk_id: u32) -> Vec<Partition> {
    log::debug!("Command: get_partitions({})", disk_id);
    StorageCollector::get_partitions(disk_id)
}

/// Get all volumes (logical drives)
#[tauri::command]
pub fn get_volumes() -> Vec<Volume> {
    log::debug!("Command: get_volumes");
    StorageCollector::get_volumes()
}

/// Get disk health information (S.M.A.R.T. data)
#[tauri::command]
pub fn get_disk_health(disk_id: u32) -> DiskHealth {
    log::debug!("Command: get_disk_health({})", disk_id);
    StorageCollector::get_disk_health(disk_id)
}

/// Get real-time disk performance metrics
#[tauri::command]
pub fn get_disk_performance() -> Vec<DiskPerformance> {
    log::trace!("Command: get_disk_performance");
    StorageCollector::get_disk_performance()
}

/// Get mapped network drives
#[tauri::command]
pub fn get_network_drives() -> Vec<NetworkDrive> {
    log::debug!("Command: get_network_drives");
    StorageCollector::get_network_drives()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_volumes() {
        let volumes = get_volumes();
        // Should have at least one volume
        assert!(!volumes.is_empty());
    }

    #[test]
    fn test_get_disk_health() {
        let health = get_disk_health(0);
        // Should return valid health object even if no SMART data
        assert!(health.device_id == 0);
    }
}
