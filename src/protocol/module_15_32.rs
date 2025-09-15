//! StreamDeck Module HID Protocol Handler (15/32 keys)
//!
//! Unified handler for Module 15 and Module 32 per Elgato HID API.
//! Reference: https://docs.elgato.com/streamdeck/hid/module-15_32

use super::{ButtonMapping, ImageProcessResult, ProtocolHandlerTrait};
use crate::device::ProtocolVersion;
use crate::protocol::module::{FirmwareType, ModuleGetCommand, ModuleSetCommand};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ModuleModel {
    Module15,
    Module32,
}

#[derive(Debug)]
pub struct Module15_32KeysHandler {
    model: ModuleModel,
}

impl Module15_32KeysHandler {
    pub fn new() -> Self {
        Self {
            model: ModuleModel::Module15,
        }
    }
    pub fn with_model(model: ModuleModel) -> Self {
        Self { model }
    }

    fn parse_module_set_command(&self, report_id: u8, data: &[u8]) -> Option<ModuleSetCommand> {
        match report_id {
            // Set Backlight Brightness (Feature report ID 0x03, Command 0x08)
            0x03 => {
                if data.len() >= 3 && data[1] == 0x08 {
                    Some(ModuleSetCommand::SetBrightness { value: data[2] })
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn parse_module_get_command(&self, report_id: u8) -> Option<ModuleGetCommand> {
        match report_id {
            0x04 => Some(ModuleGetCommand::GetFirmwareVersion(FirmwareType::LD)),
            0x05 => Some(ModuleGetCommand::GetFirmwareVersion(FirmwareType::AP2)),
            0x07 => Some(ModuleGetCommand::GetFirmwareVersion(FirmwareType::AP1)),
            0x06 => Some(ModuleGetCommand::GetUnitSerialNumber),
            0x0A => Some(ModuleGetCommand::GetIdleTime),
            _ => None,
        }
    }

    fn get_firmware_version(&self, firmware_type: FirmwareType) -> &'static [u8] {
        match firmware_type {
            FirmwareType::LD => b"1.00.000",
            FirmwareType::AP2 => b"1.00.000",
            FirmwareType::AP1 => b"1.00.000",
        }
    }

    fn get_unit_serial_number(&self) -> &'static [u8] {
        b"A1B2C3D4E5F6G7"
    }
}

impl Default for Module15_32KeysHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl ProtocolHandlerTrait for Module15_32KeysHandler {
    fn version(&self) -> ProtocolVersion {
        ProtocolVersion::Module15_32Keys
    }

    fn process_image_packet(&mut self, _data: &[u8]) -> ImageProcessResult {
        // Image upload uses Output report ID 0x02 with Command 0x07 (key), 0x08 (full), 0x09 (boot)
        ImageProcessResult::Incomplete
    }

    fn map_buttons(
        &self,
        physical_buttons: &[bool],
        cols: usize,
        rows: usize,
        left_to_right: bool,
    ) -> ButtonMapping {
        let max = match self.model {
            ModuleModel::Module15 => 15,
            ModuleModel::Module32 => 32,
        };
        let mut mapped = [false; 32];
        for y in 0..rows {
            for x in 0..cols {
                let src_index = if left_to_right {
                    y * cols + x
                } else {
                    y * cols + (cols - 1 - x)
                };
                let dst_index = y * cols + x;
                if src_index < physical_buttons.len() && dst_index < max {
                    mapped[dst_index] = physical_buttons[src_index];
                }
            }
        }
        ButtonMapping {
            mapped_buttons: mapped,
            active_count: max,
        }
    }

    fn hid_descriptor(&self) -> &'static [u8] {
        // Input(0x01), Output(0x02), Feature IDs (0x03,0x04,0x05,0x06,0x07,0x0A)
        const DESC: &[u8] = &[
            0x05, 0x0C, 0x09, 0x01, 0xA1, 0x01, 0x85, 0x01, 0x05, 0x09, 0x19, 0x01, 0x29, 0x20,
            0x15, 0x00, 0x26, 0xFF, 0x00, 0x75, 0x08, 0x95, 0x20, 0x81, 0x02, 0x85, 0x02, 0x0A,
            0x00, 0xFF, 0x15, 0x00, 0x26, 0xFF, 0x00, 0x75, 0x08, 0x96, 0xFF, 0x03, 0x91, 0x02,
            0x85, 0x03, 0x0A, 0x00, 0xFF, 0x15, 0x00, 0x26, 0xFF, 0x00, 0x75, 0x08, 0x95, 0x10,
            0xB1, 0x04, 0x85, 0x04, 0x0A, 0x00, 0xFF, 0x15, 0x00, 0x26, 0xFF, 0x00, 0x75, 0x08,
            0x95, 0x10, 0xB1, 0x04, 0x85, 0x05, 0x0A, 0x00, 0xFF, 0x15, 0x00, 0x26, 0xFF, 0x00,
            0x75, 0x08, 0x95, 0x10, 0xB1, 0x04, 0x85, 0x06, 0x0A, 0x00, 0xFF, 0x15, 0x00, 0x26,
            0xFF, 0x00, 0x75, 0x08, 0x95, 0x10, 0xB1, 0x04, 0x85, 0x07, 0x0A, 0x00, 0xFF, 0x15,
            0x00, 0x26, 0xFF, 0x00, 0x75, 0x08, 0x95, 0x10, 0xB1, 0x04, 0x85, 0x0A, 0x0A, 0x00,
            0xFF, 0x15, 0x00, 0x26, 0xFF, 0x00, 0x75, 0x08, 0x95, 0x10, 0xB1, 0x04, 0xC0,
        ];
        DESC
    }

    fn input_report_size(&self, _button_count: usize) -> usize {
        512
    }

    fn format_button_report(&self, buttons: &ButtonMapping, report: &mut [u8]) -> usize {
        // Report ID 0x01, Command 0x00, Length = number of keys, then states
        let max_keys = match self.model {
            ModuleModel::Module15 => 15,
            ModuleModel::Module32 => 32,
        };
        let used = core::cmp::min(max_keys, buttons.mapped_buttons.len());
        let needed = 4 + used;
        if report.len() < needed {
            return 0;
        }
        report[0] = 0x01; // Report ID
        report[1] = 0x00; // Command: key state change
        report[2] = used as u8; // length LSB (fits within u8 for 32)
        report[3] = 0x00; // length MSB
        for i in 0..used {
            report[4 + i] = if buttons.mapped_buttons[i] { 1 } else { 0 };
        }
        needed
    }

    fn handle_feature_report(&mut self, report_id: u8, data: &[u8]) -> Option<ModuleSetCommand> {
        if let Some(cmd) = self.parse_module_set_command(report_id, data) {
            return Some(cmd);
        }
        None
    }

    fn get_feature_report(&mut self, report_id: u8, buf: &mut [u8]) -> Option<usize> {
        let total_len = 32.min(buf.len());
        buf.iter_mut().take(total_len).for_each(|b| *b = 0);
        if let Some(cmd) = self.parse_module_get_command(report_id) {
            match cmd {
                ModuleGetCommand::GetFirmwareVersion(ftype) => {
                    let ver = self.get_firmware_version(ftype);
                    buf[0] = report_id;
                    buf[1] = 0x0C; // data length
                                   // bytes 2..5 checksum ignored (0)
                                   // version ASCII at offset 6, 8 bytes
                    let start = 6;
                    let end = (start + ver.len()).min(total_len);
                    if end > start {
                        buf[start..end].copy_from_slice(&ver[..(end - start)]);
                    }
                    return Some(total_len);
                }
                ModuleGetCommand::GetUnitSerialNumber => {
                    let serial = self.get_unit_serial_number();
                    buf[0] = 0x06;
                    let data_len = core::cmp::min(serial.len(), 14) as u8;
                    buf[1] = data_len; // 0x0C or 0x0E
                    let start = 2;
                    let end = (start + data_len as usize).min(total_len);
                    if end > start {
                        buf[start..end].copy_from_slice(&serial[..(end - start)]);
                    }
                    return Some(total_len);
                }
                ModuleGetCommand::GetIdleTime => {
                    buf[0] = 0x0A;
                    buf[1] = 0x04; // data length
                    let secs = crate::config::get_idle_time_seconds();
                    let le = secs.to_le_bytes();
                    buf[2] = le[0];
                    buf[3] = le[1];
                    buf[4] = le[2];
                    buf[5] = le[3];
                    return Some(total_len);
                }
                ModuleGetCommand::GetUnitInformation => {
                    // Module 15/32 specific unit info not implemented here
                    return None;
                }
            }
        }
        None
    }
}
