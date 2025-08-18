# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

ProductionDeck is an open-source RP2040-based StreamDeck Mini alternative implemented in Rust using the Embassy async framework. It provides full compatibility with official StreamDeck software by implementing the exact USB HID protocol. The device features 6 programmable keys with a shared 80x80 pixel TFT display.

## Build Commands

### Primary Build Process
- `cargo check` - Quick compilation check without building
- `cargo build` - Build in debug mode
- `cargo build --release` - Build in release mode (recommended for embedded)
- `cargo fmt` - Format code according to Rust standards
- `cargo clippy` - Run linter for code quality

### Development Commands
```bash
# Clean build artifacts
cargo clean

# Update dependencies (use with caution)
cargo update

# Generate documentation
cargo doc --open
```

### Build Targets
- `cargo check` - Fast compilation check
- `cargo fmt` - Code formatting
- `cargo clippy` - Code linting
- `cargo build --release` - Release build with optimizations

## Prerequisites

1. **Rust Toolchain** - Latest stable Rust (1.75+ recommended)
2. **thumbv6m-none-eabi target** - `rustup target add thumbv6m-none-eabi`
3. **elf2uf2-rs** - `cargo install elf2uf2-rs` (for UF2 conversion)
4. **flip-link** - `cargo install flip-link` (stack overflow protection)

## Architecture Overview

### Hardware Configuration
- **Target**: Raspberry Pi Pico (RP2040 dual-core ARM Cortex-M0+)
- **USB Identity**: VID 0x0fd9 (Elgato), PID 0x0063 (StreamDeck Mini)
- **Display**: 1x ST7735 TFT (80x80 pixels) shared by all buttons via SPI
- **Buttons**: 3x2 matrix scan or direct GPIO
- **Protocol**: USB HID compatible with StreamDeck Mini

### Core Architecture
- **Async Embassy framework**: Modern Rust async/await for embedded
- **Dual-core design**: Core 0 handles USB/protocol, Core 1 handles displays/buttons
- **USB HID interface**: Exact StreamDeck Mini protocol implementation
- **Channel communication**: Embassy channels for inter-task communication
- **Hardware abstraction**: Configurable pin assignments via `config.rs`

### Key Files
- `src/main.rs` - Application entry point and task coordination
- `src/config.rs` - Hardware configuration constants and pin assignments
- `src/usb.rs` - USB HID implementation and StreamDeck protocol handling
- `src/display.rs` - Display handling and graphics rendering
- `src/buttons.rs` - Button scanning and debouncing logic
- `Cargo.toml` - Rust project manifest and dependencies
- `.cargo/config.toml` - Build configuration and target settings
- `memory.x` - RP2040 memory layout for linker

### Pin Assignments (Critical for Hardware)
- **SPI Display**: MOSI(GP19), SCK(GP18), DC(GP14), RST(GP15)
- **Display CS**: GP8 (single chip select for shared display)
- **Button Matrix**: Rows(GP2,GP3), Cols(GP4,GP5,GP6)
- **Control**: Backlight(GP17), Status LED(GP25)

### USB Protocol Implementation
The device implements StreamDeck Mini's exact USB HID protocol:
- Input reports: Button states (6 bytes for 6 keys)
- Output reports: Image data packets (1024 bytes)
- Feature reports: Commands (brightness, reset, version)
- Embassy USB stack with usbd-hid for HID functionality

### Development Notes
- Uses Embassy USB stack for USB functionality
- Async/await pattern throughout for better resource utilization
- Debug output via RTT (Real-Time Transfer) with defmt logging
- Build outputs: `.uf2` (main firmware), ELF with debug symbols
- No test framework currently implemented
- Debug levels controlled via `DEFMT_LOG` environment variable
- Memory layout defined in `memory.x` for RP2040 boot2 section

### Critical Configuration
All USB descriptors and protocol handling must exactly match StreamDeck Mini for software compatibility. Changes to VID/PID or protocol structure will break compatibility with official StreamDeck software.

### Current Status
- **Version**: v0.1.0
- **Status**: Development/Alpha - Code compilation needs fixes
- **Dependencies**: Resolved version conflicts, Embassy framework updated
- **Known Issues**: Embassy API migration in progress, some code needs updating for latest Embassy versions