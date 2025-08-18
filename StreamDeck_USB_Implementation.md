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

### HID Report Descriptor (C++)

```cpp
// StreamDeck HID Report Descriptor
const uint8_t streamdeck_hid_report_descriptor[] = {
    0x05, 0x01,        // Usage Page (Generic Desktop)
    0x09, 0x00,        // Usage (0x00)
    0xa1, 0x01,        // Collection (Application)
    
    // Input Report (Button States)
    0x09, 0x00,        // Usage (0x00)
    0x15, 0x00,        // Logical Minimum (0)
    0x25, 0x01,        // Logical Maximum (1)
    0x75, 0x08,        // Report Size (8 bits)
    0x95, 0x20,        // Report Count (32 bytes max)
    0x81, 0x02,        // Input (Data, Variable, Absolute)
    
    // Output Report (Image Data)
    0x09, 0x00,        // Usage (0x00)
    0x15, 0x00,        // Logical Minimum (0)
    0x26, 0xFF, 0x00,  // Logical Maximum (255)
    0x75, 0x08,        // Report Size (8 bits)
    0x96, 0x00, 0x04,  // Report Count (1024 bytes)
    0x91, 0x02,        // Output (Data, Variable, Absolute)
    
    // Feature Report (Commands)
    0x09, 0x00,        // Usage (0x00)
    0x15, 0x00,        // Logical Minimum (0)
    0x26, 0xFF, 0x00,  // Logical Maximum (255)
    0x75, 0x08,        // Report Size (8 bits)
    0x95, 0x20,        // Report Count (32 bytes)
    0xb1, 0x02,        // Feature (Data, Variable, Absolute)
    
    0xc0               // End Collection
};
```

### USB Device Descriptor

```cpp
#include <USB.h>
#include <USBHID.h>

// StreamDeck Mini configuration
#define STREAMDECK_VID 0x0fd9
#define STREAMDECK_PID 0x0063  // Mini
#define STREAMDECK_KEYS 6
#define STREAMDECK_KEY_SIZE 80
#define STREAMDECK_IMAGE_SIZE (STREAMDECK_KEY_SIZE * STREAMDECK_KEY_SIZE * 3)

class StreamDeckDevice {
private:
    USBHID hid;
    uint8_t button_states[STREAMDECK_KEYS];
    uint8_t image_buffer[STREAMDECK_KEYS][STREAMDECK_IMAGE_SIZE];
    bool receiving_image[STREAMDECK_KEYS];
    uint16_t image_sequence[STREAMDECK_KEYS];
    uint16_t image_received[STREAMDECK_KEYS];
    
public:
    StreamDeckDevice() : hid() {
        memset(button_states, 0, sizeof(button_states));
        memset(receiving_image, 0, sizeof(receiving_image));
        memset(image_sequence, 0, sizeof(image_sequence));
        memset(image_received, 0, sizeof(image_received));
    }
    
    void begin() {
        // Initialize USB HID with StreamDeck identifiers
        USB.VID(STREAMDECK_VID);
        USB.PID(STREAMDECK_PID);
        USB.productName("Stream Deck Mini");
        USB.manufacturerName("Elgato Systems");
        
        hid.setReportDescriptor(streamdeck_hid_report_descriptor, 
                               sizeof(streamdeck_hid_report_descriptor));
        
        // Set up callbacks
        hid.onGetReport(std::bind(&StreamDeckDevice::onGetReport, this, 
                                 std::placeholders::_1, std::placeholders::_2, 
                                 std::placeholders::_3, std::placeholders::_4));
        hid.onSetReport(std::bind(&StreamDeckDevice::onSetReport, this,
                                 std::placeholders::_1, std::placeholders::_2,
                                 std::placeholders::_3, std::placeholders::_4));
        
        hid.begin();
        USB.begin();
    }
};
```

## Protocol Implementation

### Feature Report Handling

```cpp
bool StreamDeckDevice::onGetReport(uint8_t report_id, hid_report_type_t report_type, 
                                  uint8_t* buffer, uint16_t reqlen) {
    if (report_type == HID_REPORT_TYPE_FEATURE) {
        if (buffer[0] == 0x04 || buffer[0] == 0x05) {
            // Version request
            const char* version = "1.0.0.0";
            uint8_t offset = (buffer[0] == 0x05) ? 6 : 5;
            
            memset(buffer, 0, reqlen);
            buffer[0] = buffer[0]; // Keep report ID
            strncpy((char*)&buffer[offset], version, reqlen - offset);
            return true;
        }
    }
    return false;
}

bool StreamDeckDevice::onSetReport(uint8_t report_id, hid_report_type_t report_type,
                                  const uint8_t* buffer, uint16_t bufsize) {
    if (report_type == HID_REPORT_TYPE_FEATURE) {
        if (buffer[0] == 0x03 || buffer[0] == 0x0b) {
            // Reset command
            if ((buffer[0] == 0x03 && bufsize >= 2 && buffer[1] == 0x02) ||
                (buffer[0] == 0x0b && bufsize >= 2 && buffer[1] == 0x63)) {
                resetDevice();
                return true;
            }
        }
        
        if (buffer[0] == 0x03 || buffer[0] == 0x05) {
            // Brightness control
            if ((buffer[0] == 0x03 && bufsize >= 3 && buffer[1] == 0x08) ||
                (buffer[0] == 0x05 && bufsize >= 6)) {
                uint8_t brightness = (buffer[0] == 0x03) ? buffer[2] : buffer[5];
                setBrightness(brightness);
                return true;
            }
        }
    }
    
    if (report_type == HID_REPORT_TYPE_OUTPUT) {
        if (buffer[0] == 0x02) {
            handleImageData(buffer, bufsize);
            return true;
        }
    }
    
    return false;
}
```

### Image Data Processing

```cpp
void StreamDeckDevice::handleImageData(const uint8_t* buffer, uint16_t bufsize) {
    if (bufsize < 8) return;
    
    // Parse V2 header format
    uint8_t command = buffer[1];
    uint8_t key_id = buffer[2];
    uint8_t is_last = buffer[3];
    uint16_t payload_len = buffer[4] | (buffer[5] << 8);
    uint16_t sequence = buffer[6] | (buffer[7] << 8);
    
    if (command != 0x07 || key_id >= STREAMDECK_KEYS) return;
    
    // Handle image data
    if (sequence == 0) {
        // First packet - reset state
        receiving_image[key_id] = true;
        image_sequence[key_id] = 0;
        image_received[key_id] = 0;
    }
    
    if (receiving_image[key_id] && sequence == image_sequence[key_id]) {
        // Copy payload data
        uint16_t data_offset = 8;
        uint16_t copy_len = min(payload_len, bufsize - data_offset);
        uint16_t buffer_offset = image_received[key_id];
        
        if (buffer_offset + copy_len <= STREAMDECK_IMAGE_SIZE) {
            memcpy(&image_buffer[key_id][buffer_offset], &buffer[data_offset], copy_len);
            image_received[key_id] += copy_len;
            image_sequence[key_id]++;
        }
        
        if (is_last) {
            // Image complete - process and display
            processReceivedImage(key_id);
            receiving_image[key_id] = false;
        }
    }
}

void StreamDeckDevice::processReceivedImage(uint8_t key_id) {
    // For Mini devices, image is BMP format after 54-byte header
    const uint8_t bmp_header[54] = {
        0x42, 0x4d, 0xf6, 0x3c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x36, 0x00, 0x00, 0x00, 0x28, 0x00,
        0x00, 0x00, 0x50, 0x00, 0x00, 0x00, 0x50, 0x00, 0x00, 0x00, 0x01, 0x00, 0x18, 0x00, 0x00, 0x00,
        0x00, 0x00, 0xc0, 0x3c, 0x00, 0x00, 0xc4, 0x0e, 0x00, 0x00, 0xc4, 0x0e, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00
    };
    
    // Skip BMP header if present
    uint8_t* image_data = image_buffer[key_id];
    if (memcmp(image_data, bmp_header, 54) == 0) {
        image_data += 54;
    }
    
    // Display image on LCD/TFT for this key
    displayKeyImage(key_id, image_data, STREAMDECK_KEY_SIZE, STREAMDECK_KEY_SIZE);
}
```

### Button State Reporting

```cpp
void StreamDeckDevice::updateButtonStates() {
    // Read physical buttons (implement based on your hardware)
    uint8_t new_states[STREAMDECK_KEYS];
    readPhysicalButtons(new_states);
    
    // Check for changes
    bool changed = false;
    for (int i = 0; i < STREAMDECK_KEYS; i++) {
        if (new_states[i] != button_states[i]) {
            button_states[i] = new_states[i];
            changed = true;
        }
    }
    
    // Send input report if changed
    if (changed) {
        sendButtonReport();
    }
}

void StreamDeckDevice::sendButtonReport() {
    uint8_t report[STREAMDECK_KEYS + 1]; // +1 for potential offset
    
    // Mini uses direct mapping, no offset
    memcpy(report, button_states, STREAMDECK_KEYS);
    
    hid.sendInputReport(report, STREAMDECK_KEYS);
}
```

## Hardware Integration Examples

### RP2040 Implementation

```cpp
#include <Adafruit_TinyUSB.h>
#include <Adafruit_ST7735.h>  // For TFT displays

// RP2040-specific setup
void setup() {
    // Initialize TinyUSB stack
    TinyUSB_Device_Init(0);
    
    StreamDeckDevice streamdeck;
    streamdeck.begin();
    
    // Initialize displays and buttons
    initializeDisplays();
    initializeButtons();
}

void loop() {
    streamdeck.updateButtonStates();
    delay(10); // 100Hz polling rate
}
```

### STM32 Implementation

```cpp
#include "usbd_hid.h"
#include "usbd_desc.h"

// STM32 HAL setup
int main(void) {
    HAL_Init();
    SystemClock_Config();
    
    // Initialize USB Device Library
    USBD_Init(&USBD_Device, &HID_Desc, 0);
    USBD_RegisterClass(&USBD_Device, &USBD_HID);
    USBD_Start(&USBD_Device);
    
    StreamDeckDevice streamdeck;
    streamdeck.begin();
    
    while (1) {
        streamdeck.updateButtonStates();
        HAL_Delay(10);
    }
}
```

## Testing and Validation

### Windows Testing
1. Install official Stream Deck software
2. Connect your device
3. Verify recognition in Device Manager (should show as "Stream Deck Mini")
4. Test button presses and image updates in Stream Deck software

### Protocol Validation
```cpp
void StreamDeckDevice::debugProtocol(const uint8_t* buffer, uint16_t len) {
    Serial.print("Received: ");
    for (int i = 0; i < min(len, 16); i++) {
        Serial.printf("%02X ", buffer[i]);
    }
    Serial.println();
}
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