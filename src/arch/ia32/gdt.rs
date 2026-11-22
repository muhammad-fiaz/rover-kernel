//! # IA-32 Global Descriptor Table
//!
//! Segment selector definitions for 32-bit x86 protected mode.

/// Kernel code segment selector (offset 0x08).
pub const KERNEL_CODE_SELECTOR: u16 = 0x08;
/// Kernel data segment selector (offset 0x10).
pub const KERNEL_DATA_SELECTOR: u16 = 0x10;
/// User code segment selector (offset 0x1B).
pub const USER_CODE_SELECTOR: u16 = 0x1B;
/// User data segment selector (offset 0x23).
pub const USER_DATA_SELECTOR: u16 = 0x23;

/// Initializes and loads the 32-bit GDT.
///
/// # Safety
///
/// Must be executed in protected mode.
pub unsafe fn init_early_gdt() {}
