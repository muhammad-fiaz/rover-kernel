//! # Rover Kernel Executable Entry Point
//!
//! Native `#![no_std]` and `#![no_main]` entry point for the Rover operating system kernel.

#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]

mod panic;

use rover_kernel::arch::current::{CpuCapabilities, enable_interrupts, halt, halt_loop};
#[cfg(target_arch = "x86_64")]
use rover_kernel::arch::x86_64::{init_early_gdt, init_early_idt};
use rover_kernel::boot::{RoverBootInfo, dump_boot_info, validate_boot_info};
use rover_kernel::diagnostics::dump_memory_diagnostics;
use rover_kernel::drivers::pci::scan_pci_bus;
use rover_kernel::drivers::serial::SERIAL1;
#[cfg(target_arch = "x86_64")]
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
#[cfg(target_arch = "x86_64")]
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

/// Naked kernel entry point for AArch64.
///
/// # Safety
///
/// Must be invoked with identity paging active and valid stack.
#[cfg(target_arch = "aarch64")]
#[unsafe(naked)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn _start() -> ! {
    core::arch::naked_asm!(
        "msr daifset, #2",
        "bl {entry}",
        "1: wfi",
        "b 1b",
        entry = sym kernel_main,
    );
}

/// Naked kernel entry point for RISC-V 64.
///
/// # Safety
///
/// Must be invoked with identity paging active and valid stack.
#[cfg(target_arch = "riscv64")]
#[unsafe(naked)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn _start() -> ! {
    core::arch::naked_asm!(
        "csrci sstatus, 2",
        "call {entry}",
        "1: wfi",
        "j 1b",
        entry = sym kernel_main,
    );
}

/// Fallback entry point for generic targets.
///
/// # Safety
///
/// Delegates to [`kernel_main`].
#[cfg(not(any(
    target_arch = "x86_64",
    target_arch = "aarch64",
    target_arch = "riscv64"
)))]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn _start() -> ! {
    kernel_main(core::ptr::null());
}

/// Fallback entry point for MSVC host test compilation.
///
/// # Safety
///
/// Delegates to [`_start`].
#[cfg(target_os = "windows")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mainCRTStartup() -> ! {
    unsafe { _start() }
}

/// High-level Rust kernel entry point.
///
/// # Safety
///
/// `boot_info_ptr` must point to a valid, initialized [`RoverBootInfo`] structure
/// passed by the bootloader.
#[allow(clippy::too_many_lines)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kernel_main(boot_info_ptr: *const RoverBootInfo) -> ! {
    // 1. Initialize serial port (COM1: 115200 8N1) for early logging.
    SERIAL1.lock().init();

    // 2. Display kernel startup banner.
    kprintln!();
    kprintln!("  ____                               _  __                    _ ");
    kprintln!(" |  _ \\  _____   _____ _ __         | |/ /___ _ __ _ __   ___| |");
    kprintln!(" | |_) |/ _ \\ \\ / / _ \\ '__|  _____ | ' // _ \\ '__| '_ \\ / _ \\ |");
    kprintln!(" |  _ <| (_) \\ V /  __/ |    |_____|| . \\  __/ |  | | | |  __/ |");
    kprintln!(" |_| \\_\\\\___/ \\_/ \\___|_|           |_|\\_\\___|_|  |_| |_|\\___|_|");
    kprintln!();
    kinfo!("Rover Operating System Kernel v0.1.0 (Native `#![no_std]`)");
    kinfo!("Build: Rust 2024 Edition, Native Bare-Metal Architecture");
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

    // 5. Initialize CPU descriptor tables and interrupt controller for x86_64.
    #[cfg(target_arch = "x86_64")]
    {
        kinfo!("Initializing CPU Global Descriptor Table (GDT)...");
        unsafe {
            init_early_gdt();
        }
        kinfo!("GDT successfully initialized and loaded.");

        kinfo!("Initializing CPU Interrupt Descriptor Table (IDT)...");
        unsafe {
            init_early_idt();
        }
        kinfo!("IDT successfully initialized with 22 exception handlers.");

        kinfo!("Remapping and disabling legacy 8259 PIC...");
        unsafe {
            remap_pic(32, 40);
        }
        disable_pic();
        kinfo!("Legacy 8259 PIC successfully disabled and masked.");

        kinfo!("Registering hardware IRQ handlers (Timer: 32, Keyboard: 33, Spurious: 255)...");
        register_hardware_interrupts();
        kinfo!("Hardware IRQs successfully registered in IDT.");
    }

    // 6. Initialize Physical Frame Allocator from RoverBootInfo memory map.
    kinfo!("Initializing Physical Frame Allocator from UEFI memory map...");
    unsafe {
        FRAME_ALLOCATOR.lock().init(boot_info);
    }
    kinfo!("Physical Frame Allocator initialized.");

    // 7. Initialize Kernel Heap Allocator.
    kinfo!("Initializing Kernel Heap Allocator (4 MiB capacity)...");
    let heap_start = core::ptr::addr_of_mut!(EARLY_HEAP_BUFFER) as usize;
    let heap_size = 4 * 1024 * 1024;
    unsafe {
        init_kernel_heap(heap_start, heap_size);
    }
    kinfo!("Kernel Heap Allocator successfully initialized.");

    // 8. Probe CPU hardware features via CPUID.
    kinfo!("Probing CPU capabilities...");
    let cpu = CpuCapabilities::probe();
    kinfo!("CPU Vendor: {}", cpu.vendor_str());

    // 9. Enable hardware security features (SMEP, SMAP, NXE).
    kinfo!("Configuring hardware security protections...");
    unsafe {
        enable_hardware_protections(&cpu);
    }

    // 10. Initialize Local APIC and periodic timer on x86_64.
    #[cfg(target_arch = "x86_64")]
    {
        kinfo!("Initializing Local APIC...");
        unsafe {
            LOCAL_APIC.lock().init();
        }
        kinfo!("Local APIC enabled (APIC ID: {}).", LOCAL_APIC.lock().id());

        // Setup periodic timer: vector 32, initial countdown = 100,000.
        LOCAL_APIC.lock().setup_periodic_timer(32, 100_000);
        kinfo!("Local APIC periodic timer configured on IRQ vector 32.");
    }

    // 11. Initialize hardware system call architecture.
    kinfo!("Initializing System Call architecture...");
    unsafe {
        init_syscall_architecture();
    }
    kinfo!("System Call ABI infrastructure initialized.");

    // 12. Scan PCI Bus for attached peripherals.
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

    // 13. Initialize Graphical Framebuffer Console if linear aperture present.
    if boot_info.framebuffer.is_valid() {
        kinfo!("Initializing Graphical Framebuffer Console...");
        rover_kernel::drivers::fbcon::init_framebuffer_console(boot_info.framebuffer);
    }

    // 14. Display detailed Memory Diagnostics.
    dump_memory_diagnostics();

    // 15. Initialize Preemptive Kernel Scheduler.
    kinfo!("Initializing Preemptive Kernel Scheduler...");
    SCHEDULER.lock().init();
    kinfo!(
        "Kernel Scheduler initialized with {} active tasks.",
        SCHEDULER.lock().task_count()
    );

    // 16. Enable Hardware Interrupts.
    kinfo!("Enabling hardware interrupts...");
    unsafe {
        enable_interrupts();
    }
    kinfo!("Hardware interrupts enabled.");

    kprintln!();
    kinfo!("============================================================");
    kinfo!("Rover Kernel complete modular architecture successfully running!");
    kinfo!("============================================================");

    // 17. Safely halt the processor, waking on hardware interrupts / timer ticks.
    loop {
        halt();
    }
}
