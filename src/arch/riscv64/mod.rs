//! # RISC-V 64 Architecture Subsystem
//!
//! Provides 64-bit RISC-V CPU control, CSR access, context switching, and PLIC interrupt handling.

pub mod context;
pub mod cpu;
pub mod io;
pub mod plic;

pub use context::{CpuContext, switch_context};
pub use cpu::{
    CpuCapabilities, are_interrupts_enabled, disable_interrupts, enable_interrupts, halt,
    halt_loop, read_cycle_counter, read_msr, write_msr,
};
pub use io::{inb, inl, inw, io_wait, outb, outl, outw};
pub use plic::PlatformLevelInterruptController;
