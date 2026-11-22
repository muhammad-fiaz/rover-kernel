//! # IA-32 Interrupt Descriptor Table
//!
//! 32-bit IDT gate descriptors and exception vector dispatch.

/// Registers a gate handler for the specified interrupt vector.
pub fn register_idt_gate(_vector: usize, _handler: usize) {}

/// Initializes the 32-bit early IDT.
///
/// # Safety
///
/// Must be called during single-threaded boot.
pub unsafe fn init_early_idt() {}
