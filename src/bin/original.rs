//! ProductionDeck - StreamDeck Original Compatible Firmware
//! 
//! This binary builds firmware specifically for StreamDeck Original compatibility:
//! - 15 keys in 5x3 layout
//! - 72x72 pixel images per key
//! - USB VID:PID 0x0fd9:0x0060
//! - V1 BMP protocol

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use panic_halt as _;
use defmt_rtt as _;

// Set compile-time device selection
const DEVICE: productiondeck::device::Device = productiondeck::device::Device::Original;

// Import all modules from library
extern crate productiondeck;
use productiondeck::*;

// USB interrupt binding
// Use Irqs from the library to avoid duplicate definitions

/// Main application entry point for StreamDeck Original
#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Initialize hardware
    let p = embassy_rp::init(Default::default());
    
    // Create application supervisor for Original
    let mut supervisor = supervisor::AppSupervisor::new_for_device(DEVICE);
    
    // Print startup information
    supervisor.print_startup_banner();
    
    // Initialize and spawn all hardware tasks for Original
    match hardware::init_hardware_tasks_for_device(&spawner, p, DEVICE).await {
        Ok(()) => {
            info!("StreamDeck Original firmware initialized successfully");
            supervisor.print_init_success();
        }
        Err(e) => {
            error!("Failed to spawn hardware tasks: {:?}", e);
            core::panic!("Hardware initialization failed");
        }
    }
    
    // Run the main supervisor loop
    supervisor.run().await;
}