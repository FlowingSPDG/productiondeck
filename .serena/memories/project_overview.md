# ProductionDeck Project Overview

## Purpose
ProductionDeck is an open-source RP2040-based StreamDeck Mini alternative that provides full compatibility with official StreamDeck software by implementing the exact USB HID protocol. The device features 6 programmable keys with individual 80x80 pixel TFT displays.

## Project Status
- **Current State**: Project has been migrated from C++ to Rust using Embassy framework
- **Architecture**: RP2040 dual-core ARM Cortex-M0+ microcontroller
- **Compatibility**: USB HID compatible with StreamDeck Mini (VID: 0x0fd9, PID: 0x0063)
- **Hardware**: 6x ST7735 TFT displays via SPI, 3x2 button matrix

## Key Features
- Full StreamDeck Mini compatibility
- USB HID protocol implementation
- 6 programmable keys with individual displays
- 80x80 pixel full-color LCD displays
- Plug-and-play recognition as authentic StreamDeck Mini
- Open source hardware design and firmware
- RP2040 dual-core architecture

## Recent Changes
- Migrated from C++ to Rust using Embassy async framework
- Moved from CMake build system to Cargo
- Updated hardware abstraction layer for Embassy
- Currently has dependency version conflicts that need resolution