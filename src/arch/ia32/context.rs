//! # IA-32 Task Execution Context
//!
//! Callee-saved general-purpose register state for 32-bit x86 cooperative task switching.

/// Architecture-specific register state preserved across IA-32 context switches.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct CpuContext {
    /// ebx register.
    pub ebx: u32,
    /// esi register.
    pub esi: u32,
    /// edi register.
    pub edi: u32,
    /// ebp register.
    pub ebp: u32,
    /// esp stack pointer.
    pub esp: u32,
    /// eip instruction pointer / entry point.
    pub eip: u32,
}

impl CpuContext {
    /// Constructs a blank CPU context initialized to zero.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            ebx: 0,
            esi: 0,
            edi: 0,
            ebp: 0,
            esp: 0,
            eip: 0,
        }
    }

    /// Prepares an execution context with the specified entry point and stack pointer.
    #[must_use]
    pub fn new(entry_point: usize, stack_top: usize) -> Self {
        Self {
            ebx: 0,
            esi: 0,
            edi: 0,
            ebp: 0,
            esp: stack_top as u32,
            eip: entry_point as u32,
        }
    }
}

impl Default for CpuContext {
    fn default() -> Self {
        Self::empty()
    }
}

/// Switches CPU execution context from `prev` to `next` on IA-32.
///
/// # Safety
///
/// `prev` and `next` must point to valid, aligned [`CpuContext`] storage.
pub unsafe fn switch_context(prev: *mut CpuContext, next: *const CpuContext) {
    if prev.is_null() || next.is_null() {
        return;
    }

    #[cfg(all(target_arch = "x86", target_os = "none"))]
    unsafe {
        core::arch::asm!(
            "push ebp",
            "push edi",
            "push esi",
            "push ebx",
            "mov [eax], esp",
            "mov esp, [edx]",
            "pop ebx",
            "pop esi",
            "pop edi",
            "pop ebp",
            "ret",
            in("eax") prev,
            in("edx") next,
            options(noreturn)
        );
    }

    #[cfg(not(all(target_arch = "x86", target_os = "none")))]
    {
        let _ = unsafe { (&*next).eip };
    }
}
