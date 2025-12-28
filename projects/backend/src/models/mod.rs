//! Data models for Syslens
//!
//! These models are serialized to JSON and sent to the Angular frontend via Tauri IPC.

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
