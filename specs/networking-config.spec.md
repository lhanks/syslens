# Syslens Networking Configuration Specification

## Overview

The networking module provides comprehensive network configuration information, serving as a visual and enhanced alternative to command-line tools like `ipconfig`, `netstat`, and `Get-NetAdapter`.

## Data Categories

### 1. Network Adapters

Information about each physical and virtual network adapter.

| Field | Description | Source (Windows) |
|-------|-------------|------------------|
| Name | Adapter friendly name | `Get-NetAdapter` |
| Description | Full adapter description | `Get-NetAdapter` |
| Type | Adapter type (Ethernet, Wi-Fi, etc.) | `Get-NetAdapter` |
| Status | Connection status (Up/Down/Disconnected) | `Get-NetAdapter` |
| MAC Address | Physical hardware address | `Get-NetAdapter` |
| Speed | Link speed (e.g., 1 Gbps) | `Get-NetAdapter` |
| MTU | Maximum Transmission Unit | `Get-NetAdapter` |
| Driver Version | Network driver version | `Get-NetAdapterDriver` |
| Driver Date | Driver release date | `Get-NetAdapterDriver` |
| Hardware ID | PCI/USB hardware identifier | WMI |

### 2. IP Configuration

IPv4 and IPv6 configuration per adapter.

| Field | Description | Source (Windows) |
|-------|-------------|------------------|
| IPv4 Address | Primary IPv4 address | `Get-NetIPAddress` |
| IPv4 Subnet Mask | Subnet mask | `Get-NetIPAddress` |
| IPv4 CIDR | CIDR notation (e.g., /24) | Calculated |
| IPv4 Gateway | Default gateway | `Get-NetRoute` |
| IPv6 Address | Primary IPv6 address | `Get-NetIPAddress` |
| IPv6 Prefix Length | IPv6 prefix (e.g., /64) | `Get-NetIPAddress` |
| IPv6 Gateway | IPv6 default gateway | `Get-NetRoute` |
| DHCP Enabled | Whether DHCP is active | `Get-NetIPInterface` |
| DHCP Server | DHCP server address | `ipconfig /all` |
| Lease Obtained | DHCP lease start time | `ipconfig /all` |
| Lease Expires | DHCP lease expiration | `ipconfig /all` |

### 3. DNS Configuration

DNS server and resolution settings.

| Field | Description | Source (Windows) |
|-------|-------------|------------------|
| DNS Servers | List of DNS server addresses | `Get-DnsClientServerAddress` |
| DNS Suffix | Connection-specific DNS suffix | `Get-DnsClient` |
| DNS Search List | DNS suffix search list | `Get-DnsClientGlobalSetting` |
| Register Connection | DNS registration enabled | `Get-DnsClient` |

### 4. Network Statistics

Real-time network statistics per adapter.

| Field | Description | Update Frequency |
|-------|-------------|------------------|
| Bytes Sent | Total bytes transmitted | 1 second |
| Bytes Received | Total bytes received | 1 second |
| Packets Sent | Total packets transmitted | 1 second |
| Packets Received | Total packets received | 1 second |
| Errors (In/Out) | Transmission errors | 1 second |
| Discards (In/Out) | Discarded packets | 1 second |
| Current Bandwidth | Real-time throughput | 1 second |

### 5. Active Connections

Current network connections (similar to `netstat`).

| Field | Description |
|-------|-------------|
| Protocol | TCP/UDP |
| Local Address | Local IP:Port |
| Remote Address | Remote IP:Port |
| State | Connection state (Established, Listening, etc.) |
| Process ID | Associated process ID |
| Process Name | Associated process name |

### 6. Routing Table

Network routing information.

| Field | Description |
|-------|-------------|
| Destination | Network destination |
| Netmask | Network mask |
| Gateway | Gateway address |
| Interface | Interface IP |
| Metric | Route metric |
| Type | Route type (Local, Remote) |

### 7. Wireless Information (Wi-Fi Only)

Additional information for wireless adapters.

| Field | Description |
|-------|-------------|
| SSID | Connected network name |
| BSSID | Access point MAC address |
| Signal Strength | Signal quality (%) |
| Radio Type | 802.11 standard (a/b/g/n/ac/ax) |
| Channel | Wi-Fi channel |
| Frequency | Operating frequency (2.4/5/6 GHz) |
| Authentication | Security type (WPA2, WPA3, etc.) |
| Encryption | Encryption type (AES, TKIP) |

## UI Components

### Adapter Overview Card
- Shows all adapters in a card/tile layout
- Visual status indicator (green/yellow/red)
- Quick stats: IP, MAC, Speed
- Click to expand for details

### IP Configuration Panel
- Tabbed view for IPv4/IPv6
- DHCP status with lease timer
- Gateway and DNS at a glance

### Network Statistics Graph
- Real-time line chart for bandwidth
- Sent/Received differentiation
- Configurable time window (1 min, 5 min, 15 min)

### Connection Table
- Sortable/filterable table
- Process name lookup
- Quick copy functionality

## Rust Implementation Notes

### Primary Crate
- `sysinfo` - Basic network interface information
- `netstat2` or `netstat-rs` - Connection information
- `windows` crate - Windows-specific APIs for detailed info

### Data Collection

```rust
// Example command structure
#[tauri::command]
pub fn get_network_adapters() -> Vec<NetworkAdapter> {
    // Implementation using windows crate or sysinfo
}

#[tauri::command]
pub fn get_network_stats(adapter_id: String) -> NetworkStats {
    // Real-time statistics
}

#[tauri::command]
pub fn get_active_connections() -> Vec<NetworkConnection> {
    // Active TCP/UDP connections
}
```

## Refresh Behavior

| Data Type | Default Refresh | Configurable |
|-----------|-----------------|--------------|
| Adapter List | On demand | No |
| IP Configuration | 30 seconds | Yes |
| Statistics | 1 second | Yes |
| Connections | 5 seconds | Yes |
| Routing Table | On demand | No |

## Export Format

### JSON Export
```json
{
  "timestamp": "2024-01-15T10:30:00Z",
  "adapters": [
    {
      "name": "Ethernet",
      "mac": "AA:BB:CC:DD:EE:FF",
      "ipv4": {
        "address": "192.168.1.100",
        "subnet": "255.255.255.0",
        "gateway": "192.168.1.1"
      },
      "dns": ["8.8.8.8", "8.8.4.4"]
    }
  ]
}
```

### CSV Export
Flattened format suitable for spreadsheet analysis.

## Comparison to ipconfig

| ipconfig Feature | Syslens Enhancement |
|------------------|---------------------|
| IP Address | Visual display + copy button |
| Subnet Mask | CIDR notation included |
| Default Gateway | Hop count and latency |
| DNS Servers | Response time testing |
| DHCP Info | Countdown timer for lease |
| MAC Address | Vendor lookup |
| - | Real-time bandwidth graph |
| - | Active connection viewer |
| - | Wi-Fi signal strength |
| - | Historical statistics |
