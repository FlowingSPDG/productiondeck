// ===================================================================
// USB HID Descriptors for StreamDeck Mini Compatibility
// 
// This file implements the exact USB descriptors needed for the host
// to recognize this device as a StreamDeck Mini and communicate using
// the official StreamDeck software protocol.
// ===================================================================

#include "usb_descriptors.h"
#include "tusb.h"
#include "productiondeck.h"

// Global ProductionDeck instance (declared in main.cpp)
extern ProductionDeck* g_productiondeck;

// ===================================================================
// USB Device Descriptor
// ===================================================================

const tusb_desc_device_t desc_device = {
    .bLength            = sizeof(tusb_desc_device_t),
    .bDescriptorType    = TUSB_DESC_DEVICE,
    .bcdUSB             = 0x0200,           // USB 2.0
    .bDeviceClass       = 0x00,             // Device class defined at interface level
    .bDeviceSubClass    = 0x00,
    .bDeviceProtocol    = 0x00,
    .bMaxPacketSize0    = CFG_TUD_ENDPOINT0_SIZE,
    .idVendor           = USB_VID,          // 0x0fd9 (Elgato)
    .idProduct          = USB_PID,          // 0x0063 (StreamDeck Mini)
    .bcdDevice          = USB_BCD_DEVICE,   // Device version 1.0
    .iManufacturer      = STRING_INDEX_MANUFACTURER,
    .iProduct           = STRING_INDEX_PRODUCT,
    .iSerialNumber      = STRING_INDEX_SERIAL,
    .bNumConfigurations = 1
};

// ===================================================================
// HID Report Descriptor (Critical for StreamDeck Compatibility)
// ===================================================================

const uint8_t desc_hid_report[] = {
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
    0x95, STREAMDECK_KEYS,              // Report Count (6 buttons)
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
    0x95, HID_REPORT_SIZE_FEATURE,      // Report Count (32 bytes)
    0xb1, 0x02,                         // Feature (Data, Variable, Absolute)
    
    // End Collection
    0xc0
};

const uint16_t desc_hid_report_len = sizeof(desc_hid_report);

// ===================================================================
// Configuration Descriptor
// ===================================================================

#define CONFIG_TOTAL_LEN (TUD_CONFIG_DESC_LEN + TUD_HID_DESC_LEN)

const uint8_t desc_configuration[] = {
    // Configuration Descriptor
    TUD_CONFIG_DESCRIPTOR(1, 1, 0, CONFIG_TOTAL_LEN, 0x00, 100),
    
    // HID Descriptor
    TUD_HID_DESCRIPTOR(ITF_NUM_HID, 0, HID_ITF_PROTOCOL_NONE, 
                       sizeof(desc_hid_report), USB_HID_EP_IN, 
                       CFG_TUD_HID_EP_BUFSIZE, 1)
};

// ===================================================================
// String Descriptors
// ===================================================================

const char* string_desc_arr[] = {
    (const char[]) { 0x09, 0x04 },      // 0: Language (English US)
    USB_MANUFACTURER,                    // 1: Manufacturer
    USB_PRODUCT,                        // 2: Product  
    USB_SERIAL,                         // 3: Serial Number
};

// ===================================================================
// TinyUSB Descriptor Callbacks
// ===================================================================

extern "C" {

// Invoked when received GET DEVICE DESCRIPTOR
uint8_t const* tud_descriptor_device_cb(void) {
    return (uint8_t const*) &desc_device;
}

// Invoked when received GET CONFIGURATION DESCRIPTOR
uint8_t const* tud_descriptor_configuration_cb(uint8_t index) {
    (void) index; // For multiple configurations
    return desc_configuration;
}

// Invoked when received GET STRING DESCRIPTOR request
uint16_t const* tud_descriptor_string_cb(uint8_t index, uint16_t langid) {
    (void) langid;
    
    static uint16_t _desc_str[32];
    uint8_t chr_count;
    
    if (index == 0) {
        memcpy(&_desc_str[1], string_desc_arr[0], 2);
        chr_count = 1;
    } else {
        // Convert ASCII string into UTF-16
        if (index >= sizeof(string_desc_arr) / sizeof(string_desc_arr[0])) {
            return NULL;
        }
        
        const char* str = string_desc_arr[index];
        
        // Cap at max chars
        chr_count = strlen(str);
        if (chr_count > 31) chr_count = 31;
        
        // Convert ASCII to UTF-16
        for (uint8_t i = 0; i < chr_count; i++) {
            _desc_str[1 + i] = str[i];
        }
    }
    
    // First byte is length (including header), second byte is string type
    _desc_str[0] = (TUSB_DESC_STRING << 8) | (2 * chr_count + 2);
    
    return _desc_str;
}

// Invoked when received GET HID REPORT DESCRIPTOR
uint8_t const* tud_hid_descriptor_report_cb(uint8_t itf) {
    (void) itf;
    return desc_hid_report;
}

// ===================================================================
// HID Report Callbacks (Protocol Implementation)
// ===================================================================

// Invoked when received GET_REPORT control request
// Application must fill buffer report's content and return its length.
// Return zero will cause the stack to STALL request
uint16_t tud_hid_get_report_cb(uint8_t itf, uint8_t report_id, 
                               hid_report_type_t report_type, 
                               uint8_t* buffer, uint16_t reqlen) {
    (void) itf;
    
    if (report_type == HID_REPORT_TYPE_FEATURE) {
        // Handle feature report requests (version, etc.)
        if (report_id == FEATURE_REPORT_VERSION_V1 || report_id == FEATURE_REPORT_VERSION_V2) {
            // Version request
            memset(buffer, 0, reqlen);
            buffer[0] = report_id;
            
            // Version string offset depends on V1/V2
            uint8_t offset = (report_id == FEATURE_REPORT_VERSION_V2) ? 6 : 5;
            const char* version = "1.0.0";
            strncpy((char*)&buffer[offset], version, reqlen - offset);
            
            return reqlen;
        }
    }
    
    return 0; // STALL unknown requests
}

// Invoked when received SET_REPORT control request or data from OUT endpoint
void tud_hid_set_report_cb(uint8_t itf, uint8_t report_id, 
                           hid_report_type_t report_type, 
                           uint8_t const* buffer, uint16_t bufsize) {
    (void) itf;
    
    if (report_type == HID_REPORT_TYPE_FEATURE) {
        // Handle feature commands
        usb_process_feature_report(report_id, buffer, bufsize);
    } else if (report_type == HID_REPORT_TYPE_OUTPUT) {
        // Handle image data
        usb_process_output_report(buffer, bufsize);
    }
}

} // extern "C"

// ===================================================================
// USB Protocol Implementation
// ===================================================================

bool usb_hid_ready(void) {
    return tud_hid_ready();
}

bool usb_send_button_report(const uint8_t* button_states) {
    if (!tud_hid_ready()) {
        return false;
    }
    
    // Send input report (no report ID, raw button data)
    return tud_hid_report(0, button_states, STREAMDECK_KEYS);
}

void usb_process_feature_report(uint8_t report_id, const uint8_t* buffer, uint16_t bufsize) {
    if (!g_productiondeck) return;
    
    switch (report_id) {
        case FEATURE_REPORT_RESET_V1:
            // V1 Reset: [0x0B, 0x63, ...]
            if (bufsize >= 2 && buffer[1] == 0x63) {
                printf("USB: Reset command (V1)\n");
                g_productiondeck->reset_device();
            }
            break;
            
        case 0x03: // FEATURE_REPORT_RESET_V2 and FEATURE_REPORT_BRIGHTNESS_V2
            // V2 commands: [0x03, command_byte, ...]
            if (bufsize >= 2) {
                if (buffer[1] == 0x02) {
                    // V2 Reset: [0x03, 0x02, ...]
                    printf("USB: Reset command (V2)\n");
                    g_productiondeck->reset_device();
                } else if (buffer[1] == 0x08) {
                    // V2 Brightness: [0x03, 0x08, brightness, ...]
                    if (bufsize >= 3) {
                        uint8_t brightness = buffer[2];
                        printf("USB: Set brightness %d%% (V2)\n", brightness);
                        g_productiondeck->set_brightness(brightness);
                    }
                }
            }
            break;
            
        case FEATURE_REPORT_BRIGHTNESS_V1:
            // V1 Brightness: [0x05, 0x55, 0xAA, 0xD1, 0x01, brightness, ...]
            if (bufsize >= 6 && buffer[1] == 0x55 && buffer[2] == 0xAA && 
                buffer[3] == 0xD1 && buffer[4] == 0x01) {
                uint8_t brightness = buffer[5];
                printf("USB: Set brightness %d%% (V1)\n", brightness);
                g_productiondeck->set_brightness(brightness);
            }
            break;
            
        default:
            printf("USB: Unknown feature report ID: 0x%02X\n", report_id);
            break;
    }
}

void usb_process_output_report(const uint8_t* buffer, uint16_t bufsize) {
    if (!g_productiondeck || bufsize < 8) return;
    
    printf("[DEBUG] USB Output Report: %d bytes received\n", bufsize);
    printf("[DEBUG] Header: [0x%02X, 0x%02X, 0x%02X, 0x%02X, 0x%02X, 0x%02X, 0x%02X, 0x%02X]\n",
           buffer[0], buffer[1], buffer[2], buffer[3], buffer[4], buffer[5], buffer[6], buffer[7]);
    
    // Parse output report header (StreamDeck Mini V2 protocol)
    if (buffer[0] == OUTPUT_REPORT_IMAGE && buffer[1] == IMAGE_COMMAND_V2) {
        // V2 Image protocol: [0x02, 0x07, key_id, is_last, len_low, len_high, seq_low, seq_high, data...]
        uint8_t key_id = buffer[2];
        uint8_t is_last = buffer[3];
        uint16_t payload_len = buffer[4] | (buffer[5] << 8);
        uint16_t sequence = buffer[6] | (buffer[7] << 8);
        
        printf("[DEBUG] Image packet: key=%d seq=%d len=%d last=%d (for 72x72 key region)\n", 
               key_id, sequence, payload_len, is_last);
        
        if (key_id < STREAMDECK_KEYS) {
            g_productiondeck->receive_image_packet(buffer, bufsize);
        } else {
            printf("[ERROR] Invalid key_id %d (max %d)\n", key_id, STREAMDECK_KEYS - 1);
        }
    } else {
        printf("[DEBUG] Unknown output report format: [0x%02X, 0x%02X]\n", buffer[0], buffer[1]);
    }
}