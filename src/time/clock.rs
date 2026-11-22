//! # Monotonic System Clock
//!
//! Provides system uptime queries in milliseconds and blocking tick delays.

use crate::interrupts::TICK_COUNTER;
use core::sync::atomic::Ordering;

/// Number of timer ticks per second (100 Hz = 10 ms per tick).
pub const TICKS_PER_SECOND: u64 = 100;

/// Returns the total number of hardware timer ticks elapsed since system boot.
#[must_use]
pub fn uptime_ticks() -> u64 {
    TICK_COUNTER.load(Ordering::Relaxed)
}

/// Returns the system uptime in milliseconds based on the 100 Hz timer tick frequency.
#[must_use]
pub fn uptime_ms() -> u64 {
    uptime_ticks().saturating_mul(1000) / TICKS_PER_SECOND
}

/// Pauses execution until `ticks` timer ticks have elapsed.
pub fn sleep_ticks(ticks: u64) {
    let start = uptime_ticks();
    while uptime_ticks().saturating_sub(start) < ticks {
        core::hint::spin_loop();
    }
}
