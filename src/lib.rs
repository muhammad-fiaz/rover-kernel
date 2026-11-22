//! # Rover Kernel
//!
//! A modern, modular, high-performance operating system kernel written natively in `#![no_std]` Rust.
//!
//! Designed as an independent, modular kernel with its own architecture, implementations,
//! memory management, CPU control, drivers, synchronization primitives, and system interfaces.

#![no_std]
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]

extern crate alloc;

pub mod arch;
pub mod block;
pub mod boot;
pub mod diagnostics;
pub mod drivers;
pub mod fs;
pub mod interrupts;
pub mod logging;
pub mod mm;
pub mod process;
pub mod sched;
pub mod security;
pub mod sync;
pub mod syscall;
pub mod time;

pub use arch::current::*;
pub use block::{BlockDevice, RamBlockDevice};
pub use boot::{
    ROVER_BOOT_MAGIC, ROVER_BOOT_PROTOCOL_VERSION, RoverArchId, RoverBootInfo, RoverCommandLine,
    RoverFramebuffer, RoverKernelImage, RoverMemoryDescriptor, RoverMemoryType, RoverModule,
    RoverPixelFormat, RoverSystemInfo, dump_boot_info, validate_boot_info,
};
pub use diagnostics::{MemoryInfo, dump_cpu_diagnostics, dump_memory_diagnostics, get_memory_info};
pub use drivers::fbcon::{FRAMEBUFFER_CONSOLE, FramebufferConsole, init_framebuffer_console};
pub use drivers::pci::{PciDevice, pci_read_u16, pci_read_u32, scan_pci_bus};
pub use drivers::ps2::Ps2Keyboard;
pub use drivers::serial::{COM1_BASE, SERIAL1, SerialPort};
pub use fs::{FdTable, FileStat, FileSystem, Inode, NodeType, OpenFile, RamDir, RamFile, RamFs};
pub use interrupts::{
    DEFAULT_APIC_BASE, LOCAL_APIC, LocalApic, TICK_COUNTER, disable_pic, end_of_interrupt,
    register_hardware_interrupts, remap_pic,
};
pub use logging::LogLevel;
pub use mm::{
    BitmapFrameAllocator, FRAME_ALLOCATOR, LockedHeap, PAGE_SIZE, PageTable, PageTableEntry,
    PageTableFlags, PhysAddr, VirtAddr, heap_stats, init_kernel_heap, invalidate_page, map_page,
    translate_virtual_address,
};
pub use process::{Process, ProcessId};
pub use sched::{
    CpuContext, SCHEDULER, Scheduler, TASK_STACK_SIZE, Task, TaskId, TaskState, sched_yield,
    spawn_task,
};
pub use security::{
    Capability, CapabilitySet, USER_VIRTUAL_MAX, copy_from_user, copy_to_user,
    enable_hardware_protections, is_valid_user_range,
};
pub use sync::{
    IrqSafeSpinLock, IrqSafeSpinLockGuard, Mutex, MutexGuard, OnceLock, RwLock, RwLockReadGuard,
    RwLockWriteGuard, SpinLock, SpinLockGuard,
};
pub use syscall::{
    EBADF, EFAULT, ENOSYS, SYS_EXIT, SYS_GETPID, SYS_MMAP, SYS_READ, SYS_SUCCESS, SYS_TIME,
    SYS_WRITE, SYS_YIELD, handle_syscall_dispatch, init_syscall_architecture,
};
pub use time::{TICKS_PER_SECOND, delay_cycles, read_tsc, sleep_ticks, uptime_ms, uptime_ticks};
