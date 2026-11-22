//! # x86-64 CPU control and register access
//!
//! Control registers (CR0, CR2, CR3, CR4), MSRs, CPUID feature query, and instruction controls.

/// Structure holding the 4 registers returned by the `cpuid` instruction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CpuidResult {
    /// EAX register result.
    pub eax: u32,
    /// EBX register result.
    pub ebx: u32,
    /// ECX register result.
    pub ecx: u32,
    /// EDX register result.
    pub edx: u32,
}

/// Executes the `cpuid` instruction with the specified `leaf` and `sub_leaf`.
#[must_use]
pub fn cpuid(leaf: u32, sub_leaf: u32) -> CpuidResult {
    let eax: u32;
    let ebx: u32;
    let ecx: u32;
    let edx: u32;

    // SAFETY: cpuid is a read-only unprivileged instruction in x86-64 long mode.
    unsafe {
        core::arch::asm!(
            "push rbx",
            "cpuid",
            "mov {ebx_tmp:e}, ebx",
            "pop rbx",
            inout("eax") leaf => eax,
            inout("ecx") sub_leaf => ecx,
            ebx_tmp = out(reg) ebx,
            out("edx") edx,
            options(nomem, nostack, preserves_flags)
        );
    }

    CpuidResult { eax, ebx, ecx, edx }
}

/// Reads the CR0 control register.
#[inline]
#[must_use]
pub fn read_cr0() -> u64 {
    let val: u64;
    // SAFETY: Reading CR0 is allowed in kernel mode (ring 0).
    unsafe {
        core::arch::asm!("mov {0}, cr0", out(reg) val, options(nomem, nostack, preserves_flags));
    }
    val
}

/// Writes a new value to the CR0 control register.
///
/// # Safety
///
/// The caller must ensure the new flags maintain memory paging and protected mode.
#[inline]
pub unsafe fn write_cr0(val: u64) {
    // SAFETY: Verified by caller.
    unsafe {
        core::arch::asm!("mov cr0, {0}", in(reg) val, options(nomem, nostack, preserves_flags));
    }
}

/// Reads the CR2 control register (page fault linear address).
#[inline]
#[must_use]
pub fn read_cr2() -> u64 {
    let val: u64;
    // SAFETY: Reading CR2 is allowed in kernel mode (ring 0).
    unsafe {
        core::arch::asm!("mov {0}, cr2", out(reg) val, options(nomem, nostack, preserves_flags));
    }
    val
}

/// Reads the CR3 control register (PML4 base address and flags).
#[inline]
#[must_use]
pub fn read_cr3() -> u64 {
    let val: u64;
    // SAFETY: Reading CR3 is allowed in kernel mode (ring 0).
    unsafe {
        core::arch::asm!("mov {0}, cr3", out(reg) val, options(nomem, nostack, preserves_flags));
    }
    val
}

/// Writes a new PML4 base address and flags to CR3, flushing the TLB.
///
/// # Safety
///
/// The caller must ensure `val` points to a valid 4-level page table structure.
#[inline]
pub unsafe fn write_cr3(val: u64) {
    // SAFETY: Verified by caller.
    unsafe {
        core::arch::asm!("mov cr3, {0}", in(reg) val, options(nomem, nostack, preserves_flags));
    }
}

/// Reads the CR4 control register.
#[inline]
#[must_use]
pub fn read_cr4() -> u64 {
    let val: u64;
    // SAFETY: Reading CR4 is allowed in kernel mode (ring 0).
    unsafe {
        core::arch::asm!("mov {0}, cr4", out(reg) val, options(nomem, nostack, preserves_flags));
    }
    val
}

/// Writes a new value to the CR4 control register.
///
/// # Safety
///
/// The caller must ensure that the CPU features enabled in CR4 are supported.
#[inline]
pub unsafe fn write_cr4(val: u64) {
    // SAFETY: Verified by caller.
    unsafe {
        core::arch::asm!("mov cr4, {0}", in(reg) val, options(nomem, nostack, preserves_flags));
    }
}

/// Reads the CPU RFLAGS register.
#[inline]
#[must_use]
pub fn read_rflags() -> u64 {
    let val: u64;
    // SAFETY: Reading RFLAGS via pushfq/pop.
    unsafe {
        core::arch::asm!("pushfq", "pop {0}", out(reg) val, options(nomem, preserves_flags));
    }
    val
}

/// Reads a 64-bit Model-Specific Register (MSR).
///
/// # Safety
///
/// The caller must ensure that the CPU supports the specified MSR index.
#[inline]
#[must_use]
pub unsafe fn read_msr(msr: u32) -> u64 {
    let low: u32;
    let high: u32;
    // SAFETY: rdmsr reads into edx:eax.
    unsafe {
        core::arch::asm!(
            "rdmsr",
            in("ecx") msr,
            out("eax") low,
            out("edx") high,
            options(nomem, nostack, preserves_flags)
        );
    }
    (u64::from(high) << 32) | u64::from(low)
}

/// Writes a 64-bit value to a Model-Specific Register (MSR).
///
/// # Safety
///
/// The caller must ensure that the CPU supports the specified MSR index and the value is valid.
#[inline]
pub unsafe fn write_msr(msr: u32, val: u64) {
    let low = (val & 0xFFFF_FFFF) as u32;
    let high = (val >> 32) as u32;
    // SAFETY: wrmsr writes from edx:eax.
    unsafe {
        core::arch::asm!(
            "wrmsr",
            in("ecx") msr,
            in("eax") low,
            in("edx") high,
            options(nomem, nostack, preserves_flags)
        );
    }
}

/// Halts the CPU until the next external interrupt occurs.
#[inline]
pub fn halt() {
    // SAFETY: hlt instruction pauses the processor core.
    unsafe {
        core::arch::asm!("hlt", options(nomem, nostack, preserves_flags));
    }
}

/// Disables hardware interrupts and enters an infinite halt loop.
pub fn halt_loop() -> ! {
    // SAFETY: cli disables interrupts, then infinite hlt loop prevents any execution.
    unsafe {
        core::arch::asm!("cli", options(nomem, nostack, preserves_flags));
    }
    loop {
        halt();
    }
}

/// Enables hardware interrupts (`sti`).
///
/// # Safety
///
/// The caller must ensure that an IDT has been initialized and interrupts can be safely handled.
#[inline]
pub unsafe fn enable_interrupts() {
    // SAFETY: sti sets IF flag.
    unsafe {
        core::arch::asm!("sti", options(nomem, nostack, preserves_flags));
    }
}

/// Disables hardware interrupts (`cli`).
#[inline]
pub fn disable_interrupts() {
    // SAFETY: cli clears IF flag.
    unsafe {
        core::arch::asm!("cli", options(nomem, nostack, preserves_flags));
    }
}

/// Returns `true` if hardware interrupts are currently enabled.
#[must_use]
pub fn are_interrupts_enabled() -> bool {
    (read_rflags() & (1 << 9)) != 0
}

/// Reads the current 64-bit cycle count from the processor's Time Stamp Counter.
#[inline]
#[must_use]
pub fn read_cycle_counter() -> u64 {
    let low: u32;
    let high: u32;
    // SAFETY: rdtsc instruction reads cycle count into edx:eax.
    unsafe {
        core::arch::asm!(
            "rdtsc",
            out("eax") low,
            out("edx") high,
            options(nomem, nostack, preserves_flags)
        );
    }
    (u64::from(high) << 32) | u64::from(low)
}

/// High-level CPU hardware identification and supported capability flags.
#[derive(Clone, Debug)]
#[allow(clippy::struct_excessive_bools)]
pub struct CpuCapabilities {
    /// Processor vendor identifier string (e.g., "`GenuineIntel`", "`AuthenticAMD`").
    pub vendor: [u8; 12],
    /// On-chip APIC present.
    pub has_apic: bool,
    /// SSE instructions supported.
    pub has_sse: bool,
    /// SSE2 instructions supported.
    pub has_sse2: bool,
    /// SSE3 instructions supported.
    pub has_sse3: bool,
    /// SSE4.1 instructions supported.
    pub has_sse4_1: bool,
    /// SSE4.2 instructions supported.
    pub has_sse4_2: bool,
    /// AVX instructions supported.
    pub has_avx: bool,
    /// AVX2 instructions supported.
    pub has_avx2: bool,
    /// SMEP (Supervisor Mode Execution Prevention) supported.
    pub has_smep: bool,
    /// SMAP (Supervisor Mode Access Prevention) supported.
    pub has_smap: bool,
    /// Long Mode (64-bit) supported.
    pub has_long_mode: bool,
    /// No-Execute (NX / XD) bit supported.
    pub has_nx: bool,
    /// 1 GiB huge pages supported.
    pub has_page_1gb: bool,
}

impl CpuCapabilities {
    /// Detects CPU features by probing CPUID leaves.
    #[must_use]
    #[allow(clippy::similar_names)]
    pub fn probe() -> Self {
        // Leaf 0: Vendor String
        let res0 = cpuid(0, 0);
        let mut vendor = [0u8; 12];
        vendor[0..4].copy_from_slice(&res0.ebx.to_le_bytes());
        vendor[4..8].copy_from_slice(&res0.edx.to_le_bytes());
        vendor[8..12].copy_from_slice(&res0.ecx.to_le_bytes());

        // Leaf 1: Standard Features
        let res1 = cpuid(1, 0);
        let has_apic = (res1.edx & (1 << 9)) != 0;
        let has_sse = (res1.edx & (1 << 25)) != 0;
        let has_sse2 = (res1.edx & (1 << 26)) != 0;
        let has_sse3 = (res1.ecx & (1 << 0)) != 0;
        let has_sse4_1 = (res1.ecx & (1 << 19)) != 0;
        let has_sse4_2 = (res1.ecx & (1 << 20)) != 0;
        let has_avx = (res1.ecx & (1 << 28)) != 0;

        // Leaf 7: Extended Features
        let res7 = cpuid(7, 0);
        let has_avx2 = (res7.ebx & (1 << 5)) != 0;
        let has_smep = (res7.ebx & (1 << 7)) != 0;
        let has_smap = (res7.ebx & (1 << 20)) != 0;

        // Leaf 0x80000001: Extended Processor Info
        let res_ext = cpuid(0x8000_0001, 0);
        let has_long_mode = (res_ext.edx & (1 << 29)) != 0;
        let has_nx = (res_ext.edx & (1 << 20)) != 0;
        let has_page_1gb = (res_ext.edx & (1 << 26)) != 0;

        Self {
            vendor,
            has_apic,
            has_sse,
            has_sse2,
            has_sse3,
            has_sse4_1,
            has_sse4_2,
            has_avx,
            has_avx2,
            has_smep,
            has_smap,
            has_long_mode,
            has_nx,
            has_page_1gb,
        }
    }

    /// Returns the vendor identifier as a string slice.
    #[must_use]
    pub fn vendor_str(&self) -> &str {
        core::str::from_utf8(&self.vendor).unwrap_or("Unknown")
    }
}
