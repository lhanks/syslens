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

        // Need to wait a bit and refresh again for accurate usage
        std::thread::sleep(std::time::Duration::from_millis(200));
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
        let mut sys = System::new();
        sys.refresh_memory_specifics(MemoryRefreshKind::everything());

        MemoryInfo {
            total_bytes: sys.total_memory(),
            usable_bytes: sys.total_memory(), // Same as total for basic info
            memory_type: "DDR4".to_string(), // Would need WMI for actual type
            speed_mhz: 0, // Would need WMI
            slots_used: 0,
            slots_total: 0,
            max_capacity_bytes: sys.total_memory(),
            modules: Vec::new(), // Would need WMI for module details
        }
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
        // Would need platform-specific implementation or NVML for NVIDIA
        // For now, return placeholder
        #[cfg(target_os = "windows")]
        {
            // Would use WMI: Win32_VideoController
            Vec::new()
        }

        #[cfg(not(target_os = "windows"))]
        {
            Vec::new()
        }
    }

    /// Get real-time GPU metrics
    pub fn get_gpu_metrics() -> Vec<GpuMetrics> {
        // Would need NVML for NVIDIA or vendor-specific APIs
        Vec::new()
    }

    /// Get motherboard information
    pub fn get_motherboard_info() -> MotherboardInfo {
        MotherboardInfo {
            manufacturer: Self::get_board_manufacturer(),
            product: Self::get_board_product(),
            version: String::new(),
            serial_number: String::new(),
            chipset: None,
        }
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
    pub fn get_monitors() -> Vec<Monitor> {
        // Would need platform-specific implementation
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
