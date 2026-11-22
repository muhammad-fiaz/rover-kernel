//! # Interrupt Management Subsystem
//!
//! Programmable Interrupt Controllers (8259 PIC, Local APIC), IRQ routing,
//! End-Of-Interrupt handling, and hardware timer interrupt handlers.

pub mod apic;
pub mod handlers;
pub mod pic8259;

pub use apic::{DEFAULT_APIC_BASE, LOCAL_APIC, LocalApic, end_of_interrupt};
pub use handlers::{TICK_COUNTER, register_hardware_interrupts};
pub use pic8259::{disable_pic, remap_pic};
