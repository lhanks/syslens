//! Syslens - System Monitoring Desktop Application
//!
//! A modern system information and monitoring tool built with Tauri and Angular.

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use syslens::commands;

fn main() {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();

    log::info!("Starting Syslens v{}", env!("CARGO_PKG_VERSION"));

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            // Network commands
            commands::get_network_adapters,
            commands::get_adapter_stats,
            commands::get_active_connections,
            commands::get_routing_table,
            // System commands
            commands::get_device_info,
            commands::get_bios_info,
            commands::get_boot_config,
            commands::get_os_info,
            commands::get_uptime,
            commands::get_domain_info,
            commands::get_user_info,
            // Hardware commands
            commands::get_cpu_info,
            commands::get_cpu_metrics,
            commands::get_memory_info,
            commands::get_memory_metrics,
            commands::get_gpu_info,
            commands::get_gpu_metrics,
            commands::get_motherboard_info,
            commands::get_usb_devices,
            commands::get_audio_devices,
            commands::get_monitors,
            // Storage commands
            commands::get_physical_disks,
            commands::get_partitions,
            commands::get_volumes,
            commands::get_disk_health,
            commands::get_disk_performance,
            commands::get_network_drives,
            // Process commands
            commands::get_processes,
            commands::get_process_summary,
        ])
        .run(tauri::generate_context!())
        .expect("Error while running Syslens application");
}
