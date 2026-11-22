//! # RISC-V 64 Task Execution Context
//!
//! Callee-saved general-purpose register state for RISC-V cooperative task switching.

/// Architecture-specific register state preserved across RISC-V 64 context switches.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct CpuContext {
    /// Return address (ra / x1).
    pub ra: u64,
    /// Stack pointer (sp / x2).
    pub sp: u64,
    /// Callee-saved s0 / fp (x8).
    pub s0: u64,
    /// Callee-saved s1 (x9).
    pub s1: u64,
    /// Callee-saved s2 (x18).
    pub s2: u64,
    /// Callee-saved s3 (x19).
    pub s3: u64,
    /// Callee-saved s4 (x20).
    pub s4: u64,
    /// Callee-saved s5 (x21).
    pub s5: u64,
    /// Callee-saved s6 (x22).
    pub s6: u64,
    /// Callee-saved s7 (x23).
    pub s7: u64,
    /// Callee-saved s8 (x24).
    pub s8: u64,
    /// Callee-saved s9 (x25).
    pub s9: u64,
    /// Callee-saved s10 (x26).
    pub s10: u64,
    /// Callee-saved s11 (x27).
    pub s11: u64,
}

impl CpuContext {
    /// Constructs a blank CPU context initialized to zero.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            ra: 0,
            sp: 0,
            s0: 0,
            s1: 0,
            s2: 0,
            s3: 0,
            s4: 0,
            s5: 0,
            s6: 0,
            s7: 0,
            s8: 0,
            s9: 0,
            s10: 0,
            s11: 0,
        }
    }

    /// Prepares an execution context with the specified entry point and stack pointer.
    #[must_use]
    pub fn new(entry_point: usize, stack_top: usize) -> Self {
        Self {
            ra: entry_point as u64,
            sp: stack_top as u64,
            s0: 0,
            s1: 0,
            s2: 0,
            s3: 0,
            s4: 0,
            s5: 0,
            s6: 0,
            s7: 0,
            s8: 0,
            s9: 0,
            s10: 0,
            s11: 0,
        }
    }
}

impl Default for CpuContext {
    fn default() -> Self {
        Self::empty()
    }
}

/// Switches CPU execution context from `prev` to `next` on RISC-V 64.
///
/// # Safety
///
/// `prev` and `next` must point to valid, aligned [`CpuContext`] storage.
pub unsafe fn switch_context(prev: *mut CpuContext, next: *const CpuContext) {
    if prev.is_null() || next.is_null() {
        return;
    }

    #[cfg(all(target_arch = "riscv64", target_os = "none"))]
    unsafe {
        core::arch::asm!(
            "sd ra, 0(a0)",
            "sd sp, 8(a0)",
            "sd s0, 16(a0)",
            "sd s1, 24(a0)",
            "sd s2, 32(a0)",
            "sd s3, 40(a0)",
            "sd s4, 48(a0)",
            "sd s5, 56(a0)",
            "sd s6, 64(a0)",
            "sd s7, 72(a0)",
            "sd s8, 80(a0)",
            "sd s9, 88(a0)",
            "sd s10, 96(a0)",
            "sd s11, 104(a0)",
            "ld ra, 0(a1)",
            "ld sp, 8(a1)",
            "ld s0, 16(a1)",
            "ld s1, 24(a1)",
            "ld s2, 32(a1)",
            "ld s3, 40(a1)",
            "ld s4, 48(a1)",
            "ld s5, 56(a1)",
            "ld s6, 64(a1)",
            "ld s7, 72(a1)",
            "ld s8, 80(a1)",
            "ld s9, 88(a1)",
            "ld s10, 96(a1)",
            "ld s11, 104(a1)",
            "ret",
            in("a0") prev,
            in("a1") next,
            options(noreturn)
        );
    }

    #[cfg(not(all(target_arch = "riscv64", target_os = "none")))]
    {
        let _ = unsafe { (&*next).ra };
    }
}
