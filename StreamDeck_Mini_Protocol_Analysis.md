# StreamDeck Mini USB HID Protocol Analysis

## Overview

This document summarizes the complete USB HID protocol analysis for StreamDeck Mini compatibility, based on extensive debugging and Wireshark packet analysis. The findings were used to successfully implement a working StreamDeck Mini clone using RP2040 (Raspberry Pi Pico) with Embassy USB stack.

## Key Findings

### 1. USB Device Configuration

**Required USB Descriptors:**
```
Vendor ID: 0x0fd9 (Elgato Systems)
Product ID: 0x0063 (StreamDeck Mini)
Manufacturer: "Elgato"
Product: "StreamDeck Mini"
Serial Number: "PRODUCTIONDK" (or any 12-char alphanumeric)
Device Release: 0x0200 (USB 2.0)
```

**Configuration Descriptor:**
- `bcdUSB`: 0x0200 (USB 2.0)
- `bmAttributes`: 0xA0 (Self-powered, Remote Wakeup)
- `bMaxPower`: 100mA
- `wTotalLength`: Must include HID Report Descriptor length (173 bytes)

### 2. HID Report Descriptor

**Critical: Exact 173-byte descriptor required**
```c
const HID_REPORT_DESCRIPTOR: &[u8] = &[
    // Usage Page (Consumer) - 0x05, 0x0c
    0x05, 0x0c,
    // Usage (Consumer Control) - 0x09, 0x01  
    0x09, 0x01,
    // Collection (Application) - 0xa1, 0x01
    0xa1, 0x01,
    // Usage (Consumer Control) - 0x09, 0x01
    0x09, 0x01,
    // Usage Page (Button) - 0x05, 0x09
    0x05, 0x09,
    // Usage Minimum (0x01) - 0x19, 0x01
    0x19, 0x01,
    // Usage Maximum (0x10) - 0x29, 0x10
    0x29, 0x10,
    // Logical Minimum (0) - 0x15, 0x00
    0x15, 0x00,
    // Logical Maximum (255) - 0x26, 0xff, 0x00
    0x26, 0xff, 0x00,
    // Report Size (8) - 0x75, 0x08
    0x75, 0x08,
    // Report Count (16) - 0x95, 0x10
    0x95, 0x10,
    // Report ID (0x01) - 0x85, 0x01
    0x85, 0x01,
    // Input (Data,Var,Abs) - 0x81, 0x02
    0x81, 0x02,
    // ... (continues for all 10 report IDs)
    // Report IDs: 0x01, 0x02, 0x03, 0x04, 0x05, 0x07, 0x0b, 0xa0, 0xa1, 0xa2
    // End Collection - 0xc0
    0xc0
];
```

**Total length: 173 bytes (0xAD) - MUST match exactly**

### 3. Feature Report Commands (GET_REPORT)

#### Firmware Version Requests

**Report ID 0xA1 (Primary):**
```
Host → Device: GET_REPORT Feature 0xA1, wLength=32
Device → Host: 32 bytes
Format: [0xa1, 0x0c, 0x31, 0x33, 0x00, "3.00.000", ...]
```

**Report ID 0x05 (Compatibility):**
```
Host → Device: GET_REPORT Feature 0x05, wLength=32
Device → Host: 32 bytes (same format as 0xA1)
Format: [0x05, 0x0c, 0x31, 0x33, 0x00, "3.00.000", ...]
```

**Report ID 0x04 (Legacy V1):**
```
Host → Device: GET_REPORT Feature 0x04, wLength=17
Device → Host: 17 bytes
Format: [0x04, 0x00, 0x00, 0x00, 0x00, "3.00.000", ...]
```

#### Serial Number Request

**Report ID 0x03:**
```
Host → Device: GET_REPORT Feature 0x03, wLength=32
Device → Host: 32 bytes
Format: [0x03, 0x0c, 0x31, 0x33, 0x00, "PRODUCTIONDK", ...]
```

### 4. Feature Report Commands (SET_REPORT)

#### Reset Commands

**Report ID 0x0B (V1 Legacy):**
```
Host → Device: SET_REPORT Feature 0x0B, 17 bytes
Data: [0x0b, 0x63, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
```

**Report ID 0x05 (V1 Reset):**
```
Host → Device: SET_REPORT Feature 0x05, 17 bytes
Data: [0x05, 0x55, 0xAA, 0xD1, 0x01, 0x3e, ...]
```

**Report ID 0x03 (V2 Reset):**
```
Host → Device: SET_REPORT Feature 0x03, 17 bytes
Data: [0x03, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
```

#### Brightness Commands

**Report ID 0x05 (V1 Brightness):**
```
Host → Device: SET_REPORT Feature 0x05, 17 bytes
Data: [0x05, 0x55, 0xAA, 0xD1, 0x01, brightness, ...]
```

**Report ID 0x03 (V2 Brightness):**
```
Host → Device: SET_REPORT Feature 0x03, 17 bytes
Data: [0x03, 0x08, brightness, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
```

### 5. Input Reports (Button States)

**Report ID 0x01:**
```
Device → Host: 16 bytes
Format: [0x01, button1, button2, button3, button4, button5, button6, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
```

### 6. Output Reports (Image Data)

**V2 Protocol (Recommended):**
```
Host → Device: SET_REPORT Output 0x02, 1024 bytes
Format: [0x02, 0x07, key_id, is_last, payload_len_low, payload_len_high, sequence_low, sequence_high, image_data...]
```

## Critical Implementation Details

### 1. Response Length Requirements

**32-byte responses are mandatory for:**
- GET_REPORT Feature 0x03 (Serial Number)
- GET_REPORT Feature 0xA1 (Firmware Version)
- GET_REPORT Feature 0x05 (Firmware Version - compatibility)

**12-byte or 16-byte responses will cause "Read FW version: FAILED"**

### 2. Report ID Conflicts

**Important:** Report ID 0x05 serves dual purposes:
- **GET_REPORT**: Firmware Version (32 bytes)
- **SET_REPORT**: Reset/Brightness commands

Ensure proper handling in both directions.

### 3. Embassy USB Implementation

**Key configuration:**
```rust
let hid_config = HidConfig {
    report_descriptor: HID_REPORT_DESCRIPTOR,
    request_handler: Some(handler),
    poll_ms: 1,
    max_packet_size: 64, // RP2040 hardware limitation
};
```

**Critical:** Use `HidReaderWriter::<_, 64, 64>::new()` for RP2040 compatibility.

### 4. Error Patterns

**Common failure modes:**
1. **"Read FW version: FAILED"**: Wrong response length or missing Report ID handlers
2. **"Problem with connecting HID device: -4"**: USB descriptor mismatch
3. **"Upload Image: FAILED"**: Output report handling not implemented

## Success Indicators

**When properly implemented, StreamDeck software logs show:**
```
Device connected, id: @(1)[4057/99/PRODUCTIONDK], serial number: PRODUCTIONDK, firmware version: 3.00.000, bcdDevice: 2.00
```

**Pico logs show:**
```
HID Get Report 0xA1: returning 32 bytes, version='3.00.000'
HID Get Report 0x03: returning 32 bytes, serial='PRODUCTIONDK'
```

## Implementation Checklist

- [ ] USB Device Descriptor matches real StreamDeck Mini
- [ ] HID Report Descriptor is exactly 173 bytes
- [ ] GET_REPORT Feature 0x03 returns 32 bytes with serial number
- [ ] GET_REPORT Feature 0xA1 returns 32 bytes with firmware version
- [ ] GET_REPORT Feature 0x05 returns 32 bytes with firmware version (compatibility)
- [ ] SET_REPORT Feature 0x05 handles both Reset and Brightness commands
- [ ] Input Report 0x01 sends button states
- [ ] Output Report 0x02 handles image data (for full functionality)

## References

- Real StreamDeck Mini USB traffic analysis via Wireshark
- Embassy USB documentation
- RP2040 USB hardware limitations
- StreamDeck software logs analysis

## Notes

This protocol analysis was validated through extensive debugging with actual StreamDeck software, ensuring 100% compatibility for device recognition and basic functionality. Image upload functionality requires additional Output Report implementation.
