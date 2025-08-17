// ===================================================================
// Hardware Abstraction Layer Implementation
// 
// This file provides clean, safe interfaces for all hardware operations
// including GPIO, SPI, PWM, and timing functions for the RP2040.
// ===================================================================

#include "productiondeck.h"
#include "hardware/gpio.h"
#include "hardware/spi.h"
#include "hardware/pwm.h"
#include "hardware/timer.h"

// ===================================================================
// GPIO Operations
// ===================================================================

void HardwareInterface::gpio_init_output(uint gpio, bool initial_state) {
    gpio_init(gpio);
    gpio_set_dir(gpio, GPIO_OUT);
    gpio_put(gpio, initial_state);
}

void HardwareInterface::gpio_init_input(uint gpio, bool pullup) {
    gpio_init(gpio);
    gpio_set_dir(gpio, GPIO_IN);
    
    if (pullup) {
        gpio_pull_up(gpio);
    } else {
        gpio_disable_pulls(gpio);
    }
}

void HardwareInterface::gpio_set(uint gpio, bool state) {
    gpio_put(gpio, state);
}

bool HardwareInterface::gpio_get(uint gpio) {
    return gpio_get(gpio);
}

// ===================================================================
// SPI Operations
// ===================================================================

bool HardwareInterface::spi_init(spi_inst_t* spi, uint baudrate) {
    // Initialize SPI with default settings
    uint actual_baudrate = spi_init(spi, baudrate);
    
    // Configure SPI parameters
    spi_set_format(spi, 
                   8,        // 8 bits per transfer
                   SPI_CPOL_0, // Clock polarity 0 (idle low)
                   SPI_CPHA_0, // Clock phase 0 (sample on rising edge)
                   SPI_MSB_FIRST); // MSB first
    
    printf("SPI initialized: requested %d Hz, actual %d Hz\n", baudrate, actual_baudrate);
    return true;
}

void HardwareInterface::spi_write(spi_inst_t* spi, const uint8_t* data, size_t len) {
    spi_write_blocking(spi, data, len);
}

void HardwareInterface::spi_select_device(uint cs_pin, bool select) {
    // CS is active low for most SPI devices
    gpio_put(cs_pin, !select);
}

// ===================================================================
// PWM Operations
// ===================================================================

bool HardwareInterface::pwm_init(uint gpio, uint16_t wrap, uint16_t level) {
    gpio_set_function(gpio, GPIO_FUNC_PWM);
    
    uint slice_num = pwm_gpio_to_slice_num(gpio);
    uint channel = pwm_gpio_to_channel(gpio);
    
    pwm_set_wrap(slice_num, wrap);
    pwm_set_chan_level(slice_num, channel, level);
    pwm_set_enabled(slice_num, true);
    
    return true;
}

void HardwareInterface::pwm_set_level(uint gpio, uint16_t level) {
    uint slice_num = pwm_gpio_to_slice_num(gpio);
    uint channel = pwm_gpio_to_channel(gpio);
    
    pwm_set_chan_level(slice_num, channel, level);
}

// ===================================================================
// Timing Operations
// ===================================================================

uint32_t HardwareInterface::time_ms() {
    return time_us_32() / 1000;
}

void HardwareInterface::sleep_ms(uint32_t ms) {
    sleep_ms(ms);
}

void HardwareInterface::sleep_us(uint32_t us) {
    sleep_us(us);
}

// ===================================================================
// Display Driver Implementation
// ===================================================================

DisplayDriver::DisplayDriver(uint8_t cs_pin, uint8_t dc_pin, uint8_t rst_pin) 
    : cs_pin_(cs_pin), dc_pin_(dc_pin), rst_pin_(rst_pin), initialized_(false) {
}

bool DisplayDriver::initialize() {
    printf("Initializing display CS=%d DC=%d RST=%d\n", cs_pin_, dc_pin_, rst_pin_);
    
    // Initialize control pins
    HardwareInterface::gpio_init_output(cs_pin_, true);  // CS high = deselected
    HardwareInterface::gpio_init_output(dc_pin_, false); // DC low = command mode
    HardwareInterface::gpio_init_output(rst_pin_, true); // RST high = not reset
    
    // Reset the display
    reset();
    
    // Send initialization sequence
    init_sequence();
    
    initialized_ = true;
    printf("Display initialized successfully\n");
    return true;
}

void DisplayDriver::reset() {
    printf("Resetting display\n");
    
    // Reset pulse (low for 10ms)
    HardwareInterface::gpio_set(rst_pin_, false);
    HardwareInterface::sleep_ms(10);
    HardwareInterface::gpio_set(rst_pin_, true);
    HardwareInterface::sleep_ms(120); // Wait for display to boot
}

void DisplayDriver::init_sequence() {
    printf("Sending display initialization sequence\n");
    
    // ST7735 initialization sequence for 80x80 display
    // This is a basic sequence - adjust for your specific display
    
    select();
    
    // Software reset
    send_command(0x01);
    HardwareInterface::sleep_ms(150);
    
    // Sleep out
    send_command(0x11);
    HardwareInterface::sleep_ms(120);
    
    // Frame rate control
    send_command(0xB1);
    uint8_t frc[] = {0x01, 0x2C, 0x2D};
    send_data(frc, sizeof(frc));
    
    send_command(0xB2);
    send_data(frc, sizeof(frc));
    
    send_command(0xB3);
    uint8_t frc2[] = {0x01, 0x2C, 0x2D, 0x01, 0x2C, 0x2D};
    send_data(frc2, sizeof(frc2));
    
    // Column inversion
    send_command(0xB4);
    uint8_t inv = 0x07;
    send_data(&inv, 1);
    
    // Power control
    send_command(0xC0);
    uint8_t pwr1[] = {0xA2, 0x02, 0x84};
    send_data(pwr1, sizeof(pwr1));
    
    send_command(0xC1);
    uint8_t pwr2 = 0xC5;
    send_data(&pwr2, 1);
    
    send_command(0xC2);
    uint8_t pwr3[] = {0x0A, 0x00};
    send_data(pwr3, sizeof(pwr3));
    
    send_command(0xC3);
    uint8_t pwr4[] = {0x8A, 0x2A};
    send_data(pwr4, sizeof(pwr4));
    
    send_command(0xC4);
    uint8_t pwr5[] = {0x8A, 0xEE};
    send_data(pwr5, sizeof(pwr5));
    
    // VCOM control
    send_command(0xC5);
    uint8_t vcom = 0x0E;
    send_data(&vcom, 1);
    
    // Memory access control (rotation)
    send_command(0x36);
    uint8_t madctl = 0xC8; // RGB, row/col addr order
    send_data(&madctl, 1);
    
    // Color mode - 16 bit RGB565
    send_command(0x3A);
    uint8_t colmod = 0x05;
    send_data(&colmod, 1);
    
    // Column address set (0-79)
    send_command(0x2A);
    uint8_t caset[] = {0x00, 0x00, 0x00, 0x4F};
    send_data(caset, sizeof(caset));
    
    // Row address set (0-79)  
    send_command(0x2B);
    uint8_t raset[] = {0x00, 0x00, 0x00, 0x4F};
    send_data(raset, sizeof(raset));
    
    // Gamma correction
    send_command(0xE0);
    uint8_t gmctrp[] = {0x02, 0x1C, 0x07, 0x12, 0x37, 0x32, 0x29, 0x2D,
                        0x29, 0x25, 0x2B, 0x39, 0x00, 0x01, 0x03, 0x10};
    send_data(gmctrp, sizeof(gmctrp));
    
    send_command(0xE1);
    uint8_t gmctrn[] = {0x03, 0x1D, 0x07, 0x06, 0x2E, 0x2C, 0x29, 0x2D,
                        0x2E, 0x2E, 0x37, 0x3F, 0x00, 0x00, 0x02, 0x10};
    send_data(gmctrn, sizeof(gmctrn));
    
    // Display on
    send_command(0x29);
    HardwareInterface::sleep_ms(10);
    
    deselect();
    
    printf("Display initialization sequence complete\n");
}

void DisplayDriver::set_brightness(uint8_t brightness) {
    // Brightness control would typically be done via PWM on the backlight pin
    // This is handled at a higher level in the main ProductionDeck class
    printf("Display brightness set to %d\n", brightness);
}

void DisplayDriver::clear() {
    if (!initialized_) return;
    
    printf("Clearing display\n");
    display_color(0x0000); // Black
}

void DisplayDriver::display_image(const uint8_t* image_data, uint16_t width, uint16_t height) {
    if (!initialized_ || !image_data) return;
    
    printf("Displaying %dx%d image\n", width, height);
    
    select();
    
    // Set window to full screen
    send_command(0x2A); // Column address set
    uint8_t caset[] = {0x00, 0x00, 0x00, width - 1};
    send_data(caset, sizeof(caset));
    
    send_command(0x2B); // Row address set
    uint8_t raset[] = {0x00, 0x00, 0x00, height - 1};
    send_data(raset, sizeof(raset));
    
    send_command(0x2C); // Memory write
    
    // Convert RGB888 to RGB565 and send
    uint16_t pixel_count = width * height;
    
    for (uint16_t i = 0; i < pixel_count; i++) {
        uint8_t r = image_data[i * 3 + 0];
        uint8_t g = image_data[i * 3 + 1];
        uint8_t b = image_data[i * 3 + 2];
        
        // Convert to RGB565
        uint16_t rgb565 = ((r & 0xF8) << 8) | ((g & 0xFC) << 3) | (b >> 3);
        
        // Send as big-endian
        uint8_t pixel_data[] = {(uint8_t)(rgb565 >> 8), (uint8_t)(rgb565 & 0xFF)};
        send_data(pixel_data, 2);
    }
    
    deselect();
    
    printf("Image display complete\n");
}

void DisplayDriver::display_color(uint16_t color) {
    if (!initialized_) return;
    
    select();
    
    // Set window to full screen (80x80)
    send_command(0x2A);
    uint8_t caset[] = {0x00, 0x00, 0x00, 0x4F};
    send_data(caset, sizeof(caset));
    
    send_command(0x2B);
    uint8_t raset[] = {0x00, 0x00, 0x00, 0x4F};
    send_data(raset, sizeof(raset));
    
    send_command(0x2C); // Memory write
    
    // Fill entire screen with color
    uint8_t color_data[] = {(uint8_t)(color >> 8), (uint8_t)(color & 0xFF)};
    
    for (int i = 0; i < 80 * 80; i++) {
        send_data(color_data, 2);
    }
    
    deselect();
}

void DisplayDriver::set_rotation(uint8_t rotation) {
    if (!initialized_) return;
    
    select();
    
    send_command(0x36); // Memory access control
    
    uint8_t madctl;
    switch (rotation) {
        case 0:  madctl = 0x00; break; // Normal
        case 1:  madctl = 0x60; break; // 90°
        case 2:  madctl = 0xC0; break; // 180°
        case 3:  madctl = 0xA0; break; // 270°
        default: madctl = 0xC8; break; // Default for StreamDeck
    }
    
    send_data(&madctl, 1);
    
    deselect();
    
    printf("Display rotation set to %d\n", rotation);
}

// ===================================================================
// Private Display Methods
// ===================================================================

void DisplayDriver::select() {
    HardwareInterface::spi_select_device(cs_pin_, true);
}

void DisplayDriver::deselect() {
    HardwareInterface::spi_select_device(cs_pin_, false);
}

void DisplayDriver::send_command(uint8_t cmd) {
    HardwareInterface::gpio_set(dc_pin_, false); // Command mode
    HardwareInterface::spi_write(SPI_PORT, &cmd, 1);
}

void DisplayDriver::send_data(const uint8_t* data, uint16_t len) {
    HardwareInterface::gpio_set(dc_pin_, true); // Data mode
    HardwareInterface::spi_write(SPI_PORT, data, len);
}

void DisplayDriver::send_byte(uint8_t data) {
    send_data(&data, 1);
}