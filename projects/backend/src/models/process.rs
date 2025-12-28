//! Process-related data models

use serde::{Deserialize, Serialize};

/// Information about a running process
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessInfo {
    /// Process ID
    pub pid: u32,
    /// Parent process ID
    pub parent_pid: Option<u32>,
    /// Process name
    pub name: String,
    /// CPU usage percentage (0-100)
    pub cpu_usage: f32,
    /// Memory usage in bytes
    pub memory_bytes: u64,
    /// Virtual memory usage in bytes
    pub virtual_memory_bytes: u64,
    /// Process status (Running, Sleeping, etc.)
    pub status: String,
    /// User who owns the process
    pub user: Option<String>,
    /// Command line used to start the process
    pub command: String,
    /// Process start time (Unix timestamp in seconds)
    pub start_time: u64,
    /// Disk read bytes
    pub disk_read_bytes: u64,
    /// Disk write bytes
    pub disk_write_bytes: u64,
}

/// Summary of system processes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessSummary {
    /// Total number of processes
    pub total_count: usize,
    /// Number of running processes
    pub running_count: usize,
    /// Number of sleeping processes
    pub sleeping_count: usize,
    /// Total CPU usage across all processes
    pub total_cpu_usage: f32,
    /// Total memory usage across all processes
    pub total_memory_bytes: u64,
}
