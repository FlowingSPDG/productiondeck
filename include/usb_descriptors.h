#pragma once

#include "tusb.h"
#include "opendeck_config.h"

// ===================================================================
// USB HID Descriptors for StreamDeck Mini Compatibility
// Based on reverse-engineered StreamDeck protocol
// ===================================================================

// USB Device Descriptor
extern const tusb_desc_device_t desc_device;

// USB HID Report Descriptor  
extern const uint8_t desc_hid_report[];
extern const uint16_t desc_hid_report_len;

// USB Configuration Descriptor
extern const uint8_t desc_configuration[];

// USB String Descriptors
extern const char* string_desc_arr[];

// ===================================================================
// HID Report IDs and Types (StreamDeck Protocol)
// ===================================================================

// Report Types (using TinyUSB definitions)
// Note: These are already defined in TinyUSB's hid.h
// typedef enum {
//     HID_REPORT_TYPE_INPUT = 1,
//     HID_REPORT_TYPE_OUTPUT = 2,
//     HID_REPORT_TYPE_FEATURE = 3
// } hid_report_type_t;

// Feature Report Commands (StreamDeck Protocol)
#define FEATURE_REPORT_VERSION_V1       0x04    // Get firmware version (V1)
#define FEATURE_REPORT_VERSION_V2       0x05    // Get firmware version (V2)
#define FEATURE_REPORT_RESET_V1         0x0B    // Reset device (V1)
#define FEATURE_REPORT_RESET_V2         0x03    // Reset device (V2)
#define FEATURE_REPORT_BRIGHTNESS_V1    0x05    // Set brightness (V1)
#define FEATURE_REPORT_BRIGHTNESS_V2    0x03    // Set brightness (V2)

// Output Report Commands
#define OUTPUT_REPORT_IMAGE             0x02    // Image data transfer

// Image Command Types
#define IMAGE_COMMAND_V1                0x01    // V1 image protocol
#define IMAGE_COMMAND_V2                0x07    // V2 image protocol

// ===================================================================
// HID Report Structures
// ===================================================================

// Input Report - Button States (Device → Host)
typedef struct __attribute__((packed)) {
    uint8_t buttons[STREAMDECK_KEYS];   // Button states (0=released, 1=pressed)
} hid_input_report_t;

// Output Report - Image Data (Host → Device) 
typedef struct __attribute__((packed)) {
    uint8_t report_id;      // 0x02
    uint8_t command;        // 0x07 for V2, 0x01 for V1
    uint8_t key_id;         // Button index (0-5)
    uint8_t is_last;        // 1 if final packet, 0 otherwise
    uint16_t payload_len;   // Length of image data in this packet (little-endian)
    uint16_t sequence;      // Packet sequence number (little-endian)
    uint8_t image_data[HID_REPORT_SIZE_OUTPUT - 8]; // Image payload
} hid_output_report_t;

// Feature Report - Version Request/Response
typedef struct __attribute__((packed)) {
    uint8_t report_id;      // 0x04 (V1) or 0x05 (V2)
    uint8_t reserved[5];    // V1: 4 bytes, V2: 5 bytes
    char version[12];       // Firmware version string
} hid_feature_version_t;

// Feature Report - Reset Command
typedef struct __attribute__((packed)) {
    uint8_t report_id;      // 0x0B (V1) or 0x03 (V2)
    uint8_t command;        // 0x63 (V1) or 0x02 (V2)
    uint8_t reserved[15];   // Padding to 17 bytes
} hid_feature_reset_t;

// Feature Report - Brightness Command
typedef struct __attribute__((packed)) {
    uint8_t report_id;      // 0x05 (V1) or 0x03 (V2)
    uint8_t command_v1[4];  // V1: [0x55, 0xAA, 0xD1, 0x01]
    uint8_t command_v2;     // V2: 0x08
    uint8_t brightness;     // 0-100 percentage
    uint8_t reserved[10];   // Padding to 17 bytes
} hid_feature_brightness_t;

// ===================================================================
// USB Descriptor Constants
// ===================================================================

// Device Descriptor Values
#define USB_BCD_DEVICE          0x0100      // Device version 1.0
#define USB_CLASS_DEVICE        0x00        // Defined at interface level
#define USB_SUBCLASS_DEVICE     0x00        
#define USB_PROTOCOL_DEVICE     0x00
#define USB_MAX_PACKET_SIZE     64          // Control endpoint packet size

// Configuration Descriptor Values
#define USB_CONFIG_TOTAL_LEN    (TUD_CONFIG_DESC_LEN + TUD_HID_DESC_LEN)
#define USB_CONFIG_NUM          1
#define USB_CONFIG_ATTRIBUTES   (TUSB_DESC_CONFIG_ATT_REMOTE_WAKEUP)
#define USB_CONFIG_POWER_MA     100         // 100mA power consumption

// Interface Descriptor Values
#define ITF_NUM_HID             0
#define USB_HID_EP_IN           0x81        // HID input endpoint
#define USB_HID_EP_OUT          0x01        // HID output endpoint
#define USB_HID_INTERVAL_MS     1           // 1ms polling interval

// String Descriptor Indices
enum {
    STRING_INDEX_LANGUAGE = 0,
    STRING_INDEX_MANUFACTURER,
    STRING_INDEX_PRODUCT,
    STRING_INDEX_SERIAL,
    STRING_INDEX_COUNT
};

// ===================================================================
// Function Declarations
// ===================================================================

// USB descriptor callbacks (required by TinyUSB)
uint8_t const* tud_descriptor_device_cb(void);
uint8_t const* tud_descriptor_configuration_cb(uint8_t index);
uint16_t const* tud_descriptor_string_cb(uint8_t index, uint16_t langid);
uint8_t const* tud_hid_descriptor_report_cb(uint8_t itf);

// HID callbacks
uint16_t tud_hid_get_report_cb(uint8_t itf, uint8_t report_id, 
                               hid_report_type_t report_type, 
                               uint8_t* buffer, uint16_t reqlen);
void tud_hid_set_report_cb(uint8_t itf, uint8_t report_id, 
                           hid_report_type_t report_type, 
                           uint8_t const* buffer, uint16_t bufsize);

// Utility functions
bool usb_hid_ready(void);
bool usb_send_button_report(const uint8_t* button_states);
void usb_process_feature_report(uint8_t report_id, const uint8_t* buffer, uint16_t bufsize);
void usb_process_output_report(const uint8_t* buffer, uint16_t bufsize);