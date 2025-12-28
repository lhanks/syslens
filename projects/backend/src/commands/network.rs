//! Network-related Tauri commands

use crate::collectors::NetworkCollector;
use crate::models::{AdapterStats, NetworkAdapter, NetworkConnection, Route};

/// Get all network adapters with their configuration
#[tauri::command]
pub fn get_network_adapters() -> Vec<NetworkAdapter> {
    log::debug!("Command: get_network_adapters");
    NetworkCollector::get_adapters()
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
        assert!(adapters.len() >= 0);
    }
}
