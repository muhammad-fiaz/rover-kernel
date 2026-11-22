//! # Architecture abstraction layer
//!
//! Provides hardware-specific primitives, CPU control, descriptor tables,
//! and low-level execution context handling across supported processor architectures.

#[cfg(target_arch = "x86_64")]
pub mod x86_64;

#[cfg(target_arch = "x86_64")]
pub use x86_64 as current;
