//! Hardware-related Tauri commands

use crate::collectors::HardwareCollector;
use crate::models::{
    AudioDevice, CpuInfo, CpuMetrics, GpuInfo, GpuMetrics, MemoryInfo, MemoryMetrics,
    Monitor, MotherboardInfo, UsbDevice,
};

/// Get CPU static information
#[tauri::command]
pub fn get_cpu_info() -> CpuInfo {
    log::debug!("Command: get_cpu_info");
    HardwareCollector::get_cpu_info()
}

/// Get real-time CPU metrics
#[tauri::command]
pub fn get_cpu_metrics() -> CpuMetrics {
    log::trace!("Command: get_cpu_metrics");
    HardwareCollector::get_cpu_metrics()
}

/// Get memory static information
#[tauri::command]
pub fn get_memory_info() -> MemoryInfo {
    log::debug!("Command: get_memory_info");
    HardwareCollector::get_memory_info()
}

/// Get real-time memory metrics
#[tauri::command]
pub fn get_memory_metrics() -> MemoryMetrics {
    log::trace!("Command: get_memory_metrics");
    HardwareCollector::get_memory_metrics()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_cpu_info() {
        let info = get_cpu_info();
        assert!(info.logical_processors > 0);
    }

    #[test]
    fn test_get_cpu_metrics() {
        let metrics = get_cpu_metrics();
        assert!(metrics.total_usage >= 0.0);
    }

    #[test]
    fn test_get_memory_info() {
        let info = get_memory_info();
        assert!(info.total_bytes > 0);
    }
}
