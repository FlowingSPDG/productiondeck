//! StreamDeck protocol abstraction layer
//! 
//! Handles different protocol versions (V1 and V2) with unified interface

pub mod v1;
pub mod v2;
pub mod module;

use heapless::Vec;
use crate::config::IMAGE_BUFFER_SIZE;
use crate::device::ProtocolVersion;
use crate::protocol::module::{ModuleHandler, ModuleRotationType};

/// Protocol-specific image processing result
#[derive(Debug)]
pub enum ImageProcessResult {
    /// Image processing complete, ready to display
    Complete(Vec<u8, IMAGE_BUFFER_SIZE>),
    /// More packets needed to complete image
    Incomplete,
    /// Error processing image
    Error(&'static str),
}

/// Button mapping result for different devices
#[derive(Debug)]
pub struct ButtonMapping {
    pub mapped_buttons: [bool; 32], // Max buttons supported (XL has 32)
    pub active_count: usize,
}

/// Protocol handler trait for different StreamDeck versions
pub trait ProtocolHandlerTrait {
    /// Get protocol version
    fn version(&self) -> ProtocolVersion;
    
    /// Process incoming image data packet
    fn process_image_packet(&mut self, data: &[u8]) -> ImageProcessResult;
    
    /// Map physical button layout to protocol button order
    fn map_buttons(&self, physical_buttons: &[bool], cols: usize, rows: usize, left_to_right: bool) -> ButtonMapping;
    
    /// Generate HID report descriptor for this protocol
    fn hid_descriptor(&self) -> &'static [u8];
    
    /// Get input report format size
    fn input_report_size(&self, button_count: usize) -> usize;
    
    /// Format button state into input report
    fn format_button_report(&self, buttons: &ButtonMapping, report: &mut [u8]) -> usize;
    
    /// Process feature report commands
    fn handle_feature_report(&mut self, report_id: u8, data: &[u8]) -> Option<ProtocolCommand>;
}

/// Commands that can be generated from protocol processing
#[derive(Debug, Clone)]
pub enum ProtocolCommand {
    Reset,
    SetBrightness(u8),
    ImageData { key_id: u8, data: Vec<u8, IMAGE_BUFFER_SIZE> },
    /// Set idle time before entering Sleep Mode (seconds). 0 disables sleep.
    SetIdleTime(i32),
}

/// Enum-based protocol handler for no_std environment
#[derive(Debug)]
pub enum ProtocolHandler {
    V1(v1::V1Handler),
    V2(v2::V2Handler),
    Module(ModuleHandler),
}

impl ProtocolHandler {
    /// Create appropriate protocol handler based on version
    pub fn create(version: ProtocolVersion) -> Self {
        match version {
            ProtocolVersion::V1 => ProtocolHandler::V1(v1::V1Handler::new()),
            ProtocolVersion::V2 => ProtocolHandler::V2(v2::V2Handler::new()),
            ProtocolVersion::Module => ProtocolHandler::Module(ModuleHandler::new()),
        }
    }
    
    /// Create Module protocol handler with specific rotation type
    pub fn create_module_with_rotation(rotation_type: ModuleRotationType) -> Self {
        ProtocolHandler::Module(ModuleHandler::new_with_rotation(rotation_type))
    }
    
    /// Get protocol version
    pub fn version(&self) -> ProtocolVersion {
        match self {
            ProtocolHandler::V1(_) => ProtocolVersion::V1,
            ProtocolHandler::V2(_) => ProtocolVersion::V2,
            ProtocolHandler::Module(_) => ProtocolVersion::Module,
        }
    }
    
    /// Process incoming image data packet
    pub fn process_image_packet(&mut self, data: &[u8]) -> ImageProcessResult {
        match self {
            ProtocolHandler::V1(handler) => handler.process_image_packet(data),
            ProtocolHandler::V2(handler) => handler.process_image_packet(data),
            ProtocolHandler::Module(handler) => handler.process_image_packet(data),
        }
    }
    
    /// Map physical button layout to protocol button order
    pub fn map_buttons(&self, physical_buttons: &[bool], cols: usize, rows: usize, left_to_right: bool) -> ButtonMapping {
        match self {
            ProtocolHandler::V1(handler) => handler.map_buttons(physical_buttons, cols, rows, left_to_right),
            ProtocolHandler::V2(handler) => handler.map_buttons(physical_buttons, cols, rows, left_to_right),
            ProtocolHandler::Module(handler) => handler.map_buttons(physical_buttons, cols, rows, left_to_right),
        }
    }
    
    /// Generate HID report descriptor for this protocol
    pub fn hid_descriptor(&self) -> &'static [u8] {
        match self {
            ProtocolHandler::V1(handler) => handler.hid_descriptor(),
            ProtocolHandler::V2(handler) => handler.hid_descriptor(),
            ProtocolHandler::Module(handler) => handler.hid_descriptor(),
        }
    }
    
    /// Get input report format size
    pub fn input_report_size(&self, button_count: usize) -> usize {
        match self {
            ProtocolHandler::V1(handler) => handler.input_report_size(button_count),
            ProtocolHandler::V2(handler) => handler.input_report_size(button_count),
            ProtocolHandler::Module(handler) => handler.input_report_size(button_count),
        }
    }
    
    /// Format button state into input report
    pub fn format_button_report(&self, buttons: &ButtonMapping, report: &mut [u8]) -> usize {
        match self {
            ProtocolHandler::V1(handler) => handler.format_button_report(buttons, report),
            ProtocolHandler::V2(handler) => handler.format_button_report(buttons, report),
            ProtocolHandler::Module(handler) => handler.format_button_report(buttons, report),
        }
    }
    
    /// Process feature report commands
    pub fn handle_feature_report(&mut self, report_id: u8, data: &[u8]) -> Option<ProtocolCommand> {
        match self {
            ProtocolHandler::V1(handler) => handler.handle_feature_report(report_id, data),
            ProtocolHandler::V2(handler) => handler.handle_feature_report(report_id, data),
            ProtocolHandler::Module(handler) => handler.handle_feature_report(report_id, data),
        }
    }
}

/// Image format utilities
pub mod image {
    use super::*;
    
    /// Convert RGB888 to RGB565 for display
    pub fn rgb888_to_rgb565(rgb888: &[u8]) -> Vec<u8, 2048> {
        let mut rgb565_data = Vec::new();
        
        for chunk in rgb888.chunks_exact(3) {
            if let [r, g, b] = chunk {
                let r5 = (r >> 3) as u16;
                let g6 = (g >> 2) as u16;
                let b5 = (b >> 3) as u16;
                
                let rgb565 = (r5 << 11) | (g6 << 5) | b5;
                
                // Store as big-endian for display
                let _ = rgb565_data.push((rgb565 >> 8) as u8);
                let _ = rgb565_data.push((rgb565 & 0xFF) as u8);
            }
        }
        
        rgb565_data
    }
    
    /// Rotate image 270 degrees clockwise (for Mini devices)
    pub fn rotate_270(image_data: &[u8], width: usize, height: usize) -> Vec<u8, IMAGE_BUFFER_SIZE> {
        let mut rotated = Vec::new();
        
        // 270° rotation: new[y][x] = old[width - 1 - x][y]
        for new_y in 0..width {
            for new_x in 0..height {
                let old_x = width - 1 - new_y;
                let old_y = new_x;
                
                let old_idx = (old_y * width + old_x) * 3;
                if old_idx + 2 < image_data.len() {
                    let _ = rotated.push(image_data[old_idx]);     // R
                    let _ = rotated.push(image_data[old_idx + 1]); // G
                    let _ = rotated.push(image_data[old_idx + 2]); // B
                }
            }
        }
        
        rotated
    }
    
    /// Flip image horizontally
    pub fn flip_horizontal(image_data: &[u8], width: usize, height: usize) -> Vec<u8, IMAGE_BUFFER_SIZE> {
        let mut flipped = Vec::new();
        
        for y in 0..height {
            for x in 0..width {
                let src_x = width - 1 - x;
                let src_idx = (y * width + src_x) * 3;
                
                if src_idx + 2 < image_data.len() {
                    let _ = flipped.push(image_data[src_idx]);     // R
                    let _ = flipped.push(image_data[src_idx + 1]); // G
                    let _ = flipped.push(image_data[src_idx + 2]); // B
                }
            }
        }
        
        flipped
    }
    
    /// Flip image vertically  
    pub fn flip_vertical(image_data: &[u8], width: usize, height: usize) -> Vec<u8, IMAGE_BUFFER_SIZE> {
        let mut flipped = Vec::new();
        
        for y in 0..height {
            let src_y = height - 1 - y;
            for x in 0..width {
                let src_idx = (src_y * width + x) * 3;
                
                if src_idx + 2 < image_data.len() {
                    let _ = flipped.push(image_data[src_idx]);     // R
                    let _ = flipped.push(image_data[src_idx + 1]); // G
                    let _ = flipped.push(image_data[src_idx + 2]); // B
                }
            }
        }
        
        flipped
    }
    
    /// Apply device-specific image transformations
    pub fn apply_transformations(
        image_data: &[u8], 
        width: usize, 
        height: usize,
        needs_rotation: bool,
        should_flip_horizontal: bool,
        should_flip_vertical: bool,
    ) -> Vec<u8, IMAGE_BUFFER_SIZE> {
        let mut result_data = Vec::new();
        let _ = result_data.extend_from_slice(image_data);
        
        if needs_rotation {
            result_data = rotate_270(&result_data, width, height);
        }
        
        if should_flip_horizontal {
            result_data = flip_horizontal(&result_data, width, height);
        }
        
        if should_flip_vertical {
            result_data = flip_vertical(&result_data, width, height);
        }
        
        result_data
    }
}