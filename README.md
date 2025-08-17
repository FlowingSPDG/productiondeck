# OpenDeck - Open Source StreamDeck Alternative

![OpenDeck](https://img.shields.io/badge/Platform-RP2040-green)
![Status](https://img.shields.io/badge/Status-Alpha-yellow)
![License](https://img.shields.io/badge/License-MIT-blue)

**OpenDeck** is an open-source hardware and software implementation of a StreamDeck-compatible device using the Raspberry Pi Pico (RP2040). It provides full compatibility with the official StreamDeck software by implementing the exact USB HID protocol used by Elgato StreamDeck Mini.

## 🚀 Features

- **Full StreamDeck Mini Compatibility** - Works with official StreamDeck software
- **USB HID Protocol** - Exact implementation of Elgato's communication protocol
- **6 Programmable Keys** - 3x2 button layout with individual TFT displays
- **80x80 Pixel Displays** - Full-color LCD displays for each key
- **Plug-and-Play** - Recognized as authentic StreamDeck Mini by Windows/macOS/Linux
- **Open Source** - Complete hardware design and firmware source code
- **RP2040 Based** - Uses the powerful dual-core Raspberry Pi Pico microcontroller

## 📋 Hardware Requirements

### Core Components
- **Raspberry Pi Pico** (RP2040 microcontroller)
- **6x ST7735 TFT Displays** (80x80 pixels, SPI interface)
- **6x Tactile Switches** (for button matrix)
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
│   ├── KEY0: GP8  ┐
│   ├── KEY1: GP9  │
│   ├── KEY2: GP10 ├─ Individual display selection
│   ├── KEY3: GP11 │
│   ├── KEY4: GP12 │
│   └── KEY5: GP13 ┘
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
1. **Pico SDK** - Raspberry Pi Pico SDK installed and configured
2. **CMake** - Version 3.13 or higher
3. **ARM GCC Toolchain** - arm-none-eabi-gcc
4. **Git** - For cloning repositories

### Installation Steps

#### 1. Install Pico SDK
```bash
# Clone Pico SDK
git clone https://github.com/raspberrypi/pico-sdk.git
cd pico-sdk
git submodule update --init

# Set environment variable
export PICO_SDK_PATH=/path/to/pico-sdk
```

#### 2. Install Build Tools
**Ubuntu/Debian:**
```bash
sudo apt update
sudo apt install cmake gcc-arm-none-eabi libnewlib-arm-none-eabi 
sudo apt install build-essential libstdc++-arm-none-eabi-newlib
```

**macOS (with Homebrew):**
```bash
brew install cmake
brew tap ArmMbed/homebrew-formulae
brew install arm-none-eabi-gcc
```

**Windows:**
- Install [CMake](https://cmake.org/download/)
- Install [ARM GCC Toolchain](https://developer.arm.com/tools-and-software/open-source-software/developer-tools/gnu-toolchain/gnu-rm/downloads)

#### 3. Build OpenDeck
```bash
# Clone the repository
git clone https://github.com/your-username/opendeck.git
cd opendeck

# Create build directory
mkdir build
cd build

# Configure with CMake
cmake .. -DPICO_SDK_PATH=/path/to/pico-sdk

# Build the firmware
make -j4

# Build info (optional)
make info
```

### Build Output
After successful build, you'll find these files in `build/output/`:
- `opendeck.uf2` - Main firmware file for flashing
- `opendeck.bin` - Binary file
- `opendeck.hex` - Hex file
- `opendeck.elf` - ELF file with debug symbols

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
   cp build/output/opendeck.uf2 /Volumes/RPI-RP2/  # macOS
   cp build/output/opendeck.uf2 /media/RPI-RP2/    # Linux
   # On Windows: drag opendeck.uf2 to RPI-RP2 drive
   ```

3. **Automatic Reboot:**
   - Pico automatically reboots with new firmware
   - Should appear as "Stream Deck Mini" in Device Manager

### Method 2: Debug Probe (Advanced)
```bash
# Using OpenOCD with debug probe
openocd -f interface/picoprobe.cfg -f target/rp2040.cfg -c "program build/opendeck.elf verify reset exit"
```

## 💻 Software Setup

### Windows
1. Flash the firmware to your Pico
2. Install [Stream Deck Software](https://www.elgato.com/gaming/stream-deck)
3. Connect OpenDeck via USB
4. Should be recognized as "Stream Deck Mini"
5. Configure keys in Stream Deck software

### macOS/Linux
Same as Windows - the device uses standard USB HID drivers.

## 🔧 Hardware Assembly

### PCB Design
The project includes KiCad PCB files for a complete board design:
- **Schematic:** `hardware/opendeck.sch`
- **PCB Layout:** `hardware/opendeck.kicad_pcb`
- **Bill of Materials:** `hardware/BOM.csv`

### Display Connections
Each ST7735 display connects via SPI:
```
ST7735 Display → RP2040
VCC  → 3.3V
GND  → GND
SCL  → GP18 (SCK)
SDA  → GP19 (MOSI)
RES  → GP15 (RST, shared)
DC   → GP14 (Data/Command, shared)
CS   → GP8-GP13 (individual per display)
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
Modify `DEBUG_LEVEL` in `include/opendeck_config.h`:
- `0` - No debug output
- `1` - Info messages only
- `2` - Verbose debug output

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
# Format code
make format

# Clean build
make clean-all

# Flash firmware
make flash
```

## 📚 Protocol Documentation

OpenDeck implements the exact StreamDeck Mini USB HID protocol:

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

## 🤝 Contributing

We welcome contributions! Please see:
- [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines
- [Issues](https://github.com/your-username/opendeck/issues) for bug reports
- [Discussions](https://github.com/your-username/opendeck/discussions) for questions

### Development Setup
1. Fork the repository
2. Create feature branch: `git checkout -b feature-name`
3. Make changes and test thoroughly
4. Submit pull request

## 📄 License

This project is licensed under the MIT License - see [LICENSE](LICENSE) file for details.

## ⚠️ Legal Notice

This project is not affiliated with Elgato Systems. StreamDeck is a trademark of Elgato Systems. This project implements a compatible device through reverse engineering for educational and interoperability purposes.

## 🙏 Acknowledgments

- [rust-streamdeck](https://github.com/ryankurte/rust-streamdeck) - Protocol reference
- [Raspberry Pi Foundation](https://www.raspberrypi.org/) - RP2040 microcontroller
- [TinyUSB](https://github.com/hathach/tinyusb) - USB stack
- StreamDeck reverse engineering community

## 📞 Support

- **Documentation:** [Wiki](https://github.com/your-username/opendeck/wiki)
- **Hardware Help:** [Hardware Guide](hardware/README.md)
- **Firmware Issues:** [Submit Issue](https://github.com/your-username/opendeck/issues)
- **Community:** [Discord Server](https://discord.gg/opendeck)

---

**Made with ❤️ for the maker community**

*Build your own StreamDeck and join the open hardware revolution!*