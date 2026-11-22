//! # Hardware Interrupt Handlers and Dispatchers
//!
//! Assembly stubs and Rust handlers for timer ticks (vector 32), keyboard (vector 33),
//! and spurious interrupts (vector 255).

use super::apic::end_of_interrupt;
use crate::arch::x86_64::register_idt_gate;
use core::sync::atomic::{AtomicU64, Ordering};

/// Global system tick counter incremented on every timer interrupt.
pub static TICK_COUNTER: AtomicU64 = AtomicU64::new(0);

// Macro for generating naked hardware IRQ stubs.
macro_rules! hardware_irq_stub {
    ($name:ident, $handler:ident) => {
        #[unsafe(no_mangle)]
        #[unsafe(naked)]
        unsafe extern "C" fn $name() {
            core::arch::naked_asm!(
                "push rax",
                "push rcx",
                "push rdx",
                "push rbx",
                "push rbp",
                "push rsi",
                "push rdi",
                "push r8",
                "push r9",
                "push r10",
                "push r11",
                "push r12",
                "push r13",
                "push r14",
                "push r15",
                concat!("call ", stringify!($handler)),
                "pop r15",
                "pop r14",
                "pop r13",
                "pop r12",
                "pop r11",
                "pop r10",
                "pop r9",
                "pop r8",
                "pop rdi",
                "pop rsi",
                "pop rbp",
                "pop rbx",
                "pop rdx",
                "pop rcx",
                "pop rax",
                "iretq"
            );
        }
    };
}

hardware_irq_stub!(irq32_timer_entry, handle_timer_tick);
hardware_irq_stub!(irq33_keyboard_entry, handle_keyboard_irq);

#[unsafe(no_mangle)]
extern "C" fn handle_timer_tick() {
    TICK_COUNTER.fetch_add(1, Ordering::Relaxed);
    end_of_interrupt();
}

#[unsafe(no_mangle)]
extern "C" fn handle_keyboard_irq() {
    // Read keyboard scancode from I/O port 0x60.
    // SAFETY: Reading keyboard controller data port.
    let scancode = unsafe { crate::arch::x86_64::inb(0x60) };
    let _ = scancode; // Consumed
    end_of_interrupt();
}

#[unsafe(no_mangle)]
#[unsafe(naked)]
unsafe extern "C" fn irq255_spurious_entry() {
    // Spurious interrupts do NOT send an EOI.
    core::arch::naked_asm!("iretq");
}

/// Registers the hardware IRQ gate handlers into the system IDT.
pub fn register_hardware_interrupts() {
    register_idt_gate(32, irq32_timer_entry as *const () as usize);
    register_idt_gate(33, irq33_keyboard_entry as *const () as usize);
    register_idt_gate(255, irq255_spurious_entry as *const () as usize);
}
