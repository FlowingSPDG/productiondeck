# ProductionDeck Tech Stack

## Core Technology
- **Language**: Rust (Edition 2021)
- **Framework**: Embassy async framework for embedded development
- **Target**: RP2040 (thumbv6m-none-eabi)
- **Runtime**: Cortex-M runtime with critical section support

## Key Dependencies

### Embassy Framework
- `embassy-rp` - RP2040 hardware abstraction
- `embassy-usb` - USB device stack
- `embassy-time` - Async timing and delays
- `embassy-executor` - Async task executor
- `embassy-futures` - Future utilities
- `embassy-sync` - Synchronization primitives

### Hardware Abstraction
- `embedded-hal` / `embedded-hal-async` - Hardware abstraction layer
- `embedded-hal-bus` - Bus sharing utilities
- `cortex-m` / `cortex-m-rt` - Cortex-M runtime

### Display and Graphics
- `st7735-lcd` - ST7735 TFT display driver
- `embedded-graphics` - 2D graphics library
- `display-interface` / `display-interface-spi` - Display abstraction

### USB and HID
- `usbd-hid` - USB HID device implementation (currently has version conflicts)

### Utilities
- `heapless` - No-allocation collections
- `defmt` / `defmt-rtt` - Logging and debugging
- `panic-halt` - Panic handler

## Build Configuration
- **Target**: thumbv6m-none-eabi (Cortex-M0+)
- **Runner**: elf2uf2-rs for UF2 conversion
- **Linker**: flip-link with custom memory layout
- **Memory Layout**: Custom memory.x for RP2040 with boot2 section

## Current Issues
- Dependency version conflict between `byteorder` versions used by `embedded-graphics` (1.4.3) and `usbd-hid` (1.5.0)
- Project currently fails to compile due to this conflict