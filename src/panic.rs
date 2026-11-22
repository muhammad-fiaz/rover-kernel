//! # Kernel Panic Handler
//!
//! Provides a safe, diagnostic panic handler for `#![no_std]` bare-metal execution.

use core::panic::PanicInfo;
use rover_kernel::arch::x86_64::{
    disable_interrupts, halt_loop, read_cr0, read_cr2, read_cr3, read_cr4, read_rflags,
};
use rover_kernel::kerror;

/// Invoked by the Rust runtime when an unrecoverable panic occurs.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    disable_interrupts();

    kerror!("=================================================");
    kerror!("!!! KERNEL PANIC: SYSTEM EXECUTION HALTED !!!");

    if let Some(location) = info.location() {
        kerror!(
            "Panic occurred at file '{}', line {}, column {}",
            location.file(),
            location.line(),
            location.column()
        );
    } else {
        kerror!("Panic occurred at unknown location");
    }

    kerror!("Message: {}", info.message());

    let rflags = read_rflags();
    let cr0 = read_cr0();
    let cr2 = read_cr2();
    let cr3 = read_cr3();
    let cr4 = read_cr4();

    kerror!("Processor Control Registers:");
    kerror!("  RFLAGS: 0x{:016x}", rflags);
    kerror!("  CR0:    0x{:016x}", cr0);
    kerror!("  CR2:    0x{:016x} (Page Fault Linear Address)", cr2);
    kerror!("  CR3:    0x{:016x} (Page Table Directory Base)", cr3);
    kerror!("  CR4:    0x{:016x}", cr4);
    kerror!("=================================================");

    halt_loop();
}
