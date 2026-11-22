//! # Time Stamp Counter (TSC) Driver
//!
//! Provides high-precision CPU cycle counting using the `rdtsc` instruction.

/// Reads the current 64-bit cycle count from the processor's Time Stamp Counter.
#[inline]
#[must_use]
pub fn read_tsc() -> u64 {
    let low: u32;
    let high: u32;
    // SAFETY: rdtsc instruction reads cycle count into edx:eax.
    unsafe {
        core::arch::asm!(
            "rdtsc",
            out("eax") low,
            out("edx") high,
            options(nomem, nostack, preserves_flags)
        );
    }
    (u64::from(high) << 32) | u64::from(low)
}

/// Executes a calibrated spin-loop busy wait for approximately `cycles` CPU cycles.
pub fn delay_cycles(cycles: u64) {
    let start = read_tsc();
    while read_tsc().saturating_sub(start) < cycles {
        core::hint::spin_loop();
    }
}
