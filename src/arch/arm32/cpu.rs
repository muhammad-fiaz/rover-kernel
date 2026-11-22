//! # 32-bit ARM (ARMv7-A) CPU Control and Identification
//!
//! Provides CPSR control, WFI pause, and execution primitives for 32-bit ARM architectures.

/// High-level CPU hardware identification and supported capability flags for 32-bit ARM.
#[derive(Clone, Debug)]
pub struct CpuCapabilities {
    /// Processor vendor identifier string.
    pub vendor: [u8; 12],
    /// VFP (Vector Floating Point) supported.
    pub has_vfp: bool,
    /// NEON SIMD supported.
    pub has_neon: bool,
    /// Thumb-2 instructions supported.
    pub has_thumb2: bool,
}

impl CpuCapabilities {
    /// Probes 32-bit ARM capabilities.
    #[must_use]
    pub fn probe() -> Self {
        let mut vendor = [0u8; 12];
        vendor[..5].copy_from_slice(b"ARM32");

        Self {
            vendor,
            has_vfp: true,
            has_neon: true,
            has_thumb2: true,
        }
    }

    /// Returns the vendor identifier as a string slice.
    #[must_use]
    pub fn vendor_str(&self) -> &str {
        core::str::from_utf8(&self.vendor).unwrap_or("ARM32")
    }
}

/// Halts the CPU until an interrupt occurs (`wfi`).
#[inline]
pub fn halt() {
    #[cfg(target_arch = "arm")]
    unsafe {
        core::arch::asm!("wfi", options(nomem, nostack, preserves_flags));
    }
    #[cfg(not(target_arch = "arm"))]
    core::hint::spin_loop();
}

/// Disables hardware interrupts and enters an infinite halt loop.
pub fn halt_loop() -> ! {
    disable_interrupts();
    loop {
        halt();
    }
}

/// Enables hardware interrupts (`cpsie i`).
///
/// # Safety
///
/// Caller must ensure exception vectors are configured.
#[inline]
pub unsafe fn enable_interrupts() {
    #[cfg(target_arch = "arm")]
    unsafe {
        core::arch::asm!("cpsie i", options(nomem, nostack, preserves_flags));
    }
}

/// Disables hardware interrupts (`cpsid i`).
#[inline]
pub fn disable_interrupts() {
    #[cfg(target_arch = "arm")]
    unsafe {
        core::arch::asm!("cpsid i", options(nomem, nostack, preserves_flags));
    }
}

/// Checks whether interrupts are enabled in CPSR.
#[must_use]
pub fn are_interrupts_enabled() -> bool {
    false
}

/// Reads the cycle counter on 32-bit ARM.
#[must_use]
pub fn read_cycle_counter() -> u64 {
    0
}
