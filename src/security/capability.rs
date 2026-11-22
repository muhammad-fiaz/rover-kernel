//! # Capability-Based Security Model
//!
//! Fine-grained privilege bitmasks for tasks and processes.

/// A specific system privilege capability.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum Capability {
    /// Full system administrative privileges.
    SysAdmin = 1 << 0,
    /// Network configuration and raw socket access.
    NetAdmin = 1 << 1,
    /// Permission to send signals or kill other processes.
    Kill = 1 << 2,
    /// Permission to lock memory and modify paging.
    IpcLock = 1 << 3,
    /// Direct port I/O and hardware raw access.
    RawIo = 1 << 4,
}

/// Bitmask set of granted capabilities.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct CapabilitySet(pub u32);

impl CapabilitySet {
    /// Creates an empty capability set with zero privileges.
    #[must_use]
    pub const fn empty() -> Self {
        Self(0)
    }

    /// Grants a capability to this set.
    pub fn grant(&mut self, cap: Capability) {
        self.0 |= cap as u32;
    }

    /// Revokes a capability from this set.
    pub fn revoke(&mut self, cap: Capability) {
        self.0 &= !(cap as u32);
    }

    /// Returns `true` if this set has the specified capability.
    #[must_use]
    pub const fn has(&self, cap: Capability) -> bool {
        (self.0 & (cap as u32)) != 0
    }
}
