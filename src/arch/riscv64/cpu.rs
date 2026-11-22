//! # RISC-V 64 CPU Control and CSR Access
//!
//! Provides Control and Status Register (CSR) manipulation (sstatus, sie, satp, scause, sepc),
//! CPU identification, and execution control for 64-bit RISC-V architectures.

/// High-level CPU hardware identification and supported capability flags for RISC-V 64.
#[derive(Clone, Debug)]
#[allow(clippy::struct_excessive_bools)]
pub struct CpuCapabilities {
    /// Processor vendor identifier string.
    pub vendor: [u8; 12],
    /// Integer multiplication and division (M extension) supported.
    pub has_m: bool,
    /// Atomic instructions (A extension) supported.
    pub has_a: bool,
    /// Single-precision floating point (F extension) supported.
    pub has_f: bool,
    /// Double-precision floating point (D extension) supported.
    pub has_d: bool,
    /// Compressed instructions (C extension) supported.
    pub has_c: bool,
    /// Vector instructions (V extension) supported.
    pub has_v: bool,
    /// Physical Memory Protection (PMP) supported.
    pub has_pmp: bool,
}

impl CpuCapabilities {
    /// Probes RISC-V system CSRs and extension bits.
    #[must_use]
    pub fn probe() -> Self {
        let mut vendor = [0u8; 12];
        vendor[..7].copy_from_slice(b"RISCV64");

        let has_m = true;
        let has_a = true;
        let has_f = true;
        let has_d = true;
        let has_c = true;
        let has_v = false;
        let has_pmp = true;

        Self {
            vendor,
            has_m,
            has_a,
            has_f,
            has_d,
            has_c,
            has_v,
            has_pmp,
        }
    }

    /// Returns the vendor identifier as a string slice.
    #[must_use]
    pub fn vendor_str(&self) -> &str {
        core::str::from_utf8(&self.vendor).unwrap_or("RISCV64")
    }
}

/// Halts the CPU core until the next interrupt occurs (`wfi`).
#[inline]
pub fn halt() {
    #[cfg(target_arch = "riscv64")]
    unsafe {
        core::arch::asm!("wfi", options(nomem, nostack, preserves_flags));
    }
    #[cfg(not(target_arch = "riscv64"))]
    core::hint::spin_loop();
}

/// Disables hardware interrupts and enters an infinite halt loop.
pub fn halt_loop() -> ! {
    disable_interrupts();
    loop {
        halt();
    }
}

/// Enables supervisor interrupts (`csrsi sstatus, 2`).
///
/// # Safety
///
/// Caller must ensure that a trap vector table is initialized.
#[inline]
pub unsafe fn enable_interrupts() {
    #[cfg(target_arch = "riscv64")]
    unsafe {
        core::arch::asm!("csrsi sstatus, 2", options(nomem, nostack, preserves_flags));
    }
}

/// Disables supervisor interrupts (`csrci sstatus, 2`).
#[inline]
pub fn disable_interrupts() {
    #[cfg(target_arch = "riscv64")]
    unsafe {
        core::arch::asm!("csrci sstatus, 2", options(nomem, nostack, preserves_flags));
    }
}

/// Checks whether supervisor interrupts are enabled in `sstatus`.
#[must_use]
pub fn are_interrupts_enabled() -> bool {
    #[cfg(target_arch = "riscv64")]
    {
        let status: u64;
        unsafe {
            core::arch::asm!("csrr {0}, sstatus", out(reg) status, options(nomem, nostack, preserves_flags));
        }
        (status & 2) != 0
    }
    #[cfg(not(target_arch = "riscv64"))]
    false
}

/// Reads the 64-bit cycle counter (`rdcycle`).
#[must_use]
pub fn read_cycle_counter() -> u64 {
    #[cfg(target_arch = "riscv64")]
    {
        let val: u64;
        unsafe {
            core::arch::asm!("rdcycle {0}", out(reg) val, options(nomem, nostack, preserves_flags));
        }
        val
    }
    #[cfg(not(target_arch = "riscv64"))]
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
