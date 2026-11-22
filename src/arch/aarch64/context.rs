//! # AArch64 Task Execution Context
//!
//! Callee-saved general-purpose register state for ARM64 cooperative task switching.

/// Architecture-specific register state preserved across ARM64 context switches.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct CpuContext {
    /// x19 general purpose register.
    pub x19: u64,
    /// x20 general purpose register.
    pub x20: u64,
    /// x21 general purpose register.
    pub x21: u64,
    /// x22 general purpose register.
    pub x22: u64,
    /// x23 general purpose register.
    pub x23: u64,
    /// x24 general purpose register.
    pub x24: u64,
    /// x25 general purpose register.
    pub x25: u64,
    /// x26 general purpose register.
    pub x26: u64,
    /// x27 general purpose register.
    pub x27: u64,
    /// x28 general purpose register.
    pub x28: u64,
    /// Frame pointer (x29).
    pub fp: u64,
    /// Link register / return address (x30 / lr).
    pub lr: u64,
    /// Stack pointer (sp).
    pub sp: u64,
}

impl CpuContext {
    /// Constructs a blank CPU context initialized to zero.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            x19: 0,
            x20: 0,
            x21: 0,
            x22: 0,
            x23: 0,
            x24: 0,
            x25: 0,
            x26: 0,
            x27: 0,
            x28: 0,
            fp: 0,
            lr: 0,
            sp: 0,
        }
    }

    /// Prepares an execution context with the specified entry point and stack pointer.
    #[must_use]
    pub fn new(entry_point: usize, stack_top: usize) -> Self {
        Self {
            x19: 0,
            x20: 0,
            x21: 0,
            x22: 0,
            x23: 0,
            x24: 0,
            x25: 0,
            x26: 0,
            x27: 0,
            x28: 0,
            fp: 0,
            lr: entry_point as u64,
            sp: stack_top as u64,
        }
    }
}

impl Default for CpuContext {
    fn default() -> Self {
        Self::empty()
    }
}

/// Switches CPU execution context from `prev` to `next` on AArch64.
///
/// # Safety
///
/// `prev` and `next` must point to valid, aligned [`CpuContext`] storage.
pub unsafe fn switch_context(prev: *mut CpuContext, next: *const CpuContext) {
    if prev.is_null() || next.is_null() {
        return;
    }

    #[cfg(target_arch = "aarch64")]
    unsafe {
        core::arch::asm!(
            "mov x8, sp",
            "stp x19, x20, [x0, #0]",
            "stp x21, x22, [x0, #16]",
            "stp x23, x24, [x0, #32]",
            "stp x25, x26, [x0, #48]",
            "stp x27, x28, [x0, #64]",
            "stp x29, x30, [x0, #80]",
            "str x8, [x0, #96]",
            "ldp x19, x20, [x1, #0]",
            "ldp x21, x22, [x1, #16]",
            "ldp x23, x24, [x1, #32]",
            "ldp x25, x26, [x1, #48]",
            "ldp x27, x28, [x1, #64]",
            "ldp x29, x30, [x1, #80]",
            "ldr x8, [x1, #96]",
            "mov sp, x8",
            "ret",
            in("x0") prev,
            in("x1") next,
            options(noreturn)
        );
    }

    #[cfg(not(target_arch = "aarch64"))]
    {
        // Fallback for non-aarch64 environments
        unsafe {
            let next_ctx = &*next;
            let prev_ctx = &mut *prev;
            prev_ctx.x19 = 0;
            let _ = next_ctx.lr;
        }
    }
}
