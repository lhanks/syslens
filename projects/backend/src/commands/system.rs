//! System-related Tauri commands

use crate::collectors::SystemCollector;
use crate::models::{
    BiosInfo, BootConfig, DeviceInfo, DomainInfo, OsInfo, SystemUptime, UserInfo,
};

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
