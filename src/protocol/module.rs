//! StreamDeck Module Protocol Handler
//! 
//! Handles Stream Deck Module devices (6, 15, 32 keys) using Module protocol
//! - Fixed 65-byte Input Reports
//! - Fixed 1024-byte Output Reports  
//! - Fixed 32-byte Feature Reports
//! - 90° clockwise image rotation
//! - Chunked image upload

use super::{ProtocolHandlerTrait, ImageProcessResult, ButtonMapping, ProtocolCommand};
use crate::device::ProtocolVersion;
use crate::config::IMAGE_PROCESSING_BUFFER_SIZE;
use heapless::Vec;

/// Module Protocol Handler for Stream Deck Module devices
#[derive(Debug)]
pub struct ModuleHandler {
    image_buffer: Vec<u8, IMAGE_PROCESSING_BUFFER_SIZE>,
    receiving_image: bool,
    expected_key: u8,
    expected_chunk: u8,
    total_chunks: u8,
}

impl ModuleHandler {
    pub fn new() -> Self {
        Self {
            image_buffer: Vec::new(),
            receiving_image: false,
            expected_key: 0,
            expected_chunk: 0,
            total_chunks: 0,
        }
    }
    
    /// Reset image reception state
    fn reset_image_state(&mut self) {
        self.image_buffer.clear();
        self.receiving_image = false;
        self.expected_key = 0;
        self.expected_chunk = 0;
        self.total_chunks = 0;
    }
    
    /// Calculate total chunks needed for image size
    fn calculate_total_chunks(&self, image_size: usize) -> u8 {
        // Module protocol: 1024-byte output reports, 1014 bytes per chunk (1024 - 10 byte header)
        let chunk_data_size = 1014;
        ((image_size + chunk_data_size - 1) / chunk_data_size) as u8
    }
}

impl ProtocolHandlerTrait for ModuleHandler {
    fn version(&self) -> ProtocolVersion {
        ProtocolVersion::Module
    }
    
    fn process_image_packet(&mut self, data: &[u8]) -> ImageProcessResult {
        if data.len() < 16 {
            // Module protocol requires at least 16 bytes for header
            return ImageProcessResult::Incomplete;
        }
        
        // Module Output Report format: [0x02, 0x01, chunk_index, 0x00, show_flag, key_index, reserved[10], data...]
        let (chunk_index, _show_flag, key_index, data_start) = if data[0] == 0x02 && data[1] == 0x01 {
            (data[2], data[4], data[5], 16)
        } else {
            return ImageProcessResult::Incomplete;
        };
        
        // First chunk (index 0) starts image reception
        if chunk_index == 0 {
            self.reset_image_state();
            self.receiving_image = true;
            self.expected_key = key_index;
            self.expected_chunk = 0;
            
            // Estimate total chunks based on first chunk size
            let first_chunk_data_size = data.len() - data_start;
            self.total_chunks = self.calculate_total_chunks(first_chunk_data_size * 2); // Rough estimate
        }
        
        // Validate chunk sequence and key
        if !self.receiving_image || key_index != self.expected_key || chunk_index != self.expected_chunk {
            self.reset_image_state();
            return ImageProcessResult::Incomplete;
        }
        
        // Copy chunk data
        let copy_len = (data.len() - data_start).min(1014); // Max 1014 bytes per chunk
        
        if copy_len > 0 {
            if self.image_buffer.extend_from_slice(&data[data_start..data_start + copy_len]).is_err() {
                self.reset_image_state();
                return ImageProcessResult::Incomplete;
            }
        }
        
        self.expected_chunk += 1;
        
        // Check if this is the last chunk (rough estimation)
        if chunk_index >= self.total_chunks - 1 || copy_len < 1014 {
            // Image complete
            let mut complete_image = Vec::new();
            let _ = complete_image.extend_from_slice(&self.image_buffer);
            self.reset_image_state();
            
            ImageProcessResult::Complete(complete_image)
        } else {
            ImageProcessResult::Incomplete
        }
    }
    
    fn map_buttons(&self, physical_buttons: &[bool], cols: usize, rows: usize, left_to_right: bool) -> ButtonMapping {
        let mut mapped_buttons = [false; 32];
        let total_keys = cols * rows;
        
        // Module devices use left-to-right mapping
        for (physical_idx, &pressed) in physical_buttons.iter().take(total_keys).enumerate() {
            let mapped_idx = if left_to_right {
                physical_idx
            } else {
                // Right-to-left if needed (rare for Module devices)
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
        // Module StreamDeck HID descriptor
        &[
            0x05, 0x0c, // Usage Page (Consumer)
            0x09, 0x01, // Usage (Consumer Control)
            0xa1, 0x01, // Collection (Application)
            0x09, 0x01, // Usage (Consumer Control)
            0x05, 0x09, // Usage Page (Button)
            0x19, 0x01, // Usage Minimum (0x01)
            0x29, 0x20, // Usage Maximum (0x20) - Support up to 32 buttons
            0x15, 0x00, // Logical Minimum (0)
            0x26, 0xff, 0x00, // Logical Maximum (255)
            0x75, 0x08, // Report Size (8)
            0x95, 0x40, // Report Count (64) - Module uses 64-byte input reports
            0x85, 0x01, // Report ID (0x01)
            0x81, 0x02, // Input (Data,Var,Abs)
            0x0a, 0x00, 0xff, // Usage (Button 255)
            0x15, 0x00, // Logical Minimum (0)
            0x26, 0xff, 0x00, // Logical Maximum (255)
            0x75, 0x08, // Report Size (8)
            0x96, 0x00, 0x04, // Report Count (1024) - Module uses 1024-byte output reports
            0x85, 0x02, // Report ID (0x02)
            0x91, 0x02, // Output (Data,Var,Abs)
            0x0a, 0x00, 0xff, // Usage (Button 255)
            0x15, 0x00, // Logical Minimum (0)
            0x26, 0xff, 0x00, // Logical Maximum (255)
            0x75, 0x08, // Report Size (8)
            0x95, 0x20, // Report Count (32) - Module uses 32-byte feature reports
            0x85, 0x03, // Report ID (0x03)
            0xb1, 0x04, // Feature (Data,Array,Rel)
            0x0a, 0x00, 0xff, // Usage (Button 255)
            0x15, 0x00, // Logical Minimum (0)
            0x26, 0xff, 0x00, // Logical Maximum (255)
            0x75, 0x08, // Report Size (8)
            0x95, 0x20, // Report Count (32)
            0x85, 0x05, // Report ID (0x05)
            0xb1, 0x04, // Feature (Data,Array,Rel)
            0x0a, 0x00, 0xff, // Usage (Button 255)
            0x15, 0x00, // Logical Minimum (0)
            0x26, 0xff, 0x00, // Logical Maximum (255)
            0x75, 0x08, // Report Size (8)
            0x95, 0x20, // Report Count (32)
            0x85, 0x0b, // Report ID (0x0b)
            0xb1, 0x04, // Feature (Data,Array,Rel)
            0x0a, 0x00, 0xff, // Usage (Button 255)
            0x15, 0x00, // Logical Minimum (0)
            0x26, 0xff, 0x00, // Logical Maximum (255)
            0x75, 0x08, // Report Size (8)
            0x95, 0x20, // Report Count (32)
            0x85, 0xa0, // Report ID (0xa0)
            0xb1, 0x04, // Feature (Data,Array,Rel)
            0x0a, 0x00, 0xff, // Usage (Button 255)
            0x15, 0x00, // Logical Minimum (0)
            0x26, 0xff, 0x00, // Logical Maximum (255)
            0x75, 0x08, // Report Size (8)
            0x95, 0x20, // Report Count (32)
            0x85, 0xa1, // Report ID (0xa1)
            0xb1, 0x04, // Feature (Data,Array,Rel)
            0x0a, 0x00, 0xff, // Usage (Button 255)
            0x15, 0x00, // Logical Minimum (0)
            0x26, 0xff, 0x00, // Logical Maximum (255)
            0x75, 0x08, // Report Size (8)
            0x95, 0x20, // Report Count (32)
            0x85, 0xa2, // Report ID (0xa2)
            0xb1, 0x04, // Feature (Data,Array,Rel)
            0x0a, 0x00, 0xff, // Usage (Button 255)
            0x15, 0x00, // Logical Minimum (0)
            0x26, 0xff, 0x00, // Logical Maximum (255)
            0x75, 0x08, // Report Size (8)
            0x95, 0x20, // Report Count (32)
            0x85, 0xa3, // Report ID (0xa3)
            0xb1, 0x04, // Feature (Data,Array,Rel)
            0xc0 // End Collection
        ]
    }
    
    fn input_report_size(&self, _button_count: usize) -> usize {
        // Module input reports: Fixed 65 bytes (1 byte Report ID + 64 bytes payload)
        65
    }
    
    fn format_button_report(&self, buttons: &ButtonMapping, report: &mut [u8]) -> usize {
        if report.len() < 65 {
            return 0;
        }
        
        // Module format: [0x01, key_states[64]]
        report[0] = 0x01; // Report ID
        
        // Fill first 6 bytes with button states (Module 6Keys uses first 6 bytes)
        let button_bytes = buttons.active_count.min(6);
        for i in 0..button_bytes {
            report[i + 1] = if buttons.mapped_buttons[i] { 1 } else { 0 };
        }
        
        // Fill remaining bytes with 0
        for i in (button_bytes + 1)..65 {
            report[i] = 0;
        }
        
        65
    }
    
    fn handle_feature_report(&mut self, report_id: u8, data: &[u8]) -> Option<ProtocolCommand> {
        match report_id {
            0x05 => {
                // Module Brightness: [0x05, 0x55, 0xAA, 0xD1, 0x01, brightness, ...]
                if data.len() >= 6 
                    && data[1] == 0x55 
                    && data[2] == 0xAA 
                    && data[3] == 0xD1 
                    && data[4] == 0x01 {
                    Some(ProtocolCommand::SetBrightness(data[5]))
                } else {
                    None
                }
            }
            0x0b => {
                if data.len() >= 2 {
                    match data[1] {
                        0x63 => {
                            // Show Logo: [0x0b, 0x63, 0x00, ...]
                            Some(ProtocolCommand::Reset)
                        }
                        0xa2 => {
                            // Set Idle Time: [0x0b, 0xa2, seconds_le...]
                            if data.len() >= 6 {
                                let secs = i32::from_le_bytes([data[2], data[3], data[4], data[5]]);
                                Some(ProtocolCommand::SetIdleTime(secs))
                            } else {
                                None
                            }
                        }
                        _ => None,
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}
