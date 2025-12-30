//! Hardware-related Tauri commands

use crate::collectors::HardwareCollector;
use crate::models::{
    AudioDevice, CpuInfo, CpuMetrics, GpuInfo, GpuMetrics, MemoryInfo, MemoryMetrics,
    Monitor, MotherboardInfo, UsbDevice,
};
use crate::state::SysInfoState;
use sysinfo::Components;
use tauri::State;

/// Get CPU static information
#[tauri::command]
pub fn get_cpu_info() -> CpuInfo {
    log::debug!("Command: get_cpu_info");
    HardwareCollector::get_cpu_info()
}

/// Get real-time CPU metrics using shared state for efficiency
#[tauri::command]
pub fn get_cpu_metrics(state: State<SysInfoState>) -> CpuMetrics {
    log::trace!("Command: get_cpu_metrics (optimized)");

    let (total_usage, per_core_usage, current_clock_mhz) = state.get_cpu_metrics();

    // Get temperature from components (this is relatively fast)
    let components = Components::new_with_refreshed_list();
    let temperature = components
        .iter()
        .find(|c| c.label().to_lowercase().contains("cpu"))
        .map(|c| c.temperature());

    CpuMetrics {
        total_usage,
        per_core_usage,
        current_clock_mhz,
        temperature,
        power_draw: None,
    }
}

/// Get memory static information
#[tauri::command]
pub fn get_memory_info() -> MemoryInfo {
    log::debug!("Command: get_memory_info");
    HardwareCollector::get_memory_info()
}

/// Get real-time memory metrics using shared state for efficiency
#[tauri::command]
pub fn get_memory_metrics(state: State<SysInfoState>) -> MemoryMetrics {
    log::trace!("Command: get_memory_metrics (optimized)");

    let (total, used, _total_swap, _used_swap) = state.get_memory_metrics();
    let available = total.saturating_sub(used);

    MemoryMetrics {
        in_use_bytes: used,
        available_bytes: available,
        committed_bytes: used, // Approximation
        cached_bytes: 0,       // Not available from sysinfo
        paged_pool_bytes: 0,
        non_paged_pool_bytes: 0,
    }
}

/// Get GPU static information for all GPUs
#[tauri::command]
pub fn get_gpu_info() -> Vec<GpuInfo> {
    log::debug!("Command: get_gpu_info");
    HardwareCollector::get_gpu_info()
}

/// Get real-time GPU metrics for all GPUs
#[tauri::command]
pub fn get_gpu_metrics() -> Vec<GpuMetrics> {
    log::trace!("Command: get_gpu_metrics");
    HardwareCollector::get_gpu_metrics()
}

/// Get motherboard information
#[tauri::command]
pub fn get_motherboard_info() -> MotherboardInfo {
    log::debug!("Command: get_motherboard_info");
    HardwareCollector::get_motherboard_info()
}

/// Get connected USB devices
#[tauri::command]
pub fn get_usb_devices() -> Vec<UsbDevice> {
    log::debug!("Command: get_usb_devices");
    HardwareCollector::get_usb_devices()
}

/// Get audio devices
#[tauri::command]
pub fn get_audio_devices() -> Vec<AudioDevice> {
    log::debug!("Command: get_audio_devices");
    HardwareCollector::get_audio_devices()
}

/// Get connected monitors
#[tauri::command]
pub fn get_monitors() -> Vec<Monitor> {
    log::debug!("Command: get_monitors");
    HardwareCollector::get_monitors()
}

/// Response for hardware ID database update
#[derive(serde::Serialize)]
pub struct HwIdUpdateResponse {
    pub usb_updated: bool,
    pub pci_updated: bool,
    pub usb_vendors: usize,
    pub usb_products: usize,
    pub pci_vendors: usize,
    pub pci_devices: usize,
    pub error: Option<String>,
}

/// Update hardware ID databases from official sources.
/// Downloads the latest USB and PCI ID databases if they are outdated.
#[tauri::command]
pub async fn update_hardware_ids() -> HwIdUpdateResponse {
    log::info!("Command: update_hardware_ids");

    let data_dir = crate::hwids::get_data_dir();
    let result = crate::hwids::update_databases(&data_dir).await;

    HwIdUpdateResponse {
        usb_updated: result.usb_updated,
        pci_updated: result.pci_updated,
        usb_vendors: result.usb_vendors,
        usb_products: result.usb_products,
        pci_vendors: result.pci_vendors,
        pci_devices: result.pci_devices,
        error: result.error,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_cpu_info() {
        let info = get_cpu_info();
        assert!(info.logical_processors > 0);
    }

    #[test]
    fn test_get_memory_info() {
        let info = get_memory_info();
        assert!(info.total_bytes > 0);
    }

    // Note: get_cpu_metrics and get_memory_metrics tests moved to state module
    // since they now require Tauri State which cannot be easily instantiated in unit tests
}
