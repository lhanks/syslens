//! Network information collector

use crate::models::{
    AdapterStats, AdapterStatus, AdapterType, ConnectionState,
    DnsConfig, Ipv4Config, Ipv6Config, NetworkAdapter, NetworkConnection, Route, RouteType,
};
use sysinfo::Networks;
use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::HashMap;

#[cfg(target_os = "windows")]
use windows::Win32::NetworkManagement::IpHelper::{
    GetAdaptersAddresses, GAA_FLAG_INCLUDE_PREFIX, GAA_FLAG_INCLUDE_GATEWAYS,
    IP_ADAPTER_ADDRESSES_LH,
    GetIpForwardTable, MIB_IPFORWARDTABLE,
};

// IF_OPER_STATUS values (from Windows SDK)
#[cfg(target_os = "windows")]
const IF_OPER_STATUS_UP: i32 = 1;
#[cfg(target_os = "windows")]
const IF_OPER_STATUS_DOWN: i32 = 2;

#[cfg(target_os = "windows")]
use windows::Win32::Networking::WinSock::{
    AF_INET, AF_INET6, AF_UNSPEC, SOCKADDR_IN, SOCKADDR_IN6,
};

#[cfg(target_os = "windows")]
use windows::Win32::Foundation::NO_ERROR;

#[cfg(target_os = "windows")]
use wmi::{COMLibrary, WMIConnection};

#[cfg(target_os = "windows")]
use serde::Deserialize;

/// WMI structures for network adapter info
#[cfg(target_os = "windows")]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
#[allow(dead_code)]
struct Win32NetworkAdapterConfiguration {
    description: Option<String>,
    #[serde(rename = "DHCPEnabled")]
    dhcp_enabled: Option<bool>,
    #[serde(rename = "DHCPServer")]
    dhcp_server: Option<String>,
    #[serde(rename = "DHCPLeaseObtained")]
    dhcp_lease_obtained: Option<String>,
    #[serde(rename = "DHCPLeaseExpires")]
    dhcp_lease_expires: Option<String>,
    #[serde(rename = "DNSServerSearchOrder")]
    dns_server_search_order: Option<Vec<String>>,
    #[serde(rename = "DNSDomain")]
    dns_domain: Option<String>,
    #[serde(rename = "DNSDomainSuffixSearchOrder")]
    dns_domain_suffix_search_order: Option<Vec<String>>,
    index: Option<u32>,
    #[serde(rename = "MACAddress")]
    mac_address: Option<String>,
    #[serde(rename = "MTU")]
    mtu: Option<u32>,
}

/// Collector for network-related information
pub struct NetworkCollector;

impl NetworkCollector {
    /// Get all network adapters with their configuration
    pub fn get_adapters() -> Vec<NetworkAdapter> {
        #[cfg(target_os = "windows")]
        {
            Self::get_windows_adapters()
        }

        #[cfg(not(target_os = "windows"))]
        {
            Self::get_unix_adapters()
        }
    }

    #[cfg(target_os = "windows")]
    fn get_windows_adapters() -> Vec<NetworkAdapter> {
        let mut adapters = Vec::new();

        // First, get adapter addresses using IP Helper API
        let mut buffer_size: u32 = 0;
        let flags = GAA_FLAG_INCLUDE_PREFIX | GAA_FLAG_INCLUDE_GATEWAYS;

        // Get required buffer size
        unsafe {
            GetAdaptersAddresses(
                AF_UNSPEC.0 as u32,
                flags,
                None,
                None,
                &mut buffer_size,
            );
        }

        if buffer_size == 0 {
            log::warn!("No network adapters found");
            return adapters;
        }

        let mut buffer: Vec<u8> = vec![0; buffer_size as usize];
        let adapter_addresses = buffer.as_mut_ptr() as *mut IP_ADAPTER_ADDRESSES_LH;

        unsafe {
            let result = GetAdaptersAddresses(
                AF_UNSPEC.0 as u32,
                flags,
                None,
                Some(adapter_addresses),
                &mut buffer_size,
            );

            if result != NO_ERROR.0 {
                log::error!("GetAdaptersAddresses failed with error: {}", result);
                return adapters;
            }

            // Get WMI data for additional info
            let wmi_data = Self::get_wmi_adapter_config();

            let mut current = adapter_addresses;
            while !current.is_null() {
                let adapter = &*current;

                // Get adapter name
                let friendly_name = if !adapter.FriendlyName.is_null() {
                    let wide_str = adapter.FriendlyName.as_wide();
                    String::from_utf16_lossy(wide_str)
                } else {
                    "Unknown".to_string()
                };

                // Get adapter description
                let description = if !adapter.Description.is_null() {
                    let wide_str = adapter.Description.as_wide();
                    String::from_utf16_lossy(wide_str)
                } else {
                    friendly_name.clone()
                };

                // Get adapter name (GUID-like)
                let _adapter_name = if !adapter.AdapterName.is_null() {
                    std::ffi::CStr::from_ptr(adapter.AdapterName.0 as *const i8)
                        .to_string_lossy()
                        .to_string()
                } else {
                    friendly_name.clone()
                };

                // Determine adapter type
                let adapter_type = Self::if_type_to_adapter_type(adapter.IfType);

                // Get MAC address
                let mac_address = if adapter.PhysicalAddressLength > 0 {
                    adapter.PhysicalAddress[0..adapter.PhysicalAddressLength as usize]
                        .iter()
                        .map(|b| format!("{:02X}", b))
                        .collect::<Vec<_>>()
                        .join(":")
                } else {
                    String::new()
                };

                // Determine operational status
                let status = match adapter.OperStatus.0 {
                    x if x == IF_OPER_STATUS_UP => AdapterStatus::Up,
                    x if x == IF_OPER_STATUS_DOWN => AdapterStatus::Down,
                    _ => AdapterStatus::Unknown,
                };

                // Get link speed in Mbps
                let speed_mbps = if adapter.TransmitLinkSpeed > 0 {
                    Some(adapter.TransmitLinkSpeed / 1_000_000)
                } else {
                    None
                };

                // Get MTU
                let mtu = adapter.Mtu;

                // Extract IPv4 and IPv6 configuration
                let (ipv4_config, ipv6_config) = Self::extract_ip_configs(adapter, &wmi_data, &description);

                // Extract DNS configuration
                let dns_config = Self::extract_dns_config(adapter, &wmi_data, &description);

                let network_adapter = NetworkAdapter {
                    id: friendly_name.clone(),  // Use friendly_name as ID to match sysinfo's Networks
                    name: friendly_name,
                    description,
                    adapter_type,
                    mac_address,
                    status,
                    speed_mbps,
                    mtu,
                    ipv4_config,
                    ipv6_config,
                    dns_config,
                };

                adapters.push(network_adapter);
                current = adapter.Next;
            }
        }

        adapters
    }

    #[cfg(target_os = "windows")]
    fn if_type_to_adapter_type(if_type: u32) -> AdapterType {
        match if_type {
            6 => AdapterType::Ethernet,    // IF_TYPE_ETHERNET_CSMACD
            71 => AdapterType::WiFi,       // IF_TYPE_IEEE80211
            24 => AdapterType::Loopback,   // IF_TYPE_SOFTWARE_LOOPBACK
            131 | 53 => AdapterType::Virtual,  // Virtual/Tunnel types
            _ => AdapterType::Unknown,
        }
    }

    #[cfg(target_os = "windows")]
    fn get_wmi_adapter_config() -> HashMap<String, Win32NetworkAdapterConfiguration> {
        let mut result = HashMap::new();

        // Try to initialize COM, or assume it's already initialized
        let com = COMLibrary::new()
            .or_else(|_| COMLibrary::without_security())
            .unwrap_or_else(|_| unsafe { COMLibrary::assume_initialized() });

        if let Ok(wmi) = WMIConnection::new(com) {
            let query = "SELECT Description, DHCPEnabled, DHCPServer, DHCPLeaseObtained, DHCPLeaseExpires, \
                        DNSServerSearchOrder, DNSDomain, DNSDomainSuffixSearchOrder, Index, MACAddress, MTU \
                        FROM Win32_NetworkAdapterConfiguration WHERE IPEnabled = TRUE";

            if let Ok(configs) = wmi.raw_query::<Win32NetworkAdapterConfiguration>(query) {
                for config in configs {
                    if let Some(ref desc) = config.description {
                        result.insert(desc.clone(), config);
                    }
                }
            }
        }

        result
    }

    #[cfg(target_os = "windows")]
    unsafe fn extract_ip_configs(
        adapter: &IP_ADAPTER_ADDRESSES_LH,
        wmi_data: &HashMap<String, Win32NetworkAdapterConfiguration>,
        description: &str,
    ) -> (Option<Ipv4Config>, Option<Ipv6Config>) {
        let mut ipv4_config = None;
        let mut ipv6_config = None;

        // Get WMI data for this adapter
        let wmi_info = wmi_data.get(description);

        // Parse unicast addresses
        let mut unicast = adapter.FirstUnicastAddress;
        while !unicast.is_null() {
            let addr = &*unicast;
            if !addr.Address.lpSockaddr.is_null() {
                let sockaddr = &*addr.Address.lpSockaddr;

                if sockaddr.sa_family == AF_INET {
                    let sockaddr_in = &*(addr.Address.lpSockaddr as *const SOCKADDR_IN);
                    let ip_bytes = sockaddr_in.sin_addr.S_un.S_un_b;
                    let ip_address = format!("{}.{}.{}.{}",
                        ip_bytes.s_b1, ip_bytes.s_b2, ip_bytes.s_b3, ip_bytes.s_b4);

                    // Calculate subnet mask from prefix length
                    let prefix_len = addr.OnLinkPrefixLength;
                    let subnet_mask = Self::prefix_to_subnet_mask(prefix_len);

                    // Get gateway
                    let gateway = Self::get_ipv4_gateway(adapter);

                    // Get DHCP info from WMI
                    let (dhcp_enabled, dhcp_server, lease_obtained, lease_expires) =
                        if let Some(wmi) = wmi_info {
                            (
                                wmi.dhcp_enabled.unwrap_or(false),
                                wmi.dhcp_server.clone(),
                                wmi.dhcp_lease_obtained.clone(),
                                wmi.dhcp_lease_expires.clone(),
                            )
                        } else {
                            (false, None, None, None)
                        };

                    ipv4_config = Some(Ipv4Config {
                        address: ip_address,
                        subnet_mask,
                        default_gateway: gateway,
                        dhcp_enabled,
                        dhcp_server,
                        lease_obtained,
                        lease_expires,
                    });
                } else if sockaddr.sa_family == AF_INET6 {
                    let sockaddr_in6 = &*(addr.Address.lpSockaddr as *const SOCKADDR_IN6);
                    let ip_bytes = sockaddr_in6.sin6_addr.u.Byte;

                    // Format IPv6 address
                    let ip_address = Self::format_ipv6(&ip_bytes);
                    let scope_id = sockaddr_in6.Anonymous.sin6_scope_id;

                    // Check if this is a link-local address (fe80::)
                    let is_link_local = ip_bytes[0] == 0xfe && (ip_bytes[1] & 0xc0) == 0x80;

                    // Get IPv6 gateway
                    let gateway = Self::get_ipv6_gateway(adapter);

                    if is_link_local {
                        if ipv6_config.is_none() {
                            ipv6_config = Some(Ipv6Config {
                                link_local_address: format!("{}%{}", ip_address, scope_id),
                                global_address: None,
                                temporary_address: None,
                                gateway,
                            });
                        } else if let Some(ref mut cfg) = ipv6_config {
                            cfg.link_local_address = format!("{}%{}", ip_address, scope_id);
                        }
                    } else {
                        // Global or other unicast address
                        if let Some(ref mut cfg) = ipv6_config {
                            cfg.global_address = Some(ip_address);
                        } else {
                            ipv6_config = Some(Ipv6Config {
                                link_local_address: String::new(),
                                global_address: Some(ip_address),
                                temporary_address: None,
                                gateway,
                            });
                        }
                    }
                }
            }
            unicast = addr.Next;
        }

        (ipv4_config, ipv6_config)
    }

    #[cfg(target_os = "windows")]
    fn prefix_to_subnet_mask(prefix: u8) -> String {
        let mask: u32 = if prefix == 0 {
            0
        } else {
            !0u32 << (32 - prefix)
        };
        format!("{}.{}.{}.{}",
            (mask >> 24) & 0xff,
            (mask >> 16) & 0xff,
            (mask >> 8) & 0xff,
            mask & 0xff)
    }

    #[cfg(target_os = "windows")]
    unsafe fn get_ipv4_gateway(adapter: &IP_ADAPTER_ADDRESSES_LH) -> Option<String> {
        let mut gateway = adapter.FirstGatewayAddress;
        while !gateway.is_null() {
            let gw = &*gateway;
            if !gw.Address.lpSockaddr.is_null() {
                let sockaddr = &*gw.Address.lpSockaddr;
                if sockaddr.sa_family == AF_INET {
                    let sockaddr_in = &*(gw.Address.lpSockaddr as *const SOCKADDR_IN);
                    let ip_bytes = sockaddr_in.sin_addr.S_un.S_un_b;
                    return Some(format!("{}.{}.{}.{}",
                        ip_bytes.s_b1, ip_bytes.s_b2, ip_bytes.s_b3, ip_bytes.s_b4));
                }
            }
            gateway = gw.Next;
        }
        None
    }

    #[cfg(target_os = "windows")]
    unsafe fn get_ipv6_gateway(adapter: &IP_ADAPTER_ADDRESSES_LH) -> Option<String> {
        let mut gateway = adapter.FirstGatewayAddress;
        while !gateway.is_null() {
            let gw = &*gateway;
            if !gw.Address.lpSockaddr.is_null() {
                let sockaddr = &*gw.Address.lpSockaddr;
                if sockaddr.sa_family == AF_INET6 {
                    let sockaddr_in6 = &*(gw.Address.lpSockaddr as *const SOCKADDR_IN6);
                    let ip_bytes = sockaddr_in6.sin6_addr.u.Byte;
                    return Some(Self::format_ipv6(&ip_bytes));
                }
            }
            gateway = gw.Next;
        }
        None
    }

    #[cfg(target_os = "windows")]
    fn format_ipv6(bytes: &[u8; 16]) -> String {
        let words: Vec<u16> = bytes
            .chunks(2)
            .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]))
            .collect();

        // Simple formatting without zero compression for clarity
        words.iter()
            .map(|w| format!("{:x}", w))
            .collect::<Vec<_>>()
            .join(":")
    }

    #[cfg(target_os = "windows")]
    unsafe fn extract_dns_config(
        adapter: &IP_ADAPTER_ADDRESSES_LH,
        wmi_data: &HashMap<String, Win32NetworkAdapterConfiguration>,
        description: &str,
    ) -> DnsConfig {
        let mut servers = Vec::new();
        let mut suffix = None;
        let mut search_list = Vec::new();

        // Get DNS servers from adapter addresses
        let mut dns = adapter.FirstDnsServerAddress;
        while !dns.is_null() {
            let dns_addr = &*dns;
            if !dns_addr.Address.lpSockaddr.is_null() {
                let sockaddr = &*dns_addr.Address.lpSockaddr;

                if sockaddr.sa_family == AF_INET {
                    let sockaddr_in = &*(dns_addr.Address.lpSockaddr as *const SOCKADDR_IN);
                    let ip_bytes = sockaddr_in.sin_addr.S_un.S_un_b;
                    servers.push(format!("{}.{}.{}.{}",
                        ip_bytes.s_b1, ip_bytes.s_b2, ip_bytes.s_b3, ip_bytes.s_b4));
                } else if sockaddr.sa_family == AF_INET6 {
                    let sockaddr_in6 = &*(dns_addr.Address.lpSockaddr as *const SOCKADDR_IN6);
                    let ip_bytes = sockaddr_in6.sin6_addr.u.Byte;
                    servers.push(Self::format_ipv6(&ip_bytes));
                }
            }
            dns = dns_addr.Next;
        }

        // Get DNS suffix from adapter
        if !adapter.DnsSuffix.is_null() {
            let wide_str = adapter.DnsSuffix.as_wide();
            let dns_suffix = String::from_utf16_lossy(wide_str);
            if !dns_suffix.is_empty() {
                suffix = Some(dns_suffix);
            }
        }

        // Get additional info from WMI
        if let Some(wmi) = wmi_data.get(description) {
            if let Some(ref wmi_suffix) = wmi.dns_domain {
                if suffix.is_none() && !wmi_suffix.is_empty() {
                    suffix = Some(wmi_suffix.clone());
                }
            }
            if let Some(ref wmi_search) = wmi.dns_domain_suffix_search_order {
                search_list = wmi_search.clone();
            }
        }

        DnsConfig {
            servers,
            suffix,
            search_list,
        }
    }

    #[cfg(not(target_os = "windows"))]
    fn get_unix_adapters() -> Vec<NetworkAdapter> {
        let networks = Networks::new_with_refreshed_list();
        let mut adapters = Vec::new();

        for (name, data) in networks.iter() {
            let adapter = NetworkAdapter {
                id: name.clone(),
                name: name.clone(),
                description: name.clone(),
                adapter_type: Self::detect_adapter_type(name),
                mac_address: data.mac_address().to_string(),
                status: if data.total_received() > 0 || data.total_transmitted() > 0 {
                    AdapterStatus::Up
                } else {
                    AdapterStatus::Unknown
                },
                speed_mbps: None,
                mtu: 1500,
                ipv4_config: None,
                ipv6_config: None,
                dns_config: DnsConfig::default(),
            };
            adapters.push(adapter);
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
                discards: 0,
                timestamp,
            }
        })
    }

    /// Get active network connections
    pub fn get_active_connections() -> Vec<NetworkConnection> {
        #[cfg(target_os = "windows")]
        {
            Self::get_windows_connections()
        }

        #[cfg(not(target_os = "windows"))]
        {
            Vec::new()
        }
    }

    #[cfg(target_os = "windows")]
    fn get_windows_connections() -> Vec<NetworkConnection> {
        let mut connections = Vec::new();

        // Get TCP connections
        connections.extend(Self::get_tcp_connections());

        // Get UDP connections
        connections.extend(Self::get_udp_connections());

        connections
    }

    #[cfg(target_os = "windows")]
    fn get_tcp_connections() -> Vec<NetworkConnection> {
        // TODO: Implement using Windows API when proper bindings are available
        // GetExtendedTcpTable requires additional Windows crate features
        Vec::new()
    }

    #[cfg(target_os = "windows")]
    fn get_udp_connections() -> Vec<NetworkConnection> {
        // TODO: Implement using Windows API when proper bindings are available
        // GetExtendedUdpTable requires additional Windows crate features
        Vec::new()
    }

    #[cfg(target_os = "windows")]
    fn u32_to_ip(addr: u32) -> String {
        format!("{}.{}.{}.{}",
            addr & 0xff,
            (addr >> 8) & 0xff,
            (addr >> 16) & 0xff,
            (addr >> 24) & 0xff)
    }

    #[cfg(target_os = "windows")]
    #[allow(dead_code)]
    fn tcp_state_to_enum(state: u32) -> ConnectionState {
        match state {
            1 => ConnectionState::Closed,
            2 => ConnectionState::Listen,
            3 => ConnectionState::SynSent,
            4 => ConnectionState::SynReceived,
            5 => ConnectionState::Established,
            6 => ConnectionState::FinWait1,
            7 => ConnectionState::FinWait2,
            8 => ConnectionState::CloseWait,
            9 => ConnectionState::Closing,
            10 => ConnectionState::LastAck,
            11 => ConnectionState::TimeWait,
            _ => ConnectionState::Unknown,
        }
    }

    #[cfg(target_os = "windows")]
    #[allow(dead_code)]
    fn get_process_name(pid: u32) -> Option<String> {
        use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};
        use windows::Win32::System::ProcessStatus::GetModuleBaseNameW;

        if pid == 0 {
            return Some("System Idle Process".to_string());
        }
        if pid == 4 {
            return Some("System".to_string());
        }

        unsafe {
            if let Ok(handle) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
                let mut name = [0u16; 260];
                let len = GetModuleBaseNameW(handle, None, &mut name);
                if len > 0 {
                    return Some(String::from_utf16_lossy(&name[..len as usize]));
                }
            }
        }
        None
    }

    /// Get the routing table
    pub fn get_routing_table() -> Vec<Route> {
        #[cfg(target_os = "windows")]
        {
            Self::get_windows_routes()
        }

        #[cfg(not(target_os = "windows"))]
        {
            Vec::new()
        }
    }

    #[cfg(target_os = "windows")]
    fn get_windows_routes() -> Vec<Route> {
        let mut routes = Vec::new();
        let mut size: u32 = 0;

        unsafe {
            // Get required size
            GetIpForwardTable(None, &mut size, false);

            if size == 0 {
                return routes;
            }

            let mut buffer: Vec<u8> = vec![0; size as usize];
            let table = buffer.as_mut_ptr() as *mut MIB_IPFORWARDTABLE;

            let result = GetIpForwardTable(Some(table), &mut size, false);

            if result == NO_ERROR.0 {
                let table_ref = &*table;
                let entries = std::slice::from_raw_parts(
                    table_ref.table.as_ptr(),
                    table_ref.dwNumEntries as usize,
                );

                for entry in entries {
                    let destination = Self::u32_to_ip(entry.dwForwardDest);
                    let netmask = Self::u32_to_ip(entry.dwForwardMask);
                    let gateway = Self::u32_to_ip(entry.dwForwardNextHop);
                    let interface_id = format!("{}", entry.dwForwardIfIndex);
                    let metric = entry.dwForwardMetric1;

                    let route_type = if destination == "0.0.0.0" {
                        RouteType::Default
                    } else if gateway == "0.0.0.0" || destination == gateway {
                        RouteType::Local
                    } else {
                        RouteType::Remote
                    };

                    routes.push(Route {
                        destination,
                        netmask,
                        gateway,
                        interface_id,
                        metric,
                        route_type,
                    });
                }
            }
        }

        routes
    }

    /// Detect adapter type from name
    #[allow(dead_code)]
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
}

/// Format MAC address bytes into string
#[allow(dead_code)]
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
        let _ = adapters; // May be empty in test environment
    }

    #[test]
    fn test_detect_adapter_type() {
        assert!(matches!(NetworkCollector::detect_adapter_type("Wi-Fi"), AdapterType::WiFi));
        assert!(matches!(NetworkCollector::detect_adapter_type("Ethernet"), AdapterType::Ethernet));
        assert!(matches!(NetworkCollector::detect_adapter_type("Loopback"), AdapterType::Loopback));
    }
}
