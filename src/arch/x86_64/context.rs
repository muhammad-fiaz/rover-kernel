//! # x86-64 Task Execution Context
//!
//! Callee-saved general-purpose register state for x86-64 cooperative task switching.

/// Architecture-specific register state preserved across x86-64 context switches.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct CpuContext {
    /// RFLAGS register value.
    pub rflags: u64,
    /// R15 register.
    pub r15: u64,
    /// R14 register.
    pub r14: u64,
    /// R13 register.
    pub r13: u64,
    /// R12 register.
    pub r12: u64,
    /// RBP frame pointer.
    pub rbp: u64,
    /// RBX general register.
    pub rbx: u64,
    /// Return address / Instruction pointer (RIP).
    pub rip: u64,
}

impl CpuContext {
    /// Constructs a blank CPU context initialized with interrupts enabled (RFLAGS bit 9).
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            rflags: 0x202,
            r15: 0,
            r14: 0,
            r13: 0,
            r12: 0,
            rbp: 0,
            rbx: 0,
            rip: 0,
        }
    }

    /// Prepares an execution context with the specified entry point and stack pointer.
    #[must_use]
    pub fn new(entry_point: usize, _stack_top: usize) -> Self {
        Self {
            rflags: 0x202,
            r15: 0,
            r14: 0,
            r13: 0,
            r12: 0,
            rbp: 0,
            rbx: 0,
            rip: entry_point as u64,
        }
    }
}

impl Default for CpuContext {
    fn default() -> Self {
        Self::empty()
    }
}
