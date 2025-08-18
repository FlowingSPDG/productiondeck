# Code Style and Conventions

## Rust Style Guidelines
- Follow standard Rust naming conventions:
  - `snake_case` for functions, variables, modules
  - `PascalCase` for types, structs, enums
  - `SCREAMING_SNAKE_CASE` for constants and statics
- Use `cargo fmt` for code formatting (standard Rust formatting)

## Project Structure
```
src/
├── main.rs        - Main entry point, task coordination
├── config.rs      - Hardware configuration constants
├── usb.rs         - USB HID implementation
├── display.rs     - Display handling and graphics
└── buttons.rs     - Button scanning and debouncing
```

## Code Organization Patterns
- **Constants**: Defined in `config.rs` with descriptive names
- **Async Tasks**: Embassy framework with async/await pattern
- **Channel Communication**: Inter-task communication via Embassy channels
- **Hardware Abstraction**: Clean separation between hardware and logic layers

## Key Architectural Patterns
- **Dual-core design**: Core 0 handles USB/protocol, Core 1 handles displays/buttons
- **Channel-based communication**: `BUTTON_CHANNEL`, `USB_COMMAND_CHANNEL`, `DISPLAY_CHANNEL`
- **Embassy async framework**: All I/O operations are async
- **Type safety**: Strong typing for commands, states, and hardware interfaces

## Configuration Management
- Hardware pin assignments in `config.rs`
- USB descriptors and protocol constants centralized
- Configurable debug levels and timing parameters
- Feature flags for different build configurations

## Documentation Standards
- Inline documentation for public APIs
- Clear module-level documentation
- Hardware pin assignments clearly documented
- Protocol implementation details documented

## Error Handling
- Use `Result<T, E>` for fallible operations
- Panic only for unrecoverable errors
- `panic-halt` for embedded panic handler
- Graceful error handling in USB and display operations