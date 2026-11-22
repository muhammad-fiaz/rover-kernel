//! # x86-64 Architecture Implementation
//!
//! Subsystem implementations for x86-64 long mode execution.

pub mod context;
pub mod cpu;
pub mod gdt;
pub mod idt;
pub mod io;

pub use context::CpuContext;
pub use cpu::{
    CpuCapabilities, CpuidResult, are_interrupts_enabled, cpuid, disable_interrupts,
    enable_interrupts, halt, halt_loop, read_cr0, read_cr2, read_cr3, read_cr4, read_cycle_counter,
    read_msr, read_rflags, write_cr0, write_cr3, write_cr4, write_msr,
};
pub use gdt::{KERNEL_CODE_SELECTOR, KERNEL_DATA_SELECTOR, init_early_gdt};
pub use idt::{ExceptionStackFrame, init_early_idt, register_idt_gate};
pub use io::{inb, inl, inw, io_wait, outb, outl, outw};
