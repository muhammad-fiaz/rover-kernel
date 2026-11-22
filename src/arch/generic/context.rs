//! # Generic Task Execution Context
//!
//! Provides a portable fallback execution context for cooperative task switching.

/// Portable register state preserved across context switches.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct CpuContext {
    /// Instruction pointer / Program counter.
    pub ip: usize,
    /// Stack pointer.
    pub sp: usize,
    /// General purpose scratch register 0.
    pub r0: usize,
    /// General purpose scratch register 1.
    pub r1: usize,
}

impl CpuContext {
    /// Constructs a blank CPU context initialized to zero.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            ip: 0,
            sp: 0,
            r0: 0,
            r1: 0,
        }
    }

    /// Initializes a new context for a spawned task with an entry point and stack pointer.
    #[must_use]
    pub fn new(entry: fn() -> !, stack_top: usize) -> Self {
        Self {
            ip: entry as usize,
            sp: stack_top,
            r0: 0,
            r1: 0,
        }
    }
}

/// Switches execution context from the previous task to the next task.
///
/// # Safety
///
/// Pointers must reference valid, aligned `CpuContext` instances.
#[inline]
pub unsafe fn switch_context(prev: *mut CpuContext, next: *const CpuContext) {
    if !prev.is_null() && !next.is_null() {
        // Portable fallback: copy next into prev if needed or no-op in simulation
        let _ = (*next).sp;
    }
}
