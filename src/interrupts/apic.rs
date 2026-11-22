//! # Local Advanced Programmable Interrupt Controller (LAPIC)
//!
//! Controls per-CPU Local APIC registers via MMIO, End-Of-Interrupt (EOI) signaling,
//! Spurious vector configuration, and LAPIC periodic timer setup.

use crate::arch::current::{read_msr, write_msr};
use crate::sync::SpinLock;

/// `IA32_APIC_BASE` MSR index.
pub const IA32_APIC_BASE_MSR: u32 = 0x1B;

/// Standard LAPIC MMIO base physical address.
pub const DEFAULT_APIC_BASE: u64 = 0xFEE0_0000;

// LAPIC Register Offsets
const LAPIC_ID: usize = 0x020;
const LAPIC_VERSION: usize = 0x030;
const LAPIC_EOI: usize = 0x0B0;
const LAPIC_SVR: usize = 0x0F0;
const LAPIC_TIMER_LVT: usize = 0x320;
const LAPIC_TIMER_INIT_CNT: usize = 0x380;
const LAPIC_TIMER_CURR_CNT: usize = 0x390;
const LAPIC_TIMER_DIV: usize = 0x3E0;

/// Controller for the per-CPU Local APIC.
pub struct LocalApic {
    base_addr: u64,
    enabled: bool,
}

impl LocalApic {
    /// Creates a new uninitialized Local APIC instance.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            base_addr: DEFAULT_APIC_BASE,
            enabled: false,
        }
    }

    /// Initializes and enables the Local APIC.
    ///
    /// # Safety
    ///
    /// The physical MMIO aperture at `DEFAULT_APIC_BASE` must be mapped into virtual memory.
    pub unsafe fn init(&mut self) {
        // 1. Read hardware APIC base physical address from MSR.
        let msr_val = unsafe { read_msr(IA32_APIC_BASE_MSR) };
        let base = msr_val & 0x000F_FFFF_FFFF_F000;
        self.base_addr = if base != 0 { base } else { DEFAULT_APIC_BASE };

        // 2. Ensure global APIC enable bit (bit 11) is set in MSR.
        unsafe {
            write_msr(IA32_APIC_BASE_MSR, msr_val | (1 << 11));
        }

        // 3. Set Spurious Interrupt Vector to 0xFF (vector 255) and enable APIC software bit (bit 8).
        self.write_reg(LAPIC_SVR, 0x1FF);

        self.enabled = true;
    }

    /// Reads a 32-bit register from the Local APIC MMIO aperture.
    #[must_use]
    pub fn read_reg(&self, offset: usize) -> u32 {
        let reg_ptr = (self.base_addr + (offset as u64)) as *const u32;
        // SAFETY: Reading MMIO register.
        unsafe { core::ptr::read_volatile(reg_ptr) }
    }

    /// Writes a 32-bit value to a Local APIC MMIO register.
    pub fn write_reg(&mut self, offset: usize, val: u32) {
        let reg_ptr = (self.base_addr + (offset as u64)) as *mut u32;
        // SAFETY: Writing MMIO register.
        unsafe { core::ptr::write_volatile(reg_ptr, val) };
    }

    /// Sends an End-Of-Interrupt (EOI) signal to the Local APIC.
    pub fn send_eoi(&mut self) {
        self.write_reg(LAPIC_EOI, 0);
    }

    /// Returns the local APIC ID for this processor core.
    #[must_use]
    pub fn id(&self) -> u32 {
        self.read_reg(LAPIC_ID) >> 24
    }

    /// Returns the Local APIC version register value.
    #[must_use]
    pub fn version(&self) -> u32 {
        self.read_reg(LAPIC_VERSION) & 0xFF
    }

    /// Configures the Local APIC timer in periodic mode with the specified vector and countdown.
    pub fn setup_periodic_timer(&mut self, vector: u8, initial_count: u32) {
        // Divide by 16: bits 0..3 of divider register = 0x03.
        self.write_reg(LAPIC_TIMER_DIV, 0x03);

        // LVT Timer: Periodic mode (bit 17) | unmasked (bit 16 = 0) | vector.
        let lvt = (1 << 17) | u32::from(vector);
        self.write_reg(LAPIC_TIMER_LVT, lvt);

        // Set initial countdown value.
        self.write_reg(LAPIC_TIMER_INIT_CNT, initial_count);
    }

    /// Reads the current timer countdown value.
    #[must_use]
    pub fn timer_current_count(&self) -> u32 {
        self.read_reg(LAPIC_TIMER_CURR_CNT)
    }

    /// Returns `true` if the Local APIC has been successfully initialized and enabled.
    #[must_use]
    pub const fn is_enabled(&self) -> bool {
        self.enabled
    }
}

impl Default for LocalApic {
    fn default() -> Self {
        Self::new()
    }
}

/// Global thread-safe Local APIC instance.
pub static LOCAL_APIC: SpinLock<LocalApic> = SpinLock::new(LocalApic::new());

/// Sends an EOI signal via the global [`LOCAL_APIC`] instance.
pub fn end_of_interrupt() {
    LOCAL_APIC.lock().send_eoi();
}
