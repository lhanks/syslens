//! System configuration collector

use crate::models::{
    ActivationStatus, BiosInfo, BootConfig, BootMode, DeviceInfo, DomainInfo, DomainRole,
    OsInfo, RestorePoint, RestorePointType, SystemUptime, TpmStatus, UserInfo,
};
use sysinfo::System;
use chrono::{DateTime, Local, Utc};

#[cfg(target_os = "windows")]
use wmi::{COMLibrary, WMIConnection};

#[cfg(target_os = "windows")]
use serde::Deserialize;

/// WMI query structures for Windows
#[cfg(target_os = "windows")]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct Win32ComputerSystem {
    manufacturer: Option<String>,
    model: Option<String>,
    system_type: Option<String>,
    #[serde(rename = "SystemSKUNumber")]
    system_sku_number: Option<String>,
}

#[cfg(target_os = "windows")]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct Win32BIOS {
    manufacturer: Option<String>,
    serial_number: Option<String>,
    #[serde(rename = "SMBIOSBIOSVersion")]
    smbios_bios_version: Option<String>,
    version: Option<String>,
    release_date: Option<String>,
}

#[cfg(target_os = "windows")]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
#[allow(dead_code)]
struct Win32BaseBoard {
    manufacturer: Option<String>,
    product: Option<String>,
    serial_number: Option<String>,
    version: Option<String>,
}

#[cfg(target_os = "windows")]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
#[allow(dead_code)]
struct Win32OperatingSystem {
    caption: Option<String>,
    version: Option<String>,
    build_number: Option<String>,
    install_date: Option<String>,
    os_architecture: Option<String>,
}

#[cfg(target_os = "windows")]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct Win32TPM {
    spec_version: Option<String>,
    is_activated_initial_value: Option<bool>,
    is_enabled_initial_value: Option<bool>,
}

#[cfg(target_os = "windows")]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct SystemRestorePoint {
    sequence_number: Option<u32>,
    description: Option<String>,
    restore_point_type: Option<u32>,
    creation_time: Option<String>,
}

/// Collector for system configuration information
pub struct SystemCollector;

// Thread-local WMI connection for Windows
#[cfg(target_os = "windows")]
thread_local! {
    static WMI_CON: std::cell::RefCell<Option<WMIConnection>> = const { std::cell::RefCell::new(None) };
}

#[cfg(target_os = "windows")]
fn get_wmi_connection() -> Option<WMIConnection> {
    // Try to initialize COM, or assume it's already initialized
    let com = COMLibrary::new()
        .or_else(|_| COMLibrary::without_security())
        .unwrap_or_else(|_| {
            // COM already initialized (e.g., by Tauri or sysinfo) - use existing
            // SAFETY: We've tried other methods, COM must be initialized by the runtime
            unsafe { COMLibrary::assume_initialized() }
        });

    match WMIConnection::new(com) {
        Ok(wmi) => Some(wmi),
        Err(e) => {
            log::warn!("Failed to create WMI connection: {}", e);
            None
        }
    }
}

impl SystemCollector {
    /// Get device identification information
    pub fn get_device_info() -> DeviceInfo {
        DeviceInfo {
            computer_name: System::host_name().unwrap_or_else(|| "Unknown".to_string()),
            device_name: System::host_name().unwrap_or_else(|| "Unknown".to_string()),
            manufacturer: Self::get_manufacturer(),
            model: Self::get_model(),
            system_type: Self::get_system_type(),
            serial_number: Self::get_serial_number(),
            product_id: Self::get_product_id(),
            system_sku: Self::get_system_sku(),
        }
    }

    /// Get system type description
    #[cfg(target_os = "windows")]
    fn get_system_type() -> String {
        Self::get_computer_system_info()
            .and_then(|cs| cs.system_type)
            .unwrap_or_else(|| std::env::consts::ARCH.to_string())
    }

    #[cfg(not(target_os = "windows"))]
    fn get_system_type() -> String {
        std::env::consts::ARCH.to_string()
    }

    /// Get Windows product ID
    #[cfg(target_os = "windows")]
    fn get_product_id() -> Option<String> {
        use windows::Win32::System::Registry::{
            RegOpenKeyExW, RegQueryValueExW, HKEY_LOCAL_MACHINE, KEY_READ, REG_VALUE_TYPE,
        };
        use windows::core::PCWSTR;

        unsafe {
            let mut key = windows::Win32::System::Registry::HKEY::default();
            let subkey: Vec<u16> = "SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\0"
                .encode_utf16()
                .collect();

            if RegOpenKeyExW(
                HKEY_LOCAL_MACHINE,
                PCWSTR::from_raw(subkey.as_ptr()),
                0,
                KEY_READ,
                &mut key,
            ).is_ok() {
                let value_name: Vec<u16> = "ProductId\0".encode_utf16().collect();
                let mut value_type = REG_VALUE_TYPE::default();
                let mut data = [0u8; 512];
                let mut data_size: u32 = 512;

                if RegQueryValueExW(
                    key,
                    PCWSTR::from_raw(value_name.as_ptr()),
                    None,
                    Some(&mut value_type),
                    Some(data.as_mut_ptr()),
                    Some(&mut data_size),
                ).is_ok() {
                    let wide_slice = std::slice::from_raw_parts(
                        data.as_ptr() as *const u16,
                        (data_size as usize / 2).saturating_sub(1),
                    );
                    return Some(String::from_utf16_lossy(wide_slice));
                }
            }
        }
        None
    }

    #[cfg(not(target_os = "windows"))]
    fn get_product_id() -> Option<String> {
        None
    }

    /// Get BIOS/UEFI information
    pub fn get_bios_info() -> BiosInfo {
        #[cfg(target_os = "windows")]
        let tpm_status = Self::get_tpm_status();
        #[cfg(not(target_os = "windows"))]
        let tpm_status = Self::get_tpm_status();

        BiosInfo {
            vendor: Self::get_bios_vendor(),
            version: Self::get_bios_version(),
            firmware_version: Self::get_firmware_version(),
            release_date: Self::get_bios_date(),
            uefi_version: Self::get_uefi_version(),
            secure_boot: Self::is_secure_boot_enabled(),
            tpm_version: Self::get_tpm_version(),
            tpm_status,
        }
    }

    /// Get firmware version string
    #[cfg(target_os = "windows")]
    fn get_firmware_version() -> String {
        Self::get_bios_version()
    }

    #[cfg(not(target_os = "windows"))]
    fn get_firmware_version() -> String {
        std::fs::read_to_string("/sys/class/dmi/id/bios_version")
            .unwrap_or_default()
            .trim()
            .to_string()
    }

    /// Get UEFI version if available
    #[cfg(target_os = "windows")]
    fn get_uefi_version() -> Option<String> {
        if Self::is_uefi() {
            // UEFI version is typically part of the BIOS version on Windows
            Some(Self::get_bios_version())
        } else {
            None
        }
    }

    #[cfg(not(target_os = "windows"))]
    fn get_uefi_version() -> Option<String> {
        if Self::is_uefi() {
            std::fs::read_to_string("/sys/firmware/efi/fw_platform_size")
                .ok()
                .map(|_| "UEFI".to_string())
        } else {
            None
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

    /// Get system restore points (Windows only)
    #[cfg(target_os = "windows")]
    pub fn get_restore_points() -> Vec<RestorePoint> {
        // System restore points are in root\default namespace
        let com = COMLibrary::new()
            .or_else(|_| COMLibrary::without_security())
            .unwrap_or_else(|_| unsafe { COMLibrary::assume_initialized() });

        if let Ok(wmi) = WMIConnection::with_namespace_path("root\\default", com) {
            let results: Result<Vec<SystemRestorePoint>, _> = wmi.raw_query(
                "SELECT SequenceNumber, Description, RestorePointType, CreationTime FROM SystemRestore"
            );

            if let Ok(points) = results {
                return points
                    .into_iter()
                    .map(|p| RestorePoint {
                        sequence_number: p.sequence_number.unwrap_or(0),
                        description: p.description.unwrap_or_else(|| "Unknown".to_string()),
                        restore_point_type: Self::parse_restore_point_type(p.restore_point_type),
                        creation_time: p.creation_time
                            .map(|dt| Self::parse_wmi_datetime(&dt))
                            .unwrap_or_else(|| "Unknown".to_string()),
                    })
                    .collect();
            }
        }
        Vec::new()
    }

    #[cfg(not(target_os = "windows"))]
    pub fn get_restore_points() -> Vec<RestorePoint> {
        Vec::new()
    }

    #[cfg(target_os = "windows")]
    fn parse_restore_point_type(type_code: Option<u32>) -> RestorePointType {
        match type_code {
            Some(0) => RestorePointType::ApplicationInstall,
            Some(1) => RestorePointType::ApplicationUninstall,
            Some(10) => RestorePointType::DeviceDriverInstall,
            Some(12) => RestorePointType::ModifySettings,
            Some(13) => RestorePointType::CancelledOperation,
            Some(14) => RestorePointType::BackupRecovery,
            Some(16) => RestorePointType::ManualCheckpoint,
            Some(17) => RestorePointType::WindowsUpdate,
            _ => RestorePointType::Unknown,
        }
    }

    #[cfg(target_os = "windows")]
    fn parse_wmi_datetime(wmi_dt: &str) -> String {
        // WMI datetime format: 20231015123456.123456+000
        if wmi_dt.len() >= 14 {
            let year = &wmi_dt[0..4];
            let month = &wmi_dt[4..6];
            let day = &wmi_dt[6..8];
            let hour = &wmi_dt[8..10];
            let minute = &wmi_dt[10..12];
            let second = &wmi_dt[12..14];
            format!("{}-{}-{} {}:{}:{}", year, month, day, hour, minute, second)
        } else {
            wmi_dt.to_string()
        }
    }

    // Platform-specific helper methods

    #[cfg(target_os = "windows")]
    fn get_computer_system_info() -> Option<Win32ComputerSystem> {
        let wmi = get_wmi_connection()?;
        let results: Vec<Win32ComputerSystem> = wmi
            .raw_query("SELECT Manufacturer, Model, SystemType, SystemSKUNumber FROM Win32_ComputerSystem")
            .ok()?;
        results.into_iter().next()
    }

    #[cfg(target_os = "windows")]
    fn get_bios_wmi_info() -> Option<Win32BIOS> {
        let wmi = get_wmi_connection()?;
        let results: Vec<Win32BIOS> = wmi
            .raw_query("SELECT Manufacturer, SerialNumber, SMBIOSBIOSVersion, Version, ReleaseDate FROM Win32_BIOS")
            .ok()?;
        results.into_iter().next()
    }

    #[cfg(target_os = "windows")]
    fn get_manufacturer() -> String {
        Self::get_computer_system_info()
            .and_then(|cs| cs.manufacturer)
            .unwrap_or_else(|| "Unknown".to_string())
    }

    #[cfg(not(target_os = "windows"))]
    fn get_manufacturer() -> String {
        std::fs::read_to_string("/sys/class/dmi/id/sys_vendor")
            .unwrap_or_else(|_| "Unknown".to_string())
            .trim()
            .to_string()
    }

    #[cfg(target_os = "windows")]
    fn get_model() -> String {
        Self::get_computer_system_info()
            .and_then(|cs| cs.model)
            .unwrap_or_else(|| "Unknown".to_string())
    }

    #[cfg(not(target_os = "windows"))]
    fn get_model() -> String {
        std::fs::read_to_string("/sys/class/dmi/id/product_name")
            .unwrap_or_else(|_| "Unknown".to_string())
            .trim()
            .to_string()
    }

    #[cfg(target_os = "windows")]
    fn get_system_sku() -> Option<String> {
        Self::get_computer_system_info()
            .and_then(|cs| cs.system_sku_number)
            .filter(|s| !s.is_empty())
    }

    #[cfg(not(target_os = "windows"))]
    fn get_system_sku() -> Option<String> {
        std::fs::read_to_string("/sys/class/dmi/id/product_sku")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }

    #[cfg(target_os = "windows")]
    fn get_serial_number() -> String {
        Self::get_bios_wmi_info()
            .and_then(|bios| bios.serial_number)
            .unwrap_or_default()
    }

    #[cfg(not(target_os = "windows"))]
    fn get_serial_number() -> String {
        std::fs::read_to_string("/sys/class/dmi/id/product_serial")
            .unwrap_or_default()
            .trim()
            .to_string()
    }

    #[cfg(target_os = "windows")]
    fn get_bios_vendor() -> String {
        Self::get_bios_wmi_info()
            .and_then(|bios| bios.manufacturer)
            .unwrap_or_else(|| "Unknown".to_string())
    }

    #[cfg(not(target_os = "windows"))]
    fn get_bios_vendor() -> String {
        std::fs::read_to_string("/sys/class/dmi/id/bios_vendor")
            .unwrap_or_else(|_| "Unknown".to_string())
            .trim()
            .to_string()
    }

    #[cfg(target_os = "windows")]
    fn get_bios_version() -> String {
        Self::get_bios_wmi_info()
            .and_then(|bios| bios.smbios_bios_version.or(bios.version))
            .unwrap_or_default()
    }

    #[cfg(not(target_os = "windows"))]
    fn get_bios_version() -> String {
        std::fs::read_to_string("/sys/class/dmi/id/bios_version")
            .unwrap_or_default()
            .trim()
            .to_string()
    }

    #[cfg(target_os = "windows")]
    fn get_bios_date() -> String {
        Self::get_bios_wmi_info()
            .and_then(|bios| bios.release_date)
            .map(|date| Self::parse_wmi_date(&date))
            .unwrap_or_default()
    }

    #[cfg(target_os = "windows")]
    fn parse_wmi_date(wmi_date: &str) -> String {
        // WMI date format: 20231015000000.000000+000
        if wmi_date.len() >= 8 {
            let year = &wmi_date[0..4];
            let month = &wmi_date[4..6];
            let day = &wmi_date[6..8];
            format!("{}-{}-{}", year, month, day)
        } else {
            wmi_date.to_string()
        }
    }

    #[cfg(not(target_os = "windows"))]
    fn get_bios_date() -> String {
        std::fs::read_to_string("/sys/class/dmi/id/bios_date")
            .unwrap_or_default()
            .trim()
            .to_string()
    }

    #[cfg(target_os = "windows")]
    fn is_uefi() -> bool {
        // Check if system is UEFI boot by looking for EFI system partition info in firmware
        use windows::Win32::System::SystemInformation::GetFirmwareType;
        use windows::Win32::System::SystemInformation::FIRMWARE_TYPE;

        let mut firmware_type = FIRMWARE_TYPE::default();
        unsafe {
            if GetFirmwareType(&mut firmware_type).is_ok() {
                // FirmwareTypeUefi = 2
                return firmware_type.0 == 2;
            }
        }
        // Fallback: check for EFI loader
        std::path::Path::new("C:\\Windows\\System32\\winload.efi").exists()
    }

    #[cfg(not(target_os = "windows"))]
    fn is_uefi() -> bool {
        std::path::Path::new("/sys/firmware/efi").exists()
    }

    #[cfg(target_os = "windows")]
    fn is_secure_boot_enabled() -> bool {
        use windows::Win32::System::Registry::{
            RegOpenKeyExW, RegQueryValueExW, HKEY_LOCAL_MACHINE, KEY_READ, REG_VALUE_TYPE,
        };
        use windows::core::PCWSTR;

        unsafe {
            let mut key = windows::Win32::System::Registry::HKEY::default();
            let subkey: Vec<u16> = "SYSTEM\\CurrentControlSet\\Control\\SecureBoot\\State\0"
                .encode_utf16()
                .collect();

            if RegOpenKeyExW(
                HKEY_LOCAL_MACHINE,
                PCWSTR::from_raw(subkey.as_ptr()),
                0,
                KEY_READ,
                &mut key,
            ).is_ok() {
                let value_name: Vec<u16> = "UEFISecureBootEnabled\0".encode_utf16().collect();
                let mut value_type = REG_VALUE_TYPE::default();
                let mut data: u32 = 0;
                let mut data_size: u32 = std::mem::size_of::<u32>() as u32;

                if RegQueryValueExW(
                    key,
                    PCWSTR::from_raw(value_name.as_ptr()),
                    None,
                    Some(&mut value_type),
                    Some(&mut data as *mut u32 as *mut u8),
                    Some(&mut data_size),
                ).is_ok() {
                    return data == 1;
                }
            }
        }
        false
    }

    #[cfg(not(target_os = "windows"))]
    fn is_secure_boot_enabled() -> bool {
        // Check EFI variable for secure boot status
        std::fs::read_to_string("/sys/firmware/efi/efivars/SecureBoot-8be4df61-93ca-11d2-aa0d-00e098032b8c")
            .map(|s| s.contains('\x01'))
            .unwrap_or(false)
    }

    #[cfg(target_os = "windows")]
    fn get_tpm_info() -> (Option<String>, TpmStatus) {
        // Try to query TPM via WMI in the root\cimv2\Security\MicrosoftTpm namespace
        let com = COMLibrary::new()
            .or_else(|_| COMLibrary::without_security())
            .unwrap_or_else(|_| unsafe { COMLibrary::assume_initialized() });

        if let Ok(wmi) = WMIConnection::with_namespace_path("root\\cimv2\\Security\\MicrosoftTpm", com) {
            let results: Result<Vec<Win32TPM>, _> = wmi.raw_query("SELECT SpecVersion, IsActivated_InitialValue, IsEnabled_InitialValue FROM Win32_Tpm");
            if let Ok(mut tpms) = results {
                if let Some(tpm) = tpms.pop() {
                    let status = match (tpm.is_enabled_initial_value, tpm.is_activated_initial_value) {
                        (Some(true), Some(true)) => TpmStatus::Enabled,
                        (Some(true), _) => TpmStatus::Enabled,
                        (Some(false), _) => TpmStatus::Disabled,
                        _ => TpmStatus::Unknown,
                    };
                    return (tpm.spec_version, status);
                }
            }
        }
        (None, TpmStatus::NotPresent)
    }

    #[cfg(target_os = "windows")]
    fn get_tpm_version() -> Option<String> {
        Self::get_tpm_info().0
    }

    #[cfg(not(target_os = "windows"))]
    fn get_tpm_version() -> Option<String> {
        // Check for TPM device
        if std::path::Path::new("/dev/tpm0").exists() {
            Some("2.0".to_string())
        } else if std::path::Path::new("/dev/tpm").exists() {
            Some("1.2".to_string())
        } else {
            None
        }
    }

    #[cfg(target_os = "windows")]
    fn get_tpm_status() -> TpmStatus {
        Self::get_tpm_info().1
    }

    #[cfg(not(target_os = "windows"))]
    fn get_tpm_status() -> TpmStatus {
        if std::path::Path::new("/dev/tpm0").exists() || std::path::Path::new("/dev/tpm").exists() {
            TpmStatus::Enabled
        } else {
            TpmStatus::NotPresent
        }
    }

    #[cfg(target_os = "windows")]
    fn get_boot_device() -> String {
        use windows::Win32::System::Registry::{
            RegOpenKeyExW, RegQueryValueExW, HKEY_LOCAL_MACHINE, KEY_READ, REG_VALUE_TYPE,
        };
        use windows::core::PCWSTR;

        unsafe {
            let mut key = windows::Win32::System::Registry::HKEY::default();
            let subkey: Vec<u16> = "SYSTEM\\CurrentControlSet\\Control\0"
                .encode_utf16()
                .collect();

            if RegOpenKeyExW(
                HKEY_LOCAL_MACHINE,
                PCWSTR::from_raw(subkey.as_ptr()),
                0,
                KEY_READ,
                &mut key,
            ).is_ok() {
                let value_name: Vec<u16> = "SystemBootDevice\0".encode_utf16().collect();
                let mut value_type = REG_VALUE_TYPE::default();
                let mut data = [0u8; 512];
                let mut data_size: u32 = 512;

                if RegQueryValueExW(
                    key,
                    PCWSTR::from_raw(value_name.as_ptr()),
                    None,
                    Some(&mut value_type),
                    Some(data.as_mut_ptr()),
                    Some(&mut data_size),
                ).is_ok() {
                    // Convert wide string to Rust string
                    let wide_slice = std::slice::from_raw_parts(
                        data.as_ptr() as *const u16,
                        (data_size as usize / 2).saturating_sub(1),
                    );
                    return String::from_utf16_lossy(wide_slice);
                }
            }
        }
        "\\Device\\HarddiskVolume1".to_string()
    }

    #[cfg(not(target_os = "windows"))]
    fn get_boot_device() -> String {
        std::fs::read_to_string("/proc/cmdline")
            .ok()
            .and_then(|cmdline| {
                cmdline
                    .split_whitespace()
                    .find(|s| s.starts_with("root="))
                    .map(|s| s.strip_prefix("root=").unwrap_or("").to_string())
            })
            .unwrap_or_else(|| "/dev/sda1".to_string())
    }

    #[cfg(target_os = "windows")]
    fn is_restart_pending() -> bool {
        use windows::Win32::System::Registry::{
            RegOpenKeyExW, HKEY_LOCAL_MACHINE, KEY_READ,
        };
        use windows::core::PCWSTR;

        // Check common registry keys that indicate pending restart
        let keys_to_check = [
            "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Component Based Servicing\\RebootPending",
            "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\WindowsUpdate\\Auto Update\\RebootRequired",
            "SYSTEM\\CurrentControlSet\\Control\\Session Manager\\PendingFileRenameOperations",
        ];

        unsafe {
            for key_path in keys_to_check {
                let mut key = windows::Win32::System::Registry::HKEY::default();
                let subkey: Vec<u16> = format!("{}\0", key_path).encode_utf16().collect();

                if RegOpenKeyExW(
                    HKEY_LOCAL_MACHINE,
                    PCWSTR::from_raw(subkey.as_ptr()),
                    0,
                    KEY_READ,
                    &mut key,
                ).is_ok() {
                    return true;
                }
            }
        }
        false
    }

    #[cfg(not(target_os = "windows"))]
    fn is_restart_pending() -> bool {
        // Check if reboot is required (e.g., after kernel update on Debian/Ubuntu)
        std::path::Path::new("/var/run/reboot-required").exists()
    }

    #[cfg(target_os = "windows")]
    fn is_admin() -> bool {
        use windows::Win32::Security::{
            GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY,
        };
        use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
        use windows::Win32::Foundation::HANDLE;

        unsafe {
            let mut token_handle = HANDLE::default();
            if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token_handle).is_ok() {
                let mut elevation = TOKEN_ELEVATION::default();
                let mut return_length: u32 = 0;

                if GetTokenInformation(
                    token_handle,
                    TokenElevation,
                    Some(&mut elevation as *mut _ as *mut std::ffi::c_void),
                    std::mem::size_of::<TOKEN_ELEVATION>() as u32,
                    &mut return_length,
                ).is_ok() {
                    return elevation.TokenIsElevated != 0;
                }
            }
        }
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
