//! Syslens - System Monitoring Library
//!
//! This library provides system information collection and Tauri command handlers
//! for the Syslens desktop application.

pub mod collectors;
pub mod commands;
pub mod models;

// Re-export commonly used items without ambiguity
pub use collectors::hardware::HardwareCollector;
pub use collectors::network::NetworkCollector;
pub use collectors::process::ProcessCollector;
pub use collectors::storage::StorageCollector;
pub use collectors::system::SystemCollector;
