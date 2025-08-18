//! Hardware configuration for ProductionDeck
//! RP2040-based StreamDeck Mini compatible device

// ===================================================================
// USB Configuration - CRITICAL: Must match StreamDeck Mini
// ===================================================================

pub const USB_VID: u16 = 0x0fd9; // Elgato Systems VID
pub const USB_PID: u16 = 0x0063; // StreamDeck Mini PID
pub const USB_MANUFACTURER: &str = "Elgato Systems";
pub const USB_PRODUCT: &str = "Stream Deck Mini";
pub const USB_SERIAL: &str = "PD240100001"; // ProductionDeck serial format

// ===================================================================
// Device Specifications (StreamDeck Mini)
// ===================================================================

pub const STREAMDECK_KEYS: usize = 6; // Number of keys (3x2 layout)
pub const STREAMDECK_COLS: usize = 3; // Keys per row
pub const STREAMDECK_ROWS: usize = 2; // Number of rows
pub const KEY_IMAGE_SIZE: usize = 72; // 72x72 pixels per key
pub const KEY_IMAGE_BYTES: usize = KEY_IMAGE_SIZE * KEY_IMAGE_SIZE * 3; // RGB

// ===================================================================
// USB HID Configuration
// ===================================================================

#[allow(dead_code)]
pub const HID_REPORT_SIZE_INPUT: usize = 32; // Button state report size
#[allow(dead_code)]
pub const HID_REPORT_SIZE_OUTPUT: usize = 1024; // Image data report size
pub const HID_REPORT_SIZE_FEATURE: usize = 32; // Feature report size

// ===================================================================
// GPIO Pin Assignments - Raspberry Pi Pico
// ===================================================================

// Button Matrix (6 buttons arranged as 3x2)
#[allow(dead_code)]
pub const BTN_ROW_PINS: [u8; STREAMDECK_ROWS] = [2, 3]; // GPIO 2, 3
#[allow(dead_code)]
pub const BTN_COL_PINS: [u8; STREAMDECK_COLS] = [4, 5, 6]; // GPIO 4, 5, 6

// SPI Display Interface
#[allow(dead_code)]
pub const SPI_MOSI_PIN: u8 = 19; // Data to display
#[allow(dead_code)]
pub const SPI_SCK_PIN: u8 = 18; // Clock to display
#[allow(dead_code)]
pub const SPI_BAUDRATE: u32 = 10_000_000; // 10MHz SPI clock

// Single Display Control Pins
#[allow(dead_code)]
pub const DISPLAY_CS_PIN: u8 = 8; // Chip select
#[allow(dead_code)]
pub const DISPLAY_DC_PIN: u8 = 14; // Data/Command select
#[allow(dead_code)]
pub const DISPLAY_RST_PIN: u8 = 15; // Reset
#[allow(dead_code)]
pub const DISPLAY_BL_PIN: u8 = 17; // Backlight control (PWM)

// Status LEDs
#[allow(dead_code)]
pub const LED_STATUS_PIN: u8 = 25; // Built-in LED on Pico
#[allow(dead_code)]
pub const LED_USB_PIN: u8 = 20; // USB status LED
#[allow(dead_code)]
pub const LED_ERROR_PIN: u8 = 21; // Error indication LED

// ===================================================================
// Hardware Configuration Options
// ===================================================================

#[allow(dead_code)]
pub const USE_BUTTON_MATRIX: bool = true; // Use matrix scan
pub const BUTTON_DEBOUNCE_MS: u64 = 20; // Button debounce time
pub const BUTTON_SCAN_RATE_HZ: u64 = 100; // Button scan frequency

// Display configuration
#[allow(dead_code)]
pub const DISPLAY_ROTATION: u8 = 3; // 270° rotation
pub const DISPLAY_BRIGHTNESS: u8 = 255; // Default brightness (0-255)
pub const DISPLAY_TOTAL_WIDTH: usize = STREAMDECK_COLS * KEY_IMAGE_SIZE; // 216 pixels
pub const DISPLAY_TOTAL_HEIGHT: usize = STREAMDECK_ROWS * KEY_IMAGE_SIZE; // 144 pixels

// USB Configuration
pub const USB_POLL_RATE_MS: u64 = 1; // 1ms USB polling (1000Hz)
pub const IMAGE_BUFFER_SIZE: usize = KEY_IMAGE_BYTES + 100; // Extra space for headers

// Performance options
#[allow(dead_code)]
pub const USE_DUAL_CORE: bool = true; // Use both RP2040 cores

// Development options

// ===================================================================
// USB HID Report IDs and Commands
// ===================================================================

// Report types
pub const OUTPUT_REPORT_IMAGE: u8 = 0x02;
pub const IMAGE_COMMAND_V2: u8 = 0x07;

// Feature report IDs
pub const FEATURE_REPORT_VERSION_V1: u8 = 0x04;
pub const FEATURE_REPORT_VERSION_V2: u8 = 0x05;
pub const FEATURE_REPORT_RESET_V1: u8 = 0x0B;
pub const FEATURE_REPORT_BRIGHTNESS_V1: u8 = 0x05;

// V2 commands (using report ID 0x03)
#[allow(dead_code)]
pub const FEATURE_REPORT_RESET_V2: u8 = 0x02;
#[allow(dead_code)]
pub const FEATURE_REPORT_BRIGHTNESS_V2: u8 = 0x08;