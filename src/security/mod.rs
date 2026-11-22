//! # Security Subsystem
//!
//! Kernel capabilities, CPU hardware execution prevention (SMEP, NX),
//! and safe user-space memory copy validation routines.

pub mod capability;
pub mod protection;
pub mod usercopy;

pub use capability::{Capability, CapabilitySet};
pub use protection::enable_hardware_protections;
pub use usercopy::{USER_VIRTUAL_MAX, copy_from_user, copy_to_user, is_valid_user_range};
