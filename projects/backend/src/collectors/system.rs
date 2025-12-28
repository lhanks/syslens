//! System configuration collector

use crate::models::{
    ActivationStatus, BiosInfo, BootConfig, BootMode, DeviceInfo, DomainInfo, DomainRole,
    OsInfo, SystemUptime, TpmStatus, UserInfo,
};
use sysinfo::System;
use chrono::{DateTime, Local, Utc};

/// Collector for system configuration information
pub struct SystemCollector;

impl SystemCollector {
    /// Get device identification information
    pub fn get_device_info() -> DeviceInfo {
        let sys = System::new_all();

        DeviceInfo {
            computer_name: System::host_name().unwrap_or_else(|| "Unknown".to_string()),
            device_name: System::host_name().unwrap_or_else(|| "Unknown".to_string()),
            manufacturer: Self::get_manufacturer(),
            model: Self::get_model(),
            system_type: std::env::consts::ARCH.to_string(),
            serial_number: Self::get_serial_number(),
            product_id: None,
            system_sku: None,
        }
    }

    /// Get BIOS/UEFI information
    pub fn get_bios_info() -> BiosInfo {
        BiosInfo {
            vendor: Self::get_bios_vendor(),
            version: Self::get_bios_version(),
            firmware_version: String::new(),
            release_date: Self::get_bios_date(),
            uefi_version: None,
            secure_boot: Self::is_secure_boot_enabled(),
            tpm_version: Self::get_tpm_version(),
            tpm_status: TpmStatus::Unknown,
        }
    }

    /// Get boot configuration
    pub fn get_boot_config() -> BootConfig {
        let boot_time = System::boot_time();
        let boot_datetime = DateTime::<Utc>::from_timestamp(boot_time as i64, 0)
            .map(|dt| dt.with_timezone(&Local).to_rfc3339())
            .unwrap_or_else(|| "Unknown".to_string());

        BootConfig {
            boot_mode: if Self::is_uefi() { BootMode::UEFI } else { BootMode::Legacy },
            secure_boot_enabled: Self::is_secure_boot_enabled(),
            boot_device: Self::get_boot_device(),
            boot_order: Vec::new(),
            boot_priority: String::new(),
            fast_startup: false,
            hibernation: false,
            last_boot_time: boot_datetime,
            boot_duration_seconds: 0,
        }
    }

    /// Get operating system information
    pub fn get_os_info() -> OsInfo {
        OsInfo {
            name: System::name().unwrap_or_else(|| "Unknown".to_string()),
            version: System::os_version().unwrap_or_else(|| "Unknown".to_string()),
            build: System::kernel_version().unwrap_or_else(|| "Unknown".to_string()),
            architecture: std::env::consts::ARCH.to_string(),
            install_date: String::new(),
            last_update: None,
            activation_status: ActivationStatus::Unknown,
            product_key: None,
        }
    }

    /// Get system uptime
    pub fn get_uptime() -> SystemUptime {
        let boot_time = System::boot_time();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        SystemUptime {
            uptime_seconds: now.saturating_sub(boot_time),
            last_shutdown: None,
            restart_pending: Self::is_restart_pending(),
            sleep_count: 0,
        }
    }

    /// Get domain/workgroup information
    pub fn get_domain_info() -> DomainInfo {
        DomainInfo {
            domain: None,
            workgroup: Some("WORKGROUP".to_string()),
            domain_role: DomainRole::Workstation,
            ad_site: None,
            logon_server: None,
        }
    }

    /// Get current user information
    pub fn get_user_info() -> UserInfo {
        let username = std::env::var("USERNAME")
            .or_else(|_| std::env::var("USER"))
            .unwrap_or_else(|_| "Unknown".to_string());

        let user_profile = std::env::var("USERPROFILE")
            .or_else(|_| std::env::var("HOME"))
            .unwrap_or_else(|_| String::new());

        UserInfo {
            username,
            user_sid: String::new(),
            user_profile,
            is_admin: Self::is_admin(),
            login_time: Local::now().to_rfc3339(),
        }
    }

    // Platform-specific helper methods

    #[cfg(target_os = "windows")]
    fn get_manufacturer() -> String {
        // Would use WMI: Win32_ComputerSystem.Manufacturer
        "Unknown".to_string()
    }

    #[cfg(not(target_os = "windows"))]
    fn get_manufacturer() -> String {
        "Unknown".to_string()
    }

    #[cfg(target_os = "windows")]
    fn get_model() -> String {
        // Would use WMI: Win32_ComputerSystem.Model
        "Unknown".to_string()
    }

    #[cfg(not(target_os = "windows"))]
    fn get_model() -> String {
        "Unknown".to_string()
    }

    #[cfg(target_os = "windows")]
    fn get_serial_number() -> String {
        // Would use WMI: Win32_BIOS.SerialNumber
        String::new()
    }

    #[cfg(not(target_os = "windows"))]
    fn get_serial_number() -> String {
        String::new()
    }

    #[cfg(target_os = "windows")]
    fn get_bios_vendor() -> String {
        // Would use WMI: Win32_BIOS.Manufacturer
        "Unknown".to_string()
    }

    #[cfg(not(target_os = "windows"))]
    fn get_bios_vendor() -> String {
        "Unknown".to_string()
    }

    #[cfg(target_os = "windows")]
    fn get_bios_version() -> String {
        // Would use WMI: Win32_BIOS.Version
        String::new()
    }

    #[cfg(not(target_os = "windows"))]
    fn get_bios_version() -> String {
        String::new()
    }

    #[cfg(target_os = "windows")]
    fn get_bios_date() -> String {
        // Would use WMI: Win32_BIOS.ReleaseDate
        String::new()
    }

    #[cfg(not(target_os = "windows"))]
    fn get_bios_date() -> String {
        String::new()
    }

    #[cfg(target_os = "windows")]
    fn is_uefi() -> bool {
        // Check if system is UEFI boot
        std::path::Path::new("C:\\Windows\\System32\\winload.efi").exists()
    }

    #[cfg(not(target_os = "windows"))]
    fn is_uefi() -> bool {
        std::path::Path::new("/sys/firmware/efi").exists()
    }

    #[cfg(target_os = "windows")]
    fn is_secure_boot_enabled() -> bool {
        // Would check registry or use Windows API
        false
    }

    #[cfg(not(target_os = "windows"))]
    fn is_secure_boot_enabled() -> bool {
        false
    }

    #[cfg(target_os = "windows")]
    fn get_tpm_version() -> Option<String> {
        // Would use WMI: Win32_Tpm
        None
    }

    #[cfg(not(target_os = "windows"))]
    fn get_tpm_version() -> Option<String> {
        None
    }

    #[cfg(target_os = "windows")]
    fn get_boot_device() -> String {
        // Would use bcdedit or registry
        "\\Device\\HarddiskVolume1".to_string()
    }

    #[cfg(not(target_os = "windows"))]
    fn get_boot_device() -> String {
        "/dev/sda1".to_string()
    }

    #[cfg(target_os = "windows")]
    fn is_restart_pending() -> bool {
        // Check registry for pending restart
        false
    }

    #[cfg(not(target_os = "windows"))]
    fn is_restart_pending() -> bool {
        false
    }

    #[cfg(target_os = "windows")]
    fn is_admin() -> bool {
        // Would check if running as administrator
        false
    }

    #[cfg(not(target_os = "windows"))]
    fn is_admin() -> bool {
        unsafe { libc::geteuid() == 0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_device_info() {
        let info = SystemCollector::get_device_info();
        assert!(!info.computer_name.is_empty());
    }

    #[test]
    fn test_get_uptime() {
        let uptime = SystemCollector::get_uptime();
        assert!(uptime.uptime_seconds > 0);
    }

    #[test]
    fn test_get_os_info() {
        let info = SystemCollector::get_os_info();
        assert!(!info.name.is_empty());
    }
}
