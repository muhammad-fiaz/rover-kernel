//! # Time Stamp Counter (TSC) and Cycle Timing Driver
//!
//! Provides high-precision CPU cycle counting across architectures.

/// Reads the current 64-bit cycle count from the processor's cycle counter.
#[inline]
#[must_use]
pub fn read_tsc() -> u64 {
    crate::arch::current::read_cycle_counter()
}

/// Executes a calibrated spin-loop busy wait for approximately `cycles` CPU cycles.
pub fn delay_cycles(cycles: u64) {
    let start = read_tsc();
    while read_tsc().saturating_sub(start) < cycles {
        core::hint::spin_loop();
    }
}
