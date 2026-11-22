//! # Generic I/O Emulation
//!
//! Provides portable I/O accessors for generic targets.

/// Reads an 8-bit byte from an MMIO address or port index.
///
/// # Safety
///
/// Caller must ensure port or memory mapped register is valid.
#[inline]
#[must_use]
pub unsafe fn inb(_port: u16) -> u8 {
    0
}

/// Writes an 8-bit byte to an MMIO address or port index.
///
/// # Safety
///
/// Caller must ensure port or memory mapped register is valid.
#[inline]
pub unsafe fn outb(_port: u16, _val: u8) {}

/// Reads a 16-bit word from an MMIO address or port index.
///
/// # Safety
///
/// Caller must ensure port or memory mapped register is valid.
#[inline]
#[must_use]
pub unsafe fn inw(_port: u16) -> u16 {
    0
}

/// Writes a 16-bit word to an MMIO address or port index.
///
/// # Safety
///
/// Caller must ensure port or memory mapped register is valid.
#[inline]
pub unsafe fn outw(_port: u16, _val: u16) {}

/// Reads a 32-bit double-word from an MMIO address or port index.
///
/// # Safety
///
/// Caller must ensure port or memory mapped register is valid.
#[inline]
#[must_use]
pub unsafe fn inl(_port: u16) -> u32 {
    0
}

/// Writes a 32-bit double-word to an MMIO address or port index.
///
/// # Safety
///
/// Caller must ensure port or memory mapped register is valid.
#[inline]
pub unsafe fn outl(_port: u16, _val: u32) {}

/// Short delay for I/O operations.
#[inline]
pub fn io_wait() {
    core::hint::spin_loop();
}
