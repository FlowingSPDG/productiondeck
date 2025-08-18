# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

ProductionDeck is an open-source RP2040-based StreamDeck Mini alternative that provides full compatibility with official StreamDeck software by implementing the exact USB HID protocol. The device features 6 programmable keys with individual 80x80 pixel TFT displays.

## Build Commands

### Primary Build Process
- `./build.sh` - Main build script (recommended)
- `./build.sh clean` - Clean and rebuild from scratch  
- `./build.sh flash` - Build and attempt to flash to connected Pico
- `./build.sh info` - Show build information

### Manual CMake Build
```bash
mkdir build && cd build
cmake .. -DPICO_SDK_PATH=/path/to/pico-sdk
make -j4
```

### Build Targets
- `make info` - Display build configuration
- `make flash` - Show flashing instructions  
- `make clean-all` - Remove all build files
- `make format` - Format code (mentioned in README)

## Prerequisites

1. **Pico SDK** - Must be installed and `PICO_SDK_PATH` environment variable set
2. **CMake** 3.13 or higher
3. **ARM GCC Toolchain** - arm-none-eabi-gcc

## Architecture Overview

### Hardware Configuration
- **Target**: Raspberry Pi Pico (RP2040 dual-core ARM Cortex-M0+)
- **USB Identity**: VID 0x0fd9 (Elgato), PID 0x0063 (StreamDeck Mini)
- **Displays**: 6x ST7735 TFT (80x80 pixels each) via SPI
- **Buttons**: 3x2 matrix scan or direct GPIO
- **Protocol**: USB HID compatible with StreamDeck Mini

### Core Architecture
- **Dual-core design**: Core 0 handles USB/protocol, Core 1 handles displays/buttons
- **USB HID interface**: Exact StreamDeck Mini protocol implementation
- **Real-time constraints**: 1ms USB polling, 100Hz button scanning
- **Hardware abstraction**: Configurable pin assignments via `productiondeck_config.h`

### Key Files
- `src/main.cpp` - Application entry point and initialization
- `include/productiondeck_config.h` - Hardware configuration and pin assignments
- `src/productiondeck.cpp` - Main device logic and protocol handling
- `src/usb_descriptors.cpp` - USB HID descriptors for StreamDeck compatibility
- `src/hardware.cpp` - Low-level hardware control (displays, buttons, GPIO)

### Pin Assignments (Critical for Hardware)
- **SPI Displays**: MOSI(GP19), SCK(GP18), DC(GP14), RST(GP15)
- **Display CS**: GP8-GP13 (individual chip selects)
- **Button Matrix**: Rows(GP2,GP3), Cols(GP4,GP5,GP6)
- **Control**: Backlight(GP17), Status LED(GP25)

### USB Protocol Implementation
The device implements StreamDeck Mini's exact USB HID protocol:
- Input reports: Button states (32 bytes)
- Output reports: Image data packets (1024 bytes)
- Feature reports: Commands (brightness, reset, version)

### Development Notes
- Uses TinyUSB stack for USB functionality
- Hardware watchdog enabled for stability (8s timeout)
- Debug output via UART (GP0/GP1) at 115200 baud
- Build outputs: `.uf2` (main firmware), `.bin`, `.hex`, `.elf`
- No test framework currently implemented
- Debug levels: 0=None, 1=Info, 2=Verbose (set in `productiondeck_config.h`)

### Critical Configuration
All USB descriptors and protocol handling must exactly match StreamDeck Mini for software compatibility. Changes to VID/PID or protocol structure will break compatibility with official StreamDeck software.