//! # Rover Boot Protocol Specification
//!
//! Authoritative data structures, memory descriptors, framebuffer geometry,
//! and protocol constants passed from the firmware bootloader to Rover Kernel.

/// The Rover Boot Protocol magic byte sequence ("`ROVER_BOOT_V1\0\0\0`").
pub const ROVER_BOOT_MAGIC: [u8; 16] = *b"ROVER_BOOT_V1\0\0\0";

/// Current Rover Boot Protocol version.
pub const ROVER_BOOT_PROTOCOL_VERSION: u32 = 1;

/// Flag indicating that a valid linear framebuffer is present.
pub const ROVER_FLAG_FRAMEBUFFER: u32 = 1 << 0;

/// Flag indicating that an ACPI RSDP table pointer is present.
pub const ROVER_FLAG_ACPI: u32 = 1 << 1;

/// Flag indicating that an SMBIOS entry point pointer is present.
pub const ROVER_FLAG_SMBIOS: u32 = 1 << 2;

/// Flag indicating that an initial ramdisk (initrd) was loaded.
pub const ROVER_FLAG_INITRD: u32 = 1 << 3;

/// Flag indicating that a kernel command-line string is present.
pub const ROVER_FLAG_CMDLINE: u32 = 1 << 4;

/// Flag indicating that Secure Boot is enabled.
pub const ROVER_FLAG_SECURE_BOOT: u32 = 1 << 5;

/// Pixel format enumeration for the graphics display.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum RoverPixelFormat {
    /// 32-bit color: Red 8-bit, Green 8-bit, Blue 8-bit, Reserved 8-bit.
    Rgb = 0,
    /// 32-bit color: Blue 8-bit, Green 8-bit, Red 8-bit, Reserved 8-bit.
    Bgr = 1,
    /// Custom bitmask color described by mask fields.
    Bitmask = 2,
    /// BLT-only framebuffer without direct memory aperture access.
    BltOnly = 3,
    /// Unknown or unsupported pixel format.
    Unknown = 4,
}

/// Linear framebuffer metadata provided to the kernel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct RoverFramebuffer {
    /// Physical base address of the video memory aperture.
    pub base: u64,
    /// Total size in bytes of the framebuffer mapping.
    pub size: u64,
    /// Horizontal resolution in pixels.
    pub width: u32,
    /// Vertical resolution in pixels.
    pub height: u32,
    /// Number of pixels per horizontal scanline (stride).
    pub stride: u32,
    /// Color format of the framebuffer pixels.
    pub format: RoverPixelFormat,
    /// Bitmask for red color channel (valid when `format == Bitmask`).
    pub red_mask: u32,
    /// Bitmask for green color channel (valid when `format == Bitmask`).
    pub green_mask: u32,
    /// Bitmask for blue color channel (valid when `format == Bitmask`).
    pub blue_mask: u32,
    /// Bitmask for reserved/alpha channel (valid when `format == Bitmask`).
    pub reserved_mask: u32,
}

impl RoverFramebuffer {
    /// Returns an empty or uninitialized framebuffer descriptor.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            base: 0,
            size: 0,
            width: 0,
            height: 0,
            stride: 0,
            format: RoverPixelFormat::Unknown,
            red_mask: 0,
            green_mask: 0,
            blue_mask: 0,
            reserved_mask: 0,
        }
    }

    /// Returns `true` if a usable linear framebuffer is present.
    #[must_use]
    pub const fn is_valid(&self) -> bool {
        self.base != 0 && self.size != 0 && self.width != 0 && self.height != 0
    }
}

/// Classification of physical memory regions passed to the kernel.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum RoverMemoryType {
    /// Available system RAM suitable for general kernel use.
    Usable = 1,
    /// Reserved memory region that must not be used by the kernel.
    Reserved = 2,
    /// ACPI reclaimable memory containing tables that may be reused after parsing.
    AcpiReclaimable = 3,
    /// ACPI Non-Volatile Storage (NVS) memory required across sleep/wake transitions.
    AcpiNvs = 4,
    /// Defective hardware memory regions.
    BadMemory = 5,
    /// Memory used by the bootloader that can be reclaimed after boot initialization.
    BootloaderReclaimable = 6,
    /// Memory containing loaded kernel image code, data, and boot modules.
    KernelAndModules = 7,
    /// Memory mapped for the graphics framebuffer.
    Framebuffer = 8,
}

/// A physical memory region descriptor in the Rover Boot Protocol.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct RoverMemoryDescriptor {
    /// The memory region type.
    pub region_type: RoverMemoryType,
    /// Padding for 64-bit alignment.
    pub pad: u32,
    /// The physical starting address of the memory region.
    pub physical_start: u64,
    /// The virtual starting address (if mapped), or zero.
    pub virtual_start: u64,
    /// The number of 4 KiB pages in the region.
    pub number_of_pages: u64,
    /// Memory cacheability and access attributes.
    pub attribute: u64,
}

impl RoverMemoryDescriptor {
    /// Creates a new memory descriptor.
    #[must_use]
    pub const fn new(
        region_type: RoverMemoryType,
        physical_start: u64,
        virtual_start: u64,
        number_of_pages: u64,
        attribute: u64,
    ) -> Self {
        Self {
            region_type,
            pad: 0,
            physical_start,
            virtual_start,
            number_of_pages,
            attribute,
        }
    }

    /// Returns the length in bytes of the region.
    #[must_use]
    pub const fn byte_length(&self) -> u64 {
        self.number_of_pages.saturating_mul(4096)
    }

    /// Returns the physical end address (exclusive).
    #[must_use]
    pub const fn physical_end(&self) -> u64 {
        self.physical_start.saturating_add(self.byte_length())
    }
}

/// Processor architecture identifier in the boot protocol.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum RoverArchId {
    /// Unknown or generic architecture.
    Unknown = 0,
    /// x86-64 (AMD64 / Intel 64).
    X86_64 = 1,
    /// 64-bit ARM (`AArch64`).
    Aarch64 = 2,
    /// 64-bit RISC-V (RV64GC).
    Riscv64 = 3,
    /// 32-bit x86 (IA-32).
    Ia32 = 4,
    /// 32-bit ARM.
    Arm32 = 5,
    /// 64-bit `LoongArch` (LA64).
    LoongArch64 = 6,
}

/// Information describing the loaded kernel executable image.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct RoverKernelImage {
    /// Physical base address where kernel code and data were placed.
    pub physical_base: u64,
    /// Virtual base address as specified by the ELF headers.
    pub virtual_base: u64,
    /// Total memory size in bytes allocated for all segments.
    pub size: u64,
    /// Kernel entry point address to which control is transferred.
    pub entry_point: u64,
}

impl RoverKernelImage {
    /// Returns an empty kernel image descriptor.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            physical_base: 0,
            virtual_base: 0,
            size: 0,
            entry_point: 0,
        }
    }
}

/// A loaded boot module, such as an initial ramdisk (initrd).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct RoverModule {
    /// Physical start address of the loaded module.
    pub physical_start: u64,
    /// Size of the module in bytes.
    pub size: u64,
    /// UTF-8 null-terminated module identifier or mount path.
    pub name: [u8; 64],
}

impl RoverModule {
    /// Creates an empty module descriptor.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            physical_start: 0,
            size: 0,
            name: [0u8; 64],
        }
    }
}

/// Kernel command line pointer and length.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct RoverCommandLine {
    /// Physical address of the UTF-8 null-terminated command line buffer.
    pub ptr: u64,
    /// Length of the command line string in bytes (excluding null terminator).
    pub length: u64,
}

impl RoverCommandLine {
    /// Returns an empty command line.
    #[must_use]
    pub const fn empty() -> Self {
        Self { ptr: 0, length: 0 }
    }
}

/// Firmware and platform identity information passed to the kernel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct RoverSystemInfo {
    /// Firmware environment type (1 = UEFI).
    pub firmware_type: u32,
    /// Firmware revision reported by the system table.
    pub firmware_revision: u32,
    /// Active target CPU architecture.
    pub arch: RoverArchId,
    /// Flag indicating whether Secure Boot is enabled.
    pub secure_boot: u32,
}

impl RoverSystemInfo {
    /// Creates a new system info block.
    #[must_use]
    pub const fn new(firmware_revision: u32, arch: RoverArchId, secure_boot: bool) -> Self {
        Self {
            firmware_type: 1, // UEFI
            firmware_revision,
            arch,
            secure_boot: if secure_boot { 1 } else { 0 },
        }
    }
}

/// Master boot information block passed from the bootloader to the target kernel.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct RoverBootInfo {
    /// Protocol magic bytes matching [`ROVER_BOOT_MAGIC`].
    pub magic: [u8; 16],
    /// Protocol specification version matching [`ROVER_BOOT_PROTOCOL_VERSION`].
    pub version: u32,
    /// Bitmask flags indicating which fields are populated.
    pub flags: u32,
    /// Human-readable bootloader name and version.
    pub bootloader_name: [u8; 32],
    /// Physical address of the array of [`RoverMemoryDescriptor`] entries.
    pub memory_map_addr: u64,
    /// Number of memory descriptors in the memory map array.
    pub memory_map_entries: u64,
    /// Size in bytes of each descriptor in the memory map.
    pub memory_map_entry_size: u64,
    /// Memory descriptor format version.
    pub memory_map_entry_version: u32,
    /// Padding for 64-bit alignment.
    pub pad0: u32,
    /// Framebuffer configuration and aperture mapping.
    pub framebuffer: RoverFramebuffer,
    /// Physical address of the ACPI Root System Description Pointer (RSDP), or 0.
    pub acpi_rsdp: u64,
    /// Physical address of the SMBIOS entry point table, or 0.
    pub smbios_entry: u64,
    /// Loaded kernel executable image geometry.
    pub kernel_image: RoverKernelImage,
    /// Initial ramdisk module, if loaded.
    pub initrd: RoverModule,
    /// Kernel command-line arguments.
    pub cmdline: RoverCommandLine,
    /// Firmware and platform identity details.
    pub system_info: RoverSystemInfo,
    /// Reserved space for forward-compatible protocol extensions.
    pub reserved: [u64; 8],
}

impl RoverBootInfo {
    /// Validates the structure magic and version.
    #[must_use]
    pub const fn is_valid(&self) -> bool {
        let mut i = 0;
        while i < 16 {
            if self.magic[i] != ROVER_BOOT_MAGIC[i] {
                return false;
            }
            i += 1;
        }
        self.version == ROVER_BOOT_PROTOCOL_VERSION
    }

    /// Returns a slice of the memory descriptors if the pointer and count are valid.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `memory_map_addr` points to valid physical/virtual memory
    /// containing at least `memory_map_entries` descriptors.
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub unsafe fn memory_map(&self) -> &[RoverMemoryDescriptor] {
        if self.memory_map_addr == 0 || self.memory_map_entries == 0 {
            return &[];
        }
        // SAFETY: Ensured by caller.
        unsafe {
            core::slice::from_raw_parts(
                self.memory_map_addr as *const RoverMemoryDescriptor,
                self.memory_map_entries as usize,
            )
        }
    }
}
