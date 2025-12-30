//! Windows service models

use serde::{Deserialize, Serialize};

/// Information about a Windows service
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceInfo {
    /// Service name (internal identifier)
    pub name: String,
    /// Display name (shown in Services app)
    pub display_name: String,
    /// Service status (Running, Stopped, etc.)
    pub status: String,
    /// Startup type (Automatic, Manual, Disabled)
    pub startup_type: String,
    /// Service description
    pub description: Option<String>,
    /// Path to executable
    pub binary_path: Option<String>,
    /// Account the service runs as
    pub service_account: Option<String>,
    /// Process ID (if running)
    pub pid: Option<u32>,
}

/// Summary of services by status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceSummary {
    pub total: u32,
    pub running: u32,
    pub stopped: u32,
    pub start_pending: u32,
    pub stop_pending: u32,
}
