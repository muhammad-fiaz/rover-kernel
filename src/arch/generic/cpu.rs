//! # Generic CPU Control and Identification
//!
//! Provides portable CPU control and execution primitives for generic target environments.

/// High-level CPU hardware identification and supported capability flags for generic targets.
#[derive(Clone, Debug)]
pub struct CpuCapabilities {
    /// Processor vendor identifier string.
    pub vendor: [u8; 12],
    /// Virtualization support.
    pub has_virtualization: bool,
}

impl CpuCapabilities {
    /// Probes generic CPU capabilities.
    #[must_use]
    pub fn probe() -> Self {
        let mut vendor = [0u8; 12];
        vendor[..7].copy_from_slice(b"GENERIC");

        Self {
            vendor,
            has_virtualization: false,
        }
    }

    /// Returns the vendor identifier as a string slice.
    #[must_use]
    pub fn vendor_str(&self) -> &str {
        core::str::from_utf8(&self.vendor).unwrap_or("GENERIC")
    }
}

/// Halts the CPU until an interrupt occurs.
#[inline]
pub fn halt() {
    core::hint::spin_loop();
}

/// Disables hardware interrupts and enters an infinite halt loop.
pub fn halt_loop() -> ! {
    disable_interrupts();
    loop {
        halt();
    }
}

/// Enables hardware interrupts.
///
/// # Safety
///
/// Caller must ensure exception vectors are configured.
#[inline]
pub unsafe fn enable_interrupts() {}

/// Disables hardware interrupts.
#[inline]
pub fn disable_interrupts() {}

/// Checks whether interrupts are enabled.
#[must_use]
pub fn are_interrupts_enabled() -> bool {
    false
}

/// Reads the cycle counter.
#[must_use]
pub fn read_cycle_counter() -> u64 {
    0
}

/// Reads a Model-Specific Register stub for compatibility.
///
/// # Safety
///
/// Hardware register access stub.
#[inline]
#[must_use]
pub unsafe fn read_msr(_msr: u32) -> u64 {
    0
}

/// Writes a Model-Specific Register stub for compatibility.
///
/// # Safety
///
/// Hardware register access stub.
#[inline]
pub unsafe fn write_msr(_msr: u32, _val: u64) {}
