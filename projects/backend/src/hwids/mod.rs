//! Hardware ID database module for USB and PCI device identification.
//!
//! This module provides lookup services for vendor and product names
//! based on hardware IDs (VID/PID for USB, Vendor/Device for PCI).
//!
//! The databases include embedded data for common devices and can be
//! updated from official sources via the updater module.

mod pci_ids;
mod updater;
mod usb_ids;

pub use pci_ids::PciIdDatabase;
pub use updater::{needs_update, update_databases, UpdateResult};
pub use usb_ids::UsbIdDatabase;

use std::path::PathBuf;

/// Get the data directory for storing hardware ID databases.
/// Returns the app data directory on Windows, or a fallback path.
pub fn get_data_dir() -> PathBuf {
    // Try to get the app data directory
    if let Some(data_dir) = dirs::data_local_dir() {
        let app_dir = data_dir.join("syslens").join("hwids");
        if std::fs::create_dir_all(&app_dir).is_ok() {
            return app_dir;
        }
    }

    // Fallback to current directory
    PathBuf::from(".")
}
