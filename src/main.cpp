// ===================================================================
// OpenDeck - Open Source StreamDeck Alternative for RP2040
// 
// This file implements the main application entry point and core loop
// for the RP2040-based StreamDeck compatible device.
//
// Hardware: Raspberry Pi Pico (RP2040)
// Protocol: USB HID compatible with StreamDeck Mini
// Display: 6x ST7735 TFT displays (80x80 each)
// Buttons: 6 tactile switches in 3x2 matrix
// ===================================================================

#include "opendeck.h"
#include "pico/stdlib.h"
#include "pico/multicore.h"
#include "hardware/watchdog.h"
#include "tusb.h"

// Global application instance
OpenDeck* g_opendeck = nullptr;

// ===================================================================
// Main Application Entry Point
// ===================================================================

int main() {
    // Initialize basic Pico SDK
    stdio_init_all();
    
    // Wait a moment for things to settle
    sleep_ms(100);
    
    printf("\n");
    printf("========================================\n");
    printf("OpenDeck v1.0 - StreamDeck Alternative\n");
    printf("Hardware: RP2040 (Raspberry Pi Pico)\n");
    printf("Target: StreamDeck Mini Compatible\n");
    printf("USB: VID=0x%04X PID=0x%04X\n", USB_VID, USB_PID);
    printf("Keys: %d (%dx%d layout)\n", STREAMDECK_KEYS, STREAMDECK_COLS, STREAMDECK_ROWS);
    printf("Display: %dx%d per key\n", KEY_IMAGE_SIZE, KEY_IMAGE_SIZE);
    printf("========================================\n");
    
    // Enable watchdog for system stability
    #if WATCHDOG_ENABLED
    watchdog_enable(8000, 1); // 8 second timeout, pause on debug
    printf("Watchdog enabled (8s timeout)\n");
    #endif
    
    // Create main application instance
    g_opendeck = new OpenDeck();
    if (!g_opendeck) {
        printf("ERROR: Failed to create OpenDeck instance\n");
        panic("Memory allocation failed");
    }
    
    // Initialize the device
    printf("Initializing OpenDeck...\n");
    if (!g_opendeck->initialize()) {
        printf("ERROR: OpenDeck initialization failed\n");
        panic("Initialization failed");
    }
    
    printf("OpenDeck initialized successfully\n");
    printf("USB VID:PID = %04X:%04X\n", USB_VID, USB_PID);
    printf("Waiting for USB connection...\n");
    
    // Main application loop
    uint32_t last_status_print = 0;
    uint32_t status_print_interval = 10000; // Print status every 10 seconds
    
    while (true) {
        // Update watchdog
        #if WATCHDOG_ENABLED
        watchdog_update();
        #endif
        
        // Run main application logic
        g_opendeck->run();
        
        // Periodic status output (for debugging)
        uint32_t now = time_us_32() / 1000;
        if (now - last_status_print > status_print_interval) {
            if (g_opendeck->is_usb_connected()) {
                printf("Status: USB connected, uptime=%lu ms\n", g_opendeck->get_uptime_ms());
            } else {
                printf("Status: Waiting for USB connection...\n");
            }
            last_status_print = now;
        }
        
        // Small delay to prevent overwhelming the system
        sleep_ms(1);
    }
    
    // Cleanup (should never reach here)
    g_opendeck->shutdown();
    delete g_opendeck;
    
    return 0;
}

// ===================================================================
// USB Device Callbacks (required by TinyUSB)
// ===================================================================

extern "C" {

// Invoked when device is mounted (configured)
void tud_mount_cb(void) {
    printf("USB: Device mounted\n");
    if (g_opendeck) {
        g_opendeck->blink_status_led(200, 200); // Fast blink when connected
    }
}

// Invoked when device is unmounted
void tud_umount_cb(void) {
    printf("USB: Device unmounted\n");
    if (g_opendeck) {
        g_opendeck->blink_status_led(1000, 1000); // Slow blink when disconnected
    }
}

// Invoked when USB bus is suspended
// Within 7ms, device must draw an average current less than 2.5mA from bus
void tud_suspend_cb(bool remote_wakeup_en) {
    printf("USB: Device suspended (remote_wakeup=%d)\n", remote_wakeup_en);
    if (g_opendeck) {
        g_opendeck->set_brightness(0); // Turn off displays to save power
    }
}

// Invoked when USB bus is resumed
void tud_resume_cb(void) {
    printf("USB: Device resumed\n");
    if (g_opendeck) {
        g_opendeck->set_brightness(DISPLAY_BRIGHTNESS); // Restore display brightness
    }
}

} // extern "C"

// ===================================================================
// Error Handling and Panic Recovery
// ===================================================================

// Custom panic handler for better debugging
void __attribute__((weak)) panic_handler(const char* msg) {
    printf("\n*** PANIC: %s ***\n", msg);
    
    // Try to save some state information
    printf("System state at panic:\n");
    printf("- Core: %d\n", get_core_num());
    printf("- Time: %lu us\n", time_us_32());
    
    if (g_opendeck) {
        printf("- USB connected: %s\n", g_opendeck->is_usb_connected() ? "yes" : "no");
        printf("- Device ready: %s\n", g_opendeck->is_ready() ? "yes" : "no");
        printf("- Uptime: %lu ms\n", g_opendeck->get_uptime_ms());
    }
    
    // Flash error pattern on status LED
    const uint LED_PIN = 25; // Built-in LED
    gpio_init(LED_PIN);
    gpio_set_dir(LED_PIN, GPIO_OUT);
    
    // SOS pattern: ... --- ... (3 short, 3 long, 3 short)
    while (true) {
        // Three short blinks
        for (int i = 0; i < 3; i++) {
            gpio_put(LED_PIN, 1);
            sleep_ms(200);
            gpio_put(LED_PIN, 0);
            sleep_ms(200);
        }
        
        sleep_ms(400);
        
        // Three long blinks  
        for (int i = 0; i < 3; i++) {
            gpio_put(LED_PIN, 1);
            sleep_ms(600);
            gpio_put(LED_PIN, 0);
            sleep_ms(200);
        }
        
        sleep_ms(400);
        
        // Three short blinks
        for (int i = 0; i < 3; i++) {
            gpio_put(LED_PIN, 1);
            sleep_ms(200);
            gpio_put(LED_PIN, 0);
            sleep_ms(200);
        }
        
        sleep_ms(2000); // Wait before repeating
    }
}

// ===================================================================
// Memory Management and Diagnostics
// ===================================================================

// Override new/delete for better memory tracking in debug builds
#if DEBUG_LEVEL >= 2

void* operator new(size_t size) {
    void* ptr = malloc(size);
    printf("DEBUG: Allocated %zu bytes at %p\n", size, ptr);
    return ptr;
}

void operator delete(void* ptr) {
    printf("DEBUG: Freed memory at %p\n", ptr);
    free(ptr);
}

void* operator new[](size_t size) {
    void* ptr = malloc(size);
    printf("DEBUG: Allocated array %zu bytes at %p\n", size, ptr);
    return ptr;
}

void operator delete[](void* ptr) {
    printf("DEBUG: Freed array at %p\n", ptr);
    free(ptr);
}

#endif // DEBUG_LEVEL >= 2

// ===================================================================
// Build Information (embedded in binary)
// ===================================================================

extern "C" {
    
const char build_info[] __attribute__((section(".build_info"))) = 
    "OpenDeck v1.0 for RP2040\n"
    "Built: " __DATE__ " " __TIME__ "\n"
    "Compiler: " __VERSION__ "\n"
    "Target: StreamDeck Mini Compatible\n"
    "USB: " STRINGIFY(USB_VID) ":" STRINGIFY(USB_PID) "\n"
    "GPIO: See opendeck_config.h\n";

} // extern "C"

// Stringify macro for build info
#define STRINGIFY(x) #x