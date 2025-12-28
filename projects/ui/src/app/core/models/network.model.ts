// Network adapter and configuration models

export interface NetworkAdapter {
  id: string;
  name: string;
  description: string;
  adapterType: AdapterType;
  macAddress: string;
  status: AdapterStatus;
  speedMbps: number | null;
  mtu: number;
  ipv4Config: Ipv4Config | null;
  ipv6Config: Ipv6Config | null;
  dnsConfig: DnsConfig;
}

export type AdapterType = 'Ethernet' | 'WiFi' | 'Virtual' | 'Loopback' | 'Unknown';

export type AdapterStatus = 'Up' | 'Down' | 'Disconnected' | 'Unknown';

export interface Ipv4Config {
  address: string;
  subnetMask: string;
  defaultGateway: string | null;
  dhcpEnabled: boolean;
  dhcpServer: string | null;
  leaseObtained: string | null;
  leaseExpires: string | null;
}

export interface Ipv6Config {
  linkLocalAddress: string;
  globalAddress: string | null;
  temporaryAddress: string | null;
  gateway: string | null;
}

export interface DnsConfig {
  servers: string[];
  suffix: string | null;
  searchList: string[];
}

export interface AdapterStats {
  adapterId: string;
  bytesSent: number;
  bytesReceived: number;
  packetsSent: number;
  packetsReceived: number;
  errors: number;
  discards: number;
  timestamp: number;
}

export interface NetworkConnection {
  protocol: 'TCP' | 'UDP';
  localAddress: string;
  localPort: number;
  remoteAddress: string;
  remotePort: number;
  state: ConnectionState;
  processName: string | null;
  pid: number;
}

export type ConnectionState =
  | 'Listen'
  | 'Established'
  | 'TimeWait'
  | 'CloseWait'
  | 'Closed'
  | 'SynSent'
  | 'SynReceived'
  | 'FinWait1'
  | 'FinWait2'
  | 'LastAck'
  | 'Closing'
  | 'Unknown';

export interface Route {
  destination: string;
  netmask: string;
  gateway: string;
  interfaceId: string;
  metric: number;
  routeType: 'Local' | 'Remote' | 'Default';
}
