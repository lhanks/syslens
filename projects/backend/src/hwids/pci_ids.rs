//! PCI device identification database.
//!
//! This module provides lookup services for PCI vendor and device names
//! based on Vendor ID and Device ID.
//!
//! Data is loaded from:
//! 1. Bundled pci.ids file (41K+ entries)
//! 2. Embedded fallback data for common devices

use std::collections::HashMap;
use std::sync::OnceLock;

/// Bundled PCI IDs database - included at compile time
const BUNDLED_PCI_IDS: &str = include_str!("../../resources/ids/pci.ids");

/// PCI ID database for vendor and device name lookup.
pub struct PciIdDatabase {
    vendors: HashMap<u16, String>,
    devices: HashMap<(u16, u16), String>,
}

/// Global PCI database instance.
static PCI_DATABASE: OnceLock<PciIdDatabase> = OnceLock::new();

impl PciIdDatabase {
    /// Get the global PCI ID database instance.
    pub fn global() -> &'static PciIdDatabase {
        PCI_DATABASE.get_or_init(|| {
            let mut db = PciIdDatabase {
                vendors: HashMap::new(),
                devices: HashMap::new(),
            };
            // Load bundled database first (comprehensive)
            db.load_bundled_data();
            // Add embedded fallback data (ensures critical devices are present)
            db.load_embedded_data();
            log::info!(
                "PCI database loaded: {} vendors, {} devices",
                db.vendors.len(),
                db.devices.len()
            );
            db
        })
    }

    /// Load data from the bundled pci.ids file.
    fn load_bundled_data(&mut self) {
        let mut current_vendor: Option<u16> = None;

        for line in BUNDLED_PCI_IDS.lines() {
            // Skip comments and empty lines
            if line.starts_with('#') || line.is_empty() {
                continue;
            }

            // Stop at class definitions
            if line.starts_with('C') && line.len() > 1 && line.chars().nth(1) == Some(' ') {
                break;
            }

            // Device line (starts with single tab, not double tab for subsystems)
            if line.starts_with('\t') && !line.starts_with("\t\t") {
                if let Some(vid) = current_vendor {
                    let trimmed = line.trim_start_matches('\t');
                    if let Some((did, name)) = Self::parse_id_line(trimmed) {
                        self.devices.insert((vid, did), name);
                    }
                }
            } else if !line.starts_with('\t') {
                // Vendor line
                if let Some((vid, name)) = Self::parse_id_line(line) {
                    self.vendors.insert(vid, name);
                    current_vendor = Some(vid);
                }
            }
            // Skip subsystem lines (double tab)
        }
    }

    /// Parse a line in format "xxxx  Name" where xxxx is a 4-digit hex ID.
    fn parse_id_line(line: &str) -> Option<(u16, String)> {
        let trimmed = line.trim();
        if trimmed.len() < 5 {
            return None;
        }

        let id_str = &trimmed[..4];
        let rest = trimmed[4..].trim_start();

        if let Ok(id) = u16::from_str_radix(id_str, 16) {
            if !rest.is_empty() {
                return Some((id, rest.to_string()));
            }
        }
        None
    }

    /// Load embedded vendor and device data.
    fn load_embedded_data(&mut self) {
        // Common PCI vendors (Vendor ID -> Name)
        let vendors = [
            // GPU Vendors
            (0x10de, "NVIDIA Corporation"),
            (0x1002, "Advanced Micro Devices, Inc. [AMD/ATI]"),
            (0x8086, "Intel Corporation"),
            (0x1a03, "ASPEED Technology, Inc."),
            (0x102b, "Matrox Electronics Systems Ltd."),
            (0x1039, "Silicon Integrated Systems [SiS]"),
            (0x5333, "S3 Graphics Ltd."),
            // CPU/Chipset Vendors
            (0x1022, "Advanced Micro Devices, Inc. [AMD]"),
            (0x1106, "VIA Technologies, Inc."),
            // Storage Controllers
            (0x1000, "Broadcom / LSI"),
            (0x1028, "Dell"),
            (0x103c, "Hewlett-Packard Company"),
            (0x1014, "IBM"),
            (0x15ad, "VMware"),
            (0x1af4, "Red Hat, Inc."),
            (0x144d, "Samsung Electronics Co Ltd"),
            (0x1c5c, "SK hynix"),
            (0x1179, "Toshiba Corporation"),
            (0x1987, "Phison Electronics Corporation"),
            (0x126f, "Silicon Motion, Inc."),
            (0x2646, "Kingston Technology Company, Inc."),
            (0x1e0f, "KIOXIA Corporation"),
            (0x15b7, "SanDisk Corp"),
            (0x1344, "Micron Technology Inc"),
            (0x1cc1, "ADATA Technology Co., Ltd."),
            (0xc0a9, "Micron/Crucial Technology"),
            // Network Controllers
            (0x14e4, "Broadcom Inc. and subsidiaries"),
            (0x10ec, "Realtek Semiconductor Co., Ltd."),
            (0x168c, "Qualcomm Atheros"),
            (0x8087, "Intel Corporation (Wireless)"),
            (0x11ab, "Marvell Technology Group Ltd."),
            (0x1969, "Qualcomm Atheros"),
            (0x10b7, "3Com Corporation"),
            (0x197b, "JMicron Technology Corp."),
            // USB Controllers
            (0x1912, "Renesas Technology Corp."),
            (0x1b21, "ASMedia Technology Inc."),
            (0x1b73, "Fresco Logic"),
            (0x104c, "Texas Instruments"),
            (0x1033, "NEC Corporation"),
            (0x0b05, "ASUSTek Computer, Inc."),
            // Audio
            (0x1102, "Creative Labs"),
            (0x13f6, "C-Media Electronics Inc"),
            // Motherboard/System
            (0x1043, "ASUSTeK Computer Inc."),
            (0x1462, "Micro-Star International Co., Ltd. [MSI]"),
            (0x1458, "Gigabyte Technology Co., Ltd"),
            (0x1849, "ASRock Incorporation"),
            (0x1025, "Acer Incorporated"),
            (0x17aa, "Lenovo"),
            (0x1028, "Dell"),
            (0x1297, "Shuttle Inc"),
            (0x1565, "Biostar Microtech Int'l Corp"),
            (0x10b9, "ULi Electronics Inc."),
            // Thunderbolt/USB4
            (0x8086, "Intel Corporation"),
        ];

        for (vid, name) in vendors {
            self.vendors.insert(vid, name.to_string());
        }

        // Common PCI devices (Vendor ID, Device ID) -> Name
        let devices = [
            // NVIDIA GPUs
            ((0x10de, 0x2684), "GeForce RTX 4090"),
            ((0x10de, 0x2704), "GeForce RTX 4080"),
            ((0x10de, 0x2782), "GeForce RTX 4070 Ti"),
            ((0x10de, 0x2786), "GeForce RTX 4070"),
            ((0x10de, 0x2860), "GeForce RTX 4070 Ti SUPER"),
            ((0x10de, 0x2882), "GeForce RTX 4070 SUPER"),
            ((0x10de, 0x28a0), "GeForce RTX 4060 Ti"),
            ((0x10de, 0x28a1), "GeForce RTX 4060"),
            ((0x10de, 0x2206), "GeForce RTX 3080"),
            ((0x10de, 0x2204), "GeForce RTX 3090"),
            ((0x10de, 0x2208), "GeForce RTX 3080 Ti"),
            ((0x10de, 0x2216), "GeForce RTX 3080 LHR"),
            ((0x10de, 0x2484), "GeForce RTX 3070"),
            ((0x10de, 0x2488), "GeForce RTX 3070 LHR"),
            ((0x10de, 0x2503), "GeForce RTX 3060"),
            ((0x10de, 0x2504), "GeForce RTX 3060 Ti"),
            ((0x10de, 0x2520), "GeForce RTX 3060 Mobile"),
            ((0x10de, 0x1e04), "GeForce RTX 2080 Ti"),
            ((0x10de, 0x1e07), "GeForce RTX 2080"),
            ((0x10de, 0x1e84), "GeForce RTX 2070"),
            ((0x10de, 0x1f02), "GeForce RTX 2070"),
            ((0x10de, 0x1f07), "GeForce RTX 2060"),
            ((0x10de, 0x1f08), "GeForce RTX 2060 SUPER"),
            ((0x10de, 0x1b80), "GeForce GTX 1080"),
            ((0x10de, 0x1b81), "GeForce GTX 1070"),
            ((0x10de, 0x1b82), "GeForce GTX 1070 Ti"),
            ((0x10de, 0x1b83), "GeForce GTX 1060 6GB"),
            ((0x10de, 0x1c81), "GeForce GTX 1050"),
            ((0x10de, 0x1c82), "GeForce GTX 1050 Ti"),
            ((0x10de, 0x1c03), "GeForce GTX 1060 3GB"),
            // AMD GPUs
            ((0x1002, 0x744c), "Radeon RX 7900 XTX"),
            ((0x1002, 0x7448), "Radeon RX 7900 XT"),
            ((0x1002, 0x7480), "Radeon RX 7800 XT"),
            ((0x1002, 0x7470), "Radeon RX 7700 XT"),
            ((0x1002, 0x7460), "Radeon RX 7600"),
            ((0x1002, 0x73bf), "Radeon RX 6900 XT"),
            ((0x1002, 0x73a5), "Radeon RX 6800 XT"),
            ((0x1002, 0x73a3), "Radeon RX 6800"),
            ((0x1002, 0x73df), "Radeon RX 6700 XT"),
            ((0x1002, 0x73ff), "Radeon RX 6600 XT"),
            ((0x1002, 0x73e3), "Radeon RX 6600"),
            ((0x1002, 0x731f), "Radeon RX 5700 XT"),
            ((0x1002, 0x7310), "Radeon RX 5700"),
            ((0x1002, 0x7340), "Radeon RX 5500 XT"),
            ((0x1002, 0x67df), "Radeon RX 580"),
            ((0x1002, 0x67ef), "Radeon RX 560"),
            ((0x1002, 0x699f), "Radeon RX 550"),
            // Intel GPUs
            ((0x8086, 0x56a0), "Arc A770"),
            ((0x8086, 0x56a1), "Arc A750"),
            ((0x8086, 0x5690), "Arc A380"),
            ((0x8086, 0x9a49), "UHD Graphics (Tiger Lake)"),
            ((0x8086, 0x4680), "UHD Graphics 770"),
            ((0x8086, 0x4692), "UHD Graphics 730"),
            ((0x8086, 0x3e92), "UHD Graphics 630"),
            ((0x8086, 0x5912), "HD Graphics 630"),
            ((0x8086, 0x5917), "UHD Graphics 620"),
            ((0x8086, 0x191b), "HD Graphics 530"),
            // Intel/AMD Chipsets
            (
                (0x8086, 0xa382),
                "400 Series Chipset Family SATA AHCI Controller",
            ),
            ((0x8086, 0xa352), "Q370 Chipset SATA AHCI Controller"),
            (
                (0x8086, 0x43d2),
                "500 Series Chipset Family SATA AHCI Controller",
            ),
            (
                (0x8086, 0x7a83),
                "600/700 Series Chipset SATA AHCI Controller",
            ),
            ((0x1022, 0x43b7), "300 Series Chipset SATA Controller"),
            ((0x1022, 0x43eb), "500 Series Chipset SATA Controller"),
            // NVMe Controllers
            ((0x144d, 0xa808), "NVMe SSD Controller SM981/PM981/PM983"),
            ((0x144d, 0xa809), "NVMe SSD Controller 980"),
            ((0x144d, 0xa80a), "NVMe SSD Controller PM9A1/PM9A3/980PRO"),
            ((0x144d, 0xa80b), "NVMe SSD Controller 990 PRO"),
            ((0x15b7, 0x5006), "PC SN530 NVMe SSD"),
            ((0x15b7, 0x5009), "WD Blue SN550 NVMe SSD"),
            ((0x15b7, 0x5011), "WD Black SN770 NVMe SSD"),
            ((0x1987, 0x5012), "E12 NVMe Controller"),
            ((0x1987, 0x5016), "E16 NVMe Controller"),
            ((0x1987, 0x5018), "E18 NVMe Controller"),
            ((0x126f, 0x2263), "SM2263EN/SM2263XT NVMe Controller"),
            (
                (0x1c5c, 0x174a),
                "Gold P31/BC711/PC711 NVMe Solid State Drive",
            ),
            ((0x1344, 0x5410), "2200S NVMe SSD"),
            ((0x1344, 0x5411), "2300 NVMe SSD"),
            ((0x2646, 0x5008), "A1000/U-SNS8154P3 NVMe SSD"),
            ((0x2646, 0x500f), "NV1 NVMe PCIe SSD"),
            ((0x2646, 0x5013), "KC3000/FURY Renegade NVMe SSD"),
            ((0x1e0f, 0x0001), "XG6 NVMe SSD Controller"),
            // Realtek Network
            (
                (0x10ec, 0x8168),
                "RTL8111/8168/8411 PCI Express Gigabit Ethernet",
            ),
            ((0x10ec, 0x8125), "RTL8125 2.5GbE Controller"),
            ((0x10ec, 0x2600), "RTL8125 2.5GbE Controller"),
            ((0x10ec, 0x8139), "RTL-8100/8101L/8139 PCI Fast Ethernet"),
            ((0x10ec, 0x8136), "RTL810xE PCI Express Fast Ethernet"),
            // Intel Network
            ((0x8086, 0x15b8), "Ethernet Connection (2) I219-V"),
            ((0x8086, 0x15bc), "Ethernet Connection (7) I219-V"),
            ((0x8086, 0x0d4f), "Ethernet Connection (10) I219-V"),
            ((0x8086, 0x15f3), "Ethernet Controller I225-V"),
            ((0x8086, 0x125c), "Ethernet Controller I226-V"),
            ((0x8086, 0x1533), "I210 Gigabit Network Connection"),
            ((0x8086, 0x10d3), "82574L Gigabit Network Connection"),
            // Intel Wireless
            ((0x8086, 0x2723), "Wi-Fi 6 AX200"),
            ((0x8086, 0x2725), "Wi-Fi 6 AX210/AX211/AX411"),
            ((0x8086, 0xa0f0), "Wi-Fi 6 AX201"),
            ((0x8086, 0x51f0), "Wi-Fi 6 AX211"),
            ((0x8086, 0x7a70), "Wi-Fi 7 BE200"),
            ((0x8086, 0x24fd), "Wireless 8265 / 8275"),
            ((0x8086, 0x24fb), "Wireless-AC 9260"),
            ((0x8086, 0x9df0), "Cannon Point-LP CNVi [Wireless-AC]"),
            // Realtek/MediaTek Wireless
            (
                (0x10ec, 0xc822),
                "RTL8822CE 802.11ac PCIe Wireless Network Adapter",
            ),
            (
                (0x10ec, 0x8852),
                "RTL8852AE 802.11ax PCIe Wireless Network Adapter",
            ),
            (
                (0x10ec, 0xc852),
                "RTL8852CE PCIe 802.11ax Wireless Network Controller",
            ),
            // USB Controllers
            (
                (0x8086, 0xa36d),
                "Cannon Lake PCH USB 3.1 xHCI Host Controller",
            ),
            (
                (0x8086, 0x43ed),
                "Tiger Lake-H USB 3.2 Gen 2x1 xHCI Host Controller",
            ),
            (
                (0x8086, 0x7ae0),
                "Alder Lake-S PCH USB 3.2 Gen 2x2 XHCI Host Controller",
            ),
            ((0x1022, 0x149c), "Matisse USB 3.0 Host Controller"),
            (
                (0x1022, 0x43d5),
                "400 Series Chipset USB 3.1 xHCI Compliant Host Controller",
            ),
            ((0x1b21, 0x1142), "ASM1042 SuperSpeed USB Host Controller"),
            ((0x1b21, 0x2142), "ASM2142/ASM3142 USB 3.1 Host Controller"),
            ((0x1b21, 0x3241), "ASM3241 USB 3.2 Gen2 Host Controller"),
            // Audio
            ((0x8086, 0xa348), "Cannon Lake PCH cAVS"),
            (
                (0x8086, 0xf0c8),
                "Smart Sound Technology (SST) Audio Controller",
            ),
            ((0x8086, 0x7ad0), "Alder Lake-S HD Audio Controller"),
            ((0x1022, 0x1487), "Starship/Matisse HD Audio Controller"),
            ((0x1002, 0xab38), "Navi 10 HDMI Audio"),
            ((0x10de, 0x228b), "GA104 High Definition Audio Controller"),
        ];

        for ((vid, did), name) in devices {
            self.devices.insert((vid, did), name.to_string());
        }
    }

    /// Look up a vendor name by Vendor ID.
    pub fn get_vendor(&self, vid: u16) -> Option<&str> {
        self.vendors.get(&vid).map(|s| s.as_str())
    }

    /// Look up a device name by Vendor ID and Device ID.
    pub fn get_device(&self, vid: u16, did: u16) -> Option<&str> {
        self.devices.get(&(vid, did)).map(|s| s.as_str())
    }

    /// Look up both vendor and device names.
    /// Returns (vendor_name, device_name) where either may be None.
    pub fn lookup(&self, vid: u16, did: u16) -> (Option<&str>, Option<&str>) {
        (self.get_vendor(vid), self.get_device(vid, did))
    }

    /// Format a human-readable device description.
    /// Returns "Vendor Device" if both found, just "Vendor" or "Device" if one found,
    /// or formatted hex IDs if neither found.
    pub fn format_device(&self, vid: u16, did: u16) -> String {
        match self.lookup(vid, did) {
            (Some(vendor), Some(device)) => format!("{} {}", vendor, device),
            (Some(vendor), None) => format!("{} (Device {:04X})", vendor, did),
            (None, Some(device)) => format!("(Vendor {:04X}) {}", vid, device),
            (None, None) => format!("{:04X}:{:04X}", vid, did),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vendor_lookup() {
        let db = PciIdDatabase::global();
        assert_eq!(db.get_vendor(0x10de), Some("NVIDIA Corporation"));
        assert_eq!(
            db.get_vendor(0x1002),
            Some("Advanced Micro Devices, Inc. [AMD/ATI]")
        );
        assert_eq!(db.get_vendor(0x8086), Some("Intel Corporation"));
    }

    #[test]
    fn test_device_lookup() {
        let db = PciIdDatabase::global();
        assert_eq!(db.get_device(0x10de, 0x2684), Some("GeForce RTX 4090"));
        assert_eq!(
            db.get_device(0x10ec, 0x8168),
            Some("RTL8111/8168/8411 PCI Express Gigabit Ethernet")
        );
    }

    #[test]
    fn test_format_device() {
        let db = PciIdDatabase::global();
        let desc = db.format_device(0x10de, 0x2684);
        assert!(desc.contains("NVIDIA"));
        assert!(desc.contains("RTX 4090"));
    }

    #[test]
    fn test_unknown_device() {
        let db = PciIdDatabase::global();
        // Database is comprehensive (41K+ entries), test that unknown returns hex format
        // Use vendor 0x0000 which is reserved/invalid
        let desc = db.format_device(0x0000, 0x0000);
        assert_eq!(desc, "0000:0000");
    }

    #[test]
    fn test_bundled_database_loaded() {
        let db = PciIdDatabase::global();
        // Verify bundled database was loaded (should have thousands of entries)
        // With embedded data alone, we'd have ~60 vendors; with pci.ids, we have ~3000+
        assert!(db.vendors.len() > 1000, "Expected 1000+ vendors from bundled pci.ids");
        assert!(db.devices.len() > 5000, "Expected 5000+ devices from bundled pci.ids");
    }
}
