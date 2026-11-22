//! # 32-bit x86 (IA-32) Architecture Subsystem
//!
//! Provides 32-bit x86 CPU control, GDT/IDT descriptors, and port I/O.

pub mod context;
pub mod cpu;
pub mod gdt;
pub mod idt;
pub mod io;

pub use context::{CpuContext, switch_context};
pub use cpu::{
    CpuCapabilities, are_interrupts_enabled, disable_interrupts, enable_interrupts, halt,
    halt_loop, read_cycle_counter,
};
pub use gdt::{
    KERNEL_CODE_SELECTOR, KERNEL_DATA_SELECTOR, USER_CODE_SELECTOR, USER_DATA_SELECTOR,
    init_early_gdt,
};
pub use idt::{init_early_idt, register_idt_gate};
pub use io::{inb, inl, inw, io_wait, outb, outl, outw};
