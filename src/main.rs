//! ProductionDeck - Open Source StreamDeck Alternative for RP2040
//! 
//! This implements the main application entry point and core logic
//! for the RP2040-based StreamDeck compatible device using Embassy.
//!
//! Hardware: Raspberry Pi Pico (RP2040)
//! Protocol: USB HID compatible with StreamDeck Mini
//! Display: Single ST7735 TFT display (216x144) divided into 6 regions
//! Buttons: 6 tactile switches in 3x2 matrix

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::usb::{Driver, InterruptHandler};
use embassy_rp::{bind_interrupts, peripherals};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::{Duration, Timer};
use heapless::Vec;
use panic_halt as _;
use defmt_rtt as _; // global logger

mod config;
mod usb;
mod display;
mod buttons;

use config::*;
use usb::*;
use buttons::*;

// ===================================================================
// USB Interrupt Binding
// ===================================================================

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<peripherals::USB>;
});

// ===================================================================
// Global State and Communication Channels
// ===================================================================

// Channel for button state communication between tasks
static BUTTON_CHANNEL: Channel<ThreadModeRawMutex, ButtonState, 1> = Channel::new();

// Channel for USB commands
static USB_COMMAND_CHANNEL: Channel<ThreadModeRawMutex, UsbCommand, 4> = Channel::new();

// Channel for display commands
static DISPLAY_CHANNEL: Channel<ThreadModeRawMutex, DisplayCommand, 8> = Channel::new();

#[derive(Clone, Copy, Debug, Format)]
pub struct ButtonState {
    pub buttons: [bool; STREAMDECK_KEYS],
    pub changed: bool,
}

#[derive(Clone, Debug)]
pub enum UsbCommand {
    Reset,
    SetBrightness(u8),
    ImageData { key_id: u8, data: Vec<u8, IMAGE_BUFFER_SIZE> },
}

#[derive(Clone, Debug)]
pub enum DisplayCommand {
    Clear(u8),      // Clear specific key
    ClearAll,       // Clear all keys
    SetBrightness(u8),
    DisplayImage { key_id: u8, data: Vec<u8, IMAGE_BUFFER_SIZE> },
}

// ===================================================================
// Main Application Entry Point
// ===================================================================

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("========================================");
    info!("ProductionDeck v0.1 - StreamDeck Alternative");
    info!("Hardware: RP2040 (Raspberry Pi Pico)");
    info!("Target: StreamDeck Mini Compatible");
    info!("USB: VID=0x{:04X} PID=0x{:04X}", USB_VID, USB_PID);
    info!("Keys: {} ({}x{} layout)", STREAMDECK_KEYS, STREAMDECK_COLS, STREAMDECK_ROWS);
    info!("Display: {}x{} per key", KEY_IMAGE_SIZE, KEY_IMAGE_SIZE);
    info!("========================================");

    let p = embassy_rp::init(Default::default());

    info!("Initializing ProductionDeck...");

    // Create USB driver
    let driver = Driver::new(p.USB, Irqs);

    // Spawn core tasks
    spawner.spawn(usb_task(driver, Output::new(p.PIN_20, Level::Low))).unwrap(); // USB task with USB status LED
    spawner.spawn(button_task(
        Output::new(p.PIN_2, Level::High),
        Output::new(p.PIN_3, Level::High),
        Input::new(p.PIN_4, Pull::Up),
        Input::new(p.PIN_5, Pull::Up),
        Input::new(p.PIN_6, Pull::Up),
    )).unwrap(); // Button scanning task
    /*
    spawner.spawn(display_task(
        p.SPI0,
        p.PIN_18,
        p.PIN_19,
        p.PIN_8,
        p.PIN_14,
        p.PIN_15,
        p.PIN_17,
    )).unwrap(); // Display task
*/
    spawner.spawn(status_task(Output::new(p.PIN_25, Level::Low), Output::new(p.PIN_21, Level::Low))).unwrap(); // Status LED task

    info!("ProductionDeck initialized successfully");
    info!("USB VID:PID = {:04X}:{:04X}", USB_VID, USB_PID);
    info!("Waiting for USB connection...");

    // Main supervisor loop
    let mut uptime_counter = 0u32;
    loop {
        Timer::after(Duration::from_secs(10)).await;
        uptime_counter += 10;
        info!("Status: Uptime {} seconds", uptime_counter);
    }
}

// ===================================================================
// Status LED Task
// ===================================================================

#[embassy_executor::task]
async fn status_task(
    mut status_led: Output<'static>,
    _error_led: Output<'static>,
) {

    info!("Status LED task started");

    loop {
        // Heartbeat pattern
        status_led.set_high();
        Timer::after(Duration::from_millis(100)).await;
        status_led.set_low();
        Timer::after(Duration::from_millis(900)).await;
    }
}
