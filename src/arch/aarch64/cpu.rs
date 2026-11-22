//! # AArch64 CPU Control and Register Access
//!
//! Provides low-level ARMv8-A CPU controls, system register access,
//! exception level queries, and pipeline barriers.

/// High-level CPU hardware identification and supported capability flags for ARM64.
#[derive(Clone, Debug)]
#[allow(clippy::struct_excessive_bools)]
pub struct CpuCapabilities {
    /// Processor vendor identifier string.
    pub vendor: [u8; 12],
    /// Floating Point (FP) unit present.
    pub has_fp: bool,
    /// Advanced SIMD (NEON) present.
    pub has_asimd: bool,
    /// AES cryptography instructions supported.
    pub has_aes: bool,
    /// SHA2 cryptography instructions supported.
    pub has_sha2: bool,
    /// CRC32 checksum instructions supported.
    pub has_crc32: bool,
    /// Large System Extensions (LSE / Atomics) supported.
    pub has_atomics: bool,
    /// Privileged Access Never (PAN) supported.
    pub has_pan: bool,
    /// Privileged Execute Never (PXN) supported.
    pub has_pxn: bool,
}

impl CpuCapabilities {
    /// Probes ARM64 system registers and feature fields.
    #[must_use]
    pub fn probe() -> Self {
        let mut vendor = [0u8; 12];
        vendor[..5].copy_from_slice(b"ARM64");

        let has_fp = true;
        let has_asimd = true;
        let has_pxn = true;

        #[cfg(target_arch = "aarch64")]
        let (has_aes, has_sha2, has_crc32, has_atomics, has_pan) = unsafe {
            let isar0: u64;
            core::arch::asm!(
                "mrs {0}, id_aa64isar0_el1",
                out(reg) isar0,
                options(nomem, nostack, preserves_flags)
            );
            let mmfr1: u64;
            core::arch::asm!(
                "mrs {0}, id_aa64mmfr1_el1",
                out(reg) mmfr1,
                options(nomem, nostack, preserves_flags)
            );
            (
                (isar0 & 0x0000_00F0) != 0,
                (isar0 & 0x0000_F000) != 0,
                (isar0 & 0x00F0_0000) != 0,
                (isar0 & 0x0000_0F00) != 0,
                (mmfr1 & 0x00F0_0000) != 0,
            )
        };

        #[cfg(not(target_arch = "aarch64"))]
        let (has_aes, has_sha2, has_crc32, has_atomics, has_pan) =
            (false, false, false, false, false);

        Self {
            vendor,
            has_fp,
            has_asimd,
            has_aes,
            has_sha2,
            has_crc32,
            has_atomics,
            has_pan,
            has_pxn,
        }
    }

    /// Returns the vendor identifier as a string slice.
    #[must_use]
    pub fn vendor_str(&self) -> &str {
        core::str::from_utf8(&self.vendor).unwrap_or("ARM64")
    }
}

/// Halts the CPU until an interrupt occurs (`wfi`).
#[inline]
pub fn halt() {
    #[cfg(target_arch = "aarch64")]
    unsafe {
        core::arch::asm!("wfi", options(nomem, nostack, preserves_flags));
    }
    #[cfg(not(target_arch = "aarch64"))]
    core::hint::spin_loop();
}

/// Disables hardware interrupts and enters an infinite halt loop.
pub fn halt_loop() -> ! {
    disable_interrupts();
    loop {
        halt();
    }
}

/// Enables hardware interrupts (`msr daifclr, #2`).
///
/// # Safety
///
/// The caller must ensure that exception vectors are registered and valid.
#[inline]
pub unsafe fn enable_interrupts() {
    #[cfg(target_arch = "aarch64")]
    unsafe {
        core::arch::asm!("msr daifclr, #2", options(nomem, nostack, preserves_flags));
    }
}

/// Disables hardware interrupts (`msr daifset, #2`).
#[inline]
pub fn disable_interrupts() {
    #[cfg(target_arch = "aarch64")]
    unsafe {
        core::arch::asm!("msr daifset, #2", options(nomem, nostack, preserves_flags));
    }
}

/// Returns `true` if hardware interrupts are currently enabled in `DAIF`.
#[must_use]
pub fn are_interrupts_enabled() -> bool {
    #[cfg(target_arch = "aarch64")]
    {
        let daif: u64;
        unsafe {
            core::arch::asm!("mrs {0}, daif", out(reg) daif, options(nomem, nostack, preserves_flags));
        }
        (daif & (1 << 7)) == 0
    }
    #[cfg(not(target_arch = "aarch64"))]
    {
        false
    }
}

/// Queries the Current Exception Level (EL0, EL1, EL2, or EL3).
#[must_use]
pub fn read_current_el() -> u8 {
    #[cfg(target_arch = "aarch64")]
    {
        let el: u64;
        unsafe {
            core::arch::asm!("mrs {0}, CurrentEL", out(reg) el, options(nomem, nostack, preserves_flags));
        }
        ((el >> 2) & 0x3) as u8
    }
    #[cfg(not(target_arch = "aarch64"))]
    {
        1
    }
}

/// Reads the virtual counter timer frequency (`cntvct_el0`).
#[must_use]
pub fn read_cycle_counter() -> u64 {
    #[cfg(target_arch = "aarch64")]
    {
        let count: u64;
        unsafe {
            core::arch::asm!("mrs {0}, cntvct_el0", out(reg) count, options(nomem, nostack, preserves_flags));
        }
        count
    }
    #[cfg(not(target_arch = "aarch64"))]
    {
        0
    }
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
