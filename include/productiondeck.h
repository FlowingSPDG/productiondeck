#pragma once

#include <stdint.h>
#include <stdbool.h>
#include <string.h>
#include "pico/stdlib.h"
#include "pico/multicore.h"
#include "hardware/gpio.h"
#include "hardware/spi.h"
#include "hardware/pwm.h"
#include "hardware/timer.h"
#include "hardware/watchdog.h"
#include "tusb.h"

#include "productiondeck_config.h"
#include "usb_descriptors.h"

// ===================================================================
// ProductionDeck Main Application Class
// ===================================================================

class ProductionDeck {
public:
    // Constructor and basic lifecycle
    ProductionDeck();
    ~ProductionDeck();
    
    // Main application control
    bool initialize();
    void run();
    void shutdown();
    
    // Hardware interface
    bool init_hardware();
    bool init_usb();
    bool init_displays();
    bool init_buttons();
    
    // Button handling
    void scan_buttons();
    void update_button_state(uint8_t key, bool pressed);
    bool get_button_state(uint8_t key);
    void send_button_report();
    
    // Image handling
    void receive_image_packet(const uint8_t* data, uint16_t length);
    void process_complete_image(uint8_t key_id);
    void display_image(uint8_t key_id, const uint8_t* image_data, uint16_t width, uint16_t height);
    void clear_key(uint8_t key_id);
    void clear_all_keys();
    
    // Device control
    void set_brightness(uint8_t brightness);
    void reset_device();
    const char* get_firmware_version();
    
    // Status and diagnostics
    bool is_usb_connected();
    bool is_ready();
    uint32_t get_uptime_ms();
    void blink_status_led(uint16_t on_ms, uint16_t off_ms);

private:
    // Internal state
    struct {
        bool initialized;
        bool usb_connected;
        bool displays_ready;
        uint32_t startup_time;
        uint8_t current_brightness;
        uint32_t last_button_scan;
        uint32_t last_status_update;
    } state_;
    
    // Button state tracking
    struct {
        uint8_t current[STREAMDECK_KEYS];
        uint8_t previous[STREAMDECK_KEYS];
        uint32_t last_change[STREAMDECK_KEYS];
        bool changed;
    } buttons_;
    
    // Image reception buffers
    struct ImageBuffer {
        uint8_t data[IMAGE_BUFFER_SIZE];
        uint16_t bytes_received;
        uint16_t expected_sequence;
        bool receiving;
        bool complete;
        uint32_t last_packet_time;
    } image_buffers_[STREAMDECK_KEYS];
    
    // Hardware abstraction
    void setup_gpio();
    void setup_spi();
    void setup_pwm();
    
    // Button matrix scanning
    void scan_button_matrix();
    void scan_direct_buttons();
    bool debounce_button(uint8_t key, bool raw_state);
    
    // Display management (single shared display)
    void init_shared_display();
    void send_display_command(uint8_t command);
    void send_display_data(const uint8_t* data, uint16_t length);
    void set_display_brightness(uint8_t brightness);
    
    // Image processing
    bool validate_image_packet(const hid_output_report_t* packet);
    void reset_image_buffer(uint8_t key_id);
    void convert_image_format(const uint8_t* input, uint8_t* output, uint16_t size);
    
    // USB protocol handlers
    void handle_version_request(uint8_t report_id, uint8_t* response, uint16_t max_len);
    void handle_reset_command(uint8_t report_id, const uint8_t* data);
    void handle_brightness_command(uint8_t report_id, const uint8_t* data);
    
    // Status and error handling
    void set_error_state(const char* error);
    void update_status_leds();
    void watchdog_update();
    
    // Multi-core support
    static void core1_entry();
    void core1_main();
    volatile bool core1_running_;
    
    // Internal utilities
    uint32_t millis();
    void delay_ms(uint32_t ms);
    void log_debug(const char* format, ...);
    void log_info(const char* format, ...);
    void log_error(const char* format, ...);
};

// ===================================================================
// Hardware Abstraction Layer
// ===================================================================

class HardwareInterface {
public:
    // GPIO operations
    static void gpio_init_output(uint gpio, bool initial_state = false);
    static void gpio_init_input(uint gpio, bool pullup = true);
    static void gpio_set(uint gpio, bool state);
    static bool gpio_get(uint gpio);
    
    // SPI operations
    static bool spi_init(spi_inst_t* spi, uint baudrate);
    static void spi_write(spi_inst_t* spi, const uint8_t* data, size_t len);
    static void spi_select_device(uint cs_pin, bool select);
    
    // PWM operations
    static bool pwm_init(uint gpio, uint16_t wrap, uint16_t level);
    static void pwm_set_level(uint gpio, uint16_t level);
    
    // Timing
    static uint32_t time_ms();
    static void sleep_ms(uint32_t ms);
    static void sleep_us(uint32_t us);
};

// ===================================================================
// Display Driver Interface
// ===================================================================

class DisplayDriver {
public:
    DisplayDriver(uint8_t cs_pin, uint8_t dc_pin, uint8_t rst_pin);
    
    bool initialize();
    void reset();
    void set_brightness(uint8_t brightness);
    void clear();
    void display_image(const uint8_t* image_data, uint16_t width, uint16_t height);
    void display_color(uint16_t color);
    void set_rotation(uint8_t rotation);
    
private:
    uint8_t cs_pin_;
    uint8_t dc_pin_;
    uint8_t rst_pin_;
    bool initialized_;
    
    void select();
    void deselect();
    void send_command(uint8_t cmd);
    void send_data(const uint8_t* data, uint16_t len);
    void send_byte(uint8_t data);
    void init_sequence();
};

// ===================================================================
// Global Functions and Utilities
// ===================================================================

// Main application instance
extern ProductionDeck* g_productiondeck;

// Callback functions for USB stack
extern "C" {
    // TinyUSB callbacks
    void tud_mount_cb(void);
    void tud_umount_cb(void);
    void tud_suspend_cb(bool remote_wakeup_en);
    void tud_resume_cb(void);
    
    // HID callbacks
    uint16_t tud_hid_get_report_cb(uint8_t itf, uint8_t report_id, 
                                   hid_report_type_t report_type, 
                                   uint8_t* buffer, uint16_t reqlen);
    void tud_hid_set_report_cb(uint8_t itf, uint8_t report_id, 
                               hid_report_type_t report_type, 
                               uint8_t const* buffer, uint16_t bufsize);
}

// Utility macros
#define ARRAY_SIZE(a)           (sizeof(a) / sizeof(a[0]))
// MIN and MAX are already defined by Pico SDK
#define CLAMP(x, min, max)     (MIN(MAX(x, min), max))

// Debug macros
#if DEBUG_UART_ENABLED
    #define DEBUG_PRINT(...)    printf(__VA_ARGS__)
    #define DEBUG_PRINTLN(...)  printf(__VA_ARGS__); printf("\n")
#else
    #define DEBUG_PRINT(...)    
    #define DEBUG_PRINTLN(...)  
#endif

// Error handling
#define ASSERT(condition, message) \
    do { \
        if (!(condition)) { \
            DEBUG_PRINTLN("ASSERTION FAILED: %s at %s:%d", message, __FILE__, __LINE__); \
            while(1) { tight_loop_contents(); } \
        } \
    } while(0)

#define CHECK_INIT(component) \
    ASSERT(state_.initialized, "ProductionDeck not initialized"); \
    ASSERT(component, #component " not ready")
