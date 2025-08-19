// This file is kept for backwards compatibility
// The actual library is now in lib.rs and device-specific binaries are in src/bin/

#![no_std]
#![no_main]

use panic_halt as _;

#[no_mangle]
extern "C" fn main() -> ! {
    panic!("This main.rs is deprecated. Use specific device binaries like: cargo run --bin mini")
}