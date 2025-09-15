//! StreamDeck Module HID Protocol Handler (6keys)
//!
//! Implements the unified `ProtocolHandlerTrait` for the Elgato Stream Deck
//! Modules per public HID API docs. Image upload parsing is stubbed until we
//! confirm exact chunk layout from PCAPs.

use heapless::Vec;
use crate::{config::IMAGE_BUFFER_SIZE, protocol::module::{FirmwareType, ModuleSetCommand, ModuleGetCommand}};
use crate::device::ProtocolVersion;
use super::{ProtocolHandlerTrait, ImageProcessResult, ButtonMapping, ProtocolCommand};


#[derive(Debug)]
pub struct Module6KeysHandler {}


impl Module6KeysHandler {
    pub fn new() -> Self { Self {} }
}
impl Module6KeysHandler {
    fn parse_module_set_command(&self, report_id: u8, data: &[u8]) -> Option<ModuleSetCommand> {
        match report_id {
            0x05 => {
                if data.len() >= 6 && data[1] == 0x55 && data[2] == 0xAA && data[3] == 0xD1 && data[4] == 0x01 {
                    Some(ModuleSetCommand::SetBrightness { value: data[5] })
                } else { None }
            }
            0x0B => {
                if data.len() >= 2 {
                    match data[1] {
                        0x63 => {
                            if data.len() >= 3 {
                                match data[2] {
                                    0x00 => Some(ModuleSetCommand::ShowLogo),
                                    0x02 => {
                                        let slice = if data.len() >= 4 { data[3] } else { 0 };
                                        Some(ModuleSetCommand::UpdateBootLogo { slice })
                                    }
                                    _ => None,
                                }
                            } else { None }
                        }
                        0xA2 => {
                            if data.len() >= 6 {
                                let secs = i32::from_le_bytes([data[2], data[3], data[4], data[5]]);
                                Some(ModuleSetCommand::SetIdleTime { seconds: secs })
                            } else { None }
                        }
                        _ => None,
                    }
                } else { None }
            }
            _ => None,
        }
    }

    fn parse_module_get_command(&self, report_id: u8) -> Option<ModuleGetCommand> {
        match report_id {
            0xA0 => Some(ModuleGetCommand::GetFirmwareVersion(FirmwareType::LD)),
            0xA1 => Some(ModuleGetCommand::GetFirmwareVersion(FirmwareType::AP2)),
            0xA2 => Some(ModuleGetCommand::GetFirmwareVersion(FirmwareType::AP1)),
            0x03 => Some(ModuleGetCommand::GetUnitSerialNumber),
            0xA3 => Some(ModuleGetCommand::GetIdleTime),
            _ => None,
        }
    }
}

impl Module6KeysHandler {
    fn get_firmware_version(&self, firmware_type: FirmwareType) -> &'static [u8] {
        match firmware_type {
            FirmwareType::LD => b"1.00.003",
            FirmwareType::AP2 => b"1.03.000",
            FirmwareType::AP1 => b"1.03.000",
        }
    }

    fn get_unit_serial_number(&self) -> &'static [u8] {
        b"1234567890"
    }
}


impl ProtocolHandlerTrait for Module6KeysHandler {
    fn version(&self) -> ProtocolVersion { ProtocolVersion::Module6Keys }

    fn process_image_packet(&mut self, _data: &[u8]) -> ImageProcessResult {
        // Stub: will be implemented precisely after PCAP confirmation
        ImageProcessResult::Incomplete
    }

    fn map_buttons(&self, physical_buttons: &[bool], cols: usize, rows: usize, left_to_right: bool) -> ButtonMapping {
        let mut mapped = [false; 32];
        let mut count = 0usize;

        for y in 0..rows {
            for x in 0..cols {
                let src_index = if left_to_right { y * cols + x } else { y * cols + (cols - 1 - x) };
                let dst_index = y * cols + x;
                if src_index < physical_buttons.len() && dst_index < 32 {
                    mapped[dst_index] = physical_buttons[src_index];
                    if mapped[dst_index] { count += 1; }
                }
            }
        }

        ButtonMapping { mapped_buttons: mapped, active_count: count }
    }

    fn hid_descriptor(&self) -> &'static [u8] {
        // Minimal descriptor covering Input(0x01), Output(0x02), Feature(0x03/0x04/0x05/0x07/0x0B/0xA0/0xA1/0xA2/0xA3)
        // This can be fine-tuned to match exact real devices if needed.
        const DESC: &[u8] = &[
            0x05, 0x0C,             // Usage Page (Consumer)
            0x09, 0x01,             // Usage (Consumer Control)
            0xA1, 0x01,             // Collection (Application)

            // Input report 0x01 (keys)
            0x85, 0x01,             //   Report ID 0x01
            0x05, 0x09,             //   Usage Page (Button)
            0x19, 0x01,             //   Usage Minimum (Button 1)
            0x29, 0x20,             //   Usage Maximum (Button 32)
            0x15, 0x00,             //   Logical Minimum (0)
            0x26, 0xFF, 0x00,       //   Logical Maximum (255)
            0x75, 0x08,             //   Report Size (8)
            0x95, 0x20,             //   Report Count (32)
            0x81, 0x02,             //   Input (Data,Var,Abs)

            // Output report 0x02 (image/data chunks)
            0x85, 0x02,             //   Report ID 0x02
            0x0A, 0x00, 0xFF,       //   Usage (Vendor-Defined 0xFF00)
            0x15, 0x00,             //   Logical Minimum (0)
            0x26, 0xFF, 0x00,       //   Logical Maximum (255)
            0x75, 0x08,             //   Report Size (8)
            0x96, 0xFF, 0x03,       //   Report Count (1023)
            0x91, 0x02,             //   Output (Data,Var,Abs)

            // Feature reports (common IDs)
            0x85, 0x03, 0x0A, 0x00, 0xFF, 0x15, 0x00, 0x26, 0xFF, 0x00, 0x75, 0x08, 0x95, 0x10, 0xB1, 0x04,
            0x85, 0x04, 0x0A, 0x00, 0xFF, 0x15, 0x00, 0x26, 0xFF, 0x00, 0x75, 0x08, 0x95, 0x10, 0xB1, 0x04,
            0x85, 0x05, 0x0A, 0x00, 0xFF, 0x15, 0x00, 0x26, 0xFF, 0x00, 0x75, 0x08, 0x95, 0x10, 0xB1, 0x04,
            0x85, 0x07, 0x0A, 0x00, 0xFF, 0x15, 0x00, 0x26, 0xFF, 0x00, 0x75, 0x08, 0x95, 0x10, 0xB1, 0x04,
            0x85, 0x0B, 0x0A, 0x00, 0xFF, 0x15, 0x00, 0x26, 0xFF, 0x00, 0x75, 0x08, 0x95, 0x10, 0xB1, 0x04,
            0x85, 0xA0, 0x0A, 0x00, 0xFF, 0x15, 0x00, 0x26, 0xFF, 0x00, 0x75, 0x08, 0x95, 0x10, 0xB1, 0x04,
            0x85, 0xA1, 0x0A, 0x00, 0xFF, 0x15, 0x00, 0x26, 0xFF, 0x00, 0x75, 0x08, 0x95, 0x10, 0xB1, 0x04,
            0x85, 0xA2, 0x0A, 0x00, 0xFF, 0x15, 0x00, 0x26, 0xFF, 0x00, 0x75, 0x08, 0x95, 0x10, 0xB1, 0x04,
            0x85, 0xA3, 0x0A, 0x00, 0xFF, 0x15, 0x00, 0x26, 0xFF, 0x00, 0x75, 0x08, 0x95, 0x10, 0xB1, 0x04,

            0xC0,                   // End Collection
        ];
        DESC
    }

    fn input_report_size(&self, button_count: usize) -> usize {
        65
    }

    fn format_button_report(&self, buttons: &ButtonMapping, report: &mut [u8]) -> usize {
        let count = buttons.mapped_buttons.iter().take_while(|_| true).count();
        let used = core::cmp::min(32, count);
        let needed = 1 + used;
        if report.len() < needed { return 0; }
        report[0] = 0x01; // Report ID
        for i in 0..used { report[1 + i] = if buttons.mapped_buttons[i] { 1 } else { 0 }; }
        needed
    }

    fn handle_feature_report(&mut self, report_id: u8, data: &[u8]) -> Option<ProtocolCommand> {
        if let Some(cmd) = self.parse_module_set_command(report_id, data) {
            return match cmd {
                ModuleSetCommand::ShowLogo => Some(ProtocolCommand::ShowLogo),
                ModuleSetCommand::UpdateBootLogo { slice } => Some(ProtocolCommand::UpdateBootLogo { slice_index: slice }),
                ModuleSetCommand::SetBrightness { value } => Some(ProtocolCommand::SetBrightness(value)),
                ModuleSetCommand::SetIdleTime { seconds } => Some(ProtocolCommand::SetIdleTime(seconds)),
            }
        }
        None
    }

    fn get_feature_report(&mut self, report_id: u8, buf: &mut [u8]) -> Option<usize> {
        self.get_feature_report_bytes(report_id, buf)
    }
}

impl Module6KeysHandler {
    pub fn get_feature_report_bytes(&self, report_id: u8, buf: &mut [u8]) -> Option<usize> {
        let total_len = 32.min(buf.len());
        for i in 0..total_len { buf[i] = 0x00; }
        if let Some(cmd) = self.parse_module_get_command(report_id) {
            match cmd {
                ModuleGetCommand::GetFirmwareVersion(ftype) => {
                    let ver = self.get_firmware_version(ftype);
                    buf[0] = report_id;
                    // bytes 1..4 are N/A (0), version ASCII at offset 5
                    let start = 5; let end = (start + ver.len()).min(total_len);
                    // bytes 1..4 already zeroed above
                    if end > start { buf[start..end].copy_from_slice(&ver[..(end - start)]); }
                    return Some(total_len);
                }
                ModuleGetCommand::GetUnitSerialNumber => {
                    let serial = self.get_unit_serial_number();
                    buf[0] = 0x03;
                    let start = 5; let end = (start + serial.len()).min(total_len);
                    if end > start { buf[start..end].copy_from_slice(&serial[..(end - start)]); }
                    return Some(total_len);
                }
                ModuleGetCommand::GetIdleTime => {
                    buf[0] = 0xA3;
                    buf[1] = 0x06;
                    let secs = crate::config::get_idle_time_seconds() as i32;
                    let le = secs.to_le_bytes();
                    buf[2] = le[0]; buf[3] = le[1]; buf[4] = le[2]; buf[5] = le[3];
                    return Some(total_len);
                }
            }
        }
        None
    }
}


