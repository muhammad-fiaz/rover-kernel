//! # RISC-V Platform-Level Interrupt Controller (PLIC) Abstraction
//!
//! Provides priority assignment, interrupt enabling, and claim/complete processing.

/// Default QEMU virt machine PLIC MMIO base address.
pub const PLIC_BASE: usize = 0x0C00_0000;

/// Platform-Level Interrupt Controller manager.
pub struct PlatformLevelInterruptController {
    base: usize,
}

impl PlatformLevelInterruptController {
    /// Creates a new PLIC interface with the specified base address.
    #[must_use]
    pub const fn new(base: usize) -> Self {
        Self { base }
    }

    /// Sets the priority for an interrupt source.
    pub fn set_priority(&mut self, source: u32, priority: u32) {
        if self.base != 0 && source > 0 && source < 1024 {
            let reg = (self.base + (source as usize) * 4) as *mut u32;
            unsafe {
                core::ptr::write_volatile(reg, priority);
            }
        }
    }

    /// Enables an interrupt source for context 1 (supervisor mode hart 0).
    pub fn enable_source(&mut self, context: usize, source: u32) {
        if self.base != 0 && source > 0 && source < 1024 {
            let offset = 0x0020_0000 + context * 0x1000 + (source as usize / 32) * 4;
            let reg = (self.base + offset) as *mut u32;
            unsafe {
                let current = core::ptr::read_volatile(reg);
                core::ptr::write_volatile(reg, current | (1 << (source % 32)));
            }
        }
    }

    /// Claims an active interrupt for the specified context.
    #[must_use]
    pub fn claim(&self, context: usize) -> u32 {
        if self.base == 0 {
            return 0;
        }
        let claim_reg = (self.base + 0x0020_0004 + context * 0x1000) as *const u32;
        unsafe { core::ptr::read_volatile(claim_reg) }
    }

    /// Signals completion of an interrupt for the specified context.
    pub fn complete(&self, context: usize, source: u32) {
        if self.base != 0 {
            let complete_reg = (self.base + 0x0020_0004 + context * 0x1000) as *mut u32;
            unsafe {
                core::ptr::write_volatile(complete_reg, source);
            }
        }
    }
}
