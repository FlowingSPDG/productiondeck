# ProductionDeck Codebase Structure

## Root Directory Layout
```
productiondeck/
├── .cargo/config.toml          - Cargo build configuration (thumbv6m-none-eabi target)
├── .gitignore                  - Git ignore patterns
├── Cargo.toml                  - Rust project manifest and dependencies
├── Cargo.lock                  - Dependency version lock file
├── memory.x                    - RP2040 memory layout for linker
├── CLAUDE.md                   - Project instructions for AI assistance
├── README.md                   - Comprehensive project documentation
├── StreamDeck_Protocol_Reference.md    - Protocol documentation
├── StreamDeck_USB_Implementation.md    - USB implementation details
└── src/                        - Source code directory
```

## Source Code Organization (`src/`)
```
src/
├── main.rs       - Application entry point, task coordination, main executor
├── config.rs     - Hardware configuration constants and pin assignments
├── usb.rs        - USB HID implementation and StreamDeck protocol handling
├── display.rs    - ST7735 display driver and graphics rendering
└── buttons.rs    - Button matrix scanning and debouncing logic
```

## Key Modules and Responsibilities

### `main.rs`
- Embassy executor initialization
- Task spawning and coordination
- Channel setup for inter-task communication
- Core 0/Core 1 task distribution
- Main application entry point

### `config.rs`
- USB device descriptors (VID/PID for StreamDeck compatibility)
- Hardware pin assignments (SPI, buttons, displays)
- Protocol constants (report sizes, commands)
- Timing and performance parameters
- Debug and build configuration options

### `usb.rs`
- USB HID device implementation
- StreamDeck Mini protocol handling
- HID report descriptors
- USB request/response processing
- Feature report handling (version, brightness, reset)

### `display.rs`
- ST7735 TFT display driver integration
- Graphics rendering and buffering
- Image data processing from USB
- Display initialization and control
- Per-key display management (6 displays)

### `buttons.rs`
- Button matrix scanning (3x2 layout)
- Debouncing logic
- Button state management
- Hardware pin multiplexing for rows/columns

## Build Configuration Files

### `.cargo/config.toml`
- Target specification: thumbv6m-none-eabi
- Custom runner: elf2uf2-rs for UF2 conversion
- Linker flags for RP2040
- Environment variables (DEFMT_LOG)

### `memory.x`
- RP2040-specific memory layout
- Boot2 section for second-stage bootloader
- Flash and RAM memory regions
- Required for proper RP2040 firmware generation

## Communication Architecture
- **Channel-based**: Embassy channels for inter-task communication
- **Dual-core**: Core 0 (USB/protocol) and Core 1 (displays/buttons)
- **Async**: Embassy async framework throughout
- **Type-safe**: Strong typing for commands and state management