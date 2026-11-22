//! # Rover Boot Protocol Integration
//!
//! Validates and interprets the [`RoverBootInfo`] structure provided by `rover-bootloader`.

pub use rover_bootloader::protocol::{
    ROVER_BOOT_MAGIC, ROVER_BOOT_PROTOCOL_VERSION, ROVER_FLAG_ACPI, ROVER_FLAG_CMDLINE,
    ROVER_FLAG_FRAMEBUFFER, ROVER_FLAG_INITRD, ROVER_FLAG_SECURE_BOOT, ROVER_FLAG_SMBIOS,
    RoverArchId, RoverBootInfo, RoverCommandLine, RoverFramebuffer, RoverKernelImage,
    RoverMemoryDescriptor, RoverMemoryType, RoverModule, RoverPixelFormat, RoverSystemInfo,
};

/// Validates that the provided pointer to [`RoverBootInfo`] is non-null, aligned,
/// matches the expected magic bytes, and has a compatible protocol version.
///
/// # Errors
///
/// Returns an error string slice if the boot information block is invalid.
///
/// # Safety
///
/// The caller must ensure that if `ptr` is non-null, it points to a readable [`RoverBootInfo`].
pub unsafe fn validate_boot_info(
    ptr: *const RoverBootInfo,
) -> Result<&'static RoverBootInfo, &'static str> {
    if ptr.is_null() {
        return Err("BootInfo pointer is null");
    }

    if (ptr as usize) % core::mem::align_of::<RoverBootInfo>() != 0 {
        return Err("BootInfo pointer is not properly aligned");
    }

    // SAFETY: Verified non-null and properly aligned.
    let info = unsafe { &*ptr };

    if !info.is_valid() {
        return Err("BootInfo magic or protocol version mismatch");
    }

    if info.memory_map_addr == 0 || info.memory_map_entries == 0 {
        return Err("BootInfo memory map is empty or null");
    }

    Ok(info)
}

/// Dumps comprehensive boot diagnostic information to the kernel log.
#[allow(clippy::cast_possible_truncation)]
pub fn dump_boot_info(info: &RoverBootInfo) {
    let name = core::str::from_utf8(&info.bootloader_name)
        .unwrap_or("Unknown Bootloader")
        .trim_matches('\0');

    crate::kinfo!("Bootloader: {}", name);
    crate::kinfo!("Protocol Version: {}", info.version);
    crate::kinfo!("Flags: 0x{:08x}", info.flags);

    // Kernel Image Details
    crate::kinfo!("Kernel Image:");
    crate::kinfo!(
        "  Physical Base: 0x{:016x} - 0x{:016x}",
        info.kernel_image.physical_base,
        info.kernel_image
            .physical_base
            .saturating_add(info.kernel_image.size)
    );
    crate::kinfo!("  Virtual Base:  0x{:016x}", info.kernel_image.virtual_base);
    crate::kinfo!("  Size:          {} KiB", info.kernel_image.size / 1024);
    crate::kinfo!("  Entry Point:   0x{:016x}", info.kernel_image.entry_point);

    // Memory Map
    let mut total_bytes: u64 = 0;
    let mut usable_bytes: u64 = 0;

    // SAFETY: We validated memory_map_addr and count in validate_boot_info.
    let descriptors = unsafe { info.memory_map() };
    for desc in descriptors {
        let len = desc.byte_length();
        total_bytes = total_bytes.saturating_add(len);
        if desc.region_type == RoverMemoryType::Usable {
            usable_bytes = usable_bytes.saturating_add(len);
        }
    }

    crate::kinfo!(
        "Memory Map: {} descriptors, Total: {} MiB, Usable RAM: {} MiB",
        descriptors.len(),
        total_bytes / (1024 * 1024),
        usable_bytes / (1024 * 1024)
    );

    // Framebuffer Details
    if (info.flags & ROVER_FLAG_FRAMEBUFFER) != 0 && info.framebuffer.is_valid() {
        let fb = &info.framebuffer;
        crate::kinfo!(
            "Framebuffer: {}x{} @ {} stride, Format: {:?}, Base: 0x{:016x}, Size: {} KiB",
            fb.width,
            fb.height,
            fb.stride,
            fb.format,
            fb.base,
            fb.size / 1024
        );
    } else {
        crate::kinfo!("Framebuffer: Not configured or unavailable");
    }

    // Platform Firmware Details
    if (info.flags & ROVER_FLAG_ACPI) != 0 && info.acpi_rsdp != 0 {
        crate::kinfo!("ACPI RSDP Pointer: 0x{:016x}", info.acpi_rsdp);
    }
    if (info.flags & ROVER_FLAG_SMBIOS) != 0 && info.smbios_entry != 0 {
        crate::kinfo!("SMBIOS Entry Table: 0x{:016x}", info.smbios_entry);
    }

    // Command Line
    if (info.flags & ROVER_FLAG_CMDLINE) != 0 && info.cmdline.ptr != 0 && info.cmdline.length > 0 {
        // SAFETY: Pointer is provided by bootloader.
        let slice = unsafe {
            core::slice::from_raw_parts(info.cmdline.ptr as *const u8, info.cmdline.length as usize)
        };
        let cmd_str = core::str::from_utf8(slice).unwrap_or("<invalid utf8>");
        crate::kinfo!("Kernel Command Line: \"{}\"", cmd_str);
    }

    // System Info
    crate::kinfo!(
        "System Info: Arch: {:?}, Secure Boot: {}",
        info.system_info.arch,
        if info.system_info.secure_boot != 0 {
            "Enabled"
        } else {
            "Disabled"
        }
    );
}
