# Development Environment

## Current System
- **Platform**: Linux (WSL2 on Windows)
- **Shell**: Bash-compatible
- **Git**: Available and configured
- **Cargo**: Version 1.89.0 (latest stable)

## Required Tools

### Rust Toolchain
- **Target**: thumbv6m-none-eabi (Cortex-M0+ for RP2040)
- **Toolchain**: Stable Rust with embedded target support

### Additional Tools
- `elf2uf2-rs` - Converts ELF to UF2 format for Pico flashing
- `flip-link` - Stack overflow protection for embedded Rust
- ARM GCC toolchain (for some dependencies)

## Installation Commands
```bash
# Add embedded Rust target
rustup target add thumbv6m-none-eabi

# Install required tools
cargo install elf2uf2-rs
cargo install flip-link

# Useful additional tools
cargo install cargo-audit     # Security audit
cargo install cargo-expand    # Macro expansion
```

## IDE Integration
Project supports:
- **VS Code**: With rust-analyzer extension
- **CLion**: With Rust plugin
- **Any editor**: With Language Server Protocol support

Recommended VS Code extensions:
- rust-analyzer
- Better TOML
- GitLens
- Error Lens

## Hardware Requirements
For full development:
- **Raspberry Pi Pico** (RP2040)
- **6x ST7735 TFT displays** (80x80 pixels)
- **Tactile switches** for button matrix
- **Breadboard/PCB** for prototyping
- **USB cable** for programming and power

## Current Development Blockers
1. **Dependency Conflict**: byteorder version mismatch prevents compilation
2. **USB HID Dependencies**: usbd-hid version conflicts with embedded-graphics
3. **Build System**: Project cannot currently be built until dependencies are resolved

## Debug Output
- **RTT (Real-Time Transfer)**: Via defmt-rtt for debug logging
- **UART**: GP0/GP1 pins at 115200 baud for serial debugging
- **USB**: Protocol debugging via HID reports

## Memory Constraints
- **Flash**: 2048KB - 256 bytes (for boot2)
- **RAM**: 256KB total
- **Optimization**: Release builds use size optimization (-Os)