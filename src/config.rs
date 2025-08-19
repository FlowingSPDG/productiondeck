//! Hardware configuration for ProductionDeck
//! RP2040-based StreamDeck Mini compatible device

// ===================================================================
// USB Configuration - CRITICAL: Must match StreamDeck Mini
// ===================================================================

pub const USB_VID: u16 = 0x0fd9; // Elgato Systems VID
pub const USB_PID: u16 = 0x0063; // StreamDeck Mini PID
pub const USB_MANUFACTURER: &str = "Elgato Systems";
pub const USB_PRODUCT: &str = "Stream Deck Mini";
// Serial is provided at compile-time from build.rs as an env var; fallback to a static value if missing.
// Use a fixed 12-character uppercase alphanumeric serial to match real device length
pub const USB_SERIAL: &str = "PRODUCTIONDK"; // 12 chars

// USB version settings to match real StreamDeck Mini
pub const USB_BCD_DEVICE: u16 = 0x0200; // Device version 2.0 (matches real device)

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

pub const HID_REPORT_SIZE_INPUT: usize = 16; // Button state report size (actual size used)
pub const HID_REPORT_SIZE_FEATURE: usize = 32; // Feature report size

// ===================================================================
// GPIO Pin Assignments - Raspberry Pi Pico
// ===================================================================

// Button Matrix (6 buttons arranged as 3x2) - Currently used in main.rs
pub const BTN_ROW_PINS: [u8; STREAMDECK_ROWS] = [2, 3]; // GPIO 2, 3
pub const BTN_COL_PINS: [u8; STREAMDECK_COLS] = [4, 5, 6]; // GPIO 4, 5, 6

// SPI Display Interface - Currently disabled in main.rs
pub const SPI_MOSI_PIN: u8 = 19; // Data to display
pub const SPI_SCK_PIN: u8 = 18; // Clock to display
pub const SPI_BAUDRATE: u32 = 10_000_000; // 10MHz SPI clock

// Single Display Control Pins - Currently disabled in main.rs
pub const DISPLAY_CS_PIN: u8 = 8; // Chip select
pub const DISPLAY_DC_PIN: u8 = 14; // Data/Command select
pub const DISPLAY_RST_PIN: u8 = 15; // Reset
pub const DISPLAY_BL_PIN: u8 = 17; // Backlight control (PWM)

// Status LEDs - Currently used in main.rs
pub const LED_STATUS_PIN: u8 = 25; // Built-in LED on Pico
pub const LED_USB_PIN: u8 = 20; // USB status LED
pub const LED_ERROR_PIN: u8 = 21; // Error indication LED

// ===================================================================
// Hardware Configuration Options
// ===================================================================

pub const BUTTON_DEBOUNCE_MS: u64 = 20; // Button debounce time
pub const BUTTON_SCAN_RATE_HZ: u64 = 100; // Button scan frequency

// Display configuration
pub const DISPLAY_BRIGHTNESS: u8 = 255; // Default brightness (0-255)
pub const DISPLAY_TOTAL_WIDTH: usize = STREAMDECK_COLS * KEY_IMAGE_SIZE; // 216 pixels
pub const DISPLAY_TOTAL_HEIGHT: usize = STREAMDECK_ROWS * KEY_IMAGE_SIZE; // 144 pixels

// USB Configuration
pub const USB_POLL_RATE_MS: u64 = 1; // 1ms USB polling (1000Hz)
pub const IMAGE_BUFFER_SIZE: usize = 1024; // 1KB buffer size (reduced to prevent HardFault)

// Development options

// ===================================================================
// USB HID Report IDs and Commands
// ===================================================================

// Report types
pub const OUTPUT_REPORT_IMAGE: u8 = 0x02;
pub const IMAGE_COMMAND_V2: u8 = 0x07;

// Feature report IDs and commands
pub const FEATURE_REPORT_VERSION_V1: u8 = 0x04;
pub const FEATURE_REPORT_VERSION_V2: u8 = 0x05;
pub const FEATURE_REPORT_SERIAL_NUMBER: u8 = 0x03;
pub const FEATURE_REPORT_FIRMWARE_INFO: u8 = 0xA1;
pub const FEATURE_REPORT_RESET_V1: u8 = 0x0B;
pub const FEATURE_REPORT_BRIGHTNESS_V1: u8 = 0x05;
pub const FEATURE_REPORT_V2_COMMANDS: u8 = 0x03; // V2 command container

// V2 sub-commands (used with FEATURE_REPORT_V2_COMMANDS)
pub const V2_COMMAND_RESET: u8 = 0x02;
pub const V2_COMMAND_BRIGHTNESS: u8 = 0x08;

// StreamDeck protocol magic bytes
pub const STREAMDECK_MAGIC_1: u8 = 0x55;
pub const STREAMDECK_MAGIC_2: u8 = 0xAA;
pub const STREAMDECK_MAGIC_3: u8 = 0xD1;
pub const STREAMDECK_RESET_MAGIC: u8 = 0x63;
pub const STREAMDECK_BRIGHTNESS_RESET_MAGIC: u8 = 0x3E;

// ===================================================================
// ST7735 Display Commands
// ===================================================================

pub const ST7735_SWRESET: u8 = 0x01; // Software reset
pub const ST7735_SLPOUT: u8 = 0x11;  // Sleep out
pub const ST7735_COLMOD: u8 = 0x3A;  // Color mode
pub const ST7735_CASET: u8 = 0x2A;   // Column address set
pub const ST7735_RASET: u8 = 0x2B;   // Row address set
pub const ST7735_INVOFF: u8 = 0x20;  // Display inversion off
pub const ST7735_NORON: u8 = 0x13;   // Normal display mode
pub const ST7735_DISPON: u8 = 0x29;  // Display on
pub const ST7735_RAMWR: u8 = 0x2C;   // Memory write

// ST7735 Color format constants
pub const ST7735_COLOR_MODE_16BIT: u8 = 0x05; // RGB565 format

// RGB565 conversion masks
pub const RGB565_RED_MASK: u16 = 0xF8;
pub const RGB565_GREEN_MASK: u16 = 0xFC;
pub const RGB565_BLUE_SHIFT: u8 = 3;