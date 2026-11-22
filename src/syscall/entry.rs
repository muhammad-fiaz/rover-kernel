//! # Architecture System Call Entry Point
//!
//! Configures x86-64 `EFER.SCE`, `STAR`, `LSTAR`, `FMASK` MSRs and naked `syscall_entry` stub.

use crate::arch::x86_64::{read_msr, write_msr};

const IA32_EFER: u32 = 0xC000_0080;
const IA32_STAR: u32 = 0xC000_0081;
const IA32_LSTAR: u32 = 0xC000_0082;
const IA32_FMASK: u32 = 0xC000_0084;

#[unsafe(no_mangle)]
#[unsafe(naked)]
unsafe extern "C" fn syscall_entry() {
    core::arch::naked_asm!(
        "mov r10, rcx", // syscall saves user RIP in RCX
        "push r11",     // syscall saves user RFLAGS in R11
        "push r10",
        "push rbp",
        "push rbx",
        "push r12",
        "push r13",
        "push r14",
        "push r15",
        "call handle_syscall_entry_thunk",
        "pop r15",
        "pop r14",
        "pop r13",
        "pop r12",
        "pop rbx",
        "pop rbp",
        "pop rcx", // Restore user RIP into RCX for sysretq
        "pop r11", // Restore user RFLAGS into R11 for sysretq
        "sysretq"
    );
}

#[unsafe(no_mangle)]
extern "C" fn handle_syscall_entry_thunk(num: u64, a1: u64, a2: u64, a3: u64) -> u64 {
    super::dispatch::handle_syscall_dispatch(num, a1, a2, a3)
}

/// Initializes hardware system call infrastructure on x86-64.
///
/// Enables `EFER.SCE`, sets target address in `LSTAR`, and masks interrupt flags in `FMASK`.
///
/// # Safety
///
/// Must be executed on CPU initialization in ring 0.
pub unsafe fn init_syscall_architecture() {
    unsafe {
        // 1. Enable System Call Extension (SCE - bit 0) in EFER.
        let efer = read_msr(IA32_EFER);
        write_msr(IA32_EFER, efer | 1);

        // 2. Set STAR MSR:
        // Bits 47:32 = Kernel CS (0x08) & Kernel SS (0x10)
        // Bits 63:48 = User CS/SS base selector (0x18 | 3 = 0x1B)
        let star = (0x001Bu64 << 48) | (0x0008u64 << 32);
        write_msr(IA32_STAR, star);

        // 3. Set LSTAR to target entry address.
        let lstar = syscall_entry as *const () as usize as u64;
        write_msr(IA32_LSTAR, lstar);

        // 4. Set FMASK to clear IF (Interrupt Flag, bit 9) upon system call entry.
        write_msr(IA32_FMASK, 1 << 9);
    }
}
