//! # 16550 UART Serial Driver
//!
//! Provides polled RS-232 serial communication on standard PC COM ports (COM1 = 0x3F8).
//! Supports 115200 baud, 8N1 framing, FIFO buffering, and `core::fmt::Write`.

use crate::arch::current::io::{inb, outb};
use crate::sync::SpinLock;
use core::fmt;

/// Standard COM1 I/O base port.
pub const COM1_BASE: u16 = 0x3F8;

/// 16550 UART Serial Controller.
pub struct SerialPort {
    base: u16,
    initialized: bool,
}

impl SerialPort {
    /// Creates a new uninitialized serial port instance with the specified base I/O port.
    #[must_use]
    pub const fn new(base: u16) -> Self {
        Self {
            base,
            initialized: false,
        }
    }

    /// Initializes the UART hardware to 115200 baud, 8 data bits, no parity, 1 stop bit (8N1).
    pub fn init(&mut self) {
        // SAFETY: Programming standard 16550 UART registers.
        unsafe {
            // 1. Disable all interrupts (IER = 0).
            outb(self.base + 1, 0x00);

            // 2. Enable Divisor Latch Access Bit (DLAB = 0x80) in Line Control Register (LCR).
            outb(self.base + 3, 0x80);

            // 3. Set divisor to 1 (115200 baud: 115200 / 115200 = 1).
            outb(self.base, 0x01); // DLL
            outb(self.base + 1, 0x00); // DLH

            // 4. Set line control: 8 data bits, 1 stop bit, no parity (8N1 = 0x03).
            outb(self.base + 3, 0x03);

            // 5. Enable FIFO, clear transmit/receive queues, 14-byte threshold (0xC7).
            outb(self.base + 2, 0xC7);

            // 6. Modem control: Set Data Terminal Ready (DTR) + Request To Send (RTS) + OUT2 (0x0B).
            outb(self.base + 4, 0x0B);
        }

        self.initialized = true;
    }

    /// Checks whether the Transmit Holding Register is empty and ready to accept a byte.
    #[must_use]
    pub fn is_transmit_empty(&self) -> bool {
        // Line Status Register bit 5 (0x20): Transmitter Holding Register Empty (THRE).
        // SAFETY: Reading LSR port.
        unsafe { (inb(self.base + 5) & 0x20) != 0 }
    }

    /// Transmits a single byte over the serial port, waiting until the hardware buffer is ready.
    pub fn send_byte(&mut self, byte: u8) {
        if !self.initialized {
            self.init();
        }

        // Wait until transmit holding register is ready.
        while !self.is_transmit_empty() {
            core::hint::spin_loop();
        }

        // SAFETY: Writing data to TX register.
        unsafe {
            outb(self.base, byte);
        }
    }

    /// Sends a string slice over the serial line, translating `\n` to `\r\n`.
    pub fn write_str_buffered(&mut self, s: &str) {
        for byte in s.bytes() {
            if byte == b'\n' {
                self.send_byte(b'\r');
            }
            self.send_byte(byte);
        }
    }
}

impl fmt::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_str_buffered(s);
        Ok(())
    }
}

/// Global thread-safe COM1 serial port instance protected by a spinlock.
pub static SERIAL1: SpinLock<SerialPort> = SpinLock::new(SerialPort::new(COM1_BASE));

/// Writes formatted text directly to COM1 without taking high-level locks.
pub fn early_write_fmt(args: fmt::Arguments<'_>) {
    use fmt::Write;
    let mut port = SERIAL1.lock();
    let _ = port.write_fmt(args);
}
