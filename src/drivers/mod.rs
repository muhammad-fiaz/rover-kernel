//! # Hardware Device Drivers Subsystem
//!
//! Device drivers for 16550 UART serial consoles, PCI bus enumeration,
//! linear framebuffer graphics consoles (`fbcon`), and PS/2 keyboard controllers.

pub mod fbcon;
pub mod pci;
pub mod ps2;
pub mod serial;

pub use fbcon::{FRAMEBUFFER_CONSOLE, FramebufferConsole, init_framebuffer_console};
pub use pci::{PciDevice, pci_read_u16, pci_read_u32, scan_pci_bus};
pub use ps2::Ps2Keyboard;
pub use serial::{COM1_BASE, SERIAL1, SerialPort, early_write_fmt};
