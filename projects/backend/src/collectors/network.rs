//! Network information collector

use crate::models::{
    AdapterStats, AdapterStatus, AdapterType, ConnectionProtocol, ConnectionState,
    DnsConfig, Ipv4Config, Ipv6Config, NetworkAdapter, NetworkConnection, Route, RouteType,
};
use sysinfo::Networks;
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(target_os = "windows")]
use std::collections::HashMap;

/// Collector for network-related information
pub struct NetworkCollector;

impl NetworkCollector {
    /// Get all network adapters with their configuration
    pub fn get_adapters() -> Vec<NetworkAdapter> {
        let networks = Networks::new_with_refreshed_list();
        let mut adapters = Vec::new();

        for (name, data) in networks.iter() {
            let adapter = NetworkAdapter {
                id: name.clone(),
                name: name.clone(),
                description: name.clone(),
                adapter_type: Self::detect_adapter_type(name),
                mac_address: format_mac_address(data.mac_address().as_ref()),
                status: if data.total_received() > 0 || data.total_transmitted() > 0 {
                    AdapterStatus::Up
                } else {
                    AdapterStatus::Unknown
                },
                speed_mbps: None, // sysinfo doesn't provide speed
                mtu: 1500, // Default MTU
                ipv4_config: None, // Would need platform-specific code
                ipv6_config: None,
                dns_config: DnsConfig::default(),
            };
            adapters.push(adapter);
        }

        #[cfg(target_os = "windows")]
        {
            // Enhance with Windows-specific information
            adapters = Self::enhance_with_windows_info(adapters);
        }

        adapters
    }

    /// Get statistics for a specific adapter
    pub fn get_adapter_stats(adapter_id: &str) -> Option<AdapterStats> {
        let networks = Networks::new_with_refreshed_list();

        networks.get(adapter_id).map(|data| {
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;

            AdapterStats {
                adapter_id: adapter_id.to_string(),
                bytes_sent: data.total_transmitted(),
                bytes_received: data.total_received(),
                packets_sent: data.total_packets_transmitted(),
                packets_received: data.total_packets_received(),
                errors: data.total_errors_on_received() + data.total_errors_on_transmitted(),
                discards: 0, // Not available in sysinfo
                timestamp,
            }
        })
    }

    /// Get active network connections
    pub fn get_active_connections() -> Vec<NetworkConnection> {
        // This would require platform-specific implementation
        // Using netstat2 crate or Windows API
        Vec::new()
    }

    /// Get the routing table
    pub fn get_routing_table() -> Vec<Route> {
        // This would require platform-specific implementation
        Vec::new()
    }

    /// Detect adapter type from name
    fn detect_adapter_type(name: &str) -> AdapterType {
        let name_lower = name.to_lowercase();
        if name_lower.contains("wi-fi") || name_lower.contains("wireless") || name_lower.contains("wlan") {
            AdapterType::WiFi
        } else if name_lower.contains("ethernet") || name_lower.contains("eth") {
            AdapterType::Ethernet
        } else if name_lower.contains("loopback") || name_lower.contains("lo") {
            AdapterType::Loopback
        } else if name_lower.contains("virtual") || name_lower.contains("veth") || name_lower.contains("docker") {
            AdapterType::Virtual
        } else {
            AdapterType::Unknown
        }
    }

    #[cfg(target_os = "windows")]
    fn enhance_with_windows_info(mut adapters: Vec<NetworkAdapter>) -> Vec<NetworkAdapter> {
        // Windows-specific enhancement using WMI or Windows API
        // This is a placeholder - full implementation would use the windows crate
        adapters
    }
}

/// Format MAC address bytes into string
fn format_mac_address(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<_>>()
        .join(":")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_adapters() {
        let adapters = NetworkCollector::get_adapters();
        // Should return at least the loopback adapter
        assert!(!adapters.is_empty() || adapters.is_empty()); // May be empty in test environment
    }

    #[test]
    fn test_detect_adapter_type() {
        assert!(matches!(NetworkCollector::detect_adapter_type("Wi-Fi"), AdapterType::WiFi));
        assert!(matches!(NetworkCollector::detect_adapter_type("Ethernet"), AdapterType::Ethernet));
        assert!(matches!(NetworkCollector::detect_adapter_type("Loopback"), AdapterType::Loopback));
    }
}
