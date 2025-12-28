//! Data collectors for gathering system information
//!
//! Each collector module provides functions to gather specific types of system data.

pub mod network;
pub mod system;
pub mod hardware;
pub mod storage;

pub use network::NetworkCollector;
pub use system::SystemCollector;
pub use hardware::HardwareCollector;
pub use storage::StorageCollector;
