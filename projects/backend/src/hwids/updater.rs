//! Hardware ID database updater.
//!
//! Downloads and parses official USB and PCI ID databases from:
//! - USB: http://www.linux-usb.org/usb.ids
//! - PCI: https://pci-ids.ucw.cz/v2.2/pci.ids

use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

const USB_IDS_URL: &str = "http://www.linux-usb.org/usb.ids";
const PCI_IDS_URL: &str = "https://pci-ids.ucw.cz/v2.2/pci.ids";
const UPDATE_INTERVAL_DAYS: u64 = 30;

/// Database update result
#[derive(Debug)]
pub struct UpdateResult {
    pub usb_updated: bool,
    pub pci_updated: bool,
    pub usb_vendors: usize,
    pub usb_products: usize,
    pub pci_vendors: usize,
    pub pci_devices: usize,
    pub error: Option<String>,
}

/// Parsed USB ID data
pub struct ParsedUsbIds {
    pub vendors: HashMap<u16, String>,
    pub products: HashMap<(u16, u16), String>,
}

/// Parsed PCI ID data
pub struct ParsedPciIds {
    pub vendors: HashMap<u16, String>,
    pub devices: HashMap<(u16, u16), String>,
}

/// Check if the database file needs updating.
pub fn needs_update(file_path: &Path) -> bool {
    if !file_path.exists() {
        return true;
    }

    if let Ok(metadata) = fs::metadata(file_path) {
        if let Ok(modified) = metadata.modified() {
            if let Ok(elapsed) = SystemTime::now().duration_since(modified) {
                return elapsed > Duration::from_secs(UPDATE_INTERVAL_DAYS * 24 * 60 * 60);
            }
        }
    }

    true
}

/// Download and update both databases.
pub async fn update_databases(data_dir: &Path) -> UpdateResult {
    let mut result = UpdateResult {
        usb_updated: false,
        pci_updated: false,
        usb_vendors: 0,
        usb_products: 0,
        pci_vendors: 0,
        pci_devices: 0,
        error: None,
    };

    let usb_path = data_dir.join("usb.ids");
    let pci_path = data_dir.join("pci.ids");

    // Update USB IDs if needed
    if needs_update(&usb_path) {
        match download_file(USB_IDS_URL, &usb_path).await {
            Ok(_) => {
                result.usb_updated = true;
                log::info!("USB ID database updated");
            }
            Err(e) => {
                let msg = format!("Failed to update USB IDs: {}", e);
                log::warn!("{}", msg);
                if result.error.is_none() {
                    result.error = Some(msg);
                }
            }
        }
    }

    // Update PCI IDs if needed
    if needs_update(&pci_path) {
        match download_file(PCI_IDS_URL, &pci_path).await {
            Ok(_) => {
                result.pci_updated = true;
                log::info!("PCI ID database updated");
            }
            Err(e) => {
                let msg = format!("Failed to update PCI IDs: {}", e);
                log::warn!("{}", msg);
                if result.error.is_none() {
                    result.error = Some(msg);
                }
            }
        }
    }

    // Parse and count entries
    if usb_path.exists() {
        if let Ok(parsed) = parse_usb_ids(&usb_path) {
            result.usb_vendors = parsed.vendors.len();
            result.usb_products = parsed.products.len();
        }
    }

    if pci_path.exists() {
        if let Ok(parsed) = parse_pci_ids(&pci_path) {
            result.pci_vendors = parsed.vendors.len();
            result.pci_devices = parsed.devices.len();
        }
    }

    result
}

/// Download a file from URL to the specified path.
async fn download_file(
    url: &str,
    path: &PathBuf,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let response = reqwest::get(url).await?;

    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()).into());
    }

    let content = response.text().await?;

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(path, content)?;
    Ok(())
}

/// Parse USB IDs file format.
/// Format:
/// vendor_id  vendor_name
/// \tproduct_id  product_name
pub fn parse_usb_ids(path: &PathBuf) -> Result<ParsedUsbIds, std::io::Error> {
    let file = fs::File::open(path)?;
    let reader = BufReader::new(file);

    let mut vendors = HashMap::new();
    let mut products = HashMap::new();
    let mut current_vendor: Option<u16> = None;

    for line in reader.lines() {
        let line = line?;

        // Skip comments and empty lines
        if line.starts_with('#') || line.is_empty() {
            continue;
        }

        // Skip class definitions (lines starting with 'C ')
        if line.starts_with('C') && line.len() > 1 && line.chars().nth(1) == Some(' ') {
            break; // Classes come after vendors, so we can stop here
        }

        // Product line (starts with tab)
        if line.starts_with('\t') {
            if let Some(vid) = current_vendor {
                let trimmed = line.trim_start_matches('\t');
                if let Some((pid_str, name)) = parse_id_line(trimmed) {
                    if let Ok(pid) = u16::from_str_radix(pid_str, 16) {
                        products.insert((vid, pid), name.to_string());
                    }
                }
            }
        } else if !line.starts_with('\t') {
            // Vendor line
            if let Some((vid_str, name)) = parse_id_line(&line) {
                if let Ok(vid) = u16::from_str_radix(vid_str, 16) {
                    vendors.insert(vid, name.to_string());
                    current_vendor = Some(vid);
                }
            }
        }
    }

    Ok(ParsedUsbIds { vendors, products })
}

/// Parse PCI IDs file format (same format as USB).
pub fn parse_pci_ids(path: &PathBuf) -> Result<ParsedPciIds, std::io::Error> {
    let file = fs::File::open(path)?;
    let reader = BufReader::new(file);

    let mut vendors = HashMap::new();
    let mut devices = HashMap::new();
    let mut current_vendor: Option<u16> = None;

    for line in reader.lines() {
        let line = line?;

        // Skip comments and empty lines
        if line.starts_with('#') || line.is_empty() {
            continue;
        }

        // Skip class definitions
        if line.starts_with('C') && line.len() > 1 && line.chars().nth(1) == Some(' ') {
            break;
        }

        // Device line (starts with single tab)
        if line.starts_with('\t') && !line.starts_with("\t\t") {
            if let Some(vid) = current_vendor {
                let trimmed = line.trim_start_matches('\t');
                if let Some((did_str, name)) = parse_id_line(trimmed) {
                    if let Ok(did) = u16::from_str_radix(did_str, 16) {
                        devices.insert((vid, did), name.to_string());
                    }
                }
            }
        } else if !line.starts_with('\t') {
            // Vendor line
            if let Some((vid_str, name)) = parse_id_line(&line) {
                if let Ok(vid) = u16::from_str_radix(vid_str, 16) {
                    vendors.insert(vid, name.to_string());
                    current_vendor = Some(vid);
                }
            }
        }
        // Skip subsystem lines (start with double tab)
    }

    Ok(ParsedPciIds { vendors, devices })
}

/// Parse a line in format "id  name" or "id\tname"
fn parse_id_line(line: &str) -> Option<(&str, &str)> {
    // Split on first whitespace
    let trimmed = line.trim();

    // Find first space or tab after the ID (IDs are 4 hex chars)
    if trimmed.len() < 5 {
        return None;
    }

    // The ID is always 4 characters, followed by spaces
    let id = &trimmed[..4];
    let rest = trimmed[4..].trim_start();

    if id.chars().all(|c| c.is_ascii_hexdigit()) && !rest.is_empty() {
        Some((id, rest))
    } else {
        None
    }
}

/// Get the path to the cached USB IDs file.
#[allow(dead_code)]
pub fn get_usb_ids_path(data_dir: &Path) -> PathBuf {
    data_dir.join("usb.ids")
}

/// Get the path to the cached PCI IDs file.
#[allow(dead_code)]
pub fn get_pci_ids_path(data_dir: &Path) -> PathBuf {
    data_dir.join("pci.ids")
}

/// Load USB IDs from cached file, falling back to embedded data if not available.
#[allow(dead_code)]
pub fn load_usb_ids(data_dir: &Path) -> Option<ParsedUsbIds> {
    let path = get_usb_ids_path(data_dir);
    if path.exists() {
        parse_usb_ids(&path).ok()
    } else {
        None
    }
}

/// Load PCI IDs from cached file, falling back to embedded data if not available.
#[allow(dead_code)]
pub fn load_pci_ids(data_dir: &Path) -> Option<ParsedPciIds> {
    let path = get_pci_ids_path(data_dir);
    if path.exists() {
        parse_pci_ids(&path).ok()
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_id_line() {
        assert_eq!(
            parse_id_line("046d  Logitech, Inc."),
            Some(("046d", "Logitech, Inc."))
        );
        assert_eq!(
            parse_id_line("c52b\tUnifying Receiver"),
            Some(("c52b", "Unifying Receiver"))
        );
        assert_eq!(parse_id_line(""), None);
        assert_eq!(parse_id_line("xyz"), None);
    }
}
