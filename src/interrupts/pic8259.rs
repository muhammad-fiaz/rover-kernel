//! # Legacy 8259 Programmable Interrupt Controller (PIC)
//!
//! Remaps PIC interrupt vector offsets from 0..15 to 32..47 to prevent conflict
//! with CPU exceptions (0..31), and provides full IRQ masking when switching to APIC.

use crate::arch::x86_64::io::{inb, io_wait, outb};

const PIC1_COMMAND: u16 = 0x20;
const PIC1_DATA: u16 = 0x21;
const PIC2_COMMAND: u16 = 0xA0;
const PIC2_DATA: u16 = 0xA1;

const ICW1_INIT: u8 = 0x11;
const ICW4_8086: u8 = 0x01;

/// Remaps the legacy 8259 PIC vectors to `offset1` and `offset2` (e.g., 32 and 40).
///
/// # Safety
///
/// Writing to PIC hardware I/O ports must only be performed during interrupt initialization.
pub unsafe fn remap_pic(offset1: u8, offset2: u8) {
    // SAFETY: Verified by caller.
    unsafe {
        let mask1 = inb(PIC1_DATA);
        let mask2 = inb(PIC2_DATA);

        // Start initialization sequence in cascade mode.
        outb(PIC1_COMMAND, ICW1_INIT);
        io_wait();
        outb(PIC2_COMMAND, ICW1_INIT);
        io_wait();

        // ICW2: Vector offset.
        outb(PIC1_DATA, offset1);
        io_wait();
        outb(PIC2_DATA, offset2);
        io_wait();

        // ICW3: Tell Master PIC there is a slave PIC at IRQ2 (0000 0100).
        outb(PIC1_DATA, 4);
        io_wait();
        // ICW3: Tell Slave PIC its cascade identity (0000 0010).
        outb(PIC2_DATA, 2);
        io_wait();

        // ICW4: Set 8086/88 mode.
        outb(PIC1_DATA, ICW4_8086);
        io_wait();
        outb(PIC2_DATA, ICW4_8086);
        io_wait();

        // Restore saved masks.
        outb(PIC1_DATA, mask1);
        outb(PIC2_DATA, mask2);
    }
}

/// Disables and masks all IRQ lines on both Master and Slave 8259 PICs.
///
/// This is mandatory on modern ACPI/APIC platforms to ensure legacy PICs do not fire.
pub fn disable_pic() {
    // SAFETY: Writing 0xFF masks all 8 lines on both PICs.
    unsafe {
        outb(PIC1_DATA, 0xFF);
        outb(PIC2_DATA, 0xFF);
    }
}
