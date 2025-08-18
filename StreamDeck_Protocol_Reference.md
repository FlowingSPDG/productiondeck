# StreamDeck USB HID Protocol Reference

## Critical Protocol Details Extracted from rust-streamdeck

This document provides exact packet structures and protocol details needed for USB device implementation.

## ProductionDeck Implementation Notes

**ProductionDeck** implements the StreamDeck Mini protocol (PID: 0x0063) using:
- **Language**: Rust with Embassy async framework
- **Target**: RP2040 (Raspberry Pi Pico)
- **USB Stack**: Embassy USB with usbd-hid
- **Protocol Version**: V1 (BMP format, BGR color order)
- **Display**: Single shared 80x80 ST7735 TFT (all 6 buttons display on same screen)

The protocol implementation in `src/usb.rs` follows these exact specifications.

## USB Device Configuration

### Required Identifiers
```
Vendor ID:  0x0fd9 (Elgato Systems)
Product IDs:
  0x0060 - Original (15 keys, 72x72, BMP, V1 protocol)
  0x0063 - Mini (6 keys, 80x80, BMP, V1 protocol)
  0x006d - Original V2 (15 keys, 72x72, JPEG, V2 protocol)
  0x006c - XL (32 keys, 96x96, JPEG, V2 protocol)
  0x0080 - MK2 (15 keys, 72x72, JPEG, V2 protocol)
  0x0090 - Revised Mini (6 keys, 80x80, BMP, V1 protocol)
  0x0084 - Plus (8 keys, 120x120, JPEG, V2 protocol)
```

## Feature Report Commands

### Version Request
**Host → Device**
```
V1: [0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
V2: [0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
```

**Device → Host Response**
```
V1: [0x04, 0x??, 0x??, 0x??, 0x??, version_string...]  (offset 5)
V2: [0x05, 0x??, 0x??, 0x??, 0x??, 0x??, version_string...]  (offset 6)
```

### Reset Command
**Host → Device**
```
V1: [0x0b, 0x63, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
V2: [0x03, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
```

### Brightness Control
**Host → Device**
```
V1: [0x05, 0x55, 0xaa, 0xd1, 0x01, brightness, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
V2: [0x03, 0x08, brightness, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
```
Where `brightness` = 0-100 (percentage)

## Output Report - Image Data

### V2 Protocol (Recommended)
**Report Structure** (1024 bytes total)
```
[0x02, 0x07, key_id, is_last, payload_len_low, payload_len_high, sequence_low, sequence_high, image_data...]
```

**Field Details:**
- `0x02` - Report ID (constant)
- `0x07` - Image command (constant)
- `key_id` - Button index (0-based)
- `is_last` - 1 if final packet, 0 otherwise
- `payload_len` - Little-endian length of image data in this packet
- `sequence` - Little-endian packet sequence number (starts at 0)
- `image_data` - JPEG image data (up to 1016 bytes per packet)

### V1 Protocol (Original)
**Report Structure** (8191 bytes total)
```
Packet 1: [0x02, 0x01, 0x01, 0x00, 0x00, key_id, 0x00...0x00, BMP_header, image_data_7749_bytes]
Packet 2: [0x02, 0x01, 0x02, 0x00, 0x01, key_id, 0x00...0x00, remaining_image_data_7803_bytes]
```

**BMP Header for V1 Devices** (54 bytes)
```cpp
const uint8_t BMP_HEADER_ORIGINAL[54] = {
    0x42, 0x4d, 0xf6, 0x3c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x36, 0x00, 0x00, 0x00, 0x28, 0x00,
    0x00, 0x00, 0x48, 0x00, 0x00, 0x00, 0x48, 0x00, 0x00, 0x00, 0x01, 0x00, 0x18, 0x00, 0x00, 0x00,
    0x00, 0x00, 0xc0, 0x3c, 0x00, 0x00, 0xc4, 0x0e, 0x00, 0x00, 0xc4, 0x0e, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00
};

const uint8_t BMP_HEADER_MINI[54] = {
    0x42, 0x4d, 0xf6, 0x3c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x36, 0x00, 0x00, 0x00, 0x28, 0x00,
    0x00, 0x00, 0x50, 0x00, 0x00, 0x00, 0x50, 0x00, 0x00, 0x00, 0x01, 0x00, 0x18, 0x00, 0x00, 0x00,
    0x00, 0x00, 0xc0, 0x3c, 0x00, 0x00, 0xc4, 0x0e, 0x00, 0x00, 0xc4, 0x0e, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00
};
```

## Input Report - Button States

### Report Format
```
Device → Host (continuous polling)
```

**V1 Devices (Original, Mini)**
```
[button_0, button_1, button_2, ..., button_n]
```

**V2 Devices (All others)**
```
[0x??, 0x??, 0x??, button_0, button_1, button_2, ..., button_n]
```
3-byte header followed by button states

### Button State Values
- `0x00` - Button released
- `0x01` - Button pressed

### Button Mapping

**Original StreamDeck** (Right-to-Left mapping)
```
Physical Layout:     Data Array Index:
[01][02][03][04][05]    [05][04][03][02][01]
[06][07][08][09][10] -> [10][09][08][07][06]
[11][12][13][14][15]    [15][14][13][12][11]
```

**All Other Devices** (Left-to-Right mapping)
```
Physical Layout:     Data Array Index:
[01][02][03]           [00][01][02]
[04][05][06]        -> [03][04][05]
```

### Device Specifications

| Device | Keys | Image Size | Format | Color Order | Key Layout | Protocol |
|--------|------|------------|--------|-------------|------------|----------|
| Original | 15 | 72x72 | BMP | BGR | 5x3 (R→L) | V1 |
| Mini | 6 | 80x80 | BMP | BGR | 3x2 (L→R) | V1 |
| Original V2 | 15 | 72x72 | JPEG | RGB | 5x3 (L→R) | V2 |
| XL | 32 | 96x96 | JPEG | RGB | 8x4 (L→R) | V2 |
| MK2 | 15 | 72x72 | JPEG | RGB | 5x3 (L→R) | V2 |
| Revised Mini | 6 | 80x80 | BMP | BGR | 3x2 (L→R) | V1 |
| Plus | 8 | 120x120 | JPEG | RGB | 4x2 (L→R) | V2 |

## Image Processing Requirements

### V1 Devices (BMP Format)
1. Expect BMP header (54 bytes) + RGB pixel data
2. Color order: BGR (swap R and B channels)
3. Image dimensions must match device specifications
4. Total data: header + (width × height × 3) bytes

### V2 Devices (JPEG Format)  
1. Receive JPEG data directly
2. Color order: RGB (no conversion needed)
3. Decode JPEG to get pixel data for display
4. Support progressive JPEG if needed

### Image Transformations
- **Mini devices**: Rotate image 270° clockwise
- **Original**: Mirror horizontally (flip Y-axis)
- **Original V2/XL/MK2**: Mirror both axes
- **Plus**: No transformation needed

## Implementation Checklist

### Essential USB Functionality
- [x] Implement correct VID/PID for target device
- [x] Handle version feature report (0x04/0x05)
- [x] Handle reset feature report (0x0b/0x03)
- [x] Handle brightness feature report (0x05/0x03)
- [x] Process image output reports (0x02)
- [x] Send button input reports continuously

### Image Processing
- [x] Parse image headers correctly
- [x] Reassemble multi-packet images
- [x] Handle BMP headers for V1 devices
- [x] Apply device-specific transformations
- [x] Convert color order (BGR ↔ RGB)

### Hardware Integration
- [x] Read physical button matrix
- [x] Display images on individual key LCDs
- [x] Implement brightness control
- [x] Handle USB connection/disconnection

This protocol reference provides everything needed to implement a fully compatible StreamDeck USB device that works with official software.