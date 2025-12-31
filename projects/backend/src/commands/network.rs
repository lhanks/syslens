//! Network-related Tauri commands

use crate::collectors::NetworkCollector;
use crate::models::{AdapterStats, NetworkAdapter, NetworkConnection, Route};

/// Get all network adapters with their configuration
#[tauri::command]
pub fn get_network_adapters() -> Vec<NetworkAdapter> {
    log::debug!("Command: get_network_adapters");
    NetworkCollector::get_adapters()
}

/// Enable or disable a network adapter
/// Requires administrator privileges on Windows
#[tauri::command]
pub fn set_adapter_enabled(adapter_name: String, enabled: bool) -> Result<bool, String> {
    log::info!(
        "Command: set_adapter_enabled({}, {})",
        adapter_name,
        enabled
    );

    #[cfg(target_os = "windows")]
    {
        use std::process::Command;

        let action = if enabled { "enabled" } else { "disabled" };

        // Use netsh to enable/disable the adapter
        let output = Command::new("netsh")
            .args([
                "interface",
                "set",
                "interface",
                &adapter_name,
                &format!("admin={}", action),
            ])
            .output()
            .map_err(|e| format!("Failed to execute netsh: {}", e))?;

        if output.status.success() {
            log::info!("Successfully {} adapter: {}", action, adapter_name);
            Ok(true)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            let error_msg = if stderr.is_empty() {
                stdout.to_string()
            } else {
                stderr.to_string()
            };

            // Check for common error messages
            if error_msg.contains("requires elevation")
                || error_msg.contains("Access is denied")
                || error_msg.contains("run as administrator")
            {
                log::warn!(
                    "Administrator privileges required to {} adapter: {}",
                    if enabled { "enable" } else { "disable" },
                    adapter_name
                );
                Err("Administrator privileges required. Please run as administrator.".to_string())
            } else {
                log::error!(
                    "Failed to {} adapter {}: {}",
                    if enabled { "enable" } else { "disable" },
                    adapter_name,
                    error_msg
                );
                Err(format!(
                    "Failed to {} adapter: {}",
                    action,
                    error_msg.trim()
                ))
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        log::warn!("Network adapter control is only supported on Windows");
        Err("Network adapter control is only supported on Windows".to_string())
    }
}

/// Get statistics for a specific network adapter
#[tauri::command]
pub fn get_adapter_stats(adapter_id: String) -> Option<AdapterStats> {
    log::debug!("Command: get_adapter_stats({})", adapter_id);
    NetworkCollector::get_adapter_stats(&adapter_id)
}

/// Get all active network connections
#[tauri::command]
pub fn get_active_connections() -> Vec<NetworkConnection> {
    log::debug!("Command: get_active_connections");
    NetworkCollector::get_active_connections()
}

/// Get the system routing table
#[tauri::command]
pub fn get_routing_table() -> Vec<Route> {
    log::debug!("Command: get_routing_table");
    NetworkCollector::get_routing_table()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_network_adapters() {
        let adapters = get_network_adapters();
        // May be empty in test environment, but shouldn't panic
        let _ = adapters;
    }
}
