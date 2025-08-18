//! Button matrix scanning implementation
//! 
//! This module handles the 3x2 button matrix scanning with debouncing
//! and sends button state changes to the USB task.

use defmt::*;
use embassy_rp::gpio::{Input, Output};
use embassy_time::{Duration, Timer, Instant};

use crate::config::*;
use crate::{BUTTON_CHANNEL, ButtonState};

// ===================================================================
// Button Debouncing State
// ===================================================================

struct ButtonDebouncer {
    buttons: [ButtonDebounceState; STREAMDECK_KEYS],
}

#[derive(Clone, Copy)]
struct ButtonDebounceState {
    current: bool,
    raw: bool,
    last_change: Instant,
}

impl ButtonDebouncer {
    fn new() -> Self {
        Self {
            buttons: [ButtonDebounceState {
                current: false,
                raw: false,
                last_change: Instant::now(),
            }; STREAMDECK_KEYS],
        }
    }

    fn update(&mut self, key: usize, raw_state: bool) -> bool {
        let now = Instant::now();
        let state = &mut self.buttons[key];

        if raw_state != state.raw {
            state.raw = raw_state;
            state.last_change = now;
        }

        if now.duration_since(state.last_change) >= Duration::from_millis(BUTTON_DEBOUNCE_MS) {
            let changed = state.current != state.raw;
            state.current = state.raw;
            changed
        } else {
            false
        }
    }

    fn get_state(&self, key: usize) -> bool {
        self.buttons[key].current
    }
}

// ===================================================================
// Button Matrix Scanning
// ===================================================================

struct ButtonMatrix {
    rows: [Output<'static>; STREAMDECK_ROWS],
    cols: [Input<'static>; STREAMDECK_COLS],
}

impl ButtonMatrix {
    fn new(
        row_pin_0: Output<'static>,
        row_pin_1: Output<'static>,
        col_pin_0: Input<'static>,
        col_pin_1: Input<'static>,
        col_pin_2: Input<'static>,
    ) -> Self {
        let rows = [
            row_pin_0,
            row_pin_1,
        ];

        let cols = [
            col_pin_0,
            col_pin_1,
            col_pin_2,
        ];

        Self { rows, cols }
    }

    async fn scan(&mut self) -> [bool; STREAMDECK_KEYS] {
        let mut button_states = [false; STREAMDECK_KEYS];

        for (row_idx, row) in self.rows.iter_mut().enumerate() {
            // Pull current row low
            row.set_low();
            
            // Small settling time
            Timer::after(Duration::from_micros(10)).await;

            for (col_idx, col) in self.cols.iter().enumerate() {
                let key_index = row_idx * STREAMDECK_COLS + col_idx;
                
                // Read column pin (low = button pressed due to pull-up)
                button_states[key_index] = !col.is_high();
            }

            // Return row to high
            row.set_high();
        }

        button_states
    }
}

// ===================================================================
// Button Task Implementation
// ===================================================================

#[embassy_executor::task]
pub async fn button_task(
    row0: Output<'static>,
    row1: Output<'static>,
    col0: Input<'static>,
    col1: Input<'static>,
    col2: Input<'static>,
) {
    info!("Button task started");

    let mut matrix = ButtonMatrix::new(
        row0,
        row1,
        col0,
        col1,
        col2,
    );
    let mut debouncer = ButtonDebouncer::new();
    let mut _last_button_state = ButtonState {
        buttons: [false; STREAMDECK_KEYS],
        changed: false,
    };

    let scan_interval = Duration::from_millis(1000 / BUTTON_SCAN_RATE_HZ);
    let sender = BUTTON_CHANNEL.sender();

    info!("Button matrix initialized - scanning at {}Hz", BUTTON_SCAN_RATE_HZ);

    loop {
        // Scan button matrix
        let raw_states = matrix.scan().await;

        // Update debouncer and check for changes
        let mut changed = false;
        let mut new_state = ButtonState {
            buttons: [false; STREAMDECK_KEYS],
            changed: false,
        };

        for i in 0..STREAMDECK_KEYS {
            if debouncer.update(i, raw_states[i]) {
                changed = true;
                let pressed = debouncer.get_state(i);
                debug!("Button {} {}", i, if pressed { "pressed" } else { "released" });
            }
            new_state.buttons[i] = debouncer.get_state(i);
        }

        // Send state if changed
        if changed {
            new_state.changed = true;
            sender.send(new_state).await;
            debug!("Button state sent: {:?}", new_state.buttons);
            _last_button_state = new_state;
        }

        // Wait for next scan
        Timer::after(scan_interval).await;
    }
}

