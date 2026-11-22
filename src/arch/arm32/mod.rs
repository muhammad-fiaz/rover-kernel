//! # 32-bit ARM Architecture Subsystem
//!
//! Provides 32-bit ARM CPU control, execution context, and MMIO access.

pub mod context;
pub mod cpu;
pub mod io;

pub use context::{CpuContext, switch_context};
pub use cpu::{
    CpuCapabilities, are_interrupts_enabled, disable_interrupts, enable_interrupts, halt,
    halt_loop, read_cycle_counter, read_msr, write_msr,
};
pub use io::{inb, inl, inw, io_wait, outb, outl, outw};
