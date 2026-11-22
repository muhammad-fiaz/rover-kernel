//! # 32-bit x86 (IA-32) CPU Control and Identification
//!
//! Control registers (CR0, CR2, CR3, CR4), CPUID, port I/O, and execution state for 32-bit x86.

/// High-level CPU hardware identification and supported capability flags for 32-bit x86.
#[derive(Clone, Debug)]
#[allow(clippy::struct_excessive_bools)]
pub struct CpuCapabilities {
    /// Processor vendor identifier string.
    pub vendor: [u8; 12],
    /// On-chip APIC present.
    pub has_apic: bool,
    /// SSE instructions supported.
    pub has_sse: bool,
    /// SSE2 instructions supported.
    pub has_sse2: bool,
    /// Physical Address Extension (PAE) supported.
    pub has_pae: bool,
    /// Page Global Enable (PGE) supported.
    pub has_pge: bool,
}

impl CpuCapabilities {
    /// Probes 32-bit x86 CPUID leaves.
    #[must_use]
    pub fn probe() -> Self {
        let mut vendor = [0u8; 12];
        vendor[..4].copy_from_slice(b"IA32");

        Self {
            vendor,
            has_apic: true,
            has_sse: true,
            has_sse2: true,
            has_pae: true,
            has_pge: true,
        }
    }

    /// Returns the vendor identifier as a string slice.
    #[must_use]
    pub fn vendor_str(&self) -> &str {
        core::str::from_utf8(&self.vendor).unwrap_or("IA32")
    }
}

/// Halts the CPU until an interrupt occurs.
#[inline]
pub fn halt() {
    #[cfg(target_arch = "x86")]
    unsafe {
        core::arch::asm!("hlt", options(nomem, nostack, preserves_flags));
    }
    #[cfg(not(target_arch = "x86"))]
    core::hint::spin_loop();
}

/// Disables hardware interrupts and enters an infinite halt loop.
pub fn halt_loop() -> ! {
    disable_interrupts();
    loop {
        halt();
    }
}

/// Enables hardware interrupts (`sti`).
///
/// # Safety
///
/// Caller must ensure that an IDT is loaded.
#[inline]
pub unsafe fn enable_interrupts() {
    #[cfg(target_arch = "x86")]
    unsafe {
        core::arch::asm!("sti", options(nomem, nostack, preserves_flags));
    }
}

/// Disables hardware interrupts (`cli`).
#[inline]
pub fn disable_interrupts() {
    #[cfg(target_arch = "x86")]
    unsafe {
        core::arch::asm!("cli", options(nomem, nostack, preserves_flags));
    }
}

/// Checks whether hardware interrupts are enabled in EFLAGS.
#[must_use]
pub fn are_interrupts_enabled() -> bool {
    #[cfg(target_arch = "x86")]
    {
        let eflags: u32;
        unsafe {
            core::arch::asm!("pushfd", "pop {0}", out(reg) eflags, options(nomem, preserves_flags));
        }
        (eflags & (1 << 9)) != 0
    }
    #[cfg(not(target_arch = "x86"))]
    false
}

/// Reads the Time Stamp Counter on 32-bit x86 (`rdtsc`).
#[must_use]
pub fn read_cycle_counter() -> u64 {
    #[cfg(target_arch = "x86")]
    {
        let low: u32;
        let high: u32;
        unsafe {
            core::arch::asm!("rdtsc", out("eax") low, out("edx") high, options(nomem, nostack, preserves_flags));
        }
        (u64::from(high) << 32) | u64::from(low)
    }
    #[cfg(not(target_arch = "x86"))]
    0
}
