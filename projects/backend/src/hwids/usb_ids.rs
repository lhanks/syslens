//! USB ID database for vendor and product identification.
//!
//! This module provides lookup functionality for USB Vendor IDs (VID) and
//! Product IDs (PID) to resolve human-readable names.
//!
//! Data is loaded from:
//! 1. Bundled usb.ids file (25K+ entries)
//! 2. Embedded fallback data for common devices

use std::collections::HashMap;
use std::sync::OnceLock;

/// Bundled USB IDs database - included at compile time
const BUNDLED_USB_IDS: &str = include_str!("../../resources/ids/usb.ids");

/// USB ID Database for looking up vendor and product names
pub struct UsbIdDatabase {
    /// Map of vendor ID to vendor name
    vendors: HashMap<u16, String>,
    /// Map of (vendor ID, product ID) to product name
    products: HashMap<(u16, u16), String>,
}

/// Global static instance of the USB ID database
static USB_DATABASE: OnceLock<UsbIdDatabase> = OnceLock::new();

impl UsbIdDatabase {
    /// Get the global USB ID database instance
    pub fn global() -> &'static UsbIdDatabase {
        USB_DATABASE.get_or_init(|| {
            let mut db = UsbIdDatabase::new();
            // Load bundled database first (comprehensive)
            db.load_bundled_data();
            // Add embedded fallback data (ensures critical devices are present)
            db.load_embedded_data();
            log::info!(
                "USB database loaded: {} vendors, {} products",
                db.vendors.len(),
                db.products.len()
            );
            db
        })
    }

    /// Load data from the bundled usb.ids file.
    fn load_bundled_data(&mut self) {
        let mut current_vendor: Option<u16> = None;

        for line in BUNDLED_USB_IDS.lines() {
            // Skip comments and empty lines
            if line.starts_with('#') || line.is_empty() {
                continue;
            }

            // Stop at class definitions (lines starting with "C " for device classes)
            if line.starts_with("C ") {
                break;
            }

            // Product line (starts with single tab, not double tab for interfaces)
            if line.starts_with('\t') && !line.starts_with("\t\t") {
                if let Some(vid) = current_vendor {
                    let trimmed = line.trim_start_matches('\t');
                    if let Some((pid, name)) = Self::parse_id_line(trimmed) {
                        self.products.insert((vid, pid), name);
                    }
                }
            } else if !line.starts_with('\t') {
                // Vendor line
                if let Some((vid, name)) = Self::parse_id_line(line) {
                    self.vendors.insert(vid, name);
                    current_vendor = Some(vid);
                }
            }
            // Skip interface lines (double tab)
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

    /// Create a new empty USB ID database
    pub fn new() -> Self {
        Self {
            vendors: HashMap::new(),
            products: HashMap::new(),
        }
    }

    /// Look up a vendor name by ID
    pub fn get_vendor(&self, vid: u16) -> Option<&str> {
        self.vendors.get(&vid).map(|s| s.as_str())
    }

    /// Look up a product name by vendor and product ID
    pub fn get_product(&self, vid: u16, pid: u16) -> Option<&str> {
        self.products.get(&(vid, pid)).map(|s| s.as_str())
    }

    /// Look up both vendor and product names
    pub fn lookup(&self, vid: u16, pid: u16) -> (Option<&str>, Option<&str>) {
        (self.get_vendor(vid), self.get_product(vid, pid))
    }

    /// Add a vendor to the database
    pub fn add_vendor(&mut self, vid: u16, name: &str) {
        self.vendors.insert(vid, name.to_string());
    }

    /// Add a product to the database
    pub fn add_product(&mut self, vid: u16, pid: u16, name: &str) {
        self.products.insert((vid, pid), name.to_string());
    }

    /// Load embedded vendor/product data (most common devices)
    fn load_embedded_data(&mut self) {
        // Common USB vendors with their well-known vendor IDs
        let vendors: &[(u16, &str)] = &[
            // Major computer/peripheral manufacturers
            (0x03F0, "HP"),
            (0x0409, "NEC"),
            (0x041E, "Creative Technology"),
            (0x0424, "Microchip Technology"),
            (0x043E, "LG Electronics"),
            (0x044F, "ThrustMaster"),
            (0x045E, "Microsoft"),
            (0x046D, "Logitech"),
            (0x0471, "Philips"),
            (0x047F, "Plantronics"),
            (0x048D, "Integrated Technology Express"),
            (0x04B3, "IBM"),
            (0x04B4, "Cypress Semiconductor"),
            (0x04CA, "Lite-On Technology"),
            (0x04D8, "Microchip Technology"),
            (0x04D9, "Holtek Semiconductor"),
            (0x04E8, "Samsung Electronics"),
            (0x04F2, "Chicony Electronics"),
            (0x04F3, "Elan Microelectronics"),
            (0x050D, "Belkin"),
            (0x0518, "EzKEY"),
            (0x051D, "American Power Conversion"),
            (0x0557, "ATEN International"),
            (0x0572, "Conexant Systems"),
            (0x058F, "Alcor Micro"),
            (0x0596, "MicroTouch Systems"),
            (0x05AC, "Apple"),
            (0x05C6, "Qualcomm"),
            (0x05DC, "Lexar Media"),
            (0x05E3, "Genesys Logic"),
            (0x0609, "SMK Manufacturing"),
            (0x064E, "Suyin"),
            (0x067B, "Prolific Technology"),
            (0x0681, "Siemens"),
            (0x06CB, "Synaptics"),
            (0x0711, "Magic Control Technology"),
            (0x0738, "Mad Catz"),
            (0x0781, "SanDisk"),
            (0x07AB, "Freecom Technologies"),
            (0x07CA, "AVerMedia Technologies"),
            (0x0846, "NetGear"),
            (0x08A9, "CWAV"),
            (0x08BB, "Texas Instruments"),
            (0x08EC, "M-Systems Flash Disk Pioneers"),
            (0x08FF, "AuthenTec"),
            (0x090C, "Silicon Motion"),
            (0x0951, "Kingston Technology"),
            (0x09DA, "A4Tech"),
            (0x0A12, "Cambridge Silicon Radio"),
            (0x0A5C, "Broadcom"),
            (0x0B05, "ASUSTek Computer"),
            (0x0B95, "ASIX Electronics"),
            (0x0BB4, "HTC"),
            (0x0BDA, "Realtek Semiconductor"),
            (0x0C45, "Microdia"),
            (0x0CF3, "Qualcomm Atheros Communications"),
            (0x0D8C, "C-Media Electronics"),
            (0x0E0F, "VMware"),
            (0x0E8D, "MediaTek"),
            (0x0FCE, "Sony Ericsson"),
            (0x1005, "Apacer Technology"),
            (0x1038, "SteelSeries ApS"),
            (0x1044, "Chu Yuen Enterprise"),
            (0x1050, "Yubico.com"),
            (0x106B, "Apple"),
            (0x1093, "National Instruments"),
            (0x10C4, "Silicon Labs"),
            (0x1130, "Tenx Technology"),
            (0x1199, "Sierra Wireless"),
            (0x1209, "Generic"),
            (0x1235, "Focusrite-Novation"),
            (0x1241, "Belkin"),
            (0x125F, "A-DATA Technology"),
            (0x1266, "Pirelli Broadband Solutions"),
            (0x12D1, "Huawei Technologies"),
            (0x1307, "Transcend Information"),
            (0x1366, "SEGGER"),
            (0x13FE, "Kingston Technology"),
            (0x1415, "Nam Tai E&E Products"),
            (0x1462, "Micro Star International"),
            (0x148F, "Ralink Technology"),
            (0x1532, "Razer USA"),
            (0x154B, "PNY"),
            (0x1557, "OQO"),
            (0x15D9, "Trust International"),
            (0x1618, "Waltop International"),
            (0x16C0, "Van Ooijen Technische Informatica"),
            (0x174C, "ASMedia Technology"),
            (0x17EF, "Lenovo"),
            (0x17F4, "WaveSense"),
            (0x1852, "Gyration"),
            (0x18A5, "Verbatim"),
            (0x18D1, "Google"),
            (0x1908, "GEMBIRD"),
            (0x192F, "Avago Technologies"),
            (0x1949, "Lab126"),
            (0x195D, "Itron Technology"),
            (0x1A40, "Terminus Technology"),
            (0x1A86, "QinHeng Electronics"),
            (0x1B1C, "Corsair"),
            (0x1B3F, "Generalplus Technology"),
            (0x1BCF, "Sunplus Innovation Technology"),
            (0x1C4F, "SiGma Micro"),
            (0x1D6B, "Linux Foundation"),
            (0x1E71, "NZXT"),
            (0x1EA7, "SHARKOON Technologies"),
            (0x1EDB, "Blackmagic Design"),
            (0x1F75, "Innostor Technology"),
            (0x2109, "VIA Labs"),
            (0x214B, "Huasheng Electronics"),
            (0x2207, "Rockchip Electronics"),
            (0x22B8, "Motorola PCS"),
            (0x2341, "Arduino"),
            (0x239A, "Adafruit"),
            (0x24AE, "Shenzhen Rapoo Technology"),
            (0x2516, "Cooler Master"),
            (0x256F, "3Dconnexion"),
            (0x258A, "SINOWEALTH"),
            (0x262A, "Savitech"),
            (0x2717, "Xiaomi"),
            (0x27B8, "ThingM"),
            (0x28DE, "Valve"),
            (0x291A, "Anker"),
            (0x2B89, "Dygma"),
            (0x2C7C, "Quectel Wireless Solutions"),
            (0x2FD2, "Broadcom"),
            (0x2FE0, "TOSHIBA"),
            (0x3006, "ELECOM"),
            (0x30DE, "Topaz Systems"),
            (0x3151, "Athena Smartcard Solutions"),
            (0x3243, "SenseAir"),
            (0x3297, "ZSA Technology Labs"),
            (0x32AC, "8BitDo"),
            (0x3367, "Poslab Technology"),
            (0x3434, "Keychron"),
            (0x3553, "EGALAX_EMPIA Technology"),
            (0x3689, "Dygma"),
            (0x4653, "Xilinx"),
            (0x4E53, "Nintendo"),
            (0x5131, "INAT"),
            (0x534D, "MacroSilicon"),
            (0x584D, "Spreadtrum Communications"),
            (0x6000, "Ingenious Technologies"),
            (0x6006, "Analog Devices"),
            (0x6472, "Clavia DMI"),
            (0x7392, "Edimax Technology"),
            (0x8086, "Intel"),
            (0x8087, "Intel"),
            (0x80EE, "VirtualBox"),
            (0x8564, "Transcend Information"),
            (0x8644, "Intenso"),
            (0x9710, "MosChip Semiconductor"),
            (0xA168, "AnMo Electronics"),
            (0xB58E, "Blue Microphones"),
        ];

        for (vid, name) in vendors {
            self.add_vendor(*vid, name);
        }

        // Common products (well-known devices)
        let products: &[(u16, u16, &str)] = &[
            // Microsoft
            (0x045E, 0x028E, "Xbox360 Controller"),
            (0x045E, 0x02D1, "Xbox One Controller"),
            (0x045E, 0x02E0, "Xbox One S Controller"),
            (0x045E, 0x02FF, "Xbox One Elite Controller"),
            (0x045E, 0x0B12, "Xbox Series X Controller"),
            (0x045E, 0x00DD, "Optical Mouse"),
            (0x045E, 0x0745, "Nano Transceiver"),
            (0x045E, 0x07A5, "Wireless Receiver 1461C"),
            // Logitech
            (0x046D, 0x082D, "HD Pro Webcam C920"),
            (0x046D, 0x0825, "Webcam C270"),
            (0x046D, 0x0843, "Webcam C930e"),
            (0x046D, 0x085E, "BRIO 4K Webcam"),
            (0x046D, 0x0892, "StreamCam"),
            (0x046D, 0x0A29, "H600 Wireless Headset"),
            (0x046D, 0x0A37, "USB Headset H540"),
            (0x046D, 0x0A38, "USB Headset H340"),
            (0x046D, 0x0A44, "Headset H390"),
            (0x046D, 0x0A66, "G433 Gaming Headset"),
            (0x046D, 0xC077, "M105 Optical Mouse"),
            (0x046D, 0xC08B, "G502 HERO Gaming Mouse"),
            (0x046D, 0xC24A, "G600 Gaming Mouse"),
            (0x046D, 0xC332, "G502 SE HERO Gaming Mouse"),
            (0x046D, 0xC539, "Lightspeed Receiver"),
            (0x046D, 0xC52B, "Unifying Receiver"),
            (0x046D, 0xC548, "Logi Bolt Receiver"),
            // Apple
            (0x05AC, 0x024F, "Aluminum Keyboard (ANSI)"),
            (0x05AC, 0x0256, "Magic Keyboard"),
            (0x05AC, 0x030D, "Magic Mouse 2"),
            (0x05AC, 0x0265, "Magic Trackpad 2"),
            // SanDisk
            (0x0781, 0x5567, "Cruzer Blade"),
            (0x0781, 0x5571, "Cruzer Fit"),
            (0x0781, 0x5575, "Cruzer Glide"),
            (0x0781, 0x5580, "Ultra"),
            (0x0781, 0x5583, "Ultra Fit"),
            (0x0781, 0x5591, "Ultra Flair"),
            (0x0781, 0x55A1, "Cruzer Spark"),
            // Kingston
            (0x0951, 0x1666, "DataTraveler"),
            (0x0951, 0x16A5, "DataTraveler 3.0"),
            // Samsung
            (0x04E8, 0x6860, "Galaxy smartphone (MTP)"),
            (0x04E8, 0x6863, "Galaxy smartphone (PTP)"),
            (0x04E8, 0x61B6, "Portable SSD T5"),
            (0x04E8, 0xA003, "Flash Drive FIT"),
            (0x04E8, 0xA100, "Flash Drive BAR Plus"),
            // Razer
            (0x1532, 0x0053, "DeathAdder Elite"),
            (0x1532, 0x0078, "Viper Ultimate"),
            (0x1532, 0x007A, "Viper Ultimate (Wireless)"),
            (0x1532, 0x0084, "DeathAdder V2"),
            (0x1532, 0x008F, "DeathAdder V2 Pro"),
            (0x1532, 0x0098, "DeathAdder V3"),
            (0x1532, 0x022B, "Huntsman V2 Analog"),
            (0x1532, 0x0237, "Huntsman Mini"),
            (0x1532, 0x0256, "DeathStalker V2 Pro"),
            (0x1532, 0x0F01, "Firefly (Hyperflux)"),
            // Corsair
            (0x1B1C, 0x1B3D, "K55 RGB Keyboard"),
            (0x1B1C, 0x1B3E, "Harpoon RGB Mouse"),
            (0x1B1C, 0x1B4F, "K95 RGB Platinum"),
            (0x1B1C, 0x1B55, "K100 RGB Keyboard"),
            (0x1B1C, 0x1B75, "HS70 Bluetooth"),
            // SteelSeries
            (0x1038, 0x1702, "SteelSeries Apex Pro"),
            (0x1038, 0x1814, "Arctis 7"),
            (0x1038, 0x1830, "Aerox 3"),
            // Intel
            (0x8086, 0x0189, "Centrino Advanced-N 6230"),
            (0x8087, 0x0A2A, "Bluetooth wireless interface"),
            (0x8087, 0x0A2B, "Bluetooth 4.2"),
            (0x8087, 0x0AAA, "Bluetooth 5.0"),
            (0x8087, 0x0029, "AX201 Bluetooth"),
            (0x8087, 0x0032, "AX210 Bluetooth"),
            (0x8087, 0x0033, "AX211 Bluetooth"),
            // Realtek
            (0x0BDA, 0x8152, "RTL8152 Fast Ethernet"),
            (0x0BDA, 0x8153, "RTL8153 Gigabit Ethernet"),
            (0x0BDA, 0x8156, "RTL8156 2.5G Ethernet"),
            (0x0BDA, 0x5401, "RTL8153B Ethernet"),
            (0x0BDA, 0xB812, "RTL88x2bu [AC1200 Techkey]"),
            (0x0BDA, 0xC820, "RTL8821C Bluetooth"),
            (0x0BDA, 0xB009, "RTL8821AU Bluetooth"),
            // Google
            (0x18D1, 0x4EE1, "Nexus/Pixel MTP"),
            (0x18D1, 0x4EE2, "Nexus/Pixel PTP"),
            (0x18D1, 0x4EE7, "Nexus/Pixel (Charging)"),
            // Yubico
            (0x1050, 0x0407, "YubiKey 4/5 OTP+U2F+CCID"),
            (0x1050, 0x0402, "YubiKey 4/5 CCID"),
            (0x1050, 0x0403, "YubiKey 4/5 OTP"),
            (0x1050, 0x0406, "YubiKey 4/5 U2F+CCID"),
            (0x1050, 0x0410, "YubiKey Plus"),
            // Valve
            (0x28DE, 0x1102, "Steam Controller"),
            (0x28DE, 0x1142, "Steam Controller"),
            (0x28DE, 0x1205, "Steam Deck"),
            (0x28DE, 0x1106, "Steam Virtual Gamepad"),
            // Sony (PlayStation)
            (0x054C, 0x05C4, "DualShock 4 [CUH-ZCT1x]"),
            (0x054C, 0x09CC, "DualShock 4 [CUH-ZCT2x]"),
            (0x054C, 0x0CE6, "DualSense wireless controller"),
            // Nintendo
            (0x057E, 0x2006, "Joy-Con (L)"),
            (0x057E, 0x2007, "Joy-Con (R)"),
            (0x057E, 0x2009, "Switch Pro Controller"),
            (0x057E, 0x200E, "Joy-Con Charging Grip"),
            // Arduino/Maker
            (0x2341, 0x0043, "Arduino Uno"),
            (0x2341, 0x0042, "Arduino Mega 2560"),
            (0x2341, 0x8036, "Arduino Leonardo"),
            (0x239A, 0x801E, "Feather M0 Express"),
            // Blue Microphones
            (0xB58E, 0x9E84, "Yeti Stereo Microphone"),
            (0xB58E, 0x9E0E, "Snowball iCE"),
        ];

        for (vid, pid, name) in products {
            self.add_product(*vid, *pid, name);
        }
    }

    /// Get database statistics
    pub fn stats(&self) -> (usize, usize) {
        (self.vendors.len(), self.products.len())
    }
}

impl Default for UsbIdDatabase {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookup_known_vendor() {
        let db = UsbIdDatabase::global();
        assert_eq!(db.get_vendor(0x046D), Some("Logitech"));
        assert_eq!(db.get_vendor(0x045E), Some("Microsoft"));
        assert_eq!(db.get_vendor(0x05AC), Some("Apple"));
    }

    #[test]
    fn test_lookup_known_product() {
        let db = UsbIdDatabase::global();
        assert_eq!(db.get_product(0x046D, 0xC52B), Some("Unifying Receiver"));
        assert_eq!(db.get_product(0x0781, 0x5567), Some("Cruzer Blade"));
    }

    #[test]
    fn test_lookup_unknown() {
        let db = UsbIdDatabase::global();
        assert_eq!(db.get_vendor(0xFFFF), None);
        assert_eq!(db.get_product(0xFFFF, 0xFFFF), None);
    }

    #[test]
    fn test_stats() {
        let db = UsbIdDatabase::global();
        let (vendors, products) = db.stats();
        assert!(vendors > 100, "Should have many vendors");
        assert!(products > 50, "Should have many products");
    }

    #[test]
    fn test_bundled_database_loaded() {
        let db = UsbIdDatabase::global();
        let (vendors, products) = db.stats();
        // Verify bundled database was loaded (should have thousands of entries)
        // With embedded data alone, we'd have ~150 vendors; with usb.ids, we have ~3500+
        assert!(vendors > 2000, "Expected 2000+ vendors from bundled usb.ids, got {}", vendors);
        assert!(products > 10000, "Expected 10000+ products from bundled usb.ids, got {}", products);
    }
}
