# StreamDeck USB HID Device Implementation

## Complete Implementation Guide for RP2040 with Rust/Embassy

This document provides implementation guidance for creating a StreamDeck-compatible USB HID device that works with official StreamDeck software on Windows.

## ProductionDeck Current Implementation

**ProductionDeck** implements this protocol using:
- **Language**: Rust 2021 Edition
- **Framework**: Embassy async framework
- **Target**: RP2040 (thumbv6m-none-eabi)
- **USB Stack**: Embassy USB with usbd-hid
- **Implementation Files**:
  - `src/usb.rs` - USB HID device and protocol handling
  - `src/config.rs` - USB descriptors and constants
  - `src/main.rs` - Device initialization and task coordination

The current implementation follows the patterns described below but uses Rust/Embassy instead of C++.

## Hardware Requirements

### Supported Platforms
- **RP2040** (Raspberry Pi Pico) - Recommended for USB HID
- **STM32** with USB capability (STM32F4, STM32H7)
- **ESP32-S2/S3** with native USB
- **Arduino Leonardo/Micro** (limited capabilities)

### Display Requirements
- **ProductionDeck**: Single ST7735 TFT display (80x80px) shared by all 6 buttons
- **Traditional StreamDeck**: Individual TFT LCD per button (e.g., 15x 72x72px displays)
- Button matrix for input detection
- Optional: Status LEDs for connection/error indication

## USB HID Descriptor Implementation

### HID Report Descriptor (Rust)

```rust
// StreamDeck HID Report Descriptor - from src/usb.rs
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
```

### USB Device Configuration (Rust)

```rust
// StreamDeck Mini configuration - from src/config.rs and src/usb.rs
use embassy_usb::{Builder, Config};
use embassy_usb::class::hid::{HidReaderWriter, RequestHandler, State, Config as HidConfig};

// USB identifiers (from config.rs)
pub const USB_VID: u16 = 0x0fd9;                    // Elgato
pub const USB_PID: u16 = 0x0063;                    // StreamDeck Mini
pub const USB_MANUFACTURER: &str = "Elgato Systems";
pub const USB_PRODUCT: &str = "Stream Deck Mini";
pub const USB_SERIAL: &str = "ProductionDeck001";

// Device specifications
pub const STREAMDECK_KEYS: usize = 6;
pub const STREAMDECK_KEY_SIZE: usize = 80;
pub const HID_REPORT_SIZE_FEATURE: usize = 32;
pub const USB_POLL_RATE_MS: u16 = 1;

// USB configuration function
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

// HID Request Handler
struct StreamDeckHidHandler {
    usb_command_sender: embassy_sync::channel::Sender<'static, 
        embassy_sync::blocking_mutex::raw::ThreadModeRawMutex, UsbCommand, 4>,
}

impl StreamDeckHidHandler {
    fn new() -> Self {
        Self {
            usb_command_sender: USB_COMMAND_CHANNEL.sender(),
        }
    }
}
```

## Protocol Implementation

### Feature Report Handling (Rust)

```rust
// HID Request Handler implementation - from src/usb.rs
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
        
        None
    }
}

// Feature report command handling
impl StreamDeckHidHandler {
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
}
```

### Image Data Processing (Rust)

```rust
// Output report (image data) handling - from src/usb.rs
impl StreamDeckHidHandler {
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
                // Convert slice to heapless Vec for channel communication
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

// USB Command processing task
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
            UsbCommand::ImageData { key_id, data } => {
                debug!("Processing image data for key {}", key_id);
                let _ = DISPLAY_CHANNEL.sender().send(DisplayCommand::DisplayImage { key_id, data }).await;
            }
        }
    }
};
```

### Button State Reporting (Rust)

```rust
// Button report handling - from src/usb.rs
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

// Button state structure (from main.rs)
#[derive(Clone, Debug, Format)]
pub struct ButtonState {
    pub buttons: [bool; STREAMDECK_KEYS],
    pub changed: bool,
}

// Channel communication for button events
use embassy_sync::channel::Channel;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;

pub static BUTTON_CHANNEL: Channel<ThreadModeRawMutex, ButtonState, 4> = Channel::new();
```

## Hardware Integration Examples

### RP2040 Implementation (Rust)

```rust
// Main application - from src/main.rs
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::USB;
use embassy_rp::usb::{Driver, InterruptHandler};

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    
    // Create USB driver
    let driver = Driver::new(p.USB, Irqs);
    
    // Spawn USB task
    spawner.spawn(usb_task(driver, p.PIN_20)).unwrap();
    
    // Spawn display task on core 1
    spawner.spawn(display_task(
        p.SPI0, p.PIN_19, p.PIN_18, p.PIN_14, p.PIN_15, p.PIN_8, p.PIN_17
    )).unwrap();
    
    // Spawn button scanning task
    spawner.spawn(button_task(
        p.PIN_2, p.PIN_3, p.PIN_4, p.PIN_5, p.PIN_6
    )).unwrap();
    
    // Main loop - Embassy handles everything asynchronously
    loop {
        Timer::after(Duration::from_secs(1)).await;
    }
}

// USB task spawned on core 0
#[embassy_executor::task]
pub async fn usb_task(
    driver: Driver<'static, peripherals::USB>,
    usb_led_pin: peripherals::PIN_20,
) {
    // USB implementation as shown above...
}
```

### Embassy Async Architecture (Rust)

```rust
// Embassy provides async/await for embedded systems
// Multiple concurrent tasks instead of traditional main loop

// Task coordination using channels
use embassy_sync::channel::Channel;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;

// Communication channels between tasks
pub static BUTTON_CHANNEL: Channel<ThreadModeRawMutex, ButtonState, 4> = Channel::new();
pub static USB_COMMAND_CHANNEL: Channel<ThreadModeRawMutex, UsbCommand, 4> = Channel::new();
pub static DISPLAY_CHANNEL: Channel<ThreadModeRawMutex, DisplayCommand, 8> = Channel::new();

// USB commands enum
#[derive(Clone, Debug, Format)]
pub enum UsbCommand {
    Reset,
    SetBrightness(u8),
    ImageData {
        key_id: u8,
        data: heapless::Vec<u8, 1024>,
    },
}

// Display commands enum  
#[derive(Clone, Debug, Format)]
pub enum DisplayCommand {
    ClearAll,
    SetBrightness(u8),
    DisplayImage {
        key_id: u8,
        data: heapless::Vec<u8, 1024>,
    },
}

// All tasks run concurrently using Embassy's async executor
// - USB task handles HID protocol
// - Button task scans physical buttons
// - Display task manages TFT screen
// - All communicate via channels (lock-free, async)
```

## Testing and Validation

### Windows Testing
1. Install official Stream Deck software
2. Connect your device
3. Verify recognition in Device Manager (should show as "Stream Deck Mini")
4. Test button presses and image updates in Stream Deck software

### Protocol Validation (Rust)
```rust
// Debug logging using defmt (Real-Time Transfer)
use defmt::{debug, info, warn, error};

// Protocol debugging in handle_output_report
debug!("USB Output Report: {} bytes received", data.len());
debug!("Header: [{:02X}, {:02X}, {:02X}, {:02X}, {:02X}, {:02X}, {:02X}, {:02X}]",
       data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7]);

// Feature report debugging
info!("HID Get Report: ID={:?}", id);
info!("HID Set Report: ID={:?}, len={}", id, data.len());

// Command processing logs
info!("USB: Reset command (V1)");
info!("USB: Set brightness {}% (V2)", brightness);
debug!("Processing image data for key {}", key_id);

// Use RTT (Real-Time Transfer) for zero-overhead logging
// Set DEFMT_LOG environment variable to control log levels:
// DEFMT_LOG=debug cargo build
```

## Key Implementation Notes

1. **USB Timing**: Maintain consistent 1ms USB polling for responsive button detection
2. **Image Processing**: V1 devices use BMP with BGR color order, V2 use JPEG with RGB
3. **Memory Management**: Image buffers require significant RAM (6 * 80 * 80 * 3 = 115KB for Mini)
4. **Display Updates**: Rotate images 270° for Mini devices to match orientation
5. **Error Handling**: Implement proper USB error recovery and reconnection logic

## References

- **Protocol Analysis**: https://gist.github.com/cliffrowley/d18a9c4569537b195f2b1eb6c68469e0
- **Rust Implementation**: https://github.com/ryankurte/rust-streamdeck  
- **Elgato HID Docs**: https://docs.elgato.com/streamdeck/hid/
- **USB HID Specification**: https://www.usb.org/hid

This implementation provides a complete foundation for creating StreamDeck-compatible devices that work seamlessly with official software.