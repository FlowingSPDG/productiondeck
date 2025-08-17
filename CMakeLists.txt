cmake_minimum_required(VERSION 3.13)

# Initialize pico-sdk from submodule
# NOTE: this must happen before project()
include(pico_sdk_import.cmake)

project(opendeck)

# Initialize the SDK
pico_sdk_init()

# Set C++ standard
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Force use of Pico SDK's mbedTLS instead of system mbedTLS
set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -I${PICO_SDK_PATH}/lib/mbedtls/include")
set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -I${PICO_SDK_PATH}/lib/mbedtls/include")

# Disable system mbedTLS headers
add_compile_definitions(MBEDTLS_CONFIG_FILE="${PICO_SDK_PATH}/lib/mbedtls/include/mbedtls/config.h")

# Create directories for build outputs
file(MAKE_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR}/output)

# Include directories
include_directories(include)
include_directories(${CMAKE_CURRENT_SOURCE_DIR}/include)

# Collect all source files
file(GLOB_RECURSE SOURCES
    "src/*.cpp"
    "src/*.c"
)

# Create executable
add_executable(opendeck ${SOURCES})

# Link libraries
target_link_libraries(opendeck
    pico_stdlib
    pico_unique_id
    pico_multicore
    hardware_spi
    hardware_gpio
    hardware_pwm
    hardware_timer
    hardware_watchdog
    tinyusb_device
    tinyusb_board
)

# Enable USB output, disable UART output
pico_enable_stdio_usb(opendeck 0)
pico_enable_stdio_uart(opendeck 1)

# Create map/bin/hex file etc.
pico_add_extra_outputs(opendeck)

# Copy built files to output directory
add_custom_command(TARGET opendeck POST_BUILD
    COMMAND ${CMAKE_COMMAND} -E copy $<TARGET_FILE:opendeck>.uf2 ${CMAKE_CURRENT_BINARY_DIR}/output/
    COMMAND ${CMAKE_COMMAND} -E copy $<TARGET_FILE:opendeck>.bin ${CMAKE_CURRENT_BINARY_DIR}/output/
    COMMAND ${CMAKE_COMMAND} -E copy $<TARGET_FILE:opendeck>.hex ${CMAKE_CURRENT_BINARY_DIR}/output/
    COMMENT "Copying output files to output directory"
)

# Print build information
add_custom_target(info
    COMMAND ${CMAKE_COMMAND} -E echo "=== OpenDeck Build Information ==="
    COMMAND ${CMAKE_COMMAND} -E echo "Target: RP2040 (Raspberry Pi Pico)"
    COMMAND ${CMAKE_COMMAND} -E echo "USB VID: 0x0fd9 (Elgato)"
    COMMAND ${CMAKE_COMMAND} -E echo "USB PID: 0x0063 (StreamDeck Mini)"
    COMMAND ${CMAKE_COMMAND} -E echo "Keys: 6 (3x2 layout)"
    COMMAND ${CMAKE_COMMAND} -E echo "Display: 80x80 per key"
    COMMAND ${CMAKE_COMMAND} -E echo "Protocol: USB HID"
    COMMAND ${CMAKE_COMMAND} -E echo "Output: ${CMAKE_CURRENT_BINARY_DIR}/output/"
    COMMAND ${CMAKE_COMMAND} -E echo "=================================="
)

# Development targets
add_custom_target(flash
    COMMAND ${CMAKE_COMMAND} -E echo "=== Flash Instructions ==="
    COMMAND ${CMAKE_COMMAND} -E echo "1. Hold BOOTSEL button on Pico"
    COMMAND ${CMAKE_COMMAND} -E echo "2. Connect USB cable"
    COMMAND ${CMAKE_COMMAND} -E echo "3. Copy opendeck.uf2 to RPI-RP2 drive"
    COMMAND ${CMAKE_COMMAND} -E echo "4. Device will reboot automatically"
    COMMAND ${CMAKE_COMMAND} -E echo "=========================="
    DEPENDS opendeck
)

# Clean target
add_custom_target(clean-all
    COMMAND ${CMAKE_COMMAND} -E remove_directory ${CMAKE_CURRENT_BINARY_DIR}
    COMMAND ${CMAKE_COMMAND} -E echo "All build files cleaned"
)