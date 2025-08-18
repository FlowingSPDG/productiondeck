//! USB HID implementation for StreamDeck Mini compatibility
//! 
//! This module implements the exact USB HID protocol required for 
//! StreamDeck Mini compatibility with official software.

use defmt::*;
use embassy_rp::gpio::Output;
use embassy_rp::peripherals;
use embassy_rp::usb::Driver;
use embassy_time::{Duration, Timer};
use embassy_usb::class::hid::{HidReaderWriter, RequestHandler, ReportId, State, Config as HidConfig};
use embassy_usb::control::OutResponse;
use embassy_usb::{Builder, Config};
use heapless::Vec;

use crate::config::*;
use crate::{BUTTON_CHANNEL, USB_COMMAND_CHANNEL, DISPLAY_CHANNEL, UsbCommand, DisplayCommand};

// ===================================================================
// USB HID Report Descriptor (StreamDeck Mini Compatible)
// ===================================================================
// 
// This descriptor exactly matches the real StreamDeck Mini HID report descriptor
// as captured from USB traffic analysis. The descriptor structure is:
//
// 1. Usage Page (Consumer) - 0x05, 0x0c
// 2. Usage (Consumer Control) - 0x09, 0x01
// 3. Collection (Application) - 0xa1, 0x01
// 4. Multiple report definitions with IDs: 0x01, 0x02, 0x03, 0x04, 0x05, 0x07, 0x0b, 0xa0, 0xa1, 0xa2
// 5. End Collection - 0xc0
//
// Total length: 173 bytes (0xAD) - matches wTotalLength in configuration descriptor
// This ensures perfect compatibility with StreamDeck software.

const HID_REPORT_DESCRIPTOR: &[u8] = &[
    // Real StreamDeck Mini HID Report Descriptor (exact copy from hex dump)
    // This matches the exact byte sequence from the real StreamDeck Mini device
    // Total length: 173 bytes (0xAD) - matches wTotalLength in configuration descriptor
    
    // Usage Page (Consumer) - 0x05, 0x0c
    0x05, 0x0c,
    // Usage (Consumer Control) - 0x09, 0x01  
    0x09, 0x01,
    // Collection (Application) - 0xa1, 0x01
    0xa1, 0x01,
    // Usage (Consumer Control) - 0x09, 0x01
    0x09, 0x01,
    // Usage Page (Button) - 0x05, 0x09
    0x05, 0x09,
    // Usage Minimum (0x01) - 0x19, 0x01
    0x19, 0x01,
    // Usage Maximum (0x10) - 0x29, 0x10
    0x29, 0x10,
    // Logical Minimum (0) - 0x15, 0x00
    0x15, 0x00,
    // Logical Maximum (255) - 0x26, 0xff, 0x00
    0x26, 0xff, 0x00,
    // Report Size (8) - 0x75, 0x08
    0x75, 0x08,
    // Report Count (16) - 0x95, 0x10
    0x95, 0x10,
    // Report ID (0x01) - 0x85, 0x01
    0x85, 0x01,
    // Input (Data,Var,Abs) - 0x81, 0x02
    0x81, 0x02,
    // Usage (Button 255) - 0x0a, 0x00, 0xff
    0x0a, 0x00, 0xff,
    // Logical Minimum (0) - 0x15, 0x00
    0x15, 0x00,
    // Logical Maximum (255) - 0x26, 0xff, 0x00
    0x26, 0xff, 0x00,
    // Report Size (8) - 0x75, 0x08
    0x75, 0x08,
    // Report Count (1023) - 0x96, 0xff, 0x03
    0x96, 0xff, 0x03,
    // Report ID (0x02) - 0x85, 0x02
    0x85, 0x02,
    // Output (Data,Var,Abs) - 0x91, 0x02
    0x91, 0x02,
    // Usage (Button 255) - 0x0a, 0x00, 0xff
    0x0a, 0x00, 0xff,
    // Logical Minimum (0) - 0x15, 0x00
    0x15, 0x00,
    // Logical Maximum (255) - 0x26, 0xff, 0x00
    0x26, 0xff, 0x00,
    // Report Size (8) - 0x75, 0x08
    0x75, 0x08,
    // Report Count (16) - 0x95, 0x10
    0x95, 0x10,
    // Report ID (0x03) - 0x85, 0x03
    0x85, 0x03,
    // Feature (Data,Array,Rel) - 0xb1, 0x04
    0xb1, 0x04,
    // Usage (Button 255) - 0x0a, 0x00, 0xff
    0x0a, 0x00, 0xff,
    // Logical Minimum (0) - 0x15, 0x00
    0x15, 0x00,
    // Logical Maximum (255) - 0x26, 0xff, 0x00
    0x26, 0xff, 0x00,
    // Report Size (8) - 0x75, 0x08
    0x75, 0x08,
    // Report Count (16) - 0x95, 0x10
    0x95, 0x10,
    // Report ID (0x04) - 0x85, 0x04
    0x85, 0x04,
    // Feature (Data,Array,Rel) - 0xb1, 0x04
    0xb1, 0x04,
    // Usage (Button 255) - 0x0a, 0x00, 0xff
    0x0a, 0x00, 0xff,
    // Logical Minimum (0) - 0x15, 0x00
    0x15, 0x00,
    // Logical Maximum (255) - 0x26, 0xff, 0x00
    0x26, 0xff, 0x00,
    // Report Size (8) - 0x75, 0x08
    0x75, 0x08,
    // Report Count (16) - 0x95, 0x10
    0x95, 0x10,
    // Report ID (0x05) - 0x85, 0x05
    0x85, 0x05,
    // Feature (Data,Array,Rel) - 0xb1, 0x04
    0xb1, 0x04,
    // Usage (Button 255) - 0x0a, 0x00, 0xff
    0x0a, 0x00, 0xff,
    // Logical Minimum (0) - 0x15, 0x00
    0x15, 0x00,
    // Logical Maximum (255) - 0x26, 0xff, 0x00
    0x26, 0xff, 0x00,
    // Report Size (8) - 0x75, 0x08
    0x75, 0x08,
    // Report Count (16) - 0x95, 0x10
    0x95, 0x10,
    // Report ID (0x07) - 0x85, 0x07
    0x85, 0x07,
    // Feature (Data,Array,Rel) - 0xb1, 0x04
    0xb1, 0x04,
    // Usage (Button 255) - 0x0a, 0x00, 0xff
    0x0a, 0x00, 0xff,
    // Logical Minimum (0) - 0x15, 0x00
    0x15, 0x00,
    // Logical Maximum (255) - 0x26, 0xff, 0x00
    0x26, 0xff, 0x00,
    // Report Size (8) - 0x75, 0x08
    0x75, 0x08,
    // Report Count (16) - 0x95, 0x10
    0x95, 0x10,
    // Report ID (0x0b) - 0x85, 0x0b
    0x85, 0x0b,
    // Feature (Data,Array,Rel) - 0xb1, 0x04
    0xb1, 0x04,
    // Usage (Button 255) - 0x0a, 0x00, 0xff
    0x0a, 0x00, 0xff,
    // Logical Minimum (0) - 0x15, 0x00
    0x15, 0x00,
    // Logical Maximum (255) - 0x26, 0xff, 0x00
    0x26, 0xff, 0x00,
    // Report Size (8) - 0x75, 0x08
    0x75, 0x08,
    // Report Count (16) - 0x95, 0x10
    0x95, 0x10,
    // Report ID (0xa0) - 0x85, 0xa0
    0x85, 0xa0,
    // Feature (Data,Array,Rel) - 0xb1, 0x04
    0xb1, 0x04,
    // Usage (Button 255) - 0x0a, 0x00, 0xff
    0x0a, 0x00, 0xff,
    // Logical Minimum (0) - 0x15, 0x00
    0x15, 0x00,
    // Logical Maximum (255) - 0x26, 0xff, 0x00
    0x26, 0xff, 0x00,
    // Report Size (8) - 0x75, 0x08
    0x75, 0x08,
    // Report Count (16) - 0x95, 0x10
    0x95, 0x10,
    // Report ID (0xa1) - 0x85, 0xa1
    0x85, 0xa1,
    // Feature (Data,Array,Rel) - 0xb1, 0x04
    0xb1, 0x04,
    // Usage (Button 255) - 0x0a, 0x00, 0xff
    0x0a, 0x00, 0xff,
    // Logical Minimum (0) - 0x15, 0x00
    0x15, 0x00,
    // Logical Maximum (255) - 0x26, 0xff, 0x00
    0x26, 0xff, 0x00,
    // Report Size (8) - 0x75, 0x08
    0x75, 0x08,
    // Report Count (16) - 0x95, 0x10
    0x95, 0x10,
    // Report ID (0xa2) - 0x85, 0xa2
    0x85, 0xa2,
    // Feature (Data,Array,Rel) - 0xb1, 0x04
    0xb1, 0x04,
    // End Collection - 0xc0
    0xc0
];

// ===================================================================
// USB Configuration
// ===================================================================

fn create_usb_config() -> Config<'static> {
    let mut config = Config::new(USB_VID, USB_PID);
    config.manufacturer = Some(USB_MANUFACTURER);
    config.product = Some(USB_PRODUCT);
    config.serial_number = Some(USB_SERIAL);
    config.max_power = 100; // 200mA (matches real StreamDeck Mini)
    config.max_packet_size_0 = 64;
    config.device_class = 0x00; // Interface-defined (HID class will be set in interface)
    config.device_sub_class = 0x00;
    config.device_protocol = 0x00;
    config.composite_with_iads = false;
    
    // Set device version to match real StreamDeck Mini
    config.device_release = USB_BCD_DEVICE;
    
    // Force USB 2.0 (not 2.1) to match real StreamDeck Mini
    // Note: embassy-usb automatically sets bcd_usb to 0x0200 for USB 2.0
    
    // Enable remote wakeup to match real StreamDeck Mini bmAttributes: 0xa0
    // Note: embassy-usb automatically handles remote wakeup based on configuration
    
    config
}

// ===================================================================
// HID Request Handler
// ===================================================================

struct StreamDeckHidHandler {
    usb_command_sender: embassy_sync::channel::Sender<'static, embassy_sync::blocking_mutex::raw::ThreadModeRawMutex, UsbCommand, 4>,
}

impl RequestHandler for StreamDeckHidHandler {
    fn get_report(&mut self, id: ReportId, _buf: &mut [u8]) -> Option<usize> {
        info!("HID Get Report: ID={:?}", id);
        
        match id {
            ReportId::In(_) => {
                // Button state will be sent via separate input reports
                None
            }
            ReportId::Feature(report_id) => {
                // Handle feature report requests for StreamDeck Mini
                match report_id {
                    0x03 | 0x04 | 0x05 | 0x07 | 0x0b | 0xa0 | 0xa1 | 0xa2 => {
                        // StreamDeck Mini feature reports - return 16 bytes
                        _buf[0] = report_id;
                        // Fill remaining 15 bytes with zeros
                        for i in 1..16 {
                            _buf[i] = 0x00;
                        }
                        Some(16)
                    }
                    _ => {
                        warn!("Unknown feature report ID: 0x{:02x}", report_id);
                        None
                    }
                }
            }
            _ => None,
        }
    }

    fn set_report(&mut self, id: ReportId, data: &[u8]) -> OutResponse {
        info!("HID Set Report: ID={:?}, len={}", id, data.len());
        
        match id {
            ReportId::Feature(report_id) => {
                self.handle_feature_report(report_id, data);
            }
            ReportId::Out(_) => {
                self.handle_output_report(data);
            }
            _ => {}
        }
        
        OutResponse::Accepted
    }
}

impl StreamDeckHidHandler {
    fn new() -> Self {
        Self {
            usb_command_sender: USB_COMMAND_CHANNEL.sender(),
        }
    }

    fn handle_feature_report(&mut self, report_id: u8, data: &[u8]) {
        match report_id {
            0x00 => {
                // Feature Report ID 0x00 - StreamDeck initialization command
                // This is likely a device initialization or status request
                info!("USB: Feature Report 0x00 received (initialization)");
                // Respond with success - this is critical for StreamDeck software recognition
                debug!("Feature Report 0x00 data: {:?}", data);
            }
            FEATURE_REPORT_RESET_V1 => {
                // V1 Reset: [0x0B, 0x63, ...]
                if data.len() >= 2 && data[1] == 0x63 {
                    info!("USB: Reset command (V1)");
                    let _ = self.usb_command_sender.try_send(UsbCommand::Reset);
                }
            }
            0x03 => {
                // V2 commands: [0x03, command_byte, ...]
                if data.len() >= 2 {
                    match data[1] {
                        0x02 => {
                            // V2 Reset: [0x03, 0x02, ...]
                            info!("USB: Reset command (V2)");
                            let _ = self.usb_command_sender.try_send(UsbCommand::Reset);
                        }
                        0x08 => {
                            // V2 Brightness: [0x03, 0x08, brightness, ...]
                            if data.len() >= 3 {
                                let brightness = data[2];
                                info!("USB: Set brightness {}% (V2)", brightness);
                                let _ = self.usb_command_sender.try_send(UsbCommand::SetBrightness(brightness));
                            }
                        }
                        _ => {
                            warn!("Unknown V2 command: 0x{:02X}", data[1]);
                        }
                    }
                }
            }
            FEATURE_REPORT_BRIGHTNESS_V1 => {
                // V1 Brightness: [0x05, 0x55, 0xAA, 0xD1, 0x01, brightness, ...]
                if data.len() >= 6 && data[1] == 0x55 && data[2] == 0xAA && 
                   data[3] == 0xD1 && data[4] == 0x01 {
                    let brightness = data[5];
                    info!("USB: Set brightness {}% (V1)", brightness);
                    let _ = self.usb_command_sender.try_send(UsbCommand::SetBrightness(brightness));
                }
            }
            _ => {
                warn!("Unknown feature report ID: 0x{:02X}", report_id);
            }
        }
    }

    fn handle_output_report(&mut self, data: &[u8]) {
        if data.len() < 8 {
            warn!("Invalid output report length: {}", data.len());
            return;
        }

        debug!("USB Output Report: {} bytes received", data.len());
        debug!("Header: [{:02X}, {:02X}, {:02X}, {:02X}, {:02X}, {:02X}, {:02X}, {:02X}]",
               data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7]);

        // Parse output report header (StreamDeck Mini V2 protocol)
        if data[0] == OUTPUT_REPORT_IMAGE && data[1] == IMAGE_COMMAND_V2 {
            // V2 Image protocol: [0x02, 0x07, key_id, is_last, len_low, len_high, seq_low, seq_high, data...]
            let key_id = data[2];
            let _is_last = data[3];
            let payload_len = u16::from_le_bytes([data[4], data[5]]);
            let _sequence = u16::from_le_bytes([data[6], data[7]]);

            debug!("Image packet: key={} seq={} len={} last={}", 
                   key_id, _sequence, payload_len, _is_last);

            if key_id < STREAMDECK_KEYS as u8 {
                // Convert slice to heapless Vec
                let mut image_data = Vec::new();
                if image_data.extend_from_slice(data).is_ok() {
                    let _ = self.usb_command_sender.try_send(UsbCommand::ImageData { 
                        key_id, 
                        data: image_data 
                    });
                } else {
                    error!("Failed to copy image data to buffer");
                }
            } else {
                error!("Invalid key_id {} (max {})", key_id, STREAMDECK_KEYS - 1);
            }
        } else {
            debug!("Unknown output report format: [0x{:02X}, 0x{:02X}]", data[0], data[1]);
        }
    }
}

// ===================================================================
// USB Task Implementation
// ===================================================================

#[embassy_executor::task]
pub async fn usb_task(
    driver: Driver<'static, peripherals::USB>,
    mut usb_led: Output<'static>,
) {
    info!("USB task started");
    info!("USB HID communication will start with periodic button state reports");

    // Create USB configuration
    let config = create_usb_config();

    // Create USB builder
    static mut DEVICE_DESC_BUF: [u8; 256] = [0; 256];
    static mut CONFIG_DESC_BUF: [u8; 256] = [0; 256];
    static mut BOS_DESC_BUF: [u8; 256] = [0; 256];
    static mut CONTROL_BUF: [u8; 512] = [0; 512];
    let mut builder = unsafe {
        #[allow(static_mut_refs)]
        Builder::new(
            driver,
            config,
            &mut DEVICE_DESC_BUF,  // Device descriptor buffer
            &mut CONFIG_DESC_BUF,  // Config descriptor buffer
            &mut BOS_DESC_BUF,     // BOS descriptor buffer
            &mut CONTROL_BUF,      // Control buffer
        )
    };

    // Create HID request handler
    static mut REQUEST_HANDLER: Option<StreamDeckHidHandler> = None;
    unsafe {
        REQUEST_HANDLER = Some(StreamDeckHidHandler::new());
    }
    let hid_config = HidConfig {
        report_descriptor: HID_REPORT_DESCRIPTOR,
        #[allow(static_mut_refs)]
        request_handler: unsafe { REQUEST_HANDLER.as_mut().map(|h| h as _) },
        poll_ms: USB_POLL_RATE_MS as u8,
        max_packet_size: 64, // RP2040 USB hardware limitation (max 64 bytes)
    };
    
    // Note: embassy-usb automatically calculates wTotalLength based on all descriptors
    // The HID descriptor length (173 bytes) contributes to the total configuration descriptor length
    
    info!("HID configuration created with report descriptor size: {} bytes", HID_REPORT_DESCRIPTOR.len());
    info!("HID descriptor structure: Consumer Control (0x0C) -> Button (0x09) with 10 report IDs: 0x01,0x02,0x03,0x04,0x05,0x07,0x0b,0xa0,0xa1,0xa2");
    
    // Verify descriptor length matches real StreamDeck Mini (173 bytes = 0xAD)
    if HID_REPORT_DESCRIPTOR.len() != 173 {
        error!("HID descriptor length mismatch: expected 173 bytes, got {} bytes", HID_REPORT_DESCRIPTOR.len());
    } else {
        info!("HID descriptor length verified: 173 bytes (0xAD) - matches real StreamDeck Mini");
    }
    
    // HID class is automatically added when creating HidReaderWriter

    static mut HID_STATE: State = State::new();
    #[allow(static_mut_refs)]
    let hid = unsafe { HidReaderWriter::<_, 64, 64>::new(&mut builder, &mut HID_STATE, hid_config) };

    // HID descriptor length: 173 bytes (0xAD) - matches real StreamDeck Mini wTotalLength

    // Build USB device
    let mut usb = builder.build();

    // Split HID into reader and writer
    let (_reader, mut writer) = hid.split();

    // Spawn USB device task
    let usb_fut = usb.run();

    // Spawn USB command processor (temporarily disabled - no commands being sent)
    let command_fut = async {
        info!("USB command processor started (waiting for commands)");
        let receiver = USB_COMMAND_CHANNEL.receiver();
        loop {
            match receiver.receive().await {
                UsbCommand::Reset => {
                    info!("Processing reset command");
                    let _ = DISPLAY_CHANNEL.sender().send(DisplayCommand::ClearAll).await;
                }
                UsbCommand::SetBrightness(brightness) => {
                    info!("Processing brightness command: {}%", brightness);
                    let _ = DISPLAY_CHANNEL.sender().send(DisplayCommand::SetBrightness(brightness)).await;
                }
                // ここでVectorのバッファサイズが大きいためHardFaultが発生するためコメントアウト
                /*
                UsbCommand::ImageData { key_id, data } => {
                    debug!("Processing image data for key {}", key_id);
                    let _ = DISPLAY_CHANNEL.sender().send(DisplayCommand::DisplayImage { key_id, data }).await;
                }
                */
                _ => {
                    warn!("Unknown USB command");
                }
            }
        }
    };

    // Spawn button report sender
    let button_fut = async {
        let receiver = BUTTON_CHANNEL.receiver();
        
        // Send initial button state report (all buttons released)
        let mut initial_report = [0u8; 16];
        initial_report[0] = 0x01; // Report ID
        match writer.write(&initial_report).await {
            Ok(()) => {
                info!("Initial button state report sent: {:?}", initial_report);
            }
            Err(e) => {
                warn!("Failed to send initial button report: {:?}", e);
            }
        }
        
        loop {
            // Wait for button state updates
            let button_state = receiver.receive().await;
            
            // Convert button state to HID report format (Report ID 0x01, 16 bytes)
            let mut report = [0u8; 16];
            report[0] = 0x01; // Report ID
            
            for (i, &pressed) in button_state.buttons.iter().enumerate() {
                if i < 15 { // First 15 bytes for button states
                    report[i + 1] = if pressed { 1 } else { 0 };
                }
            }

            // Send button report
            match writer.write(&report).await {
                Ok(()) => {
                    debug!("Button report sent: {:?}", report);
                }
                Err(e) => {
                    warn!("Failed to send button report: {:?}", e);
                }
            }
        }
    };

    // USB status LED control (simplified - just turn on when USB task is running)
    let led_fut = async {
        info!("USB LED task started");
        usb_led.set_high();
        loop {
            Timer::after(Duration::from_secs(1)).await;
        }
    };

    // Run all futures concurrently
    embassy_futures::join::join4(usb_fut, command_fut, button_fut, led_fut).await;
}