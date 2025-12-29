//! Hardware information collector

use crate::models::{
    AudioDevice, CacheInfo, CpuInfo, CpuMetrics,
    GpuAdapterType, GpuInfo, GpuMetrics, MemoryInfo, MemoryMetrics, MemoryModule,
    Monitor, MotherboardInfo, UsbDevice,
};
use sysinfo::{Components, CpuRefreshKind, MemoryRefreshKind, System};

/// Collector for hardware information
pub struct HardwareCollector;

impl HardwareCollector {
    /// Get CPU static information
    pub fn get_cpu_info() -> CpuInfo {
        #[cfg(target_os = "windows")]
        {
            Self::get_cpu_info_windows()
        }

        #[cfg(not(target_os = "windows"))]
        {
            Self::get_cpu_info_generic()
        }
    }

    #[cfg(not(target_os = "windows"))]
    fn get_cpu_info_generic() -> CpuInfo {
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

    #[cfg(target_os = "windows")]
    fn get_cpu_info_windows() -> CpuInfo {
        use wmi::{COMLibrary, WMIConnection};
        use serde::Deserialize;

        let mut sys = System::new();
        sys.refresh_cpu_specifics(CpuRefreshKind::new().with_frequency());

        let cpus = sys.cpus();
        let cpu = cpus.first();

        // Get basic info from sysinfo
        let name = cpu.map(|c| c.brand().to_string()).unwrap_or_else(|| "Unknown".to_string());
        let manufacturer = cpu.map(|c| c.vendor_id().to_string()).unwrap_or_else(|| "Unknown".to_string());
        let base_clock = cpu.map(|c| c.frequency() as u32).unwrap_or(0);
        let physical_cores = sys.physical_core_count().unwrap_or(0) as u32;
        let logical_processors = cpus.len() as u32;

        // Get cache and socket info from WMI
        #[derive(Deserialize, Debug)]
        #[serde(rename = "Win32_Processor")]
        #[serde(rename_all = "PascalCase")]
        struct Win32Processor {
            #[serde(rename = "L2CacheSize")]
            l2_cache_size: Option<u32>,
            #[serde(rename = "L3CacheSize")]
            l3_cache_size: Option<u32>,
            max_clock_speed: Option<u32>,
            socket_designation: Option<String>,
        }

        #[derive(Deserialize, Debug)]
        #[serde(rename = "Win32_CacheMemory")]
        #[serde(rename_all = "PascalCase")]
        struct Win32CacheMemory {
            purpose: Option<String>,
            max_cache_size: Option<u32>,
            level: Option<u16>,
        }

        let mut cache = CacheInfo {
            l1_data_kb: 0,
            l1_instruction_kb: 0,
            l2_kb: 0,
            l3_kb: 0,
        };
        let mut socket = String::new();
        let mut max_clock = base_clock;

        if let Ok(com) = COMLibrary::new() {
            if let Ok(wmi_con) = WMIConnection::new(com) {
                // Get processor info (L2, L3, socket)
                if let Ok(processors) = wmi_con.query::<Win32Processor>() {
                    if let Some(proc) = processors.first() {
                        cache.l2_kb = proc.l2_cache_size.unwrap_or(0);
                        cache.l3_kb = proc.l3_cache_size.unwrap_or(0);
                        max_clock = proc.max_clock_speed.unwrap_or(base_clock);
                        socket = proc.socket_designation.clone().unwrap_or_default();
                    }
                }

                // Get cache info from Win32_CacheMemory
                // WMI Level values: 3=Primary(L1), 4=Secondary(L2), 5=Tertiary(L3)
                if let Ok(caches) = wmi_con.query::<Win32CacheMemory>() {
                    for cache_entry in caches {
                        let level = cache_entry.level.unwrap_or(0);
                        let size_kb = cache_entry.max_cache_size.unwrap_or(0);
                        let purpose = cache_entry.purpose.as_deref().unwrap_or("");

                        match level {
                            3 => {
                                // L1 cache (Primary)
                                if purpose.to_lowercase().contains("data") {
                                    cache.l1_data_kb = size_kb;
                                } else if purpose.to_lowercase().contains("instruction") {
                                    cache.l1_instruction_kb = size_kb;
                                } else if cache.l1_data_kb == 0 {
                                    cache.l1_data_kb = size_kb;
                                }
                            }
                            4 => {
                                // L2 cache (Secondary) - use if not set from Win32_Processor
                                if cache.l2_kb == 0 {
                                    cache.l2_kb = size_kb;
                                }
                            }
                            5 => {
                                // L3 cache (Tertiary) - use if not set from Win32_Processor
                                if cache.l3_kb == 0 {
                                    cache.l3_kb = size_kb;
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        CpuInfo {
            name,
            manufacturer,
            architecture: std::env::consts::ARCH.to_string(),
            family: String::new(),
            model: String::new(),
            stepping: String::new(),
            physical_cores,
            logical_processors,
            base_clock_mhz: base_clock,
            max_clock_mhz: max_clock,
            cache,
            socket,
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
                            .map(Self::decode_memory_type)
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

                    // Get manufacturer - prefer WMI, fall back to part number extraction
                    let wmi_manufacturer = module.manufacturer
                        .map(|m| m.trim().to_string())
                        .filter(|m| !m.is_empty() && m != "Unknown");

                    let manufacturer = wmi_manufacturer
                        .or_else(|| Self::extract_manufacturer_from_part_number(&part_number))
                        .unwrap_or_else(|| "Unknown".to_string());

                    modules.push(MemoryModule {
                        slot: module.device_locator.unwrap_or_else(|| "Unknown".to_string()),
                        capacity_bytes: capacity,
                        manufacturer,
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

    /// Extract manufacturer from memory part number prefix.
    /// Many manufacturers use consistent prefixes in their part numbers.
    #[cfg(target_os = "windows")]
    fn extract_manufacturer_from_part_number(part_number: &str) -> Option<String> {
        let upper = part_number.to_uppercase();

        // Team Group patterns: FLBD, TED, TPRD, TF, TZD, TPD, THLD, TLZGD, TLRD, TEAMGROUP
        if upper.starts_with("FLBD")
            || upper.starts_with("TED")
            || upper.starts_with("TPRD")
            || upper.starts_with("TZD")
            || upper.starts_with("TPD")
            || upper.starts_with("THLD")
            || upper.starts_with("TLZGD")
            || upper.starts_with("TLRD")
            || upper.starts_with("TEAMGROUP")
            || (upper.starts_with("TF") && upper.len() > 4)
        {
            return Some("Team Group".to_string());
        }

        // G.Skill patterns: F5-, F4-, F3-
        if upper.starts_with("F5-") || upper.starts_with("F4-") || upper.starts_with("F3-") {
            return Some("G.Skill".to_string());
        }

        // Corsair patterns: CM (CMK, CMW, CML, CMT, CMH, CMD, CMR, CMS)
        if upper.starts_with("CM") && upper.len() > 3 {
            return Some("Corsair".to_string());
        }

        // Kingston patterns: KHX, KVR, KF, FURY
        if upper.starts_with("KHX")
            || upper.starts_with("KVR")
            || upper.starts_with("KF")
            || upper.starts_with("FURY")
        {
            return Some("Kingston".to_string());
        }

        // Crucial/Micron patterns: BL, CT, BLS, MTA, MT (not followed by another letter)
        if upper.starts_with("BL") || upper.starts_with("CT") || upper.starts_with("BLS") {
            return Some("Crucial".to_string());
        }
        if upper.starts_with("MTA") || upper.starts_with("MTC") {
            return Some("Micron".to_string());
        }

        // Samsung patterns: M378, M471, M393, M391
        if upper.starts_with("M378")
            || upper.starts_with("M471")
            || upper.starts_with("M393")
            || upper.starts_with("M391")
        {
            return Some("Samsung".to_string());
        }

        // SK Hynix patterns: HM, HMA, HMAA, HMT, HMCG
        if upper.starts_with("HMA")
            || upper.starts_with("HMT")
            || upper.starts_with("HMCG")
            || upper.starts_with("HMAA")
        {
            return Some("SK Hynix".to_string());
        }

        // Patriot patterns: PV, PVE, PVSR, PSD
        if upper.starts_with("PV") || upper.starts_with("PSD") {
            return Some("Patriot".to_string());
        }

        // ADATA patterns: AD, AX
        if upper.starts_with("AD") || upper.starts_with("AX") {
            return Some("ADATA".to_string());
        }

        // PNY patterns: MD
        if upper.starts_with("MD") && upper.len() > 4 {
            return Some("PNY".to_string());
        }

        None
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
        } else if lower.contains("intel") {
            // Check Intel before ATI to avoid false positive from "Corporation" containing "ati"
            Some("Intel".to_string())
        } else if lower.contains("amd") || lower.contains("ati") {
            Some("AMD".to_string())
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
                        .unwrap_or_else(String::new)
                        .trim()
                        .to_string();
                    serial_number = board.serial_number
                        .unwrap_or_else(String::new)
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

    /// Parse EDID data to extract monitor information.
    /// EDID (Extended Display Identification Data) is a 128-byte block that contains
    /// the actual monitor name, manufacturer, and specifications.
    #[cfg(target_os = "windows")]
    fn parse_edid(edid: &[u8]) -> Option<(String, String)> {
        // EDID must be at least 128 bytes
        if edid.len() < 128 {
            return None;
        }

        // Verify EDID header (bytes 0-7 should be 00 FF FF FF FF FF FF 00)
        let expected_header = [0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00];
        if edid[0..8] != expected_header {
            return None;
        }

        // Parse manufacturer ID from bytes 8-9 (3-letter PNP ID encoded in 2 bytes)
        let mfg_bytes = ((edid[8] as u16) << 8) | (edid[9] as u16);
        let char1 = ((mfg_bytes >> 10) & 0x1F) as u8 + b'A' - 1;
        let char2 = ((mfg_bytes >> 5) & 0x1F) as u8 + b'A' - 1;
        let char3 = (mfg_bytes & 0x1F) as u8 + b'A' - 1;
        let manufacturer_code = format!("{}{}{}", char1 as char, char2 as char, char3 as char);

        // Map common manufacturer codes to names
        let manufacturer = match manufacturer_code.as_str() {
            "DEL" => "Dell",
            "SAM" => "Samsung",
            "LEN" => "Lenovo",
            "ACR" => "Acer",
            "ACI" => "Asus",
            "AUS" => "Asus",
            "BNQ" => "BenQ",
            "HWP" => "HP",
            "LGD" => "LG Display",
            "GSM" => "LG Electronics",
            "PHL" => "Philips",
            "AOC" => "AOC",
            "VSC" => "ViewSonic",
            "NEC" => "NEC",
            "EIZ" => "Eizo",
            "IVM" => "Iiyama",
            "MED" => "Medion",
            "MSI" => "MSI",
            "GBT" => "Gigabyte",
            _ => &manufacturer_code,
        }.to_string();

        // Look for monitor name in descriptor blocks (bytes 54-125)
        // Each descriptor block is 18 bytes. Descriptor type 0xFC = Monitor Name
        let mut monitor_name = None;
        for i in 0..4 {
            let offset = 54 + (i * 18);
            if offset + 18 > edid.len() {
                break;
            }

            // Check if this is a display descriptor (first 2 bytes are 0)
            if edid[offset] == 0 && edid[offset + 1] == 0 {
                let descriptor_type = edid[offset + 3];

                // 0xFC = Monitor Name
                if descriptor_type == 0xFC {
                    // Name is in bytes 5-17 (13 characters, padded with 0x0A or space)
                    let name_bytes = &edid[offset + 5..offset + 18];
                    let name: String = name_bytes
                        .iter()
                        .take_while(|&&b| b != 0x0A && b != 0x00)
                        .map(|&b| b as char)
                        .collect::<String>()
                        .trim()
                        .to_string();

                    if !name.is_empty() {
                        monitor_name = Some(name);
                        break;
                    }
                }
            }
        }

        // Fall back to product code if no name found
        let name = monitor_name.unwrap_or_else(|| {
            let product_code = ((edid[11] as u16) << 8) | (edid[10] as u16);
            format!("{} {:04X}", manufacturer, product_code)
        });

        Some((manufacturer, name))
    }

    /// Read EDID data from Windows Registry for all monitors.
    /// Returns a map of DeviceID prefix -> (manufacturer, name)
    #[cfg(target_os = "windows")]
    fn get_edid_info_from_registry() -> std::collections::HashMap<String, (String, String)> {
        use winreg::enums::*;
        use winreg::RegKey;

        let mut edid_map = std::collections::HashMap::new();

        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let display_path = r"SYSTEM\CurrentControlSet\Enum\DISPLAY";

        if let Ok(display_key) = hklm.open_subkey(display_path) {
            // Iterate over monitor types (e.g., "DELA1A2", "GSM5BBF")
            if let Ok(monitor_types) = display_key.enum_keys().collect::<Result<Vec<_>, _>>() {
                for monitor_type in monitor_types {
                    let type_path = format!(r"{}\{}", display_path, monitor_type);
                    if let Ok(type_key) = hklm.open_subkey(&type_path) {
                        // Iterate over instances
                        if let Ok(instances) = type_key.enum_keys().collect::<Result<Vec<_>, _>>() {
                            for instance in instances {
                                let edid_path = format!(r"{}\{}\Device Parameters", type_path, instance);
                                if let Ok(params_key) = hklm.open_subkey(&edid_path) {
                                    if let Ok(edid_data) = params_key.get_raw_value("EDID") {
                                        if let Some((manufacturer, name)) = Self::parse_edid(&edid_data.bytes) {
                                            // Use monitor_type as the key (e.g., "DELA1A2")
                                            // This matches the DeviceID from EnumDisplayDevices
                                            edid_map.insert(monitor_type.clone(), (manufacturer.clone(), name.clone()));
                                            log::debug!(
                                                "EDID found: {} -> {} {}",
                                                monitor_type, manufacturer, name
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        edid_map
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

        // Get EDID data from registry for real monitor names
        let edid_info = Self::get_edid_info_from_registry();

        if let Ok(com) = COMLibrary::new() {
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
                        let device_id = mon.device_id.clone().unwrap_or_else(|| format!("monitor-{}", idx));

                        // Try to find EDID info by matching device ID
                        // WMI DeviceID format: "DesktopMonitor1" or "\\.\DISPLAY1\Monitor0"
                        // Registry format: "DELA1A2" (manufacturer + product code)
                        // We need to check if any EDID key is present in a connected monitor
                        let (edid_manufacturer, edid_name) = edid_info.iter()
                            .find(|(key, _)| {
                                // Match if the key appears in the device ID or matches any connected monitor
                                device_id.contains(*key)
                            })
                            .map(|(_, (mfg, name))| (Some(mfg.clone()), Some(name.clone())))
                            .unwrap_or((None, None));

                        // Use EDID name if available, otherwise WMI name
                        let wmi_name = mon.name.clone().unwrap_or_else(|| format!("Display {}", idx + 1));
                        let name = edid_name.unwrap_or_else(|| {
                            // If WMI gives "Generic PnP Monitor", try to use EDID data by index
                            if wmi_name.contains("Generic") {
                                // Try to match by index if we have EDID data
                                if idx < edid_info.len() {
                                    if let Some((_, (_, edid_name))) = edid_info.iter().nth(idx) {
                                        return edid_name.clone();
                                    }
                                }
                            }
                            wmi_name
                        });

                        // Use EDID manufacturer if available
                        let manufacturer = edid_manufacturer.or(mon.manufacturer);

                        // Try to get resolution from monitor, fall back to video controller
                        let mon_resolution = match (mon.screen_width, mon.screen_height) {
                            (Some(w), Some(h)) if w > 0 && h > 0 => format!("{}x{}", w, h),
                            _ => resolution.clone(),
                        };

                        monitors.push(Monitor {
                            id: device_id,
                            name,
                            manufacturer,
                            resolution: mon_resolution,
                            size_inches: None,
                            connection: "Unknown".to_string(),
                            hdr_support: false,
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
            monitors = Self::get_monitors_from_gdi(&edid_info);
        }

        monitors
    }

    #[cfg(target_os = "windows")]
    fn get_monitors_from_gdi(edid_info: &std::collections::HashMap<String, (String, String)>) -> Vec<Monitor> {
        use windows::Win32::Graphics::Gdi::{
            EnumDisplayDevicesW, EnumDisplaySettingsW, DEVMODEW, DISPLAY_DEVICEW,
            ENUM_CURRENT_SETTINGS,
        };

        const DISPLAY_DEVICE_ACTIVE: u32 = 0x00000001;
        const DISPLAY_DEVICE_ATTACHED_TO_DESKTOP: u32 = 0x00000001;

        let mut monitors = Vec::new();
        let mut adapter_idx = 0u32;

        // First loop: enumerate display adapters (GPUs)
        loop {
            let mut adapter_device = DISPLAY_DEVICEW {
                cb: std::mem::size_of::<DISPLAY_DEVICEW>() as u32,
                ..Default::default()
            };

            let adapter_result = unsafe {
                EnumDisplayDevicesW(None, adapter_idx, &mut adapter_device, 0)
            };

            if !adapter_result.as_bool() {
                break;
            }

            // Check if this adapter is active
            if (adapter_device.StateFlags & DISPLAY_DEVICE_ACTIVE) != 0 {
                let adapter_name: Vec<u16> = adapter_device.DeviceName.to_vec();

                // Get display settings for this adapter
                let mut dev_mode = DEVMODEW {
                    dmSize: std::mem::size_of::<DEVMODEW>() as u16,
                    ..Default::default()
                };

                let settings_result = unsafe {
                    EnumDisplaySettingsW(
                        windows::core::PCWSTR(adapter_device.DeviceName.as_ptr()),
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

                // Second loop: enumerate monitors attached to this adapter
                let mut monitor_idx = 0u32;
                let mut found_monitor = false;

                loop {
                    let mut monitor_device = DISPLAY_DEVICEW {
                        cb: std::mem::size_of::<DISPLAY_DEVICEW>() as u32,
                        ..Default::default()
                    };

                    let monitor_result = unsafe {
                        EnumDisplayDevicesW(
                            windows::core::PCWSTR(adapter_name.as_ptr()),
                            monitor_idx,
                            &mut monitor_device,
                            0,
                        )
                    };

                    if !monitor_result.as_bool() {
                        break;
                    }

                    // Check if this monitor is attached
                    if (monitor_device.StateFlags & DISPLAY_DEVICE_ATTACHED_TO_DESKTOP) != 0 {
                        found_monitor = true;

                        // DeviceID format: "MONITOR\DELA1A2\{GUID}"
                        let monitor_id: String = monitor_device.DeviceID
                            .iter()
                            .take_while(|&&c| c != 0)
                            .map(|&c| c as u8 as char)
                            .collect();

                        let gdi_name: String = monitor_device.DeviceString
                            .iter()
                            .take_while(|&&c| c != 0)
                            .map(|&c| c as u8 as char)
                            .collect();

                        // Try to find EDID info by matching DeviceID
                        // DeviceID format: "MONITOR\DELA1A2\{GUID}"
                        let (edid_manufacturer, edid_name) = edid_info.iter()
                            .find(|(key, _)| monitor_id.contains(key.as_str()))
                            .map(|(_, (mfg, name))| (Some(mfg.clone()), Some(name.clone())))
                            .unwrap_or((None, None));

                        // Use EDID name if available, otherwise GDI name
                        let display_name = edid_name.unwrap_or_else(|| {
                            if gdi_name.is_empty() || gdi_name.contains("Generic") {
                                // Try to match by index if we have EDID data
                                let idx = monitors.len();
                                if idx < edid_info.len() {
                                    if let Some((_, (_, name))) = edid_info.iter().nth(idx) {
                                        return name.clone();
                                    }
                                }
                                format!("Display {}", monitors.len() + 1)
                            } else {
                                gdi_name
                            }
                        });

                        monitors.push(Monitor {
                            id: if monitor_id.is_empty() {
                                format!("monitor-{}-{}", adapter_idx, monitor_idx)
                            } else {
                                monitor_id
                            },
                            name: display_name,
                            manufacturer: edid_manufacturer,
                            resolution: resolution.clone(),
                            size_inches: None,
                            connection: "Unknown".to_string(),
                            hdr_support: false,
                            refresh_rate_hz: refresh_rate,
                        });
                    }

                    monitor_idx += 1;
                }

                // If no monitors found for this adapter, create a generic entry
                if !found_monitor {
                    let adapter_name_str: String = adapter_device.DeviceName
                        .iter()
                        .take_while(|&&c| c != 0)
                        .map(|&c| c as u8 as char)
                        .collect();

                    monitors.push(Monitor {
                        id: adapter_name_str.clone(),
                        name: format!("Display {}", monitors.len() + 1),
                        manufacturer: None,
                        resolution,
                        size_inches: None,
                        connection: "Unknown".to_string(),
                        hdr_support: false,
                        refresh_rate_hz: refresh_rate,
                    });
                }
            }

            adapter_idx += 1;
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
    #[allow(dead_code)]
    fn get_board_manufacturer() -> String {
        // Would use WMI: Win32_BaseBoard.Manufacturer
        "Unknown".to_string()
    }

    #[cfg(not(target_os = "windows"))]
    #[allow(dead_code)]
    fn get_board_manufacturer() -> String {
        std::fs::read_to_string("/sys/class/dmi/id/board_vendor")
            .unwrap_or_else(|_| "Unknown".to_string())
            .trim()
            .to_string()
    }

    #[cfg(target_os = "windows")]
    #[allow(dead_code)]
    fn get_board_product() -> String {
        // Would use WMI: Win32_BaseBoard.Product
        "Unknown".to_string()
    }

    #[cfg(not(target_os = "windows"))]
    #[allow(dead_code)]
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

    #[test]
    fn test_get_gpu_info() {
        let gpus = HardwareCollector::get_gpu_info();
        // Can be empty on systems without discrete GPU, but should not panic
        for gpu in &gpus {
            assert!(!gpu.id.is_empty());
            assert!(!gpu.name.is_empty());
        }
    }

    #[test]
    fn test_get_motherboard_info() {
        let info = HardwareCollector::get_motherboard_info();
        // Should return at least manufacturer and product (may be "Unknown")
        assert!(!info.manufacturer.is_empty());
        assert!(!info.product.is_empty());
    }

    #[test]
    fn test_get_monitors() {
        let monitors = HardwareCollector::get_monitors();
        // Should return at least one monitor on a desktop system
        for monitor in &monitors {
            assert!(!monitor.id.is_empty());
            assert!(!monitor.name.is_empty());
        }
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_decode_memory_type() {
        assert_eq!(HardwareCollector::decode_memory_type(0), "Unknown");
        assert_eq!(HardwareCollector::decode_memory_type(20), "DDR");
        assert_eq!(HardwareCollector::decode_memory_type(21), "DDR2");
        assert_eq!(HardwareCollector::decode_memory_type(24), "DDR3");
        assert_eq!(HardwareCollector::decode_memory_type(26), "DDR4");
        assert_eq!(HardwareCollector::decode_memory_type(99), "Type 99");
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_extract_speed_from_part_number() {
        // DDR5 speeds
        assert_eq!(HardwareCollector::extract_speed_from_part_number("FLBD516G6000HC38GBKT"), 6000);
        assert_eq!(HardwareCollector::extract_speed_from_part_number("CMK32GX5M2B5600C36"), 5600);

        // DDR4 speeds
        assert_eq!(HardwareCollector::extract_speed_from_part_number("CMK16GX4M2B3200C16"), 3200);
        assert_eq!(HardwareCollector::extract_speed_from_part_number("F4-3600C16D-16GTZNC"), 3600);

        // No valid speed found
        assert_eq!(HardwareCollector::extract_speed_from_part_number("RANDOMPART123"), 0);
        assert_eq!(HardwareCollector::extract_speed_from_part_number(""), 0);
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_extract_manufacturer_from_part_number() {
        // Team Group
        assert_eq!(
            HardwareCollector::extract_manufacturer_from_part_number("FLBD516G6000HC38GBKT"),
            Some("Team Group".to_string())
        );
        assert_eq!(
            HardwareCollector::extract_manufacturer_from_part_number("TED516G6000C40DC01"),
            Some("Team Group".to_string())
        );

        // G.Skill
        assert_eq!(
            HardwareCollector::extract_manufacturer_from_part_number("F5-6000J3038F16GX2-TZ5N"),
            Some("G.Skill".to_string())
        );
        assert_eq!(
            HardwareCollector::extract_manufacturer_from_part_number("F4-3600C16D-16GTZNC"),
            Some("G.Skill".to_string())
        );

        // Corsair
        assert_eq!(
            HardwareCollector::extract_manufacturer_from_part_number("CMK32GX5M2B5600C36"),
            Some("Corsair".to_string())
        );
        assert_eq!(
            HardwareCollector::extract_manufacturer_from_part_number("CMW16GX4M2C3200C16"),
            Some("Corsair".to_string())
        );

        // Kingston
        assert_eq!(
            HardwareCollector::extract_manufacturer_from_part_number("KF560C36BBK2-32"),
            Some("Kingston".to_string())
        );
        assert_eq!(
            HardwareCollector::extract_manufacturer_from_part_number("KVR32N22S8/8"),
            Some("Kingston".to_string())
        );

        // Crucial
        assert_eq!(
            HardwareCollector::extract_manufacturer_from_part_number("CT16G56C46S5"),
            Some("Crucial".to_string())
        );
        assert_eq!(
            HardwareCollector::extract_manufacturer_from_part_number("BL16G32C16U4B"),
            Some("Crucial".to_string())
        );

        // Samsung
        assert_eq!(
            HardwareCollector::extract_manufacturer_from_part_number("M378A2K43DB1-CTD"),
            Some("Samsung".to_string())
        );

        // SK Hynix
        assert_eq!(
            HardwareCollector::extract_manufacturer_from_part_number("HMAA2GU6CJR8N-XN"),
            Some("SK Hynix".to_string())
        );

        // Unknown
        assert_eq!(
            HardwareCollector::extract_manufacturer_from_part_number("RANDOMPART123"),
            None
        );
        assert_eq!(
            HardwareCollector::extract_manufacturer_from_part_number(""),
            None
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_extract_gpu_vendor() {
        assert_eq!(HardwareCollector::extract_gpu_vendor("NVIDIA Corporation"), Some("NVIDIA".to_string()));
        assert_eq!(HardwareCollector::extract_gpu_vendor("AMD Radeon"), Some("AMD".to_string()));
        assert_eq!(HardwareCollector::extract_gpu_vendor("ATI Technologies"), Some("AMD".to_string()));
        assert_eq!(HardwareCollector::extract_gpu_vendor("Intel Corporation"), Some("Intel".to_string()));
        assert_eq!(HardwareCollector::extract_gpu_vendor("CustomVendor"), Some("CustomVendor".to_string()));
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_get_gpu_driver_link() {
        assert!(HardwareCollector::get_gpu_driver_link("NVIDIA", "").unwrap().contains("nvidia.com"));
        assert!(HardwareCollector::get_gpu_driver_link("AMD", "").unwrap().contains("amd.com"));
        assert!(HardwareCollector::get_gpu_driver_link("Intel", "").unwrap().contains("intel.com"));
        assert!(HardwareCollector::get_gpu_driver_link("Unknown", "").is_none());
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_get_motherboard_support_url() {
        assert!(HardwareCollector::get_motherboard_support_url("ASUS", "").unwrap().contains("asus.com"));
        assert!(HardwareCollector::get_motherboard_support_url("MSI", "").unwrap().contains("msi.com"));
        assert!(HardwareCollector::get_motherboard_support_url("Gigabyte", "").unwrap().contains("gigabyte.com"));
        assert!(HardwareCollector::get_motherboard_support_url("ASRock", "").unwrap().contains("asrock.com"));
        assert!(HardwareCollector::get_motherboard_support_url("Unknown", "").is_none());
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_parse_edid_valid() {
        // Valid EDID header + manufacturer "DEL" (Dell) + monitor name
        let mut edid = vec![0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00]; // Header

        // Manufacturer ID for Dell: 'D'=4, 'E'=5, 'L'=12
        // Encoded as: ((4 << 10) | (5 << 5) | 12) = 0x10AC
        edid.push(0x10); // Byte 8
        edid.push(0xAC); // Byte 9

        // Product code (2 bytes, little endian)
        edid.push(0xA2); // Byte 10
        edid.push(0xA1); // Byte 11

        // Pad to 54 bytes (before descriptors)
        edid.resize(54, 0x00);

        // Descriptor 1: Monitor name (starts at byte 54)
        // 0x00, 0x00, 0x00, 0xFC = Monitor name descriptor
        edid.extend_from_slice(&[0x00, 0x00, 0x00, 0xFC, 0x00]); // Descriptor header
        edid.extend_from_slice(b"DELL S2722DGM"); // 13 chars

        // Pad to 128 bytes
        edid.resize(128, 0x00);

        let result = HardwareCollector::parse_edid(&edid);
        assert!(result.is_some());
        let (manufacturer, name) = result.unwrap();
        assert_eq!(manufacturer, "Dell");
        assert_eq!(name, "DELL S2722DGM");
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_parse_edid_invalid_header() {
        // Invalid EDID header
        let edid = vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        let result = HardwareCollector::parse_edid(&edid);
        assert!(result.is_none());
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_parse_edid_too_short() {
        // EDID too short (less than 128 bytes)
        let edid = vec![0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00];
        let result = HardwareCollector::parse_edid(&edid);
        assert!(result.is_none());
    }
}
