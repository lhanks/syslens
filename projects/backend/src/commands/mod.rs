//! Tauri IPC command handlers
//!
//! These commands are exposed to the Angular frontend via Tauri's invoke API.

pub mod device_info;
pub mod hardware;
pub mod network;
pub mod process;
pub mod storage;
pub mod system;

pub use device_info::*;
pub use hardware::*;
pub use network::*;
pub use process::*;
pub use storage::*;
pub use system::*;
