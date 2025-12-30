/** Information about a Windows service */
export interface ServiceInfo {
  /** Service name (internal identifier) */
  name: string;
  /** Display name (shown in Services app) */
  displayName: string;
  /** Service status (Running, Stopped, etc.) */
  status: string;
  /** Startup type (Automatic, Manual, Disabled) */
  startupType: string;
  /** Service description */
  description: string | null;
  /** Path to executable */
  binaryPath: string | null;
  /** Account the service runs as */
  serviceAccount: string | null;
  /** Process ID (if running) */
  pid: number | null;
}

/** Summary of services by status */
export interface ServiceSummary {
  total: number;
  running: number;
  stopped: number;
  startPending: number;
  stopPending: number;
}
