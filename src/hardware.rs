//! Hardware abstraction and initialization
//!
//! This module provides hardware abstraction for different StreamDeck device
//! configurations and handles device-specific pin assignments and initialization.

use embassy_executor::{Spawner, SpawnError};
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::{peripherals, Peripherals};
use embassy_rp::usb::Driver;
use defmt::*;
use heapless::Vec;


use crate::config;
use crate::device::{Device, DeviceConfig};
use crate::usb::usb_task_for_device;
use crate::buttons::{button_task_matrix, button_task_direct};

/// Hardware configuration for a specific StreamDeck device
pub struct HardwareConfig {
    pub device: Device,
    pub button_pins: ButtonPins,
    pub display_pins: DisplayPins,
    pub led_pins: LedPins,
}

/// Pin assignments for button matrix
pub struct ButtonPins {
    pub row_pins: &'static [u8],
    pub col_pins: &'static [u8],
}

/// Pin assignments for display interface
pub struct DisplayPins {
    pub spi_mosi: u8,
    pub spi_sck: u8,
    pub cs: u8,
    pub dc: u8,
    pub rst: u8,
    pub backlight: u8,
}

/// Pin assignments for status LEDs
pub struct LedPins {
    pub status: u8,
    pub usb: u8,
    pub error: u8,
}

impl HardwareConfig {
    /// Get hardware configuration for the current device
    pub fn for_current_device() -> Self {
        let device = config::get_current_device();
        Self::for_device(device)
    }
    
    /// Get hardware configuration for a specific device
    pub fn for_device(device: Device) -> Self {
        let layout = device.button_layout();
        
        // Get pin assignments based on device layout
        let (row_pins, col_pins) = match (layout.rows, layout.cols) {
            (2, 3) => (&[2u8, 3][..], &[4u8, 5, 6][..]), // Mini
            (3, 5) => (&[2u8, 3, 7][..], &[4u8, 5, 6, 10, 11][..]), // Original
            (4, 8) => (&[2u8, 3, 7, 9][..], &[4u8, 5, 6, 10, 11, 12, 13, 16][..]), // XL
            (2, 4) => (&[2u8, 3][..], &[4u8, 5, 6, 10][..]), // Plus
            _ => (&[2u8, 3][..], &[4u8, 5, 6][..]), // Fallback to Mini
        };
        
        Self {
            device,
            button_pins: ButtonPins { row_pins, col_pins },
            display_pins: DisplayPins {
                spi_mosi: 19,
                spi_sck: 18,
                cs: 8,
                dc: 14,
                rst: 15,
                backlight: 17,
            },
            led_pins: LedPins {
                status: 25,
                usb: 20,
                error: 21,
            },
        }
    }
}

/// Initialize and spawn all hardware tasks for the current device (runtime selection)
pub async fn init_hardware_tasks(
    spawner: &Spawner,
    p: Peripherals,
) -> Result<(), SpawnError> {
    let hw_config = HardwareConfig::for_current_device();
    init_hardware_tasks_with_config(spawner, p, &hw_config).await
}

/// Initialize and spawn all hardware tasks for a specific device (compile-time selection)
pub async fn init_hardware_tasks_for_device(
    spawner: &Spawner,
    p: Peripherals,
    device: Device,
) -> Result<(), SpawnError> {
    let hw_config = HardwareConfig::for_device(device);
    init_hardware_tasks_with_config(spawner, p, &hw_config).await
}

/// Initialize and spawn all hardware tasks with given configuration
async fn init_hardware_tasks_with_config(
    spawner: &Spawner,
    p: Peripherals,
    hw_config: &HardwareConfig,
) -> Result<(), SpawnError> {
    let layout = hw_config.device.button_layout();
    
    info!("Initializing hardware for {}", hw_config.device.device_name());
    info!("Button layout: {}x{} = {} keys", layout.cols, layout.rows, layout.total_keys);
    
    // Create all pins and return them with the USB driver
    let (driver, usb_led, status_led, error_led, row_pins, col_pins) = 
        create_all_pins_for_device(p, hw_config.device);
    
    // Spawn USB task
    spawner.spawn(usb_task_for_device(driver, usb_led, hw_config.device))?;
    
    // For Mini devices, prefer Direct pin mode with 6 dedicated inputs
    let device = hw_config.device;
    if matches!(device, crate::device::Device::Mini | crate::device::Device::RevisedMini) {
        crate::config::set_button_input_mode(crate::config::ButtonInputMode::Direct);
    }

    // Spawn button task with device-specific layout
    spawn_button_task_with_pins(spawner, row_pins, col_pins, device)?;
    
    // Spawn display task (commented out until hardware is ready)
    // spawn_display_task(spawner, p, &hw_config)?;
    
    // Spawn status LED task
    spawner.spawn(status_task(status_led, error_led))?;
    
    Ok(())
}

/// Create all pins for specific device layout
fn create_all_pins_for_device(
    p: Peripherals, 
    device: Device
) -> (
    Driver<'static, peripherals::USB>,
    Output<'static>,
    Output<'static>, 
    Output<'static>,
    Vec<Output<'static>, 4>, 
    Vec<Input<'static>, 32>
) {
    
    // Create USB driver and LEDs first
    let driver = Driver::new(p.USB, crate::Irqs);
    let usb_led = Output::new(p.PIN_20, Level::Low);
    let status_led = Output::new(p.PIN_25, Level::Low);
    let error_led = Output::new(p.PIN_21, Level::Low);
    
    // Create button pins
    let layout = device.button_layout();
    let mut row_pins: Vec<Output<'static>, 4> = Vec::new();
    let mut col_pins: Vec<Input<'static>, 32> = Vec::new();

    // If Direct mode is selected for Mini, build 6 direct input pins
    if matches!(crate::config::button_input_mode(), crate::config::ButtonInputMode::Direct)
        && matches!(device, Device::Mini | Device::RevisedMini)
    {
        // Build six dedicated direct-input pins for Mini to avoid partial-move issues
        let _ = col_pins.push(Input::new(p.PIN_4, Pull::Up));
        let _ = col_pins.push(Input::new(p.PIN_5, Pull::Up));
        let _ = col_pins.push(Input::new(p.PIN_6, Pull::Up));
        let _ = col_pins.push(Input::new(p.PIN_10, Pull::Up));
        let _ = col_pins.push(Input::new(p.PIN_11, Pull::Up));
        let _ = col_pins.push(Input::new(p.PIN_12, Pull::Up));
    } else {
        match (layout.rows, layout.cols) {
            (2, 3) => {
                // Mini and Revised Mini (2x3 = 6 keys)
                let _ = row_pins.push(Output::new(p.PIN_2, Level::High));
                let _ = row_pins.push(Output::new(p.PIN_3, Level::High));
                let _ = col_pins.push(Input::new(p.PIN_4, Pull::Up));
                let _ = col_pins.push(Input::new(p.PIN_5, Pull::Up));
                let _ = col_pins.push(Input::new(p.PIN_6, Pull::Up));
            }
            _ => {
                // For now, all other devices use the same pin layout as Mini
                warn!("Using Mini button layout for {} - implement device-specific layout", device.device_name());
                let _ = row_pins.push(Output::new(p.PIN_2, Level::High));
                let _ = row_pins.push(Output::new(p.PIN_3, Level::High));
                let _ = col_pins.push(Input::new(p.PIN_4, Pull::Up));
                let _ = col_pins.push(Input::new(p.PIN_5, Pull::Up));
                let _ = col_pins.push(Input::new(p.PIN_6, Pull::Up));
            }
        }
    }
    
    (driver, usb_led, status_led, error_led, row_pins, col_pins)
}

/// Spawn button task with specific pins
fn spawn_button_task_with_pins(
    spawner: &Spawner,
    mut row_pins: Vec<Output<'static>, 4>,
    mut col_pins: Vec<Input<'static>, 32>,
    device: Device,
) -> Result<(), SpawnError> {
    match crate::config::button_input_mode() {
        crate::config::ButtonInputMode::Matrix => {
            // Extract pins for matrix task based on device layout
            let layout = device.button_layout();
            match (layout.rows, layout.cols) {
                (2, 3) => {
                    let row0 = row_pins.pop().unwrap();
                    let row1 = row_pins.pop().unwrap();
                    let col0 = col_pins.pop().unwrap();
                    let col1 = col_pins.pop().unwrap();
                    let col2 = col_pins.pop().unwrap();
                    spawner.spawn(button_task_matrix(row0, row1, col0, col1, col2))
                }
                _ => {
                    // Until wider matrix support lands, warn and use first 2x3
                    warn!("Matrix mode not fully implemented for this device; using Mini layout subset");
                    let row0 = row_pins.pop().unwrap();
                    let row1 = row_pins.pop().unwrap();
                    let col0 = col_pins.pop().unwrap();
                    let col1 = col_pins.pop().unwrap();
                    let col2 = col_pins.pop().unwrap();
                    spawner.spawn(button_task_matrix(row0, row1, col0, col1, col2))
                }
            }
        }
        crate::config::ButtonInputMode::Direct => {
            // Use as many input pins as available up to 32
            let mut inputs: heapless::Vec<Input<'static>, 32> = heapless::Vec::new();
            while let Some(pin) = col_pins.pop() {
                let _ = inputs.push(pin);
            }
            // Ensure Mini has exactly 6 inputs if possible
            if matches!(device, Device::Mini | Device::RevisedMini) && inputs.len() > 6 {
                while inputs.len() > 6 { let _ = inputs.pop(); }
            }
            spawner.spawn(button_task_direct(inputs))
        }
    }
}



/// Status LED task implementation
#[embassy_executor::task]
pub async fn status_task(
    mut status_led: Output<'static>,
    _error_led: Output<'static>,
) {
    use embassy_time::{Duration, Timer};
    
    info!("Status LED task started");

    loop {
        // Heartbeat pattern - short blink every second
        status_led.set_high();
        Timer::after(Duration::from_millis(100)).await;
        status_led.set_low();
        Timer::after(Duration::from_millis(900)).await;
    }
}