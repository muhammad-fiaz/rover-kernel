//! # Block Device Subsystem
//!
//! Exposes block device interfaces and in-memory block devices.

pub mod device;

pub use device::{BlockDevice, RamBlockDevice};
