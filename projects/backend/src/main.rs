//! Syslens - System Monitoring Desktop Application
//!
//! A modern system information and monitoring tool built with Tauri and Angular.

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use syslens::commands;
use syslens::state::SysInfoState;
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder, SubmenuBuilder},
    Emitter, Manager,
};

fn main() {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();

    log::info!("Starting Syslens v{}", env!("CARGO_PKG_VERSION"));

    // Create shared system state for efficient sysinfo operations
    let sysinfo_state = SysInfoState::new();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(sysinfo_state)
        .setup(|app| {
            // Build the application menu
            let toggle_left_sidebar = MenuItemBuilder::new("Toggle Navigation")
                .id("toggle_left_sidebar")
                .accelerator("CmdOrCtrl+N")
                .build(app)?;
            let toggle_sidebar = MenuItemBuilder::new("Toggle Details Panel")
                .id("toggle_sidebar")
                .accelerator("CmdOrCtrl+B")
                .build(app)?;

            // File submenu
            let file_submenu = SubmenuBuilder::new(app, "File").quit().build()?;

            // View submenu - Show resources submenu
            let toggle_cpu = MenuItemBuilder::new("CPU").id("toggle_cpu").build(app)?;
            let toggle_memory = MenuItemBuilder::new("Memory")
                .id("toggle_memory")
                .build(app)?;
            let toggle_disk = MenuItemBuilder::new("Disk").id("toggle_disk").build(app)?;
            let toggle_gpu = MenuItemBuilder::new("GPU").id("toggle_gpu").build(app)?;
            let toggle_network = MenuItemBuilder::new("Network")
                .id("toggle_network")
                .build(app)?;

            let show_submenu = SubmenuBuilder::new(app, "Show")
                .item(&toggle_cpu)
                .item(&toggle_memory)
                .item(&toggle_disk)
                .item(&toggle_gpu)
                .item(&toggle_network)
                .build()?;

            let view_submenu = SubmenuBuilder::new(app, "View")
                .item(&toggle_left_sidebar)
                .item(&toggle_sidebar)
                .separator()
                .item(&show_submenu)
                .separator()
                .item(
                    &MenuItemBuilder::new("Refresh")
                        .id("refresh")
                        .accelerator("F5")
                        .build(app)?,
                )
                .build()?;

            // Help submenu
            let about_syslens = MenuItemBuilder::new("About Syslens")
                .id("about")
                .build(app)?;
            let github = MenuItemBuilder::new("GitHub Repository")
                .id("github")
                .build(app)?;
            let report_issue = MenuItemBuilder::new("Report Issue")
                .id("report_issue")
                .build(app)?;

            let help_submenu = SubmenuBuilder::new(app, "Help")
                .item(&about_syslens)
                .separator()
                .item(&github)
                .item(&report_issue)
                .build()?;

            // Build the full menu
            let menu = MenuBuilder::new(app)
                .items(&[&file_submenu, &view_submenu, &help_submenu])
                .build()?;

            // Set the menu on the main window
            if let Some(window) = app.get_webview_window("main") {
                window.set_menu(menu)?;
            } else {
                app.set_menu(menu)?;
            }

            // Listen for menu events
            let handle = app.handle().clone();
            app.on_menu_event(move |_app, event| {
                let id = event.id().as_ref();
                log::info!("Menu item clicked: {}", id);

                match id {
                    "toggle_left_sidebar" => {
                        let _ = handle.emit("menu:toggle-left-sidebar", ());
                    }
                    "toggle_sidebar" => {
                        let _ = handle.emit("menu:toggle-sidebar", ());
                    }
                    "toggle_cpu" => {
                        let _ = handle.emit("menu:toggle-cpu", ());
                    }
                    "toggle_memory" => {
                        let _ = handle.emit("menu:toggle-memory", ());
                    }
                    "toggle_disk" => {
                        let _ = handle.emit("menu:toggle-disk", ());
                    }
                    "toggle_gpu" => {
                        let _ = handle.emit("menu:toggle-gpu", ());
                    }
                    "toggle_network" => {
                        let _ = handle.emit("menu:toggle-network", ());
                    }
                    "refresh" => {
                        let _ = handle.emit("menu:refresh", ());
                    }
                    "about" => {
                        let _ = handle.emit("menu:about", ());
                    }
                    "github" => {
                        let _ = open::that("https://github.com/syslens/syslens");
                    }
                    "report_issue" => {
                        let _ = open::that("https://github.com/syslens/syslens/issues/new");
                    }
                    _ => {}
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Network commands
            commands::get_network_adapters,
            commands::get_adapter_stats,
            commands::get_active_connections,
            commands::get_routing_table,
            commands::set_adapter_enabled,
            // System commands
            commands::get_device_info,
            commands::get_bios_info,
            commands::get_boot_config,
            commands::get_os_info,
            commands::get_uptime,
            commands::get_domain_info,
            commands::get_user_info,
            commands::get_restore_points,
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
            commands::update_hardware_ids,
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
            commands::kill_process,
            // Service commands
            commands::get_services,
            commands::get_service_summary,
            // Device info commands
            commands::get_device_deep_info,
            commands::search_device_info,
            commands::get_cached_devices,
            commands::clear_device_cache,
            commands::cleanup_device_cache,
            commands::get_device_database_stats,
            // Image cache commands
            commands::fetch_device_image,
            commands::fetch_device_image_with_key,
            commands::get_cached_image_path,
            commands::is_image_cached,
            commands::generate_device_image_cache_key,
            commands::get_image_cache_stats,
            commands::cleanup_image_cache,
            // Device enrichment commands
            commands::enrich_device,
            commands::list_enrichment_sources,
            commands::cleanup_enrichment_cache,
        ])
        .run(tauri::generate_context!())
        .expect("Error while running Syslens application");
}
