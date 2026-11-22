//! # Boot subsystem
//!
//! Handshake, bootloader protocol validation, command line parsing, and platform configuration.

pub mod info;
pub mod protocol;

pub use info::{dump_boot_info, validate_boot_info};
pub use protocol::{
    ROVER_BOOT_MAGIC, ROVER_BOOT_PROTOCOL_VERSION, ROVER_FLAG_ACPI, ROVER_FLAG_CMDLINE,
    ROVER_FLAG_FRAMEBUFFER, ROVER_FLAG_INITRD, ROVER_FLAG_SECURE_BOOT, ROVER_FLAG_SMBIOS,
    RoverArchId, RoverBootInfo, RoverCommandLine, RoverFramebuffer, RoverKernelImage,
    RoverMemoryDescriptor, RoverMemoryType, RoverModule, RoverPixelFormat, RoverSystemInfo,
};
