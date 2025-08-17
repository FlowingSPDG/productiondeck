#!/bin/bash
# ===================================================================
# OpenDeck Build Script
# Simplified build process for OpenDeck firmware
# ===================================================================

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
BUILD_DIR="build"
OUTPUT_DIR="build/output"
PROJECT_NAME="opendeck"

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check prerequisites
check_prerequisites() {
    print_status "Checking prerequisites..."
    
    if ! command_exists cmake; then
        print_error "CMake is not installed. Please install CMake 3.13 or higher."
        exit 1
    fi
    
    if ! command_exists arm-none-eabi-gcc; then
        print_error "ARM GCC toolchain is not installed."
        print_error "Please install arm-none-eabi-gcc and related tools."
        exit 1
    fi
    
    if [ -z "$PICO_SDK_PATH" ]; then
        print_warning "PICO_SDK_PATH environment variable is not set."
        print_warning "Attempting to use default locations..."
        
        # Try common locations
        for path in "/opt/pico-sdk" "$HOME/pico-sdk" "./pico-sdk"; do
            if [ -d "$path" ]; then
                export PICO_SDK_PATH="$path"
                print_status "Found Pico SDK at: $PICO_SDK_PATH"
                break
            fi
        done
        
        if [ -z "$PICO_SDK_PATH" ]; then
            print_error "Could not find Pico SDK. Please:"
            print_error "1. Install Pico SDK: git clone https://github.com/raspberrypi/pico-sdk.git"
            print_error "2. Set PICO_SDK_PATH environment variable"
            exit 1
        fi
    fi
    
    print_success "All prerequisites satisfied"
}

# Clean build directory
clean_build() {
    if [ "$1" = "clean" ] || [ "$1" = "rebuild" ]; then
        print_status "Cleaning build directory..."
        rm -rf "$BUILD_DIR"
        print_success "Build directory cleaned"
    fi
}

# Configure CMake
configure_cmake() {
    print_status "Configuring CMake..."
    
    mkdir -p "$BUILD_DIR"
    cd "$BUILD_DIR"
    
    cmake .. \
        -DPICO_SDK_PATH="$PICO_SDK_PATH" \
        -DCMAKE_BUILD_TYPE=Release
    
    cd ..
    print_success "CMake configuration complete"
}

# Build firmware
build_firmware() {
    print_status "Building OpenDeck firmware..."
    
    cd "$BUILD_DIR"
    make -j$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 4)
    cd ..
    
    print_success "Build complete"
}

# Show build information
show_build_info() {
    cd "$BUILD_DIR"
    make info
    cd ..
}

# Display build results
show_results() {
    print_status "Build Results:"
    echo "==============================================="
    
    if [ -f "$OUTPUT_DIR/${PROJECT_NAME}.uf2" ]; then
        echo "✓ Firmware file: $OUTPUT_DIR/${PROJECT_NAME}.uf2"
        echo "  Size: $(du -h "$OUTPUT_DIR/${PROJECT_NAME}.uf2" | cut -f1)"
    else
        print_error "Main firmware file not found!"
        return 1
    fi
    
    for ext in bin hex elf; do
        if [ -f "$BUILD_DIR/${PROJECT_NAME}.$ext" ]; then
            echo "✓ ${ext^^} file: $BUILD_DIR/${PROJECT_NAME}.$ext"
        fi
    done
    
    echo "==============================================="
    print_success "All files built successfully!"
    
    echo ""
    print_status "Next steps:"
    echo "1. Hold BOOTSEL button on Raspberry Pi Pico"
    echo "2. Connect USB cable"
    echo "3. Copy $OUTPUT_DIR/${PROJECT_NAME}.uf2 to RPI-RP2 drive"
    echo "4. Device will reboot as StreamDeck Mini"
}

# Flash firmware (if possible)
flash_firmware() {
    print_status "Looking for Pico in bootloader mode..."
    
    # Check for mounted RPI-RP2 drive
    rpi_path=""
    
    # Check common mount points
    for path in "/Volumes/RPI-RP2" "/media/$USER/RPI-RP2" "/media/RPI-RP2" "/mnt/RPI-RP2"; do
        if [ -d "$path" ]; then
            rpi_path="$path"
            break
        fi
    done
    
    if [ -n "$rpi_path" ]; then
        print_status "Found Pico bootloader at: $rpi_path"
        print_status "Copying firmware..."
        
        if cp "$OUTPUT_DIR/${PROJECT_NAME}.uf2" "$rpi_path/"; then
            print_success "Firmware flashed successfully!"
            print_status "Device should reboot automatically"
        else
            print_error "Failed to copy firmware file"
            return 1
        fi
    else
        print_warning "Pico not found in bootloader mode"
        print_status "To flash manually:"
        echo "1. Hold BOOTSEL button on Pico"
        echo "2. Connect USB cable"
        echo "3. Copy $OUTPUT_DIR/${PROJECT_NAME}.uf2 to the mounted RPI-RP2 drive"
    fi
}

# Show usage information
show_usage() {
    echo "OpenDeck Build Script"
    echo "Usage: $0 [command]"
    echo ""
    echo "Commands:"
    echo "  build     - Build the firmware (default)"
    echo "  clean     - Clean build directory and rebuild"
    echo "  rebuild   - Same as clean"
    echo "  flash     - Build and attempt to flash to connected Pico"
    echo "  info      - Show build information"
    echo "  help      - Show this help message"
    echo ""
    echo "Environment variables:"
    echo "  PICO_SDK_PATH - Path to Pico SDK (required)"
    echo ""
    echo "Examples:"
    echo "  $0              # Build firmware"
    echo "  $0 clean        # Clean and rebuild"
    echo "  $0 flash        # Build and flash to Pico"
}

# Main script logic
main() {
    echo "================================================"
    echo "    OpenDeck - StreamDeck Alternative"
    echo "    RP2040 Firmware Build Script"
    echo "================================================"
    echo ""
    
    local command="${1:-build}"
    
    case "$command" in
        "help"|"-h"|"--help")
            show_usage
            exit 0
            ;;
        "info")
            check_prerequisites
            if [ -d "$BUILD_DIR" ]; then
                show_build_info
            else
                print_error "Build directory not found. Run '$0 build' first."
                exit 1
            fi
            exit 0
            ;;
        "clean"|"rebuild")
            check_prerequisites
            clean_build "$command"
            configure_cmake
            build_firmware
            show_results
            ;;
        "flash")
            check_prerequisites
            clean_build
            configure_cmake
            build_firmware
            show_results
            flash_firmware
            ;;
        "build")
            check_prerequisites
            clean_build
            configure_cmake
            build_firmware
            show_results
            ;;
        *)
            print_error "Unknown command: $command"
            echo ""
            show_usage
            exit 1
            ;;
    esac
}

# Run main function with all arguments
main "$@"