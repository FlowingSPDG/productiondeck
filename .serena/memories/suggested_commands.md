# ProductionDeck Development Commands

## Primary Build Commands
```bash
# Check compilation without building
cargo check

# Build the project (release mode recommended for embedded)
cargo build --release

# Build with verbose output
cargo build --release --verbose

# Clean build artifacts
cargo clean
```

## Code Quality Commands
```bash
# Format code according to Rust standards
cargo fmt

# Check formatting without making changes
cargo fmt --check

# Run Clippy linter for code quality
cargo clippy

# Check for security vulnerabilities
cargo audit
```

## Development Commands
```bash
# Check project structure and dependencies
cargo tree

# Update dependencies (use with caution)
cargo update

# Generate documentation
cargo doc --open

# Expand macros (useful for debugging)
cargo expand
```

## Hardware Deployment
```bash
# Build and create UF2 file for flashing
cargo build --release
# Output: target/thumbv6m-none-eabi/release/productiondeck.uf2

# Flash to Pico (when in BOOTSEL mode)
# Copy the UF2 file to the RPI-RP2 drive that appears when Pico is in bootloader mode
```

## Git Commands (Standard)
```bash
git status
git add .
git commit -m "message"
git push
git pull
```

## System Utilities (Linux/WSL)
```bash
ls -la          # List files with details
find . -name    # Find files by name
grep -r         # Search in files recursively
cat             # Display file contents
cd              # Change directory
```

## Current Issues
- Project currently has dependency conflicts
- `cargo check` fails due to byteorder version mismatch
- Need to resolve dependency conflicts before building