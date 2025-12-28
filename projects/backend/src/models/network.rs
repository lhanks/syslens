//! Network-related data models

use serde::{Deserialize, Serialize};

/// Network adapter information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkAdapter {
    pub id: String,
    pub name: String,
    pub description: String,
    pub adapter_type: AdapterType,
    pub mac_address: String,
    pub status: AdapterStatus,
    pub speed_mbps: Option<u64>,
    pub mtu: u32,
    pub ipv4_config: Option<Ipv4Config>,
    pub ipv6_config: Option<Ipv6Config>,
    pub dns_config: DnsConfig,
}

/// Type of network adapter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AdapterType {
    Ethernet,
    WiFi,
    Virtual,
    Loopback,
    Unknown,
}

/// Network adapter status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AdapterStatus {
    Up,
    Down,
    Disconnected,
    Unknown,
}

/// IPv4 configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ipv4Config {
    pub address: String,
    pub subnet_mask: String,
    pub default_gateway: Option<String>,
    pub dhcp_enabled: bool,
    pub dhcp_server: Option<String>,
    pub lease_obtained: Option<String>,
    pub lease_expires: Option<String>,
}

/// IPv6 configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ipv6Config {
    pub link_local_address: String,
    pub global_address: Option<String>,
    pub temporary_address: Option<String>,
    pub gateway: Option<String>,
}

/// DNS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DnsConfig {
    pub servers: Vec<String>,
    pub suffix: Option<String>,
    pub search_list: Vec<String>,
}

impl Default for DnsConfig {
    fn default() -> Self {
        Self {
            servers: Vec::new(),
            suffix: None,
            search_list: Vec::new(),
        }
    }
}

/// Network adapter statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdapterStats {
    pub adapter_id: String,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
    pub errors: u64,
    pub discards: u64,
    pub timestamp: u64,
}

/// Active network connection
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkConnection {
    pub protocol: ConnectionProtocol,
    pub local_address: String,
    pub local_port: u16,
    pub remote_address: String,
    pub remote_port: u16,
    pub state: ConnectionState,
    pub process_name: Option<String>,
    pub pid: u32,
}

/// Network protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionProtocol {
    TCP,
    UDP,
}

/// TCP connection state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionState {
    Listen,
    Established,
    TimeWait,
    CloseWait,
    Closed,
    SynSent,
    SynReceived,
    FinWait1,
    FinWait2,
    LastAck,
    Closing,
    Unknown,
}

/// Network route entry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Route {
    pub destination: String,
    pub netmask: String,
    pub gateway: String,
    pub interface_id: String,
    pub metric: u32,
    pub route_type: RouteType,
}

/// Type of route
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RouteType {
    Local,
    Remote,
    Default,
}
