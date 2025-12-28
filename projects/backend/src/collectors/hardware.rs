//! Hardware information collector

use crate::models::{
    AudioDevice, AudioDeviceStatus, AudioDeviceType, CacheInfo, CpuInfo, CpuMetrics,
    GpuAdapterType, GpuInfo, GpuMetrics, MemoryInfo, MemoryMetrics, MemoryModule,
    Monitor, MotherboardInfo, UsbDevice, UsbSpeed,
};
use sysinfo::{Components, Cpu, CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};

/// Collector for hardware information
pub struct HardwareCollector;

impl HardwareCollector {
    /// Get CPU static information
    pub fn get_cpu_info() -> CpuInfo {
        let mut sys = System::new();
        sys.refresh_cpu_specifics(CpuRefreshKind::new().with_frequency());

        let cpus = sys.cpus();
        let cpu = cpus.first();

        CpuInfo {
            name: cpu.map(|c| c.brand().to_string()).unwrap_or_else(|| "Unknown".to_string()),
            manufacturer: cpu.map(|c| c.vendor_id().to_string()).unwrap_or_else(|| "Unknown".to_string()),
            architecture: std::env::consts::ARCH.to_string(),
            family: String::new(),
            model: String::new(),
            stepping: String::new(),
            physical_cores: sys.physical_core_count().unwrap_or(0) as u32,
            logical_processors: cpus.len() as u32,
            base_clock_mhz: cpu.map(|c| c.frequency() as u32).unwrap_or(0),
            max_clock_mhz: cpu.map(|c| c.frequency() as u32).unwrap_or(0),
            cache: CacheInfo {
                l1_data_kb: 0,
                l1_instruction_kb: 0,
                l2_kb: 0,
                l3_kb: 0,
            },
            socket: String::new(),
            tdp_watts: None,
        }
    }

    /// Get real-time CPU metrics
    pub fn get_cpu_metrics() -> CpuMetrics {
        let mut sys = System::new();
        sys.refresh_cpu_all();

        // Need to wait a bit and refresh again for accurate usage (50ms is sufficient)
        std::thread::sleep(std::time::Duration::from_millis(50));
        sys.refresh_cpu_all();

        let cpus = sys.cpus();
        let total_usage: f32 = cpus.iter().map(|c| c.cpu_usage()).sum::<f32>() / cpus.len() as f32;
        let per_core_usage: Vec<f32> = cpus.iter().map(|c| c.cpu_usage()).collect();
        let current_clock = cpus.first().map(|c| c.frequency() as u32).unwrap_or(0);

        // Try to get temperature from components
        let components = Components::new_with_refreshed_list();
        let cpu_temp = components
            .iter()
            .find(|c| c.label().to_lowercase().contains("cpu"))
            .map(|c| c.temperature());

        CpuMetrics {
            total_usage,
            per_core_usage,
            current_clock_mhz: current_clock,
            temperature: cpu_temp,
            power_draw: None,
        }
    }

    /// Get memory static information
    pub fn get_memory_info() -> MemoryInfo {
        #[cfg(target_os = "windows")]
        {
            Self::get_memory_info_windows()
        }

        #[cfg(not(target_os = "windows"))]
        {
            let mut sys = System::new();
            sys.refresh_memory_specifics(MemoryRefreshKind::everything());

            MemoryInfo {
                total_bytes: sys.total_memory(),
                usable_bytes: sys.total_memory(),
                memory_type: "Unknown".to_string(),
                speed_mhz: 0,
                slots_used: 0,
                slots_total: 0,
                max_capacity_bytes: sys.total_memory(),
                modules: Vec::new(),
            }
        }
    }

    #[cfg(target_os = "windows")]
    fn get_memory_info_windows() -> MemoryInfo {
        use wmi::{COMLibrary, WMIConnection};
        use serde::Deserialize;

        #[derive(Deserialize, Debug)]
        #[serde(rename = "Win32_PhysicalMemory")]
        #[serde(rename_all = "PascalCase")]
        struct Win32PhysicalMemory {
            capacity: Option<u64>,
            manufacturer: Option<String>,
            part_number: Option<String>,
            serial_number: Option<String>,
            speed: Option<u32>,
            configured_clock_speed: Option<u32>,
            device_locator: Option<String>,
            memory_type: Option<u32>,
        }

        #[derive(Deserialize, Debug)]
        #[serde(rename = "Win32_PhysicalMemoryArray")]
        #[serde(rename_all = "PascalCase")]
        struct Win32PhysicalMemoryArray {
            max_capacity: Option<u64>,
            memory_devices: Option<u32>,
        }

        let mut modules = Vec::new();
        let mut total_bytes = 0u64;
        let mut memory_type = "Unknown".to_string();
        let mut speed_mhz = 0u32;
        let mut slots_total = 0u32;
        let mut max_capacity_bytes = 0u64;

        // Try to initialize COM, or assume it's already initialized
        let com = COMLibrary::new()
            .or_else(|_| COMLibrary::without_security())
            .unwrap_or_else(|_| unsafe { COMLibrary::assume_initialized() });

        if let Ok(wmi_con) = WMIConnection::new(com) {
            // Get memory modules
            if let Ok(results) = wmi_con.query::<Win32PhysicalMemory>() {
                for module in results {
                    let capacity = module.capacity.unwrap_or(0);
                    total_bytes += capacity;

                    if speed_mhz == 0 {
                        // Try to get the rated speed from WMI first
                        let wmi_speed = module.speed.unwrap_or(0);
                        // Also try to extract rated speed from part number (e.g., "6000" from "FLBD516G6000HC38GBKT")
                        let part_speed = module.part_number.as_ref()
                            .map(|pn| Self::extract_speed_from_part_number(pn))
                            .unwrap_or(0);
                        // Use the higher of the two (part number often has XMP speed, WMI has JEDEC speed)
                        speed_mhz = wmi_speed.max(part_speed);
                    }

                    if memory_type == "Unknown" {
                        memory_type = module.memory_type
                            .map(|t| Self::decode_memory_type(t))
                            .unwrap_or_else(|| "Unknown".to_string());
                    }

                    let part_number = module.part_number
                        .unwrap_or_else(|| "Unknown".to_string())
                        .trim()
                        .to_string();

                    // Get rated speed - prefer part number extraction over WMI for XMP speeds
                    let wmi_speed = module.speed.unwrap_or(0);
                    let part_speed = Self::extract_speed_from_part_number(&part_number);
                    let rated_speed = wmi_speed.max(part_speed);

                    modules.push(MemoryModule {
                        slot: module.device_locator.unwrap_or_else(|| "Unknown".to_string()),
                        capacity_bytes: capacity,
                        manufacturer: module.manufacturer
                            .unwrap_or_else(|| "Unknown".to_string())
                            .trim()
                            .to_string(),
                        part_number,
                        serial_number: module.serial_number
                            .unwrap_or_else(|| "Unknown".to_string())
                            .trim()
                            .to_string(),
                        speed_mhz: rated_speed,
                        configured_speed_mhz: module.configured_clock_speed.unwrap_or(0),
                    });
                }
            }

            // Get memory array info for max capacity and slots
            if let Ok(results) = wmi_con.query::<Win32PhysicalMemoryArray>() {
                if let Some(array) = results.into_iter().next() {
                    max_capacity_bytes = array.max_capacity.unwrap_or(0) * 1024; // Convert KB to bytes
                    slots_total = array.memory_devices.unwrap_or(0);
                }
            }
        }

        MemoryInfo {
            total_bytes,
            usable_bytes: total_bytes,
            memory_type,
            speed_mhz,
            slots_used: modules.len() as u32,
            slots_total,
            max_capacity_bytes,
            modules,
        }
    }

    #[cfg(target_os = "windows")]
    fn decode_memory_type(type_code: u32) -> String {
        match type_code {
            0 => "Unknown".to_string(),
            1 => "Other".to_string(),
            2 => "DRAM".to_string(),
            3 => "Synchronous DRAM".to_string(),
            4 => "Cache DRAM".to_string(),
            5 => "EDO".to_string(),
            6 => "EDRAM".to_string(),
            7 => "VRAM".to_string(),
            8 => "SRAM".to_string(),
            9 => "RAM".to_string(),
            10 => "ROM".to_string(),
            11 => "Flash".to_string(),
            12 => "EEPROM".to_string(),
            13 => "FEPROM".to_string(),
            14 => "EPROM".to_string(),
            15 => "CDRAM".to_string(),
            16 => "3DRAM".to_string(),
            17 => "SDRAM".to_string(),
            18 => "SGRAM".to_string(),
            19 => "RDRAM".to_string(),
            20 => "DDR".to_string(),
            21 => "DDR2".to_string(),
            22 => "DDR2 FB-DIMM".to_string(),
            24 => "DDR3".to_string(),
            25 => "FBD2".to_string(),
            26 => "DDR4".to_string(),
            27 => "LPDDR".to_string(),
            28 => "LPDDR2".to_string(),
            29 => "LPDDR3".to_string(),
            30 => "LPDDR4".to_string(),
            _ => format!("Type {}", type_code),
        }
    }

    /// Extract DDR speed rating from memory part number
    /// Common patterns: "6000" in "FLBD516G6000HC38GBKT", "5600" in "CMK32GX5M2B5600C36"
    #[cfg(target_os = "windows")]
    fn extract_speed_from_part_number(part_number: &str) -> u32 {
        use regex::Regex;

        // Look for common DDR5/DDR4 speed patterns (4-digit numbers in typical speed ranges)
        // DDR5: 4800, 5200, 5600, 6000, 6400, 6800, 7200, 7600, 8000, etc.
        // DDR4: 2133, 2400, 2666, 3000, 3200, 3600, 4000, 4400, etc.
        lazy_static::lazy_static! {
            static ref SPEED_REGEX: Regex = Regex::new(r"(\d{4})").unwrap();
        }

        for cap in SPEED_REGEX.captures_iter(part_number) {
            if let Some(m) = cap.get(1) {
                if let Ok(speed) = m.as_str().parse::<u32>() {
                    // Check if it's in a valid DDR speed range
                    if (2133..=8400).contains(&speed) {
                        return speed;
                    }
                }
            }
        }

        0
    }

    /// Get real-time memory metrics
    pub fn get_memory_metrics() -> MemoryMetrics {
        let mut sys = System::new();
        sys.refresh_memory_specifics(MemoryRefreshKind::everything());

        let total = sys.total_memory();
        let available = sys.available_memory();
        let used = total.saturating_sub(available);

        MemoryMetrics {
            in_use_bytes: used,
            available_bytes: available,
            committed_bytes: used, // Approximation
            cached_bytes: 0, // Not available from sysinfo
            paged_pool_bytes: 0,
            non_paged_pool_bytes: 0,
        }
    }

    /// Get GPU information
    pub fn get_gpu_info() -> Vec<GpuInfo> {
        #[cfg(target_os = "windows")]
        {
            Self::get_gpu_info_windows()
        }

        #[cfg(not(target_os = "windows"))]
        {
            Vec::new()
        }
    }

    #[cfg(target_os = "windows")]
    fn get_gpu_info_windows() -> Vec<GpuInfo> {
        use wmi::{COMLibrary, WMIConnection};
        use serde::Deserialize;

        #[derive(Deserialize, Debug)]
        #[serde(rename = "Win32_VideoController")]
        #[serde(rename_all = "PascalCase")]
        struct Win32VideoController {
            #[serde(rename = "PNPDeviceID")]
            pnp_device_id: Option<String>,
            name: Option<String>,
            adapter_ram: Option<u64>,
            driver_version: Option<String>,
            driver_date: Option<String>,
            video_mode_description: Option<String>,
            current_refresh_rate: Option<u32>,
            adapter_compatibility: Option<String>,
        }

        let mut gpus = Vec::new();

        // Try to initialize COM, or assume it's already initialized
        let com = COMLibrary::new()
            .or_else(|_| COMLibrary::without_security())
            .unwrap_or_else(|_| unsafe { COMLibrary::assume_initialized() });

        if let Ok(wmi_con) = WMIConnection::new(com) {
            if let Ok(results) = wmi_con.query::<Win32VideoController>() {
                for (i, gpu) in results.into_iter().enumerate() {
                    let manufacturer = gpu.adapter_compatibility
                        .as_ref()
                        .and_then(|s| Self::extract_gpu_vendor(s))
                        .unwrap_or_else(|| "Unknown".to_string());

                    let pnp_id = gpu.pnp_device_id.clone();
                    let driver_link = pnp_id.as_ref()
                        .and_then(|id| Self::get_gpu_driver_link(&manufacturer, id));

                    let adapter_type = if gpu.name.as_ref()
                        .map(|n| n.to_lowercase().contains("integrated") || n.to_lowercase().contains("intel uhd"))
                        .unwrap_or(false)
                    {
                        GpuAdapterType::Integrated
                    } else {
                        GpuAdapterType::Discrete
                    };

                    gpus.push(GpuInfo {
                        id: format!("GPU{}", i),
                        name: gpu.name.unwrap_or_else(|| "Unknown".to_string()),
                        manufacturer: manufacturer.clone(),
                        driver_version: gpu.driver_version.unwrap_or_else(|| "Unknown".to_string()),
                        driver_date: gpu.driver_date.unwrap_or_else(|| "Unknown".to_string()),
                        driver_link,
                        vram_bytes: gpu.adapter_ram.unwrap_or(0),
                        current_resolution: gpu.video_mode_description.unwrap_or_else(|| "Unknown".to_string()),
                        refresh_rate_hz: gpu.current_refresh_rate.unwrap_or(0),
                        adapter_type,
                        pnp_device_id: pnp_id,
                    });
                }
            }
        }

        gpus
    }

    #[cfg(target_os = "windows")]
    fn extract_gpu_vendor(adapter_compat: &str) -> Option<String> {
        let lower = adapter_compat.to_lowercase();
        if lower.contains("nvidia") {
            Some("NVIDIA".to_string())
        } else if lower.contains("amd") || lower.contains("ati") {
            Some("AMD".to_string())
        } else if lower.contains("intel") {
            Some("Intel".to_string())
        } else {
            Some(adapter_compat.to_string())
        }
    }

    #[cfg(target_os = "windows")]
    fn get_gpu_driver_link(vendor: &str, _pnp_id: &str) -> Option<String> {
        match vendor.to_lowercase().as_str() {
            v if v.contains("nvidia") => Some("https://www.nvidia.com/Download/index.aspx".to_string()),
            v if v.contains("amd") => Some("https://www.amd.com/en/support".to_string()),
            v if v.contains("intel") => Some("https://www.intel.com/content/www/us/en/download-center/home.html".to_string()),
            _ => None,
        }
    }

    /// Get real-time GPU metrics
    pub fn get_gpu_metrics() -> Vec<GpuMetrics> {
        // Would need NVML for NVIDIA or vendor-specific APIs
        Vec::new()
    }

    /// Get motherboard information
    pub fn get_motherboard_info() -> MotherboardInfo {
        #[cfg(target_os = "windows")]
        {
            Self::get_motherboard_info_windows()
        }

        #[cfg(not(target_os = "windows"))]
        {
            MotherboardInfo {
                manufacturer: Self::get_board_manufacturer(),
                product: Self::get_board_product(),
                version: String::new(),
                serial_number: String::new(),
                chipset: None,
                bios_vendor: None,
                bios_version: None,
                bios_release_date: None,
                support_url: None,
                image_url: None,
            }
        }
    }

    #[cfg(target_os = "windows")]
    fn get_motherboard_info_windows() -> MotherboardInfo {
        use wmi::{COMLibrary, WMIConnection};
        use serde::Deserialize;

        #[derive(Deserialize, Debug)]
        #[serde(rename = "Win32_BaseBoard")]
        #[serde(rename_all = "PascalCase")]
        struct Win32BaseBoard {
            manufacturer: Option<String>,
            product: Option<String>,
            version: Option<String>,
            serial_number: Option<String>,
        }

        #[derive(Deserialize, Debug)]
        #[serde(rename = "Win32_BIOS")]
        #[serde(rename_all = "PascalCase")]
        struct Win32BIOS {
            manufacturer: Option<String>,
            #[serde(rename = "SMBIOSBIOSVersion")]
            smbios_bios_version: Option<String>,
            release_date: Option<String>,
        }

        let mut manufacturer = "Unknown".to_string();
        let mut product = "Unknown".to_string();
        let mut version = String::new();
        let mut serial_number = String::new();
        let mut bios_vendor = None;
        let mut bios_version = None;
        let mut bios_release_date = None;

        // Try to initialize COM, or assume it's already initialized
        let com = COMLibrary::new()
            .or_else(|_| COMLibrary::without_security())
            .unwrap_or_else(|_| unsafe { COMLibrary::assume_initialized() });

        if let Ok(wmi_con) = WMIConnection::new(com) {
            // Get baseboard info
            if let Ok(results) = wmi_con.query::<Win32BaseBoard>() {
                if let Some(board) = results.into_iter().next() {
                    manufacturer = board.manufacturer
                        .unwrap_or_else(|| "Unknown".to_string())
                        .trim()
                        .to_string();
                    product = board.product
                        .unwrap_or_else(|| "Unknown".to_string())
                        .trim()
                        .to_string();
                    version = board.version
                        .unwrap_or_else(|| String::new())
                        .trim()
                        .to_string();
                    serial_number = board.serial_number
                        .unwrap_or_else(|| String::new())
                        .trim()
                        .to_string();
                }
            }

            // Get BIOS info
            if let Ok(results) = wmi_con.query::<Win32BIOS>() {
                if let Some(bios) = results.into_iter().next() {
                    bios_vendor = bios.manufacturer.map(|s| s.trim().to_string());
                    bios_version = bios.smbios_bios_version.map(|s| s.trim().to_string());
                    bios_release_date = bios.release_date.map(|s| {
                        // WMI returns date in format: 20231215000000.000000+000
                        // Extract YYYYMMDD and format as YYYY-MM-DD
                        if s.len() >= 8 {
                            let year = &s[0..4];
                            let month = &s[4..6];
                            let day = &s[6..8];
                            format!("{}-{}-{}", year, month, day)
                        } else {
                            s.trim().to_string()
                        }
                    });
                }
            }
        }

        let support_url = Self::get_motherboard_support_url(&manufacturer, &product);
        let image_url = Self::get_motherboard_image_url(&manufacturer, &product);

        MotherboardInfo {
            manufacturer,
            product,
            version,
            serial_number,
            chipset: None,
            bios_vendor,
            bios_version,
            bios_release_date,
            support_url,
            image_url,
        }
    }

    #[cfg(target_os = "windows")]
    fn get_motherboard_support_url(manufacturer: &str, _product: &str) -> Option<String> {
        let lower = manufacturer.to_lowercase();
        if lower.contains("asus") {
            Some("https://www.asus.com/support/".to_string())
        } else if lower.contains("msi") {
            Some("https://www.msi.com/support".to_string())
        } else if lower.contains("gigabyte") {
            Some("https://www.gigabyte.com/Support".to_string())
        } else if lower.contains("asrock") {
            Some("https://www.asrock.com/support/".to_string())
        } else if lower.contains("evga") {
            Some("https://www.evga.com/support/".to_string())
        } else if lower.contains("dell") {
            Some("https://www.dell.com/support".to_string())
        } else if lower.contains("hp") || lower.contains("hewlett") {
            Some("https://support.hp.com/".to_string())
        } else if lower.contains("lenovo") {
            Some("https://support.lenovo.com/".to_string())
        } else {
            None
        }
    }

    #[cfg(target_os = "windows")]
    fn get_motherboard_image_url(_manufacturer: &str, _product: &str) -> Option<String> {
        // This would require web scraping or API calls to manufacturer websites
        // For now, return None - could be enhanced with actual lookups
        None
    }

    /// Get USB devices
    pub fn get_usb_devices() -> Vec<UsbDevice> {
        // Would need platform-specific implementation
        Vec::new()
    }

    /// Get audio devices
    pub fn get_audio_devices() -> Vec<AudioDevice> {
        // Would need platform-specific implementation
        Vec::new()
    }

    /// Get connected monitors
    #[cfg(target_os = "windows")]
    pub fn get_monitors() -> Vec<Monitor> {
        use wmi::{COMLibrary, WMIConnection};

        #[derive(serde::Deserialize, Debug)]
        #[serde(rename = "Win32_DesktopMonitor")]
        struct Win32DesktopMonitor {
            #[serde(rename = "DeviceID")]
            device_id: Option<String>,
            #[serde(rename = "Name")]
            name: Option<String>,
            #[serde(rename = "MonitorManufacturer")]
            manufacturer: Option<String>,
            #[serde(rename = "ScreenWidth")]
            screen_width: Option<u32>,
            #[serde(rename = "ScreenHeight")]
            screen_height: Option<u32>,
        }

        #[derive(serde::Deserialize, Debug)]
        #[serde(rename = "Win32_VideoController")]
        struct Win32VideoController {
            #[serde(rename = "CurrentHorizontalResolution")]
            current_horizontal_resolution: Option<u32>,
            #[serde(rename = "CurrentVerticalResolution")]
            current_vertical_resolution: Option<u32>,
            #[serde(rename = "CurrentRefreshRate")]
            current_refresh_rate: Option<u32>,
            #[serde(rename = "VideoModeDescription")]
            video_mode_description: Option<String>,
        }

        let mut monitors = Vec::new();

        if let Some(com) = COMLibrary::new().ok() {
            if let Ok(wmi_con) = WMIConnection::new(com) {
                // Get video controller info for refresh rate
                let video_info: Result<Vec<Win32VideoController>, _> = wmi_con.query();
                let (refresh_rate, resolution) = if let Ok(controllers) = &video_info {
                    if let Some(controller) = controllers.first() {
                        let rate = controller.current_refresh_rate.unwrap_or(60);
                        let res = match (controller.current_horizontal_resolution, controller.current_vertical_resolution) {
                            (Some(w), Some(h)) => format!("{}x{}", w, h),
                            _ => controller.video_mode_description.clone().unwrap_or_else(|| "Unknown".to_string()),
                        };
                        (rate, res)
                    } else {
                        (60, "Unknown".to_string())
                    }
                } else {
                    (60, "Unknown".to_string())
                };

                // Get monitor info
                let monitor_info: Result<Vec<Win32DesktopMonitor>, _> = wmi_con.query();
                if let Ok(wmi_monitors) = monitor_info {
                    for (idx, mon) in wmi_monitors.into_iter().enumerate() {
                        let name = mon.name.unwrap_or_else(|| format!("Display {}", idx + 1));

                        // Try to get resolution from monitor, fall back to video controller
                        let mon_resolution = match (mon.screen_width, mon.screen_height) {
                            (Some(w), Some(h)) if w > 0 && h > 0 => format!("{}x{}", w, h),
                            _ => resolution.clone(),
                        };

                        monitors.push(Monitor {
                            id: mon.device_id.unwrap_or_else(|| format!("monitor-{}", idx)),
                            name,
                            manufacturer: mon.manufacturer,
                            resolution: mon_resolution,
                            size_inches: None, // Would need EDID parsing
                            connection: "Unknown".to_string(), // WMI doesn't provide this directly
                            hdr_support: false, // Would need more advanced API
                            refresh_rate_hz: refresh_rate,
                        });
                    }
                }

                // If no monitors from WMI, try to create from video controller
                if monitors.is_empty() {
                    if let Ok(controllers) = video_info {
                        for (idx, controller) in controllers.into_iter().enumerate() {
                            let res = match (controller.current_horizontal_resolution, controller.current_vertical_resolution) {
                                (Some(w), Some(h)) => format!("{}x{}", w, h),
                                _ => controller.video_mode_description.unwrap_or_else(|| "Unknown".to_string()),
                            };

                            monitors.push(Monitor {
                                id: format!("display-{}", idx),
                                name: format!("Display {}", idx + 1),
                                manufacturer: None,
                                resolution: res,
                                size_inches: None,
                                connection: "Unknown".to_string(),
                                hdr_support: false,
                                refresh_rate_hz: controller.current_refresh_rate.unwrap_or(60),
                            });
                        }
                    }
                }
            }
        }

        // Fallback: Use EnumDisplayMonitors if WMI returns nothing
        if monitors.is_empty() {
            monitors = Self::get_monitors_from_gdi();
        }

        monitors
    }

    #[cfg(target_os = "windows")]
    fn get_monitors_from_gdi() -> Vec<Monitor> {
        use windows::Win32::Graphics::Gdi::{
            EnumDisplayDevicesW, EnumDisplaySettingsW, DEVMODEW, DISPLAY_DEVICEW,
            ENUM_CURRENT_SETTINGS,
        };

        const DISPLAY_DEVICE_ACTIVE: u32 = 0x00000001;

        let mut monitors = Vec::new();
        let mut device_idx = 0u32;

        loop {
            let mut display_device = DISPLAY_DEVICEW {
                cb: std::mem::size_of::<DISPLAY_DEVICEW>() as u32,
                ..Default::default()
            };

            let result = unsafe {
                EnumDisplayDevicesW(None, device_idx, &mut display_device, 0)
            };

            if !result.as_bool() {
                break;
            }

            // Check if this is an active display
            if (display_device.StateFlags & DISPLAY_DEVICE_ACTIVE) != 0 {
                let device_name: String = display_device.DeviceName
                    .iter()
                    .take_while(|&&c| c != 0)
                    .map(|&c| c as u8 as char)
                    .collect();

                let device_string: String = display_device.DeviceString
                    .iter()
                    .take_while(|&&c| c != 0)
                    .map(|&c| c as u8 as char)
                    .collect();

                // Get display settings
                let mut dev_mode = DEVMODEW {
                    dmSize: std::mem::size_of::<DEVMODEW>() as u16,
                    ..Default::default()
                };

                let settings_result = unsafe {
                    EnumDisplaySettingsW(
                        windows::core::PCWSTR(display_device.DeviceName.as_ptr()),
                        ENUM_CURRENT_SETTINGS,
                        &mut dev_mode,
                    )
                };

                let (resolution, refresh_rate) = if settings_result.as_bool() {
                    (
                        format!("{}x{}", dev_mode.dmPelsWidth, dev_mode.dmPelsHeight),
                        dev_mode.dmDisplayFrequency,
                    )
                } else {
                    ("Unknown".to_string(), 60)
                };

                monitors.push(Monitor {
                    id: device_name.clone(),
                    name: if device_string.is_empty() { device_name } else { device_string },
                    manufacturer: None,
                    resolution,
                    size_inches: None,
                    connection: "Unknown".to_string(),
                    hdr_support: false,
                    refresh_rate_hz: refresh_rate,
                });
            }

            device_idx += 1;
        }

        monitors
    }

    #[cfg(not(target_os = "windows"))]
    pub fn get_monitors() -> Vec<Monitor> {
        // Linux/macOS implementation would go here
        Vec::new()
    }

    // Platform-specific helpers

    #[cfg(target_os = "windows")]
    fn get_board_manufacturer() -> String {
        // Would use WMI: Win32_BaseBoard.Manufacturer
        "Unknown".to_string()
    }

    #[cfg(not(target_os = "windows"))]
    fn get_board_manufacturer() -> String {
        std::fs::read_to_string("/sys/class/dmi/id/board_vendor")
            .unwrap_or_else(|_| "Unknown".to_string())
            .trim()
            .to_string()
    }

    #[cfg(target_os = "windows")]
    fn get_board_product() -> String {
        // Would use WMI: Win32_BaseBoard.Product
        "Unknown".to_string()
    }

    #[cfg(not(target_os = "windows"))]
    fn get_board_product() -> String {
        std::fs::read_to_string("/sys/class/dmi/id/board_name")
            .unwrap_or_else(|_| "Unknown".to_string())
            .trim()
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_cpu_info() {
        let info = HardwareCollector::get_cpu_info();
        assert!(!info.name.is_empty() || info.name == "Unknown");
    }

    #[test]
    fn test_get_cpu_metrics() {
        let metrics = HardwareCollector::get_cpu_metrics();
        assert!(metrics.total_usage >= 0.0);
    }

    #[test]
    fn test_get_memory_info() {
        let info = HardwareCollector::get_memory_info();
        assert!(info.total_bytes > 0);
    }

    #[test]
    fn test_get_memory_metrics() {
        let metrics = HardwareCollector::get_memory_metrics();
        assert!(metrics.in_use_bytes > 0);
    }
}
