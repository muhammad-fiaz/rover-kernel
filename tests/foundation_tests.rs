//! # Rover Kernel Foundation Unit & Integration Tests
//!
//! Validates boot protocol structures, synchronization, GDT/IDT descriptors,
//! and CPU feature parsing.

#![allow(
    clippy::unwrap_used,
    clippy::unreadable_literal,
    clippy::borrow_as_ptr,
    clippy::doc_markdown
)]

use rover_kernel::arch::x86_64::CpuCapabilities;
use rover_kernel::arch::x86_64::gdt::{
    GdtEntry, GlobalDescriptorTable, KERNEL_CODE_SELECTOR, KERNEL_DATA_SELECTOR,
    USER_CODE_SELECTOR, USER_DATA_SELECTOR,
};
use rover_kernel::arch::x86_64::idt::{EXCEPTION_NAMES, InterruptDescriptorTable};
use rover_kernel::boot::{
    ROVER_BOOT_MAGIC, ROVER_BOOT_PROTOCOL_VERSION, RoverArchId, RoverBootInfo, RoverCommandLine,
    RoverFramebuffer, RoverKernelImage, RoverMemoryDescriptor, RoverMemoryType, RoverModule,
    RoverSystemInfo, validate_boot_info,
};
use rover_kernel::sync::SpinLock;

#[test]
fn test_boot_protocol_magic_and_version() {
    assert_eq!(&ROVER_BOOT_MAGIC[..13], b"ROVER_BOOT_V1");
    assert_eq!(ROVER_BOOT_PROTOCOL_VERSION, 1);
}

#[test]
fn test_validate_boot_info_valid() {
    let mut memory_descriptors = [RoverMemoryDescriptor::new(
        RoverMemoryType::Usable,
        0x100000,
        0,
        256,
        0,
    )];

    let boot_info = RoverBootInfo {
        magic: ROVER_BOOT_MAGIC,
        version: ROVER_BOOT_PROTOCOL_VERSION,
        flags: 0,
        bootloader_name: [0u8; 32],
        memory_map_addr: memory_descriptors.as_mut_ptr() as u64,
        memory_map_entries: 1,
        memory_map_entry_size: size_of::<RoverMemoryDescriptor>() as u64,
        memory_map_entry_version: 1,
        pad0: 0,
        framebuffer: RoverFramebuffer::empty(),
        acpi_rsdp: 0,
        smbios_entry: 0,
        kernel_image: RoverKernelImage::empty(),
        initrd: RoverModule::empty(),
        cmdline: RoverCommandLine::empty(),
        system_info: RoverSystemInfo::new(1, RoverArchId::X86_64, false),
        reserved: [0u64; 8],
    };

    let result = unsafe { validate_boot_info(&boot_info) };
    assert!(result.is_ok());
    let validated = result.unwrap();
    assert_eq!(validated.version, 1);
}

#[test]
fn test_validate_boot_info_null_and_corrupt() {
    // Null pointer
    let null_res = unsafe { validate_boot_info(core::ptr::null()) };
    assert!(null_res.is_err());

    // Corrupted magic
    let bad_magic_info = RoverBootInfo {
        magic: *b"CORRUPTED_MAGIC!",
        version: 1,
        flags: 0,
        bootloader_name: [0u8; 32],
        memory_map_addr: 0x1000,
        memory_map_entries: 1,
        memory_map_entry_size: 64,
        memory_map_entry_version: 1,
        pad0: 0,
        framebuffer: RoverFramebuffer::empty(),
        acpi_rsdp: 0,
        smbios_entry: 0,
        kernel_image: RoverKernelImage::empty(),
        initrd: RoverModule::empty(),
        cmdline: RoverCommandLine::empty(),
        system_info: RoverSystemInfo::new(1, RoverArchId::X86_64, false),
        reserved: [0u64; 8],
    };
    assert!(unsafe { validate_boot_info(&bad_magic_info) }.is_err());

    // Incompatible version
    let mut bad_version_info = bad_magic_info;
    bad_version_info.magic = ROVER_BOOT_MAGIC;
    bad_version_info.version = 999;
    assert!(unsafe { validate_boot_info(&bad_version_info) }.is_err());
}

#[test]
fn test_spinlock_mutual_exclusion() {
    let lock = SpinLock::new(42);
    assert!(!lock.is_locked());

    {
        let mut guard = lock.lock();
        assert!(lock.is_locked());
        assert_eq!(*guard, 42);
        *guard = 100;
        assert!(lock.try_lock().is_none());
    }

    assert!(!lock.is_locked());
    let guard = lock.lock();
    assert_eq!(*guard, 100);
}

#[test]
fn test_gdt_selectors_and_entries() {
    assert_eq!(KERNEL_CODE_SELECTOR, 0x08);
    assert_eq!(KERNEL_DATA_SELECTOR, 0x10);
    assert_eq!(USER_DATA_SELECTOR, 0x1B);
    assert_eq!(USER_CODE_SELECTOR, 0x23);

    let null_entry = GdtEntry::NULL;
    assert_eq!(null_entry.0, 0);

    let kernel_code = GdtEntry::code_segment(0);
    // Must have Present (bit 47) and Long Mode (bit 53) set
    assert_ne!(kernel_code.0 & (1 << 47), 0);
    assert_ne!(kernel_code.0 & (1 << 53), 0);

    let gdt = GlobalDescriptorTable::default();
    assert_eq!(core::mem::size_of_val(&gdt), 48); // 5 entries (40 bytes) aligned to 16 bytes
}

#[test]
fn test_idt_layout_and_exception_names() {
    let mut idt = InterruptDescriptorTable::default();
    assert_eq!(core::mem::size_of_val(&idt), 256 * 16);

    let dummy_handler = 0xFFFF_8000_0010_2030usize;
    idt.set_gate(14, dummy_handler);

    assert_eq!(EXCEPTION_NAMES.len(), 32);
    assert_eq!(EXCEPTION_NAMES[0], "Divide Error (#DE)");
    assert_eq!(EXCEPTION_NAMES[13], "General Protection Fault (#GP)");
    assert_eq!(EXCEPTION_NAMES[14], "Page Fault (#PF)");
}

#[test]
fn test_cpu_capabilities_vendor_string() {
    let caps = CpuCapabilities {
        vendor: *b"AuthenticAMD",
        has_apic: true,
        has_sse: true,
        has_sse2: true,
        has_sse3: true,
        has_sse4_1: true,
        has_sse4_2: true,
        has_avx: true,
        has_avx2: true,
        has_smep: true,
        has_smap: true,
        has_long_mode: true,
        has_nx: true,
        has_page_1gb: true,
    };
    assert_eq!(caps.vendor_str(), "AuthenticAMD");
}
