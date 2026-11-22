//! # AArch64 Architecture Subsystem
//!
//! Provides 64-bit ARM CPU control, execution context, GIC interrupt handling, and MMIO access.

pub mod context;
pub mod cpu;
pub mod gic;
pub mod io;

pub use context::{CpuContext, switch_context};
pub use cpu::{
    CpuCapabilities, are_interrupts_enabled, disable_interrupts, enable_interrupts, halt,
    halt_loop, read_current_el, read_cycle_counter, read_msr, write_msr,
};
pub use gic::GenericInterruptController;
pub use io::{inb, inl, inw, io_wait, outb, outl, outw};
