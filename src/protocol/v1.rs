//! StreamDeck V1 Protocol Handler
//! 
//! Handles Original, Mini, and Revised Mini devices using BMP format

use super::{ProtocolHandlerTrait, ImageProcessResult, ButtonMapping, ProtocolCommand};
use crate::device::ProtocolVersion;
use crate::config::{
    IMAGE_BUFFER_SIZE,
    STREAMDECK_MAGIC_1, STREAMDECK_MAGIC_2, STREAMDECK_MAGIC_3,
    STREAMDECK_RESET_MAGIC, STREAMDECK_BRIGHTNESS_RESET_MAGIC,
    FEATURE_REPORT_RESET_V1, FEATURE_REPORT_BRIGHTNESS_V1
};
use heapless::Vec;

/// V1 Protocol Handler for BMP-based StreamDeck devices
#[derive(Debug)]
pub struct V1Handler {
    image_buffer: Vec<u8, IMAGE_BUFFER_SIZE>,
    receiving_image: bool,
    expected_key: u8,
}

impl V1Handler {
    pub fn new() -> Self {
        Self {
            image_buffer: Vec::new(),
            receiving_image: false,
            expected_key: 0,
        }
    }
    
    /// Reset image reception state
    fn reset_image_state(&mut self) {
        self.image_buffer.clear();
        self.receiving_image = false;
        self.expected_key = 0;
    }
}

impl ProtocolHandlerTrait for V1Handler {
    fn version(&self) -> ProtocolVersion {
        ProtocolVersion::V1
    }
    
    fn process_image_packet(&mut self, data: &[u8]) -> ImageProcessResult {
        if data.len() < 8 {
            return ImageProcessResult::Error("Packet too small for V1 protocol");
        }
        
        // V1 Protocol format: [0x02, 0x01, packet_num, 0x00, 0x00, key_id, 0x00, 0x00, image_data...]
        if data[0] != 0x02 || data[1] != 0x01 {
            return ImageProcessResult::Error("Invalid V1 packet header");
        }
        
        let packet_num = data[2];
        let key_id = data[5];
        
        // First packet starts image reception
        if packet_num == 0x01 {
            self.reset_image_state();
            self.receiving_image = true;
            self.expected_key = key_id;
            
            // Skip header and copy image data
            let data_start = 8;
            if data.len() > data_start {
                if self.image_buffer.extend_from_slice(&data[data_start..]).is_err() {
                    self.reset_image_state();
                    return ImageProcessResult::Error("Image buffer overflow");
                }
            }
            
            ImageProcessResult::Incomplete
        } else if packet_num == 0x02 && self.receiving_image && key_id == self.expected_key {
            // Second packet completes the image
            let data_start = 8;
            if data.len() > data_start {
                if self.image_buffer.extend_from_slice(&data[data_start..]).is_err() {
                    self.reset_image_state();
                    return ImageProcessResult::Error("Image buffer overflow");
                }
            }
            
            // V1 image is complete
            let mut complete_image = Vec::new();
            let _ = complete_image.extend_from_slice(&self.image_buffer);
            self.reset_image_state();
            
            ImageProcessResult::Complete(complete_image)
        } else {
            ImageProcessResult::Error("Invalid V1 packet sequence")
        }
    }
    
    fn map_buttons(&self, physical_buttons: &[bool], cols: usize, rows: usize, left_to_right: bool) -> ButtonMapping {
        let mut mapped_buttons = [false; 32];
        let total_keys = cols * rows;
        
        for (physical_idx, &pressed) in physical_buttons.iter().take(total_keys).enumerate() {
            let mapped_idx = if left_to_right {
                physical_idx // Direct mapping for Mini and Revised Mini
            } else {
                // Right-to-left mapping for Original StreamDeck
                let row = physical_idx / cols;
                let col = physical_idx % cols;
                let reversed_col = cols - 1 - col;
                row * cols + reversed_col
            };
            
            if mapped_idx < 32 {
                mapped_buttons[mapped_idx] = pressed;
            }
        }
        
        ButtonMapping {
            mapped_buttons,
            active_count: total_keys,
        }
    }
    
    fn hid_descriptor(&self) -> &'static [u8] {
        // V1 StreamDeck HID descriptor (from existing implementation)
        &[
            0x05, 0x0c, // Usage Page (Consumer)
            0x09, 0x01, // Usage (Consumer Control)
            0xa1, 0x01, // Collection (Application)
            0x09, 0x01, // Usage (Consumer Control)
            0x05, 0x09, // Usage Page (Button)
            0x19, 0x01, // Usage Minimum (0x01)
            0x29, 0x10, // Usage Maximum (0x10)
            0x15, 0x00, // Logical Minimum (0)
            0x26, 0xff, 0x00, // Logical Maximum (255)
            0x75, 0x08, // Report Size (8)
            0x95, 0x10, // Report Count (16)
            0x85, 0x01, // Report ID (0x01)
            0x81, 0x02, // Input (Data,Var,Abs)
            0x0a, 0x00, 0xff, // Usage (Button 255)
            0x15, 0x00, // Logical Minimum (0)
            0x26, 0xff, 0x00, // Logical Maximum (255)
            0x75, 0x08, // Report Size (8)
            0x96, 0xff, 0x03, // Report Count (1023)
            0x85, 0x02, // Report ID (0x02)
            0x91, 0x02, // Output (Data,Var,Abs)
            0x0a, 0x00, 0xff, // Usage (Button 255)
            0x15, 0x00, // Logical Minimum (0)
            0x26, 0xff, 0x00, // Logical Maximum (255)
            0x75, 0x08, // Report Size (8)
            0x95, 0x10, // Report Count (16)
            0x85, 0x03, // Report ID (0x03)
            0xb1, 0x04, // Feature (Data,Array,Rel)
            0x0a, 0x00, 0xff, // Usage (Button 255)
            0x15, 0x00, // Logical Minimum (0)
            0x26, 0xff, 0x00, // Logical Maximum (255)
            0x75, 0x08, // Report Size (8)
            0x95, 0x10, // Report Count (16)
            0x85, 0x04, // Report ID (0x04)
            0xb1, 0x04, // Feature (Data,Array,Rel)
            0x0a, 0x00, 0xff, // Usage (Button 255)
            0x15, 0x00, // Logical Minimum (0)
            0x26, 0xff, 0x00, // Logical Maximum (255)
            0x75, 0x08, // Report Size (8)
            0x95, 0x10, // Report Count (16)
            0x85, 0x05, // Report ID (0x05)
            0xb1, 0x04, // Feature (Data,Array,Rel)
            0x0a, 0x00, 0xff, // Usage (Button 255)
            0x15, 0x00, // Logical Minimum (0)
            0x26, 0xff, 0x00, // Logical Maximum (255)
            0x75, 0x08, // Report Size (8)
            0x95, 0x10, // Report Count (16)
            0x85, 0x07, // Report ID (0x07)
            0xb1, 0x04, // Feature (Data,Array,Rel)
            0x0a, 0x00, 0xff, // Usage (Button 255)
            0x15, 0x00, // Logical Minimum (0)
            0x26, 0xff, 0x00, // Logical Maximum (255)
            0x75, 0x08, // Report Size (8)
            0x95, 0x10, // Report Count (16)
            0x85, 0x0b, // Report ID (0x0b)
            0xb1, 0x04, // Feature (Data,Array,Rel)
            0x0a, 0x00, 0xff, // Usage (Button 255)
            0x15, 0x00, // Logical Minimum (0)
            0x26, 0xff, 0x00, // Logical Maximum (255)
            0x75, 0x08, // Report Size (8)
            0x95, 0x10, // Report Count (16)
            0x85, 0xa0, // Report ID (0xa0)
            0xb1, 0x04, // Feature (Data,Array,Rel)
            0x0a, 0x00, 0xff, // Usage (Button 255)
            0x15, 0x00, // Logical Minimum (0)
            0x26, 0xff, 0x00, // Logical Maximum (255)
            0x75, 0x08, // Report Size (8)
            0x95, 0x10, // Report Count (16)
            0x85, 0xa1, // Report ID (0xa1)
            0xb1, 0x04, // Feature (Data,Array,Rel)
            0x0a, 0x00, 0xff, // Usage (Button 255)
            0x15, 0x00, // Logical Minimum (0)
            0x26, 0xff, 0x00, // Logical Maximum (255)
            0x75, 0x08, // Report Size (8)
            0x95, 0x10, // Report Count (16)
            0x85, 0xa2, // Report ID (0xa2)
            0xb1, 0x04, // Feature (Data,Array,Rel)
            0xc0 // End Collection
        ]
    }
    
    fn input_report_size(&self, button_count: usize) -> usize {
        // V1 input reports: Report ID (1 byte) + button states
        (button_count + 15) / 16 * 16 // Round up to 16-byte boundary
    }
    
    fn format_button_report(&self, buttons: &ButtonMapping, report: &mut [u8]) -> usize {
        if report.is_empty() {
            return 0;
        }
        
        // V1 format: [0x01, button_states...]
        report[0] = 0x01; // Report ID
        
        let button_bytes = (buttons.active_count).min(report.len() - 1);
        for i in 0..button_bytes {
            report[i + 1] = if buttons.mapped_buttons[i] { 1 } else { 0 };
        }
        
        // Fill remaining bytes with 0
        for i in (button_bytes + 1)..report.len() {
            report[i] = 0;
        }
        
        report.len()
    }
    
    fn handle_feature_report(&mut self, report_id: u8, data: &[u8]) -> Option<ProtocolCommand> {
        match report_id {
            FEATURE_REPORT_RESET_V1 => {
                // V1 Reset: [0x0B, 0x63, ...]
                if data.len() >= 2 && data[1] == STREAMDECK_RESET_MAGIC {
                    Some(ProtocolCommand::Reset)
                } else {
                    None
                }
            }
            FEATURE_REPORT_BRIGHTNESS_V1 => {
                // V1 Brightness/Reset: [0x05, 0x55, 0xAA, 0xD1, 0x01, value, ...]
                if data.len() >= 6 
                    && data[1] == STREAMDECK_MAGIC_1 
                    && data[2] == STREAMDECK_MAGIC_2 
                    && data[3] == STREAMDECK_MAGIC_3 
                    && data[4] == 0x01 {
                    
                    if data[5] == STREAMDECK_BRIGHTNESS_RESET_MAGIC {
                        Some(ProtocolCommand::Reset)
                    } else {
                        Some(ProtocolCommand::SetBrightness(data[5]))
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}