//! # Timers and Timekeeping Subsystem
//!
//! Exposes TSC cycle counting, system monotonic clock, uptime queries, and tick delays.

pub mod clock;
pub mod tsc;

pub use clock::{TICKS_PER_SECOND, sleep_ticks, uptime_ms, uptime_ticks};
pub use tsc::{delay_cycles, read_tsc};
