//! # Boot subsystem
//!
//! Handshake, bootloader protocol validation, command line parsing, and platform configuration.

pub mod info;

pub use info::{
    ROVER_BOOT_MAGIC, ROVER_BOOT_PROTOCOL_VERSION, RoverArchId, RoverBootInfo, RoverCommandLine,
    RoverFramebuffer, RoverKernelImage, RoverMemoryDescriptor, RoverMemoryType, RoverModule,
    RoverPixelFormat, RoverSystemInfo, dump_boot_info, validate_boot_info,
};
