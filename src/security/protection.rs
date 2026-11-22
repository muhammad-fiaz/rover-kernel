//! # Hardware Security Features
//!
//! Enables Supervisor Mode Execution Prevention (SMEP), Supervisor Mode Access Prevention (SMAP),
//! and No-Execute (NX) bit enforcement.

use crate::arch::x86_64::{CpuCapabilities, read_cr4, read_msr, write_cr4, write_msr};

const IA32_EFER: u32 = 0xC000_0080;
const CR4_SMEP: u64 = 1 << 20;
const CR4_SMAP: u64 = 1 << 21;
const EFER_NXE: u64 = 1 << 11;

/// Enables hardware security features supported by the CPU.
///
/// # Safety
///
/// Must be executed on the BSP during kernel initialization.
pub unsafe fn enable_hardware_protections(caps: &CpuCapabilities) {
    // 1. Enable No-Execute (NX) page table bit if supported.
    if caps.has_nx {
        let efer = unsafe { read_msr(IA32_EFER) };
        unsafe {
            write_msr(IA32_EFER, efer | EFER_NXE);
        }
        crate::kinfo!("Hardware Protection: No-Execute (NX) enabled in EFER.");
    }

    // 2. Enable SMEP and SMAP in CR4 if supported.
    let mut cr4 = read_cr4();
    if caps.has_sse {
        // Enable OS support for SSE (OSFXSR) & unmasked SIMD floating point exceptions (OSXMMEXCPT)
        cr4 |= (1 << 9) | (1 << 10);
    }

    if caps.has_smep {
        cr4 |= CR4_SMEP;
        crate::kinfo!("Hardware Protection: SMEP enabled in CR4.");
    }
    if caps.has_smap {
        cr4 |= CR4_SMAP;
        crate::kinfo!("Hardware Protection: SMAP enabled in CR4.");
    }

    unsafe {
        write_cr4(cr4);
    }
}
