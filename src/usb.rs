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

const HID_REPORT_DESCRIPTOR: &[u8] = &[
    // Usage Page (Generic Desktop)
    0x05, 0x01,
    // Usage (Undefined)
    0x09, 0x00,
    // Collection (Application)
    0xa1, 0x01,
    
    // ===============================================
    // Input Report (Button States: Device → Host)
    // ===============================================
    0x09, 0x00,                         // Usage (Undefined)
    0x15, 0x00,                         // Logical Minimum (0)
    0x25, 0x01,                         // Logical Maximum (1)
    0x75, 0x08,                         // Report Size (8 bits)
    0x95, STREAMDECK_KEYS as u8,        // Report Count (6 buttons)
    0x81, 0x02,                         // Input (Data, Variable, Absolute)
    
    // ===============================================
    // Output Report (Image Data: Host → Device)
    // ===============================================
    0x09, 0x00,                         // Usage (Undefined)
    0x15, 0x00,                         // Logical Minimum (0)
    0x26, 0xFF, 0x00,                   // Logical Maximum (255)
    0x75, 0x08,                         // Report Size (8 bits)
    0x96, 0x00, 0x04,                   // Report Count (1024 bytes)
    0x91, 0x02,                         // Output (Data, Variable, Absolute)
    
    // ===============================================
    // Feature Report (Commands: Bidirectional)
    // ===============================================
    0x09, 0x00,                         // Usage (Undefined)
    0x15, 0x00,                         // Logical Minimum (0)
    0x26, 0xFF, 0x00,                   // Logical Maximum (255)
    0x75, 0x08,                         // Report Size (8 bits)
    0x95, HID_REPORT_SIZE_FEATURE as u8, // Report Count (32 bytes)
    0xb1, 0x02,                         // Feature (Data, Variable, Absolute)
    
    // End Collection
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
    config.max_power = 100; // 200mA
    config.max_packet_size_0 = 64;
    config.device_class = 0x00; // Interface-defined
    config.device_sub_class = 0x00;
    config.device_protocol = 0x00;
    config.composite_with_iads = false;
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
                // Handle feature report requests (version, etc.)
                if report_id == FEATURE_REPORT_VERSION_V1 || report_id == FEATURE_REPORT_VERSION_V2 {
                    // Version request - return firmware version
                    _buf[0] = report_id;
                    let offset = if report_id == FEATURE_REPORT_VERSION_V2 { 6 } else { 5 };
                    let version = b"1.0.0";
                    
                    if _buf.len() > offset + version.len() {
                        _buf[offset..offset + version.len()].copy_from_slice(version);
                        return Some(HID_REPORT_SIZE_FEATURE);
                    }
                }
                None
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

    // Create USB configuration
    let config = create_usb_config();

    // Create USB builder
    static mut DEVICE_DESC_BUF: [u8; 256] = [0; 256];
    static mut CONFIG_DESC_BUF: [u8; 256] = [0; 256];
    static mut BOS_DESC_BUF: [u8; 256] = [0; 256];
    static mut CONTROL_BUF: [u8; 128] = [0; 128];
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
        max_packet_size: 64,
    };

    static mut HID_STATE: State = State::new();
    #[allow(static_mut_refs)]
    let hid = unsafe { HidReaderWriter::<_, 64, 64>::new(&mut builder, &mut HID_STATE, hid_config) };

    // Build USB device
    let mut usb = builder.build();

    // Split HID into reader and writer
    let (_reader, mut writer) = hid.split();

    // Spawn USB device task
    let usb_fut = usb.run();

    // Spawn USB command processor
    let command_fut = async {
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
        loop {
            let button_state = receiver.receive().await;
            if button_state.changed {
                // Convert button state to HID report format
                let mut report = [0u8; STREAMDECK_KEYS];
                for (i, &pressed) in button_state.buttons.iter().enumerate() {
                    report[i] = if pressed { 1 } else { 0 };
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