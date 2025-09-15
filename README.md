# ProductionDeck - Open Source StreamDeck Alternative

![ProductionDeck](https://img.shields.io/badge/Platform-RP2040-green)
![Status](https://img.shields.io/badge/Status-Alpha-yellow)
![License](https://img.shields.io/badge/License-MIT-blue)

**ProductionDeck** is an open-source firmware library for creating StreamDeck-compatible devices using the Raspberry Pi Pico (RP2040). It provides full compatibility with official StreamDeck software by implementing exact USB HID protocols for multiple StreamDeck models.

## 🚀 Features

- **Multi-Device Support** - Supports 6 different StreamDeck models with dedicated firmware binaries
- **USB HID Protocol** - Exact implementation of Elgato's communication protocols (V1 BMP and V2 JPEG)
- **Device-Specific Binaries** - Compile-time optimized firmware for each model
- **Plug-and-Play** - Recognized as authentic StreamDeck devices by Windows/macOS
- **Open Source** - Complete firmware source code with modular architecture
- **RP2040 Based** - Uses the powerful dual-core Raspberry Pi Pico microcontroller

## 📱 Supported Devices

Build device-specific firmware using: `cargo run --bin <device-name>`

### StreamDeck Product Support Matrix

| Product | Keys | Display | USB Protocol | Binary Target | Status |
|---------|------|---------|--------------|---------------|--------|
| **StreamDeck Mini** | 6 (3x2) | 80x80px | V1 BMP | `mini` | ✅ Alpha |
| **StreamDeck Revised Mini** | 6 (3x2) | 80x80px | V1 BMP | `revised-mini` | ✅ Alpha |
| **StreamDeck Original** | 15 (5x3) | 72x72px | V1 BMP | `original` | ✅ Alpha |
| **StreamDeck Original V2** | 15 (5x3) | 72x72px | V2 JPEG | `original-v2` | ✅ Alpha |
| **StreamDeck XL** | 32 (8x4) | 96x96px | V2 JPEG | `xl` | ✅ Alpha |
| **StreamDeck Plus** | 8 (4x2) | 120x120px | V2 JPEG | `plus` | ✅ Alpha |
| **StreamDeck Pedal** | ❌ | ❌ | ❌ | Not implemented |
| **StreamDeck Studio** | ❌ | ❌ | ❌ | Not implemented |
| **StreamDeck Mobile** | N/A | N/A | N/A | Not Planned |
| **StreamDeck Module 6Keys** | 6 (3x2) | 80x80px | BMP | `module6` | ✅ Alpha |
| **StreamDeck Module 15Keys** | 15 (5x3) | 72x72px | JPEG | `module15` | ✅ Alpha |
| **StreamDeck Module 32Keys** | 32 (8x4) | 96x96px | JPEG | `module32` | ✅ Alpha |

### Implementation Status Legend
- ✅ **Fully implemented and working**
- ⚠️ **Implemented but disabled** (due to memory issues)
- ❌ **Not implemented**

### Current StreamDeck Mini Status
- **USB Protocol**: ✅ Complete HID implementation, device enumeration working
- **Button Input**: ✅ 6-button matrix scanning with debouncing
- **Display Output**: ⚠️ ST7735 driver implemented but disabled due to buffer memory issues
- **Software Compatibility**: ✅ Recognized as authentic StreamDeck Mini by official software

**Note**: Only StreamDeck Mini is currently targeted. Other StreamDeck variants require different USB protocols, button layouts, and display configurations.

## 📋 Hardware Requirements

### Core Components
- **Raspberry Pi Pico** (RP2040 microcontroller)
- **1x ST7735 TFT Display** (80x80 pixels, SPI interface) - shared by all buttons
- **6x Tactile Switches** 
- **Basic passive components** (resistors, capacitors)

### Optional Components
- **Status LEDs** (USB, Error, Status indication)
- **Enclosure** (3D printed or custom case)

## 🔌 Pin Assignments

### Critical GPIO Configuration
```
RP2040 Pin Layout (Raspberry Pi Pico):

USB Connection:
├── VID: 0x0fd9 (Elgato Systems)
├── PID: 0x0063 (StreamDeck Mini)
└── Protocol: USB HID

GPIO Assignments:
├── Buttons (Matrix Scan):
│   ├── ROW0: GP2  ┐
│   ├── ROW1: GP3  ├─ Button Matrix
│   ├── COL0: GP4  │  (3x2 layout)
│   ├── COL1: GP5  │
│   └── COL2: GP6  ┘
│
├── SPI Displays (SPI0):
│   ├── MOSI: GP19 (Data to displays)
│   ├── SCK:  GP18 (Clock)
│   ├── DC:   GP14 (Data/Command)
│   └── RST:  GP15 (Reset, shared)
│
├── Display CS (Chip Select):
│   └── DISPLAY: GP8  (Single shared display)
│
├── Control:
│   ├── Brightness: GP17 (PWM backlight control)
│   ├── Status LED: GP25 (Built-in LED)
│   ├── USB LED:    GP20 (Connection status)
│   └── Error LED:  GP21 (Error indication)
│
└── Debug:
    ├── UART TX: GP0 (Debug output)
    └── UART RX: GP1 (Debug input)
```

### Button Matrix Layout
```
Physical Button Layout:    GPIO Matrix:
┌─────┬─────┬─────┐       ROW0(GP2): BTN0 BTN1 BTN2
│ 0   │ 1   │ 2   │                  │    │    │
├─────┼─────┼─────┤       ROW1(GP3): BTN3 BTN4 BTN5
│ 3   │ 4   │ 5   │                  │    │    │
└─────┴─────┴─────┘                  │    │    │
                                   COL0  COL1 COL2
                                   (GP4) (GP5) (GP6)
```

## 🛠️ Building the Firmware

### Prerequisites
1. **Rust Toolchain** - Latest stable Rust (1.75+) with embedded target support
2. **elf2uf2-rs** - Tool for converting ELF to UF2 format
3. **flip-link** - Stack overflow protection for embedded Rust
4. **Git** - For cloning repositories

### Installation Steps

#### 1. Install Rust and Tools
```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Add embedded target for RP2040
rustup target add thumbv6m-none-eabi

# Install required tools
cargo install elf2uf2-rs
cargo install flip-link
```

#### 2. Optional Development Tools
**All Platforms:**
```bash
# Install additional helpful tools
cargo install cargo-audit     # Security audit
cargo install cargo-expand    # Macro expansion for debugging  
cargo install cargo-bloat     # Binary size analysis
```

#### 3. Build ProductionDeck
```bash
# Clone the repository  
git clone https://github.com/FlowingSPDG/productiondeck.git
cd productiondeck

# Check that dependencies compile
cargo check

# Build device-specific firmware
cargo build --release --bin mini        # For StreamDeck Mini
cargo build --release --bin xl          # For StreamDeck XL
cargo build --release --bin original    # For StreamDeck Original
# ... or any other device from the support matrix

# Build all devices at once (optional)
./build-devices.sh

# The UF2 files will be at: target/thumbv6m-none-eabi/release/<device-name>.uf2
```

### Build Output
After successful build, you'll find these files in `target/thumbv6m-none-eabi/release/`:
- `mini` - ELF executable for Mini (with debug symbols)  
- `mini.uf2` - Mini firmware file for flashing
- `xl` - ELF executable for XL (with debug symbols)
- `xl.uf2` - XL firmware file for flashing
- ... (and similarly for other devices)

Each device gets its own optimized binary with compile-time device selection.
The UF2 files are automatically generated thanks to the runner configuration in `.cargo/config.toml`.

## 📱 Flashing the Firmware

### Method 1: UF2 Bootloader (Recommended)
1. **Enter Bootloader Mode:**
   - Hold the **BOOTSEL** button on the Pico
   - Connect USB cable to computer
   - Release **BOOTSEL** button
   - Pico appears as `RPI-RP2` drive

2. **Flash Firmware:**
   ```bash
   # Copy UF2 file to the Pico
   cp target/thumbv6m-none-eabi/release/productiondeck.uf2 /Volumes/RPI-RP2/  # macOS
   cp target/thumbv6m-none-eabi/release/productiondeck.uf2 /media/RPI-RP2/    # Linux
   # On Windows: drag productiondeck.uf2 to RPI-RP2 drive
   ```

3. **Automatic Reboot:**
   - Pico automatically reboots with new firmware
   - Should appear as "Stream Deck Mini" in Device Manager

### Method 2: Cargo Run (with debug probe)
```bash
# Using cargo run with configured runner (requires debug probe setup)
cargo run --release

# Alternative: Use probe-rs for advanced debugging (install first: cargo install probe-rs-tools)
probe-rs run --chip RP2040 target/thumbv6m-none-eabi/release/productiondeck
```

## 💻 Software Setup

### Windows
1. Flash the firmware to your Pico
2. Install [Stream Deck Software](https://www.elgato.com/gaming/stream-deck)
3. Connect ProductionDeck via USB
4. Should be recognized as "Stream Deck Mini"
5. Configure keys in Stream Deck software

### macOS/Linux
Same as Windows - the device uses standard USB HID drivers.

## 🔧 Hardware Assembly

### Wiring Instructions
Since this is currently a firmware-only project, you'll need to wire the components manually:

### Display Connections
Single ST7735 display connects via SPI (shared by all 6 buttons):
```
ST7735 Display → RP2040
VCC  → 3.3V
GND  → GND
SCL  → GP18 (SCK)
SDA  → GP19 (MOSI)
RES  → GP15 (RST)
DC   → GP14 (Data/Command)
CS   → GP8 (Chip Select)
BLK  → GP17 (PWM backlight control)
```

### Button Matrix
Simple tactile switch matrix:
```
Button connections:
BTN0: ROW0(GP2) ↔ COL0(GP4)
BTN1: ROW0(GP2) ↔ COL1(GP5)
BTN2: ROW0(GP2) ↔ COL2(GP6)
BTN3: ROW1(GP3) ↔ COL0(GP4)
BTN4: ROW1(GP3) ↔ COL1(GP5)
BTN5: ROW1(GP3) ↔ COL2(GP6)
```

## 🐛 Debugging and Development

### Serial Debug Output
Connect UART to see debug messages:
```bash
# Linux/macOS
screen /dev/ttyUSB0 115200

# Windows
# Use PuTTY or similar terminal program
# Port: COM port of Pico
# Baud: 115200
```

### Debug Levels
Debug output is controlled via the `DEFMT_LOG` environment variable in `.cargo/config.toml`:
- Set to `debug` for detailed logging
- Set to `info` for basic information  
- Set to `warn` for warnings only
- Set to `off` to disable logging

You can also set the log level temporarily:
```bash
DEFMT_LOG=debug cargo build --release
```

### Common Issues

#### Device Not Recognized
1. Check USB VID/PID in device manager
2. Ensure firmware flashed correctly
3. Try different USB cable/port

#### Displays Not Working
1. Check SPI connections
2. Verify power supply (3.3V)
3. Test individual displays

#### Buttons Not Responding
1. Check button matrix wiring
2. Verify pull-up resistors
3. Check debounce timing

### Development Tools
```bash
# Format code according to Rust standards
cargo fmt

# Check code quality with Clippy
cargo clippy

# Clean build artifacts
cargo clean

# Check compilation without building
cargo check

# Build and flash (with debug probe configured)
cargo run --release
```

## 📚 Protocol Documentation

ProductionDeck implements the exact StreamDeck Mini USB HID protocol:

### USB Device Descriptors
- **VID:** `0x0fd9` (Elgato Systems)
- **PID:** `0x0063` (StreamDeck Mini)
- **Class:** HID (Human Interface Device)

### Report Structure
- **Input Reports:** Button states (6 bytes)
- **Output Reports:** Image data (1024 bytes per packet)
- **Feature Reports:** Commands (version, reset, brightness)

### Image Protocol
```
V2 Image Packet (1024 bytes):
[0x02][0x07][key_id][is_last][len_low][len_high][seq_low][seq_high][image_data...]
│     │     │       │        │       │         │       │         └─ Image payload
│     │     │       │        │       │         │       └─ Sequence high byte
│     │     │       │        │       │         └─ Sequence low byte
│     │     │       │        │       └─ Payload length high byte
│     │     │       │        └─ Payload length low byte
│     │     │       └─ Last packet flag (1=final, 0=more)
│     │     └─ Key ID (0-5)
│     └─ Image command (0x07)
└─ Report ID (0x02)
```

### Tech Stack
- **Language**: Rust 2021 Edition
- **Framework**: Embassy async framework for embedded
- **USB Stack**: Embassy USB with HID support
- **Graphics**: embedded-graphics with ST7735 driver
- **Target**: thumbv6m-none-eabi (Cortex-M0+)

## 🤝 Contributing

We welcome contributions! Please see:
- [Issues](https://github.com/FlowingSPDG/productiondeck/issues) for bug reports
- [Discussions](https://github.com/FlowingSPDG/productiondeck/discussions) for questions

### Development Setup
1. Fork the repository
2. Create feature branch: `git checkout -b feature-name`
3. Make changes and test thoroughly
4. Submit pull request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## ⚠️ Legal Notice

This project is not affiliated with Elgato Systems. StreamDeck is a trademark of Elgato Systems. This project implements a compatible device through reverse engineering for educational and interoperability purposes.

## 🙏 Acknowledgments

- [rust-streamdeck](https://github.com/ryankurte/rust-streamdeck) - Protocol reference
- [Raspberry Pi Foundation](https://www.raspberrypi.org/) - RP2040 microcontroller
- [TinyUSB](https://github.com/hathach/tinyusb) - USB stack
- StreamDeck reverse engineering community

## 📞 Support

- **Firmware Issues:** [Submit Issue](https://github.com/FlowingSPDG/productiondeck/issues)
- **Questions:** [GitHub Discussions](https://github.com/FlowingSPDG/productiondeck/discussions)

---

**Made with ❤️ for the maker community**

*Build your own StreamDeck and join the open hardware revolution!*