//! # Low-Level Architecture Context Switching
//!
//! Assembly routines for saving callee-saved registers and switching CPU stack pointers.

/// Switches execution context from the previous task to the next task.
///
/// Saves RFLAGS, R15..R12, RBP, RBX onto `prev_rsp`, updates `*prev_rsp = rsp`,
/// loads `rsp = next_rsp`, restores callee-saved registers, and returns to the new task's RIP.
///
/// # Safety
///
/// `prev_rsp` must be a valid pointer to store the current stack pointer.
/// `next_rsp` must be a valid pointer to an initialized [`super::task::CpuContext`] stack frame.
#[cfg(all(target_arch = "x86_64", target_os = "none"))]
#[unsafe(no_mangle)]
#[unsafe(naked)]
pub unsafe extern "C" fn switch_context(prev_rsp: *mut *mut u8, next_rsp: *const u8) {
    core::arch::naked_asm!(
        "pushfq",
        "push r15",
        "push r14",
        "push r13",
        "push r12",
        "push rbp",
        "push rbx",
        "mov [rdi], rsp",
        "mov rsp, rsi",
        "pop rbx",
        "pop rbp",
        "pop r12",
        "pop r13",
        "pop r14",
        "pop r15",
        "popfq",
        "ret"
    );
}

/// Fallback context switch routine for non-x86_64 targets and test runners.
///
/// # Safety
///
/// Pointers must be valid or null.
#[cfg(not(all(target_arch = "x86_64", target_os = "none")))]
pub unsafe extern "C" fn switch_context(_prev_rsp: *mut *mut u8, _next_rsp: *const u8) {}
