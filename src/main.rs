//! # Rover Kernel Executable Entry Point
//!
//! Native `#![no_std]` and `#![no_main]` entry point for the Rover operating system kernel.

#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]

mod panic;

use rover_kernel::arch::x86_64::{
    CpuCapabilities, enable_interrupts, halt_loop, init_early_gdt, init_early_idt,
};
use rover_kernel::boot::{RoverBootInfo, dump_boot_info, validate_boot_info};
use rover_kernel::diagnostics::dump_memory_diagnostics;
use rover_kernel::drivers::pci::scan_pci_bus;
use rover_kernel::drivers::serial::SERIAL1;
use rover_kernel::interrupts::{LOCAL_APIC, disable_pic, register_hardware_interrupts, remap_pic};
use rover_kernel::mm::{FRAME_ALLOCATOR, init_kernel_heap};
use rover_kernel::sched::SCHEDULER;
use rover_kernel::security::enable_hardware_protections;
use rover_kernel::syscall::init_syscall_architecture;
use rover_kernel::{kerror, kinfo, kprintln};

/// Static buffer for the early 4 MiB kernel heap.
static mut EARLY_HEAP_BUFFER: [u8; 4 * 1024 * 1024] = [0u8; 4 * 1024 * 1024];

/// Naked kernel entry point invoked by Rover Bootloader.
///
/// Sets up the System V AMD64 ABI stack frame alignment and branches to [`kernel_main`].
///
/// # Safety
///
/// Must be invoked by a compliant bootloader in 64-bit long mode with identity paging active
/// and a valid stack pointer.
#[unsafe(naked)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn _start() -> ! {
    core::arch::naked_asm!(
        "cli",
        "cld",
        "xor ebp, ebp",
        "and rsp, -16", // Guarantee 16-byte stack alignment
        "sub rsp, 8",   // Compensate for caller frame
        "call {entry}",
        "2:",
        "hlt",
        "jmp 2b",
        entry = sym kernel_main,
    );
}

/// Fallback entry point for MSVC host test compilation.
///
/// # Safety
///
/// Delegates to [`_start`].
#[cfg(target_os = "windows")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mainCRTStartup() -> ! {
    unsafe {
        _start();
    }
}

/// The main Rust entry point of the Rover Kernel.
///
/// Receives the authoritative [`RoverBootInfo`] structure pointer from the bootloader.
///
/// # Safety
///
/// `boot_info_ptr` must point to valid bootloader-initialized memory or be null.
#[unsafe(no_mangle)]
#[allow(clippy::too_many_lines)]
pub unsafe extern "C" fn kernel_main(boot_info_ptr: *const RoverBootInfo) -> ! {
    // 1. Initialize COM1 early serial console for diagnostic logging.
    SERIAL1.lock().init();

    // 2. Display kernel startup banner.
    kprintln!();
    kprintln!("  ____                               _  __                    _ ");
    kprintln!(" |  _ \\  _____   _____ _ __         | |/ /___ _ __ _ __   ___| |");
    kprintln!(" | |_) |/ _ \\ \\ / / _ \\ '__|  _____ | ' // _ \\ '__| '_ \\ / _ \\ |");
    kprintln!(" |  _ <| (_) \\ V /  __/ |    |_____|| . \\  __/ |  | | | |  __/ |");
    kprintln!(" |_| \\_\\\\___/ \\_/ \\___|_|           |_|\\_\\___|_|  |_| |_|\\___|_|");
    kprintln!();
    kinfo!("Rover Operating System Kernel v0.1.0 (x86_64-unknown-none)");
    kinfo!("Build: Rust 2024 Edition, Native `#![no_std]` Bare-Metal");
    kprintln!();

    // 3. Validate boot information block passed from Rover Bootloader.
    kinfo!("Validating Rover Boot Protocol v1 handshake...");
    let boot_info = match unsafe { validate_boot_info(boot_info_ptr) } {
        Ok(info) => {
            kinfo!(
                "Boot protocol validation SUCCESSFUL (protocol version {})",
                info.version
            );
            info
        }
        Err(err) => {
            kerror!("Boot protocol validation FAILED: {}", err);
            kerror!("Kernel initialization aborted.");
            halt_loop();
        }
    };

    // 4. Dump complete boot parameters and system memory layout.
    dump_boot_info(boot_info);

    // 5. Initialize CPU Global Descriptor Table (GDT) and reload segment registers.
    kinfo!("Initializing CPU Global Descriptor Table (GDT)...");
    unsafe {
        init_early_gdt();
    }
    kinfo!("GDT successfully initialized and loaded.");

    // 6. Initialize CPU Interrupt Descriptor Table (IDT) with exception gates.
    kinfo!("Initializing CPU Interrupt Descriptor Table (IDT)...");
    unsafe {
        init_early_idt();
    }
    kinfo!("IDT successfully initialized with 22 exception handlers.");

    // 7. Remap legacy 8259 PIC and disable/mask PIC lines for APIC transition.
    kinfo!("Remapping and disabling legacy 8259 PIC...");
    unsafe {
        remap_pic(32, 40);
    }
    disable_pic();
    kinfo!("Legacy 8259 PIC successfully disabled and masked.");

    // 8. Register hardware interrupt handlers into IDT.
    kinfo!("Registering hardware IRQ handlers (Timer: 32, Keyboard: 33, Spurious: 255)...");
    register_hardware_interrupts();
    kinfo!("Hardware IRQs successfully registered in IDT.");

    // 9. Initialize Physical Frame Allocator from RoverBootInfo memory map.
    kinfo!("Initializing Physical Frame Allocator from UEFI memory map...");
    unsafe {
        FRAME_ALLOCATOR.lock().init(boot_info);
    }
    kinfo!("Physical Frame Allocator initialized.");

    // 10. Initialize Kernel Heap Allocator.
    kinfo!("Initializing Kernel Heap Allocator (4 MiB capacity)...");
    let heap_start = core::ptr::addr_of_mut!(EARLY_HEAP_BUFFER) as usize;
    let heap_size = 4 * 1024 * 1024;
    unsafe {
        init_kernel_heap(heap_start, heap_size);
    }
    kinfo!("Kernel Heap Allocator successfully initialized.");

    // 11. Probe CPU hardware features via CPUID.
    kinfo!("Probing CPUID capabilities...");
    let cpu = CpuCapabilities::probe();
    kinfo!("CPU Vendor: {}", cpu.vendor_str());
    kinfo!(
        "Features: APIC={}, SSE={}, SSE2={}, SSE3={}, SSE4.1={}, SSE4.2={}, AVX={}, AVX2={}",
        cpu.has_apic,
        cpu.has_sse,
        cpu.has_sse2,
        cpu.has_sse3,
        cpu.has_sse4_1,
        cpu.has_sse4_2,
        cpu.has_avx,
        cpu.has_avx2
    );
    kinfo!(
        "Extended: LongMode={}, NXBit={}, 1GBPages={}",
        cpu.has_long_mode,
        cpu.has_nx,
        cpu.has_page_1gb
    );

    // 12. Enable hardware security features (SMEP, SMAP, NXE).
    kinfo!("Configuring hardware security protections...");
    unsafe {
        enable_hardware_protections(&cpu);
    }

    // 13. Initialize Local APIC and periodic timer.
    kinfo!("Initializing Local APIC...");
    unsafe {
        LOCAL_APIC.lock().init();
    }
    kinfo!("Local APIC enabled (APIC ID: {}).", LOCAL_APIC.lock().id());

    // Setup periodic timer: vector 32, initial countdown = 100,000.
    LOCAL_APIC.lock().setup_periodic_timer(32, 100_000);
    kinfo!("Local APIC periodic timer configured on IRQ vector 32.");

    // 14. Initialize hardware system call architecture (EFER.SCE, STAR, LSTAR, FMASK).
    kinfo!("Initializing x86-64 System Call architecture...");
    unsafe {
        init_syscall_architecture();
    }
    kinfo!("System Call ABI infrastructure initialized.");

    // 15. Scan PCI Bus for attached peripherals.
    kinfo!("Scanning PCI buses for hardware devices...");
    let pci_devices = scan_pci_bus();
    kinfo!("Discovered {} PCI device functions:", pci_devices.len());
    for dev in &pci_devices {
        kinfo!(
            "  PCI [{:02x}:{:02x}.{:x}] Vendor: 0x{:04x}, Device: 0x{:04x}, Class: 0x{:02x}, Subclass: 0x{:02x}",
            dev.bus,
            dev.slot,
            dev.func,
            dev.vendor_id,
            dev.device_id,
            dev.class_code,
            dev.subclass
        );
    }

    // 16. Initialize Graphical Framebuffer Console if linear aperture present.
    if boot_info.framebuffer.is_valid() {
        kinfo!("Initializing Graphical Framebuffer Console...");
        rover_kernel::drivers::fbcon::init_framebuffer_console(boot_info.framebuffer);
    }

    // 17. Display detailed Memory Diagnostics.
    dump_memory_diagnostics();

    // 18. Initialize Preemptive Kernel Scheduler.
    kinfo!("Initializing Preemptive Kernel Scheduler...");
    SCHEDULER.lock().init();
    kinfo!(
        "Kernel Scheduler initialized with {} active tasks.",
        SCHEDULER.lock().task_count()
    );

    // 19. Enable Hardware Interrupts.
    kinfo!("Enabling hardware interrupts (sti)...");
    unsafe {
        enable_interrupts();
    }
    kinfo!("Hardware interrupts enabled.");

    kprintln!();
    kinfo!("============================================================");
    kinfo!("Rover Kernel complete modular architecture successfully running!");
    kinfo!("============================================================");

    // 20. Safely halt the processor, waking on hardware interrupts / timer ticks.
    loop {
        rover_kernel::arch::x86_64::halt();
    }
}
