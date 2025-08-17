#pragma once

// ===================================================================
// OpenDeck Hardware Configuration
// RP2040-based StreamDeck Alternative
// ===================================================================

// USB Configuration - CRITICAL: Must match StreamDeck Mini for compatibility
#define USB_VID                 0x0fd9      // Elgato Systems VID
#define USB_PID                 0x0063      // StreamDeck Mini PID
#define USB_MANUFACTURER        "Elgato Systems"
#define USB_PRODUCT             "Stream Deck Mini"
#define USB_SERIAL              "OD240100001"  // OpenDeck serial format

// Device Specifications (StreamDeck Mini)
#define STREAMDECK_KEYS         6           // Number of keys (3x2 layout)
#define STREAMDECK_COLS         3           // Keys per row
#define STREAMDECK_ROWS         2           // Number of rows
#define KEY_IMAGE_SIZE          80          // 80x80 pixels per key
#define KEY_IMAGE_BYTES         (KEY_IMAGE_SIZE * KEY_IMAGE_SIZE * 3) // RGB

// USB HID Configuration
#define HID_REPORT_SIZE_INPUT   32          // Button state report size
#define HID_REPORT_SIZE_OUTPUT  1024        // Image data report size
#define HID_REPORT_SIZE_FEATURE 32          // Feature report size

// ===================================================================
// GPIO Pin Assignments - Raspberry Pi Pico
// ===================================================================

// Button Matrix (6 buttons arranged as 3x2)
// Use internal pull-ups, buttons connect to GND when pressed
#define BTN_ROW_PINS           {2, 3}       // GPIO 2, 3 for row scanning
#define BTN_COL_PINS           {4, 5, 6}    // GPIO 4, 5, 6 for column scanning

// Alternative: Direct button connections (if not using matrix)
#define BTN_DIRECT_PINS        {2, 3, 4, 5, 6, 7}  // One GPIO per button

// SPI Display Interface (for key LCD displays)
// Using SPI0 interface
#define SPI_PORT               spi0
#define SPI_MISO_PIN           16           // Not used for displays
#define SPI_MOSI_PIN           19           // Data to displays
#define SPI_SCK_PIN            18           // Clock to displays
#define SPI_BAUDRATE           10000000     // 10MHz SPI clock

// Display Control Pins (one set per display)
// CS (Chip Select) pins for individual display control
#define DISPLAY_CS_PINS        {8, 9, 10, 11, 12, 13}  // CS for displays 0-5

// Shared display control pins
#define DISPLAY_DC_PIN         14           // Data/Command select
#define DISPLAY_RST_PIN        15           // Reset (shared by all displays)
#define DISPLAY_BL_PIN         17           // Backlight control (PWM)

// Status LEDs (optional)
#define LED_STATUS_PIN         25           // Built-in LED on Pico
#define LED_USB_PIN            20           // USB status LED
#define LED_ERROR_PIN          21           // Error indication LED

// Debug UART (for development)
#define UART_TX_PIN            0            // UART0 TX (to computer)
#define UART_RX_PIN            1            // UART0 RX (from computer)

// ===================================================================
// Hardware Configuration Options
// ===================================================================

// Button scanning method
#define USE_BUTTON_MATRIX      1            // 1 = matrix scan, 0 = direct pins
#define BUTTON_DEBOUNCE_MS     20           // Button debounce time
#define BUTTON_SCAN_RATE_HZ    100          // Button scan frequency

// Display configuration
#define DISPLAY_TYPE_ST7735    1            // Using ST7735 TFT displays
#define DISPLAY_ROTATION       3            // 270° rotation for correct orientation
#define DISPLAY_BRIGHTNESS     255          // Default brightness (0-255)

// USB Configuration
#define USB_POLL_RATE_MS       1            // 1ms USB polling (1000Hz)
#define IMAGE_BUFFER_SIZE      (KEY_IMAGE_BYTES + 100) // Extra space for headers

// Performance options
#define USE_DUAL_CORE          1            // Use both RP2040 cores
#define CORE0_TASKS            "USB, Protocol" // Core 0 handles USB
#define CORE1_TASKS            "Display, Buttons" // Core 1 handles I/O

// Development options
#define DEBUG_UART_ENABLED     1            // Enable debug output
#define DEBUG_LEVEL            1            // 0=None, 1=Info, 2=Verbose
#define WATCHDOG_ENABLED       1            // Enable hardware watchdog

// ===================================================================
// Pin Layout Diagram (Raspberry Pi Pico)
// ===================================================================
/*
                    RP2040 (Raspberry Pi Pico)
                        USB  [===]
                             [   ]
    UART TX  GP0  [ 1]      [40] VBUS
    UART RX  GP1  [ 2]      [39] VSYS  
             GND  [ 3]      [38] GND
    BTN ROW0 GP2  [ 4]      [37] 3V3_EN
    BTN ROW1 GP3  [ 5]      [36] 3V3
    BTN COL0 GP4  [ 6]      [35] ADC_VREF
    BTN COL1 GP5  [ 7]      [34] GP28
    BTN COL2 GP6  [ 8]      [33] GND
             GND  [ 9]      [32] GP27
    DISP CS0 GP8  [10]      [31] GP26
    DISP CS1 GP9  [11]      [30] RUN
    DISP CS2 GP10 [12]      [29] GP22
    DISP CS3 GP11 [13]      [28] GND
    DISP CS4 GP12 [14]      [27] GP21  LED_ERROR
    DISP CS5 GP13 [15]      [26] GP20  LED_USB
             GND  [16]      [25] GP19  SPI_MOSI
    DISP DC  GP14 [17]      [24] GP18  SPI_SCK
    DISP RST GP15 [18]      [23] GND
    SPI MISO GP16 [19]      [22] GP17  DISP_BL (PWM)
    DISP BL  GP17 [20]      [21] GP16  SPI_MISO (unused)

    Additional connections:
    - LED_STATUS: GP25 (built-in LED)
    - Each button connects between ROW pin and COL pin
    - All displays share DC, RST, MOSI, SCK
    - Each display has individual CS pin
    
    Button Matrix Layout:
    ROW0: BTN0(GP2-GP4) BTN1(GP2-GP5) BTN2(GP2-GP6)
    ROW1: BTN3(GP3-GP4) BTN4(GP3-GP5) BTN5(GP3-GP6)
*/

// ===================================================================
// Validation and Sanity Checks
// ===================================================================

#if STREAMDECK_KEYS != 6
#error "This configuration is specifically for StreamDeck Mini (6 keys)"
#endif

#if USB_VID != 0x0fd9 || USB_PID != 0x0063
#error "USB VID/PID must match StreamDeck Mini for software compatibility"
#endif

#if KEY_IMAGE_SIZE != 80
#error "StreamDeck Mini requires 80x80 pixel images"
#endif