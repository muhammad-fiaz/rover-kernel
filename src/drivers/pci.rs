//! # PCI Bus Enumeration Driver
//!
//! PCI configuration space access via I/O ports 0xCF8 and 0xCFC, device enumeration,
//! vendor/device ID discovery, and Base Address Register (BAR) inspection.

extern crate alloc;
use crate::arch::x86_64::io::{inl, outl};
use alloc::vec::Vec;

const PCI_CONFIG_ADDRESS: u16 = 0x0CF8;
const PCI_CONFIG_DATA: u16 = 0x0CFC;

/// Reads a 32-bit register from PCI configuration space.
#[must_use]
pub fn pci_read_u32(bus: u8, slot: u8, func: u8, offset: u8) -> u32 {
    let address = (1u32 << 31)
        | (u32::from(bus) << 16)
        | (u32::from(slot) << 11)
        | (u32::from(func) << 8)
        | (u32::from(offset) & 0xFC);

    // SAFETY: Accessing standard PCI configuration space ports.
    unsafe {
        outl(PCI_CONFIG_ADDRESS, address);
        inl(PCI_CONFIG_DATA)
    }
}

/// Reads a 16-bit field from PCI configuration space.
#[must_use]
pub fn pci_read_u16(bus: u8, slot: u8, func: u8, offset: u8) -> u16 {
    let dword = pci_read_u32(bus, slot, func, offset);
    let shift = (offset & 2) * 8;
    ((dword >> shift) & 0xFFFF) as u16
}

/// Information describing a discovered PCI device function.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PciDevice {
    /// PCI bus number (0..255).
    pub bus: u8,
    /// PCI device / slot number (0..31).
    pub slot: u8,
    /// Function number (0..7).
    pub func: u8,
    /// Hardware vendor identifier (e.g. 0x8086 for Intel).
    pub vendor_id: u16,
    /// Hardware device identifier.
    pub device_id: u16,
    /// Device class code.
    pub class_code: u8,
    /// Device subclass code.
    pub subclass: u8,
    /// Programming interface register.
    pub prog_if: u8,
    /// Base Address Register 0 (BAR0).
    pub bar0: u32,
}

impl PciDevice {
    /// Probes the specified PCI coordinate, returning a device descriptor if present.
    #[must_use]
    pub fn probe(bus: u8, slot: u8, func: u8) -> Option<Self> {
        let vendor_id = pci_read_u16(bus, slot, func, 0x00);
        if vendor_id == 0xFFFF || vendor_id == 0 {
            return None; // Device not present
        }

        let device_id = pci_read_u16(bus, slot, func, 0x02);
        let class_dword = pci_read_u32(bus, slot, func, 0x08);
        let class_code = ((class_dword >> 24) & 0xFF) as u8;
        let subclass = ((class_dword >> 16) & 0xFF) as u8;
        let prog_if = ((class_dword >> 8) & 0xFF) as u8;

        let bar0 = pci_read_u32(bus, slot, func, 0x10);

        Some(Self {
            bus,
            slot,
            func,
            vendor_id,
            device_id,
            class_code,
            subclass,
            prog_if,
            bar0,
        })
    }
}

/// Scans PCI buses 0..4 and slots 0..31, returning a list of all detected devices.
#[must_use]
pub fn scan_pci_bus() -> Vec<PciDevice> {
    let mut devices = Vec::new();

    // Check root buses 0..4
    for bus in 0..4 {
        for slot in 0..32 {
            if let Some(dev) = PciDevice::probe(bus, slot, 0) {
                devices.push(dev);

                // Check header type to see if this is a multi-function device.
                let header_type = (pci_read_u32(bus, slot, 0, 0x0C) >> 16) & 0xFF;
                if (header_type & 0x80) != 0 {
                    for func in 1..8 {
                        if let Some(fdev) = PciDevice::probe(bus, slot, func) {
                            devices.push(fdev);
                        }
                    }
                }
            }
        }
    }

    devices
}
