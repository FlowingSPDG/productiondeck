//! Device abstraction layer for StreamDeck compatible devices
//! 
//! This module provides a unified interface for different StreamDeck models,
//! abstracting away device-specific configurations, protocols, and capabilities.

pub mod mini;
pub mod original;
pub mod original_v2;
pub mod xl;
pub mod plus;


/// Image format supported by StreamDeck devices
#[derive(Debug, Clone, Copy, PartialEq, defmt::Format)]
pub enum ImageFormat {
    /// BMP format (used by V1 protocol devices)
    Bmp,
    /// JPEG format (used by V2 protocol devices)
    Jpeg,
}

/// Protocol version used by StreamDeck devices
#[derive(Debug, Clone, Copy, PartialEq, defmt::Format)]
pub enum ProtocolVersion {
    /// V1 protocol (Original, Mini, Revised Mini)
    V1,
    /// V2 protocol (Original V2, XL, MK2, Plus)
    V2,
    /// Module protocol (Stream Deck Module 6, 15, 32 Keys)
    Module,
}

/// Button layout configuration
#[derive(Debug, Clone, Copy)]
pub struct ButtonLayout {
    /// Number of button columns
    pub cols: usize,
    /// Number of button rows  
    pub rows: usize,
    /// Total number of buttons (cols * rows)
    pub total_keys: usize,
    /// Button mapping order (true = left-to-right, false = right-to-left)
    pub left_to_right: bool,
}

impl ButtonLayout {
    pub const fn new(cols: usize, rows: usize, left_to_right: bool) -> Self {
        Self {
            cols,
            rows,
            total_keys: cols * rows,
            left_to_right,
        }
    }
}

/// Display configuration for StreamDeck devices
#[derive(Debug, Clone, Copy)]
pub struct DisplayConfig {
    /// Image width in pixels per key
    pub image_width: usize,
    /// Image height in pixels per key
    pub image_height: usize,
    /// Image format (BMP or JPEG)
    pub format: ImageFormat,
    /// Whether image needs rotation (Mini devices need 270° rotation)
    pub needs_rotation: bool,
    /// Whether image needs horizontal flip
    pub flip_horizontal: bool,
    /// Whether image needs vertical flip
    pub flip_vertical: bool,
}

/// USB configuration for StreamDeck devices
#[derive(Debug, Clone, Copy)]
pub struct UsbConfig {
    /// USB Vendor ID (always 0x0fd9 for Elgato)
    pub vid: u16,
    /// USB Product ID (device-specific)
    pub pid: u16,
    /// USB product name
    pub product_name: &'static str,
    /// USB manufacturer name
    pub manufacturer: &'static str,
    /// Protocol version
    pub protocol: ProtocolVersion,
}

/// Complete device configuration trait
pub trait DeviceConfig {
    /// Get device name for identification
    fn device_name(&self) -> &'static str;
    
    /// Get button layout configuration
    fn button_layout(&self) -> ButtonLayout;
    
    /// Get display configuration
    fn display_config(&self) -> DisplayConfig;
    
    /// Get USB configuration
    fn usb_config(&self) -> UsbConfig;
    
    /// Get maximum image data size in bytes
    fn max_image_size(&self) -> usize {
        let display = self.display_config();
        match display.format {
            ImageFormat::Bmp => {
                // BMP: header (54 bytes) + RGB data (width * height * 3)
                54 + (display.image_width * display.image_height * 3)
            }
            ImageFormat::Jpeg => {
                // JPEG: Variable size, use conservative estimate
                display.image_width * display.image_height / 2
            }
        }
    }
    
    /// Get HID report descriptor size
    fn hid_descriptor_size(&self) -> usize {
        173 // Standard StreamDeck HID descriptor size
    }
    
    /// Get input report size (button states)
    fn input_report_size(&self) -> usize {
        match self.usb_config().protocol {
            ProtocolVersion::V1 => self.button_layout().total_keys + 1, // +1 for report ID
            ProtocolVersion::V2 => self.button_layout().total_keys + 4, // +4 for V2 header
            ProtocolVersion::Module => 65, // Fixed 65-byte input reports for Module
        }
    }
    
    /// Get feature report size
    fn feature_report_size(&self) -> usize {
        32 // Standard feature report size
    }
    
    /// Get output report size (image data)
    fn output_report_size(&self) -> usize {
        1024 // Standard 1KB output report size
    }
}

/// Enum-based device configuration for no_std environment
#[derive(Debug, Clone, Copy)]
pub enum Device {
    Mini,
    RevisedMini,
    Original,
    OriginalV2,
    Xl,
    Plus,
    Module6,
    Module15,
    Module32,
}

impl Device {
    /// Get device by USB PID  
    pub fn from_pid(pid: u16) -> Option<Self> {
        match pid {
            0x0063 => Some(Device::Mini),
            0x0080 => Some(Device::RevisedMini), // StreamDeck Revised Mini
            0x0060 => Some(Device::Original),
            0x006d => Some(Device::OriginalV2),
            0x006c => Some(Device::Xl),
            0x0084 => Some(Device::Plus),
            0x00b8 => Some(Device::Module6), // Stream Deck Module 6 Keys
            0x00b9 => Some(Device::Module15), // Stream Deck Module 15 Keys
            0x00ba => Some(Device::Module32), // Stream Deck Module 32 Keys
            _ => None,
        }
    }
    
    /// Get all supported device PIDs
    pub fn supported_pids() -> &'static [u16] {
        &[0x0060, 0x0063, 0x0080, 0x006d, 0x006c, 0x0084, 0x00b8, 0x00b9, 0x00ba]
    }
    
    /// Get PID for this device
    pub fn pid(&self) -> u16 {
        match self {
            Device::Mini => 0x0063,
            Device::RevisedMini => 0x0080,
            Device::Original => 0x0060,
            Device::OriginalV2 => 0x006d,
            Device::Xl => 0x006c,
            Device::Plus => 0x0084,
            Device::Module6 => 0x00b8,
            Device::Module15 => 0x00b9,
            Device::Module32 => 0x00ba,
        }
    }
}

impl DeviceConfig for Device {
    fn device_name(&self) -> &'static str {
        match self {
            Device::Mini => "StreamDeck Mini",
            Device::RevisedMini => "StreamDeck Mini (Revised)",
            Device::Original => "StreamDeck Original",
            Device::OriginalV2 => "StreamDeck Original V2",
            Device::Xl => "StreamDeck XL",
            Device::Plus => "StreamDeck Plus",
            Device::Module6 => "Stream Deck Module 6 Keys",
            Device::Module15 => "Stream Deck Module 15 Keys",
            Device::Module32 => "Stream Deck Module 32 Keys",
        }
    }
    
    fn button_layout(&self) -> ButtonLayout {
        match self {
            Device::Mini | Device::RevisedMini => ButtonLayout::new(3, 2, true),
            Device::Original => ButtonLayout::new(5, 3, false), // right-to-left
            Device::OriginalV2 => ButtonLayout::new(5, 3, true),
            Device::Xl => ButtonLayout::new(8, 4, true),
            Device::Plus => ButtonLayout::new(4, 2, true),
            Device::Module6 => ButtonLayout::new(3, 2, true), // 3x2 layout, left-to-right
            Device::Module15 => ButtonLayout::new(5, 3, true), // 5x3 layout, left-to-right
            Device::Module32 => ButtonLayout::new(8, 4, true), // 8x4 layout, left-to-right
        }
    }
    
    fn display_config(&self) -> DisplayConfig {
        match self {
            Device::Mini | Device::RevisedMini => DisplayConfig {
                image_width: 80,
                image_height: 80,
                format: ImageFormat::Bmp,
                needs_rotation: true,
                flip_horizontal: false,
                flip_vertical: false,
            },
            Device::Original => DisplayConfig {
                image_width: 72,
                image_height: 72,
                format: ImageFormat::Bmp,
                needs_rotation: false,
                flip_horizontal: true,
                flip_vertical: false,
            },
            Device::OriginalV2 => DisplayConfig {
                image_width: 72,
                image_height: 72,
                format: ImageFormat::Jpeg,
                needs_rotation: false,
                flip_horizontal: true,
                flip_vertical: true,
            },
            Device::Xl => DisplayConfig {
                image_width: 96,
                image_height: 96,
                format: ImageFormat::Jpeg,
                needs_rotation: false,
                flip_horizontal: true,
                flip_vertical: true,
            },
            Device::Plus => DisplayConfig {
                image_width: 120,
                image_height: 120,
                format: ImageFormat::Jpeg,
                needs_rotation: false,
                flip_horizontal: false,
                flip_vertical: false,
            },
            Device::Module6 => DisplayConfig {
                image_width: 80,
                image_height: 80,
                format: ImageFormat::Bmp,
                needs_rotation: true, // 90° clockwise rotation
                flip_horizontal: false,
                flip_vertical: false,
            },
            Device::Module15 => DisplayConfig {
                image_width: 72,
                image_height: 72,
                format: ImageFormat::Bmp,
                needs_rotation: true, // 180° rotation for Module 15
                flip_horizontal: false,
                flip_vertical: false,
            },
            Device::Module32 => DisplayConfig {
                image_width: 96,
                image_height: 96,
                format: ImageFormat::Bmp,
                needs_rotation: true, // 180° rotation for Module 32
                flip_horizontal: false,
                flip_vertical: false,
            },
        }
    }
    
    fn usb_config(&self) -> UsbConfig {
        match self {
            Device::Mini => UsbConfig {
                vid: 0x0fd9,
                pid: 0x0063,
                product_name: "Stream Deck Mini",
                manufacturer: "Elgato Systems",
                protocol: ProtocolVersion::V1,
            },
            Device::RevisedMini => UsbConfig {
                vid: 0x0fd9,
                pid: 0x0080,
                product_name: "Stream Deck Mini",
                manufacturer: "Elgato Systems",
                protocol: ProtocolVersion::V1,
            },
            Device::Original => UsbConfig {
                vid: 0x0fd9,
                pid: 0x0060,
                product_name: "Stream Deck",
                manufacturer: "Elgato Systems",
                protocol: ProtocolVersion::V1,
            },
            Device::OriginalV2 => UsbConfig {
                vid: 0x0fd9,
                pid: 0x006d,
                product_name: "Stream Deck",
                manufacturer: "Elgato Systems",
                protocol: ProtocolVersion::V2,
            },
            Device::Xl => UsbConfig {
                vid: 0x0fd9,
                pid: 0x006c,
                product_name: "Stream Deck XL",
                manufacturer: "Elgato Systems",
                protocol: ProtocolVersion::V2,
            },
            Device::Plus => UsbConfig {
                vid: 0x0fd9,
                pid: 0x0080,
                product_name: "Stream Deck Plus",
                manufacturer: "Elgato Systems",
                protocol: ProtocolVersion::V2,
            },
            Device::Module6 => UsbConfig {
                vid: 0x0fd9,
                pid: 0x00b8,
                product_name: "Stream Deck Module 6 Keys",
                manufacturer: "Elgato Systems",
                protocol: ProtocolVersion::Module,
            },
            Device::Module15 => UsbConfig {
                vid: 0x0fd9,
                pid: 0x00b9,
                product_name: "Stream Deck Module 15 Keys",
                manufacturer: "Elgato Systems",
                protocol: ProtocolVersion::Module,
            },
            Device::Module32 => UsbConfig {
                vid: 0x0fd9,
                pid: 0x00ba,
                product_name: "Stream Deck Module 32 Keys",
                manufacturer: "Elgato Systems",
                protocol: ProtocolVersion::Module,
            },
        }
    }
}