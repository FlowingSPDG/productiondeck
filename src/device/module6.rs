//! Stream Deck Module 6 device configuration
//! 
//! Module 6 (Model 20GAI9901) — VID:PID 0x0FD9:0x00B8

use super::{DeviceConfig, ButtonLayout, DisplayConfig, UsbConfig, ImageFormat, ProtocolVersion};

/// Stream Deck Module 6 configuration (PID: 0x00B8)
pub struct Module6Config;

impl DeviceConfig for Module6Config {
    fn device_name(&self) -> &'static str {
        "Stream Deck Module 6"
    }
    
    fn button_layout(&self) -> ButtonLayout {
        ButtonLayout::new(3, 2, true) // 3x2 layout, left-to-right
    }
    
    fn display_config(&self) -> DisplayConfig {
        DisplayConfig {
            image_width: 80,
            image_height: 80,
            format: ImageFormat::Bmp,
            needs_rotation: true,    // Rotate content 90° clockwise per spec
            flip_horizontal: false,
            flip_vertical: false,
        }
    }
    
    fn usb_config(&self) -> UsbConfig {
        UsbConfig {
            vid: 0x0fd9,
            pid: 0x00b8,
            product_name: "Stream Deck Module 6",
            manufacturer: "Elgato Systems",
            protocol: ProtocolVersion::Module6Keys,
        }
    }
}