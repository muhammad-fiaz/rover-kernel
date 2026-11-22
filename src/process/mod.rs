//! # Process Management Subsystem
//!
//! Process control blocks, process IDs, and address space ownership.

pub mod pcb;

pub use pcb::{Process, ProcessId};
