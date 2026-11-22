//! # IA-32 Port I/O Primitives
//!
//! Direct x86 port I/O using `in` and `out` instructions.

/// Reads an 8-bit byte from an x86 I/O port.
#[inline]
#[must_use]
pub fn inb(port: u16) -> u8 {
    let val: u8;
    #[cfg(target_arch = "x86")]
    unsafe {
        core::arch::asm!("in al, dx", in("dx") port, out("al") val, options(nomem, nostack, preserves_flags));
    }
    #[cfg(not(target_arch = "x86"))]
    {
        let _ = port;
        val = 0;
    }
    val
}

/// Writes an 8-bit byte to an x86 I/O port.
#[inline]
pub fn outb(port: u16, val: u8) {
    #[cfg(target_arch = "x86")]
    unsafe {
        core::arch::asm!("out dx, al", in("dx") port, in("al") val, options(nomem, nostack, preserves_flags));
    }
    #[cfg(not(target_arch = "x86"))]
    {
        let _ = (port, val);
    }
}

/// Reads a 16-bit word from an x86 I/O port.
#[inline]
#[must_use]
pub fn inw(port: u16) -> u16 {
    let val: u16;
    #[cfg(target_arch = "x86")]
    unsafe {
        core::arch::asm!("in ax, dx", in("dx") port, out("ax") val, options(nomem, nostack, preserves_flags));
    }
    #[cfg(not(target_arch = "x86"))]
    {
        let _ = port;
        val = 0;
    }
    val
}

/// Writes a 16-bit word to an x86 I/O port.
#[inline]
pub fn outw(port: u16, val: u16) {
    #[cfg(target_arch = "x86")]
    unsafe {
        core::arch::asm!("out dx, ax", in("dx") port, in("ax") val, options(nomem, nostack, preserves_flags));
    }
    #[cfg(not(target_arch = "x86"))]
    {
        let _ = (port, val);
    }
}

/// Reads a 32-bit double-word from an x86 I/O port.
#[inline]
#[must_use]
pub fn inl(port: u16) -> u32 {
    let val: u32;
    #[cfg(target_arch = "x86")]
    unsafe {
        core::arch::asm!("in eax, dx", in("dx") port, out("eax") val, options(nomem, nostack, preserves_flags));
    }
    #[cfg(not(target_arch = "x86"))]
    {
        let _ = port;
        val = 0;
    }
    val
}

/// Writes a 32-bit double-word to an x86 I/O port.
#[inline]
pub fn outl(port: u16, val: u32) {
    #[cfg(target_arch = "x86")]
    unsafe {
        core::arch::asm!("out dx, eax", in("dx") port, in("eax") val, options(nomem, nostack, preserves_flags));
    }
    #[cfg(not(target_arch = "x86"))]
    {
        let _ = (port, val);
    }
}

/// Pauses momentarily by writing to port 0x80.
#[inline]
pub fn io_wait() {
    outb(0x80, 0);
}
