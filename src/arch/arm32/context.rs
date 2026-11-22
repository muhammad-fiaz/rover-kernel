//! # 32-bit ARM Task Execution Context
//!
//! Callee-saved general-purpose register state for ARMv7-A cooperative task switching.

/// Architecture-specific register state preserved across 32-bit ARM context switches.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct CpuContext {
    /// r4 register.
    pub r4: u32,
    /// r5 register.
    pub r5: u32,
    /// r6 register.
    pub r6: u32,
    /// r7 register.
    pub r7: u32,
    /// r8 register.
    pub r8: u32,
    /// r9 register.
    pub r9: u32,
    /// r10 register.
    pub r10: u32,
    /// r11 / fp register.
    pub fp: u32,
    /// sp stack pointer.
    pub sp: u32,
    /// lr link register / return address.
    pub lr: u32,
}

impl CpuContext {
    /// Constructs a blank CPU context initialized to zero.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            r4: 0,
            r5: 0,
            r6: 0,
            r7: 0,
            r8: 0,
            r9: 0,
            r10: 0,
            fp: 0,
            sp: 0,
            lr: 0,
        }
    }

    /// Prepares an execution context with the specified entry point and stack pointer.
    #[must_use]
    pub fn new(entry_point: usize, stack_top: usize) -> Self {
        Self {
            r4: 0,
            r5: 0,
            r6: 0,
            r7: 0,
            r8: 0,
            r9: 0,
            r10: 0,
            fp: 0,
            sp: stack_top as u32,
            lr: entry_point as u32,
        }
    }
}

impl Default for CpuContext {
    fn default() -> Self {
        Self::empty()
    }
}

/// Switches CPU execution context from `prev` to `next` on 32-bit ARM.
///
/// # Safety
///
/// `prev` and `next` must point to valid, aligned [`CpuContext`] storage.
pub unsafe fn switch_context(prev: *mut CpuContext, next: *const CpuContext) {
    if prev.is_null() || next.is_null() {
        return;
    }

    #[cfg(all(target_arch = "arm", target_os = "none"))]
    unsafe {
        core::arch::asm!(
            "stmia r0, {r4-r11, sp, lr}",
            "ldmia r1, {r4-r11, sp, lr}",
            "bx lr",
            in("r0") prev,
            in("r1") next,
            options(noreturn)
        );
    }

    #[cfg(not(all(target_arch = "arm", target_os = "none")))]
    {
        let _ = unsafe { (&*next).lr };
    }
}
