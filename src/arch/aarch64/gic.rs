//! # ARM Generic Interrupt Controller (GIC) Abstraction
//!
//! Provides distributor (GICD) and CPU interface (GICC) control for ARM systems.

/// Base address of GICv2 Distributor on QEMU virt machine.
pub const GICD_BASE: usize = 0x0800_0000;
/// Base address of GICv2 CPU Interface on QEMU virt machine.
pub const GICC_BASE: usize = 0x0801_0000;

/// Generic Interrupt Controller manager.
pub struct GenericInterruptController {
    dist_base: usize,
    cpu_base: usize,
}

impl GenericInterruptController {
    /// Creates a new GIC interface pointing to distributor and CPU interface MMIO bases.
    #[must_use]
    pub const fn new(dist_base: usize, cpu_base: usize) -> Self {
        Self {
            dist_base,
            cpu_base,
        }
    }

    /// Initializes distributor and CPU interface, unmasking group 0 interrupts.
    pub fn init(&mut self) {
        if self.dist_base != 0 {
            // Enable distributor (CTLR = 1)
            let ctlr = self.dist_base as *mut u32;
            unsafe {
                core::ptr::write_volatile(ctlr, 1);
            }
        }

        if self.cpu_base != 0 {
            // Set priority mask register to 0xFF (enable all priorities)
            let pmr = (self.cpu_base + 0x0004) as *mut u32;
            unsafe {
                core::ptr::write_volatile(pmr, 0xFF);
            }

            // Enable CPU interface (CTLR = 1)
            let ctlr = self.cpu_base as *mut u32;
            unsafe {
                core::ptr::write_volatile(ctlr, 1);
            }
        }
    }

    /// Acknowledges an active interrupt, returning its INTID.
    #[must_use]
    pub fn acknowledge_interrupt(&self) -> u32 {
        if self.cpu_base == 0 {
            return 1023; // Spurious
        }
        let iar = (self.cpu_base + 0x000C) as *const u32;
        unsafe { core::ptr::read_volatile(iar) & 0x03FF }
    }

    /// Signals End Of Interrupt (EOI) for the specified INTID.
    pub fn end_of_interrupt(&self, int_id: u32) {
        if self.cpu_base != 0 {
            let eoir = (self.cpu_base + 0x0010) as *mut u32;
            unsafe {
                core::ptr::write_volatile(eoir, int_id);
            }
        }
    }
}
