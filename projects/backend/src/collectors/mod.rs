//! Data collectors for gathering system information
//!
//! Each collector module provides functions to gather specific types of system data.

pub mod hardware;
pub mod network;
pub mod process;
pub mod service;
pub mod storage;
pub mod system;

pub use hardware::HardwareCollector;
pub use network::NetworkCollector;
pub use process::ProcessCollector;
pub use service::ServiceCollector;
pub use storage::StorageCollector;
pub use system::SystemCollector;
