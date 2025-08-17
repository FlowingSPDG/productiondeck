// ===================================================================
// ProductionDeck Core Implementation
// 
// This file implements the main ProductionDeck class that coordinates all
// device functionality including USB protocol, button scanning,
// image reception, and display management.
// ===================================================================

#include "productiondeck.h"
#include <cstdio>
#include <cstdarg>

// ===================================================================
// Constructor and Destructor
// ===================================================================

ProductionDeck::ProductionDeck() : core1_running_(false) {
    // Initialize state
    memset(&state_, 0, sizeof(state_));
    memset(&buttons_, 0, sizeof(buttons_));
    memset(image_buffers_, 0, sizeof(image_buffers_));
    
    state_.current_brightness = DISPLAY_BRIGHTNESS;
    state_.startup_time = time_us_32() / 1000;
}

ProductionDeck::~ProductionDeck() {
    shutdown();
}

// ===================================================================
// Main Lifecycle Methods
// ===================================================================

bool ProductionDeck::initialize() {
    log_info("ProductionDeck: Starting initialization...");
    
    // Initialize hardware first
    if (!init_hardware()) {
        log_error("Hardware initialization failed");
        return false;
    }
    
    // Initialize USB stack
    if (!init_usb()) {
        log_error("USB initialization failed");
        return false;
    }
    
    // Initialize displays
    if (!init_displays()) {
        log_error("Display initialization failed");
        return false;
    }
    
    // Initialize buttons
    if (!init_buttons()) {
        log_error("Button initialization failed");
        return false;
    }
    
    // Start second core for I/O processing
    #if USE_DUAL_CORE
    multicore_launch_core1(core1_entry);
    core1_running_ = true;
    log_info("Core1 launched for I/O processing");
    #endif
    
    // Clear all displays and show startup pattern
    clear_all_keys();
    blink_status_led(100, 100);
    
    state_.initialized = true;
    log_info("ProductionDeck initialization complete");
    
    return true;
}

void ProductionDeck::run() {
    if (!state_.initialized) {
        return;
    }
    
    // Update USB stack
    tud_task();
    
    // Update connection status
    bool was_connected = state_.usb_connected;
    state_.usb_connected = tud_mounted();
    
    if (state_.usb_connected && !was_connected) {
        log_info("USB connected - device ready");
    } else if (!state_.usb_connected && was_connected) {
        log_info("USB disconnected");
    }
    
    // Process button states and send reports (Core 0)
    uint32_t now = millis();
    if (now - state_.last_button_scan >= (1000 / BUTTON_SCAN_RATE_HZ)) {
        scan_buttons();
        if (buttons_.changed && state_.usb_connected) {
            send_button_report();
        }
        state_.last_button_scan = now;
    }
    
    // Update status indicators
    if (now - state_.last_status_update >= 100) {
        update_status_leds();
        state_.last_status_update = now;
    }
    
    // Update watchdog
    watchdog_update();
}

void ProductionDeck::shutdown() {
    if (!state_.initialized) {
        return;
    }
    
    log_info("ProductionDeck: Shutting down...");
    
    // Stop second core
    #if USE_DUAL_CORE
    if (core1_running_) {
        // Signal core1 to stop (implementation dependent)
        core1_running_ = false;
        sleep_ms(100); // Give time to stop
    }
    #endif
    
    // Clear displays
    clear_all_keys();
    set_brightness(0);
    
    // Reset hardware state
    for (int i = 0; i < STREAMDECK_KEYS; i++) {
        reset_image_buffer(i);
    }
    
    state_.initialized = false;
    log_info("ProductionDeck shutdown complete");
}

// ===================================================================
// Hardware Initialization
// ===================================================================

bool ProductionDeck::init_hardware() {
    log_info("Initializing hardware...");
    
    // Set up GPIO pins
    setup_gpio();
    
    // Set up SPI for displays
    setup_spi();
    
    // Set up PWM for brightness control
    setup_pwm();
    
    log_info("Hardware initialization complete");
    return true;
}

bool ProductionDeck::init_usb() {
    log_info("Initializing USB...");
    
    // Initialize TinyUSB
    tusb_init();
    
    log_info("USB stack initialized");
    return true;
}

bool ProductionDeck::init_displays() {
    log_info("Initializing displays...");
    
    // Initialize each display
    uint8_t cs_pins[] = DISPLAY_CS_PINS;
    
    for (int i = 0; i < STREAMDECK_KEYS; i++) {
        init_display(i);
        sleep_ms(10); // Small delay between initializations
    }
    
    state_.displays_ready = true;
    log_info("All displays initialized");
    return true;
}



bool ProductionDeck::init_buttons() {
    log_info("Initializing buttons...");
    
    #if USE_BUTTON_MATRIX
    // Set up button matrix scanning
    uint8_t row_pins[] = BTN_ROW_PINS;
    uint8_t col_pins[] = BTN_COL_PINS;
    
    // Initialize row pins as outputs (initially high)
    for (int i = 0; i < STREAMDECK_ROWS; i++) {
        HardwareInterface::gpio_init_output(row_pins[i], true);
    }
    
    // Initialize column pins as inputs with pull-ups
    for (int i = 0; i < STREAMDECK_COLS; i++) {
        HardwareInterface::gpio_init_input(col_pins[i], true);
    }
    #else
    // Set up direct button connections
    uint8_t btn_pins[] = BTN_DIRECT_PINS;
    for (int i = 0; i < STREAMDECK_KEYS; i++) {
        HardwareInterface::gpio_init_input(btn_pins[i], true);
    }
    #endif
    
    log_info("Button matrix initialized");
    return true;
}

// ===================================================================
// GPIO and Hardware Setup
// ===================================================================

void ProductionDeck::setup_gpio() {
    // Status LEDs
    HardwareInterface::gpio_init_output(LED_STATUS_PIN, false);
    HardwareInterface::gpio_init_output(LED_USB_PIN, false);
    HardwareInterface::gpio_init_output(LED_ERROR_PIN, false);
    
    // Display control pins
    HardwareInterface::gpio_init_output(DISPLAY_DC_PIN, false);
    HardwareInterface::gpio_init_output(DISPLAY_RST_PIN, true);
    
    // CS pins for each display
    uint8_t cs_pins[] = DISPLAY_CS_PINS;
    for (int i = 0; i < STREAMDECK_KEYS; i++) {
        HardwareInterface::gpio_init_output(cs_pins[i], true); // CS high = deselected
    }
}

void ProductionDeck::setup_spi() {
    // Initialize SPI0 for display communication
    spi_init(SPI_PORT, SPI_BAUDRATE);
    
    // Set up SPI pins
    gpio_set_function(SPI_MOSI_PIN, GPIO_FUNC_SPI);
    gpio_set_function(SPI_SCK_PIN, GPIO_FUNC_SPI);
    // MISO not needed for displays
}

void ProductionDeck::setup_pwm() {
    // Set up PWM for display backlight control
    gpio_set_function(DISPLAY_BL_PIN, GPIO_FUNC_PWM);
    uint slice_num = pwm_gpio_to_slice_num(DISPLAY_BL_PIN);
    pwm_set_wrap(slice_num, 255);
    pwm_set_chan_level(slice_num, PWM_CHAN_A, state_.current_brightness);
    pwm_set_enabled(slice_num, true);
}

// ===================================================================
// Button Scanning and Processing
// ===================================================================

void ProductionDeck::scan_buttons() {
    #if USE_BUTTON_MATRIX
    scan_button_matrix();
    #else
    scan_direct_buttons();
    #endif
}

void ProductionDeck::scan_button_matrix() {
    uint8_t row_pins[] = BTN_ROW_PINS;
    uint8_t col_pins[] = BTN_COL_PINS;
    
    buttons_.changed = false;
    
    for (int row = 0; row < STREAMDECK_ROWS; row++) {
        // Pull current row low
        HardwareInterface::gpio_set(row_pins[row], false);
        sleep_us(10); // Small settling time
        
        for (int col = 0; col < STREAMDECK_COLS; col++) {
            uint8_t key_index = row * STREAMDECK_COLS + col;
            
            // Read column pin (low = button pressed)
            bool raw_pressed = !HardwareInterface::gpio_get(col_pins[col]);
            bool pressed = debounce_button(key_index, raw_pressed);
            
            if (pressed != buttons_.current[key_index]) {
                buttons_.previous[key_index] = buttons_.current[key_index];
                buttons_.current[key_index] = pressed;
                buttons_.last_change[key_index] = millis();
                buttons_.changed = true;
                
                log_debug("Button %d %s", key_index, pressed ? "pressed" : "released");
            }
        }
        
        // Return row to high
        HardwareInterface::gpio_set(row_pins[row], true);
    }
}

void ProductionDeck::scan_direct_buttons() {
    uint8_t btn_pins[] = BTN_DIRECT_PINS;
    
    buttons_.changed = false;
    
    for (int i = 0; i < STREAMDECK_KEYS; i++) {
        // Read button pin (low = pressed due to pull-up)
        bool raw_pressed = !HardwareInterface::gpio_get(btn_pins[i]);
        bool pressed = debounce_button(i, raw_pressed);
        
        if (pressed != buttons_.current[i]) {
            buttons_.previous[i] = buttons_.current[i];
            buttons_.current[i] = pressed;
            buttons_.last_change[i] = millis();
            buttons_.changed = true;
            
            log_debug("Button %d %s", i, pressed ? "pressed" : "released");
        }
    }
}

bool ProductionDeck::debounce_button(uint8_t key, bool raw_state) {
    static bool debounce_state[STREAMDECK_KEYS] = {false};
    static uint32_t debounce_time[STREAMDECK_KEYS] = {0};
    
    uint32_t now = millis();
    
    if (raw_state != debounce_state[key]) {
        debounce_state[key] = raw_state;
        debounce_time[key] = now;
    }
    
    if ((now - debounce_time[key]) > BUTTON_DEBOUNCE_MS) {
        return debounce_state[key];
    }
    
    return buttons_.current[key]; // Return previous stable state
}

void ProductionDeck::send_button_report() {
    if (!usb_hid_ready()) {
        return;
    }
    
    bool success = usb_send_button_report(buttons_.current);
    if (success) {
        buttons_.changed = false;
        log_debug("Button report sent");
    }
}

// ===================================================================
// Image Reception and Processing
// ===================================================================

void ProductionDeck::receive_image_packet(const uint8_t* data, uint16_t length) {
    if (length < 8) {
        log_error("Invalid image packet length: %d", length);
        return;
    }
    
    // Parse V2 header: [0x02, 0x07, key_id, is_last, len_low, len_high, seq_low, seq_high, data...]
    uint8_t key_id = data[2];
    uint8_t is_last = data[3];
    uint16_t payload_len = data[4] | (data[5] << 8);
    uint16_t sequence = data[6] | (data[7] << 8);
    
    if (key_id >= STREAMDECK_KEYS) {
        log_error("Invalid key ID: %d", key_id);
        return;
    }
    
    ImageBuffer* buf = &image_buffers_[key_id];
    
    // Reset buffer on first packet
    if (sequence == 0) {
        reset_image_buffer(key_id);
        buf->receiving = true;
        log_debug("Starting image reception for key %d", key_id);
    }
    
    // Validate sequence
    if (!buf->receiving || sequence != buf->expected_sequence) {
        log_error("Image packet sequence error: expected %d, got %d", 
                  buf->expected_sequence, sequence);
        reset_image_buffer(key_id);
        return;
    }
    
    // Copy payload data
    uint16_t data_offset = 8;
    uint16_t copy_len = MIN(payload_len, length - data_offset);
    uint16_t available_space = IMAGE_BUFFER_SIZE - buf->bytes_received;
    
    if (copy_len > available_space) {
        log_error("Image buffer overflow for key %d", key_id);
        reset_image_buffer(key_id);
        return;
    }
    
    memcpy(&buf->data[buf->bytes_received], &data[data_offset], copy_len);
    buf->bytes_received += copy_len;
    buf->expected_sequence++;
    buf->last_packet_time = millis();
    
    log_debug("Image packet key=%d seq=%d len=%d total=%d", 
              key_id, sequence, copy_len, buf->bytes_received);
    
    // Process complete image
    if (is_last) {
        buf->complete = true;
        buf->receiving = false;
        log_info("Image complete for key %d (%d bytes)", key_id, buf->bytes_received);
        process_complete_image(key_id);
    }
}

void ProductionDeck::process_complete_image(uint8_t key_id) {
    if (key_id >= STREAMDECK_KEYS) return;
    
    ImageBuffer* buf = &image_buffers_[key_id];
    if (!buf->complete) return;
    
    log_info("Processing image for key %d", key_id);
    
    // For StreamDeck Mini, images are BMP format after potential header
    // Skip BMP header if present (54 bytes)
    uint8_t* image_data = buf->data;
    uint16_t image_size = buf->bytes_received;
    
    // Check for BMP header (starts with "BM")
    if (image_size > 54 && image_data[0] == 0x42 && image_data[1] == 0x4D) {
        image_data += 54;
        image_size -= 54;
        log_debug("Skipped BMP header for key %d", key_id);
    }
    
    // Display the image
    display_image(key_id, image_data, KEY_IMAGE_SIZE, KEY_IMAGE_SIZE);
    
    // Reset buffer for next image
    reset_image_buffer(key_id);
}

void ProductionDeck::reset_image_buffer(uint8_t key_id) {
    if (key_id >= STREAMDECK_KEYS) return;
    
    ImageBuffer* buf = &image_buffers_[key_id];
    memset(buf, 0, sizeof(ImageBuffer));
}

// ===================================================================
// Display Management
// ===================================================================

void ProductionDeck::init_display(uint8_t display_id) {
    // Validate display ID
    if (display_id >= STREAMDECK_KEYS) {
        log_error("Invalid display ID: %d", display_id);
        return;
    }
    
    log_info("Initializing display %d", display_id);
    
    // Get CS pin for this display
    uint8_t cs_pins[] = DISPLAY_CS_PINS;
    uint8_t cs_pin = cs_pins[display_id];
    
    // Select this display's CS pin
    HardwareInterface::spi_select_device(cs_pin, true);
    
    // Reset the display using shared RST pin
    log_debug("Resetting display %d", display_id);
    HardwareInterface::gpio_set(DISPLAY_RST_PIN, false);
    HardwareInterface::sleep_ms(10);
    HardwareInterface::gpio_set(DISPLAY_RST_PIN, true);
    HardwareInterface::sleep_ms(120); // Wait for display to boot
    
    // Send ST7735 initialization sequence
    log_debug("Sending initialization sequence to display %d", display_id);
    
    // Software reset
    send_display_command(0x01);
    HardwareInterface::sleep_ms(150);
    
    // Sleep out
    send_display_command(0x11);
    HardwareInterface::sleep_ms(120);
    
    // Frame rate control
    send_display_command(0xB1);
    uint8_t frc[] = {0x01, 0x2C, 0x2D};
    send_display_data(frc, sizeof(frc));
    
    send_display_command(0xB2);
    send_display_data(frc, sizeof(frc));
    
    send_display_command(0xB3);
    uint8_t frc2[] = {0x01, 0x2C, 0x2D, 0x01, 0x2C, 0x2D};
    send_display_data(frc2, sizeof(frc2));
    
    // Column inversion
    send_display_command(0xB4);
    uint8_t inv = 0x07;
    send_display_data(&inv, 1);
    
    // Power control
    send_display_command(0xC0);
    uint8_t pwr1[] = {0xA2, 0x02, 0x84};
    send_display_data(pwr1, sizeof(pwr1));
    
    send_display_command(0xC1);
    uint8_t pwr2 = 0xC5;
    send_display_data(&pwr2, 1);
    
    send_display_command(0xC2);
    uint8_t pwr3[] = {0x0A, 0x00};
    send_display_data(pwr3, sizeof(pwr3));
    
    send_display_command(0xC3);
    uint8_t pwr4[] = {0x8A, 0x2A};
    send_display_data(pwr4, sizeof(pwr4));
    
    send_display_command(0xC4);
    uint8_t pwr5[] = {0x8A, 0xEE};
    send_display_data(pwr5, sizeof(pwr5));
    
    // VCOM control
    send_display_command(0xC5);
    uint8_t vcom = 0x0E;
    send_display_data(&vcom, 1);
    
    // Memory access control (270° rotation for StreamDeck Mini)
    send_display_command(0x36);
    uint8_t madctl = 0xC8; // RGB, row/col addr order, 270° rotation
    send_display_data(&madctl, 1);
    
    // Color mode - 16 bit RGB565
    send_display_command(0x3A);
    uint8_t colmod = 0x05;
    send_display_data(&colmod, 1);
    
    // Column address set (0-79)
    send_display_command(0x2A);
    uint8_t caset[] = {0x00, 0x00, 0x00, 0x4F};
    send_display_data(caset, sizeof(caset));
    
    // Row address set (0-79)  
    send_display_command(0x2B);
    uint8_t raset[] = {0x00, 0x00, 0x00, 0x4F};
    send_display_data(raset, sizeof(raset));
    
    // Gamma correction
    send_display_command(0xE0);
    uint8_t gmctrp[] = {0x02, 0x1C, 0x07, 0x12, 0x37, 0x32, 0x29, 0x2D,
                        0x29, 0x25, 0x2B, 0x39, 0x00, 0x01, 0x03, 0x10};
    send_display_data(gmctrp, sizeof(gmctrp));
    
    send_display_command(0xE1);
    uint8_t gmctrn[] = {0x03, 0x1D, 0x07, 0x06, 0x2E, 0x2C, 0x29, 0x2D,
                        0x2E, 0x2E, 0x37, 0x3F, 0x00, 0x00, 0x02, 0x10};
    send_display_data(gmctrn, sizeof(gmctrn));
    
    // Display on
    send_display_command(0x29);
    HardwareInterface::sleep_ms(10);
    
    // Deselect this display
    HardwareInterface::spi_select_device(cs_pin, false);
    
    log_info("Display %d initialization complete", display_id);
}

void ProductionDeck::select_display(uint8_t display_id) {
    if (display_id >= STREAMDECK_KEYS) {
        return;
    }
    
    // Deselect all displays first
    uint8_t cs_pins[] = DISPLAY_CS_PINS;
    for (int i = 0; i < STREAMDECK_KEYS; i++) {
        HardwareInterface::spi_select_device(cs_pins[i], false);
    }
    
    // Select the specified display
    HardwareInterface::spi_select_device(cs_pins[display_id], true);
}

void ProductionDeck::send_display_command(uint8_t command) {
    // Set DC pin low for command mode
    HardwareInterface::gpio_set(DISPLAY_DC_PIN, false);
    
    // Send command byte
    HardwareInterface::spi_write(SPI_PORT, &command, 1);
}

void ProductionDeck::send_display_data(const uint8_t* data, uint16_t length) {
    // Set DC pin high for data mode
    HardwareInterface::gpio_set(DISPLAY_DC_PIN, true);
    
    // Send data
    HardwareInterface::spi_write(SPI_PORT, data, length);
}

void ProductionDeck::display_image(uint8_t key_id, const uint8_t* image_data, 
                            uint16_t width, uint16_t height) {
    if (key_id >= STREAMDECK_KEYS || !state_.displays_ready || !image_data) {
        return;
    }
    
    log_debug("Displaying image on key %d (%dx%d)", key_id, width, height);
    
    // Select the display
    select_display(key_id);
    
    // Set window to full screen (80x80)
    send_display_command(0x2A); // Column address set
    uint8_t caset[] = {0x00, 0x00, 0x00, 0x4F};
    send_display_data(caset, sizeof(caset));
    
    send_display_command(0x2B); // Row address set
    uint8_t raset[] = {0x00, 0x00, 0x00, 0x4F};
    send_display_data(raset, sizeof(raset));
    
    send_display_command(0x2C); // Memory write
    
    // Convert RGB888 to RGB565 and send to display
    uint16_t pixel_count = width * height;
    
    for (uint16_t i = 0; i < pixel_count; i++) {
        uint8_t r = image_data[i * 3 + 0];
        uint8_t g = image_data[i * 3 + 1];
        uint8_t b = image_data[i * 3 + 2];
        
        // Convert to RGB565
        uint16_t rgb565 = ((r & 0xF8) << 8) | ((g & 0xFC) << 3) | (b >> 3);
        
        // Send as big-endian
        uint8_t pixel_data[] = {(uint8_t)(rgb565 >> 8), (uint8_t)(rgb565 & 0xFF)};
        send_display_data(pixel_data, 2);
    }
    
    // Deselect display
    uint8_t cs_pins[] = DISPLAY_CS_PINS;
    HardwareInterface::spi_select_device(cs_pins[key_id], false);
    
    log_info("Image displayed on key %d: %d pixels", key_id, pixel_count);
}

void ProductionDeck::clear_key(uint8_t key_id) {
    if (key_id >= STREAMDECK_KEYS || !state_.displays_ready) return;
    
    log_debug("Clearing key %d", key_id);
    
    // Select the display
    select_display(key_id);
    
    // Set window to full screen (80x80)
    send_display_command(0x2A); // Column address set
    uint8_t caset[] = {0x00, 0x00, 0x00, 0x4F};
    send_display_data(caset, sizeof(caset));
    
    send_display_command(0x2B); // Row address set
    uint8_t raset[] = {0x00, 0x00, 0x00, 0x4F};
    send_display_data(raset, sizeof(raset));
    
    send_display_command(0x2C); // Memory write
    
    // Fill entire screen with black (RGB565: 0x0000)
    uint8_t black_pixel[] = {0x00, 0x00};
    for (int i = 0; i < 80 * 80; i++) {
        send_display_data(black_pixel, 2);
    }
    
    // Deselect display
    uint8_t cs_pins[] = DISPLAY_CS_PINS;
    HardwareInterface::spi_select_device(cs_pins[key_id], false);
}

void ProductionDeck::clear_all_keys() {
    for (int i = 0; i < STREAMDECK_KEYS; i++) {
        clear_key(i);
    }
    log_info("All keys cleared");
}

void ProductionDeck::set_brightness(uint8_t brightness) {
    brightness = MIN(brightness, 100);
    state_.current_brightness = brightness;
    
    // Update PWM level (0-255 range)
    uint16_t pwm_level = (brightness * 255) / 100;
    uint slice_num = pwm_gpio_to_slice_num(DISPLAY_BL_PIN);
    pwm_set_chan_level(slice_num, PWM_CHAN_A, pwm_level);
    
    log_info("Brightness set to %d%% (PWM=%d)", brightness, pwm_level);
}

void ProductionDeck::set_display_brightness(uint8_t brightness) {
    // This function is an alias for set_brightness for compatibility
    set_brightness(brightness);
}

// ===================================================================
// Device Control and Status
// ===================================================================

void ProductionDeck::reset_device() {
    log_info("Device reset requested");
    
    // Clear all images and reset state
    clear_all_keys();
    
    // Reset image buffers
    for (int i = 0; i < STREAMDECK_KEYS; i++) {
        reset_image_buffer(i);
    }
    
    // Reset button states
    memset(&buttons_, 0, sizeof(buttons_));
    
    // Reset brightness
    set_brightness(DISPLAY_BRIGHTNESS);
    
    log_info("Device reset complete");
}

const char* ProductionDeck::get_firmware_version() {
    return "1.0.0";
}

bool ProductionDeck::is_usb_connected() {
    return state_.usb_connected;
}

bool ProductionDeck::is_ready() {
    return state_.initialized && state_.displays_ready;
}

uint32_t ProductionDeck::get_uptime_ms() {
    return millis() - state_.startup_time;
}

// ===================================================================
// Utility Functions
// ===================================================================

uint32_t ProductionDeck::millis() {
    return time_us_32() / 1000;
}

void ProductionDeck::delay_ms(uint32_t ms) {
    sleep_ms(ms);
}

void ProductionDeck::blink_status_led(uint16_t on_ms, uint16_t off_ms) {
    static uint32_t last_toggle = 0;
    static bool led_state = false;
    static uint16_t current_on_ms = 500;
    static uint16_t current_off_ms = 500;
    
    // Update timing if changed
    current_on_ms = on_ms;
    current_off_ms = off_ms;
    
    uint32_t now = millis();
    uint32_t interval = led_state ? current_on_ms : current_off_ms;
    
    if (now - last_toggle >= interval) {
        led_state = !led_state;
        HardwareInterface::gpio_set(LED_STATUS_PIN, led_state);
        last_toggle = now;
    }
}

void ProductionDeck::update_status_leds() {
    // USB status LED
    HardwareInterface::gpio_set(LED_USB_PIN, state_.usb_connected);
    
    // Error LED (off for now - implement error tracking later)
    HardwareInterface::gpio_set(LED_ERROR_PIN, false);
}

void ProductionDeck::watchdog_update() {
    #if WATCHDOG_ENABLED
    watchdog_update();
    #endif
}

// ===================================================================
// Logging Functions
// ===================================================================

void ProductionDeck::log_debug(const char* format, ...) {
    #if DEBUG_LEVEL >= 2
    printf("[DEBUG] ");
    va_list args;
    va_start(args, format);
    vprintf(format, args);
    va_end(args);
    printf("\n");
    #endif
}

void ProductionDeck::log_info(const char* format, ...) {
    #if DEBUG_LEVEL >= 1
    printf("[INFO] ");
    va_list args;
    va_start(args, format);
    vprintf(format, args);
    va_end(args);
    printf("\n");
    #endif
}

void ProductionDeck::log_error(const char* format, ...) {
    printf("[ERROR] ");
    va_list args;
    va_start(args, format);
    vprintf(format, args);
    va_end(args);
    printf("\n");
}

// ===================================================================
// Multi-core Support
// ===================================================================

void ProductionDeck::core1_entry() {
    if (g_productiondeck) {
        g_productiondeck->core1_main();
    }
}

void ProductionDeck::core1_main() {
    log_info("Core1: I/O processing started");
    
    while (core1_running_) {
        // Handle display updates and other I/O intensive tasks
        // This helps keep USB communication responsive on Core0
        
        // TODO: Implement display refresh, button scanning optimization, etc.
        
        sleep_ms(10);
    }
    
    log_info("Core1: Stopped");
}