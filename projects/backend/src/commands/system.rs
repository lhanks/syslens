//! System-related Tauri commands

use crate::collectors::{HardwareCollector, NetworkCollector, ServiceCollector, StorageCollector, SystemCollector};
use crate::models::{
    BiosInfo, BootConfig, DeviceInfo, DomainInfo, OsInfo, RestorePoint, SystemReport, SystemUptime, UserInfo,
};
use chrono::Utc;

/// Get device identification information
#[tauri::command]
pub fn get_device_info() -> DeviceInfo {
    log::debug!("Command: get_device_info");
    SystemCollector::get_device_info()
}

/// Get BIOS/UEFI information
#[tauri::command]
pub fn get_bios_info() -> BiosInfo {
    log::debug!("Command: get_bios_info");
    SystemCollector::get_bios_info()
}

/// Get boot configuration
#[tauri::command]
pub fn get_boot_config() -> BootConfig {
    log::debug!("Command: get_boot_config");
    SystemCollector::get_boot_config()
}

/// Get operating system information
#[tauri::command]
pub fn get_os_info() -> OsInfo {
    log::debug!("Command: get_os_info");
    SystemCollector::get_os_info()
}

/// Get system uptime and related info
#[tauri::command]
pub fn get_uptime() -> SystemUptime {
    log::trace!("Command: get_uptime");
    SystemCollector::get_uptime()
}

/// Get domain/workgroup information
#[tauri::command]
pub fn get_domain_info() -> DomainInfo {
    log::debug!("Command: get_domain_info");
    SystemCollector::get_domain_info()
}

/// Get current user information
#[tauri::command]
pub fn get_user_info() -> UserInfo {
    log::debug!("Command: get_user_info");
    SystemCollector::get_user_info()
}

/// Get system restore points
#[tauri::command]
pub fn get_restore_points() -> Vec<RestorePoint> {
    log::debug!("Command: get_restore_points");
    SystemCollector::get_restore_points()
}

/// Generate a complete system report
/// Collects all static system information into a single report
#[tauri::command]
pub fn generate_system_report() -> SystemReport {
    log::info!("Command: generate_system_report");

    SystemReport {
        report_generated_at: Utc::now().to_rfc3339(),
        syslens_version: env!("CARGO_PKG_VERSION").to_string(),

        // System information
        device_info: SystemCollector::get_device_info(),
        bios_info: SystemCollector::get_bios_info(),
        boot_config: SystemCollector::get_boot_config(),
        os_info: SystemCollector::get_os_info(),
        uptime: SystemCollector::get_uptime(),
        domain_info: SystemCollector::get_domain_info(),
        user_info: SystemCollector::get_user_info(),

        // Hardware information
        cpu_info: HardwareCollector::get_cpu_info(),
        memory_info: HardwareCollector::get_memory_info(),
        gpu_info: HardwareCollector::get_gpu_info(),
        motherboard_info: HardwareCollector::get_motherboard_info(),
        usb_devices: HardwareCollector::get_usb_devices(),
        audio_devices: HardwareCollector::get_audio_devices(),
        monitors: HardwareCollector::get_monitors(),

        // Storage information
        physical_disks: StorageCollector::get_physical_disks(),
        volumes: StorageCollector::get_volumes(),

        // Network information
        network_adapters: NetworkCollector::get_adapters(),

        // Services summary
        services_summary: ServiceCollector::get_service_summary(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_device_info() {
        let info = get_device_info();
        assert!(!info.computer_name.is_empty());
    }

    #[test]
    fn test_get_uptime() {
        let uptime = get_uptime();
        assert!(uptime.uptime_seconds > 0);
    }
}
