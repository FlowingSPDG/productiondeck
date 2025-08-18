# Task Completion Workflow

## Pre-Development Setup
Before making changes, ensure:
1. Dependency conflicts are resolved (currently blocking compilation)
2. `cargo check` passes without errors
3. Understanding of Embassy async patterns and RP2040 constraints

## Code Quality Checks (When Working)
When the project compiles successfully, run these after making changes:

### 1. Code Formatting
```bash
cargo fmt --check    # Check if code is properly formatted
cargo fmt           # Apply Rust standard formatting
```

### 2. Linting and Quality
```bash
cargo clippy        # Run Clippy linter for code quality issues
cargo check         # Quick compilation check without building
```

### 3. Full Build Test
```bash
cargo build --release    # Build in release mode (required for embedded)
```

## Testing Strategy
**Note**: Currently no test framework is implemented in this project.

Future testing considerations:
- Unit tests for individual modules (buttons, display, USB protocol)
- Hardware-in-the-loop testing for actual RP2040 functionality
- Protocol compliance testing with StreamDeck software

## Hardware Validation
When deploying to hardware:

### 1. Build UF2 Firmware
```bash
cargo build --release
# Output: target/thumbv6m-none-eabi/release/productiondeck.uf2
```

### 2. Flash to Hardware
1. Hold BOOTSEL button on Pico
2. Connect USB cable
3. Copy UF2 file to RPI-RP2 drive
4. Device reboots automatically

### 3. Verify Operation
- Check USB device recognition as "Stream Deck Mini"
- Test button responsiveness
- Verify display functionality
- Test with official StreamDeck software

## Dependency Management
Currently blocked by version conflicts:
- Need to resolve byteorder version mismatch
- May require updating or pinning specific dependency versions
- Use `cargo tree` to analyze dependency chains

## Documentation Updates
After significant changes:
- Update inline documentation
- Keep CLAUDE.md current with build instructions
- Update protocol documentation if USB implementation changes

## Version Control Best Practices
- Test compilation before committing
- Include meaningful commit messages
- Tag releases when firmware is hardware-tested
- Keep hardware pin assignments documented in commits