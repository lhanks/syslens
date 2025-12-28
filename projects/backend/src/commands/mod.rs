//! Tauri IPC command handlers
//!
//! These commands are exposed to the Angular frontend via Tauri's invoke API.

pub mod network;
pub mod system;
pub mod hardware;
pub mod storage;
pub mod process;

pub use network::*;
pub use system::*;
pub use hardware::*;
pub use storage::*;
pub use process::*;
