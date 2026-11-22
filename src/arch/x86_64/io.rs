//! # x86-64 Port I/O instructions
//!
//! Provides type-safe wrappers for low-level `in` and `out` assembly operations.

/// Reads an 8-bit unsigned integer from the specified I/O port.
///
/// # Safety
///
/// The caller must ensure that reading from `port` has no harmful hardware side effects.
#[inline]
#[must_use]
pub unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    // SAFETY: Verified by caller.
    unsafe {
        core::arch::asm!("in al, dx", in("dx") port, out("al") value, options(nomem, nostack, preserves_flags));
    }
    value
}

/// Writes an 8-bit unsigned integer to the specified I/O port.
///
/// # Safety
///
/// The caller must ensure that writing `val` to `port` is valid for the target device.
#[inline]
pub unsafe fn outb(port: u16, val: u8) {
    // SAFETY: Verified by caller.
    unsafe {
        core::arch::asm!("out dx, al", in("dx") port, in("al") val, options(nomem, nostack, preserves_flags));
    }
}

/// Reads a 16-bit unsigned integer from the specified I/O port.
///
/// # Safety
///
/// The caller must ensure that reading from `port` has no harmful hardware side effects.
#[inline]
#[must_use]
pub unsafe fn inw(port: u16) -> u16 {
    let value: u16;
    // SAFETY: Verified by caller.
    unsafe {
        core::arch::asm!("in ax, dx", in("dx") port, out("ax") value, options(nomem, nostack, preserves_flags));
    }
    value
}

/// Writes a 16-bit unsigned integer to the specified I/O port.
///
/// # Safety
///
/// The caller must ensure that writing `val` to `port` is valid for the target device.
#[inline]
pub unsafe fn outw(port: u16, val: u16) {
    // SAFETY: Verified by caller.
    unsafe {
        core::arch::asm!("out dx, ax", in("dx") port, in("ax") val, options(nomem, nostack, preserves_flags));
    }
}

/// Reads a 32-bit unsigned integer from the specified I/O port.
///
/// # Safety
///
/// The caller must ensure that reading from `port` has no harmful hardware side effects.
#[inline]
#[must_use]
pub unsafe fn inl(port: u16) -> u32 {
    let value: u32;
    // SAFETY: Verified by caller.
    unsafe {
        core::arch::asm!("in eax, dx", in("dx") port, out("eax") value, options(nomem, nostack, preserves_flags));
    }
    value
}

/// Writes a 32-bit unsigned integer to the specified I/O port.
///
/// # Safety
///
/// The caller must ensure that writing `val` to `port` is valid for the target device.
#[inline]
pub unsafe fn outl(port: u16, val: u32) {
    // SAFETY: Verified by caller.
    unsafe {
        core::arch::asm!("out dx, eax", in("dx") port, in("eax") val, options(nomem, nostack, preserves_flags));
    }
}

/// Pauses CPU execution for a brief moment to allow slow I/O buses to settle.
#[inline]
pub fn io_wait() {
    // Port 0x80 is traditionally used as an unused diagnostic port for brief I/O delay.
    // SAFETY: Writing to port 0x80 is harmless on standard PC platforms.
    unsafe {
        outb(0x80, 0);
    }
}
