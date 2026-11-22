//! # PS/2 Keyboard Controller Driver
//!
//! Handles keyboard I/O on port 0x60/0x64 and translates Scancode Set 1 into ASCII characters.

use crate::arch::current::io::inb;

const PS2_DATA_PORT: u16 = 0x60;
const PS2_STATUS_PORT: u16 = 0x64;

/// Scancode translation lookup table for unshifted US QWERTY keyboard (Scancode Set 1).
static SCANCODE_TABLE: [char; 58] = [
    '\0', '\x1B', '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', '-', '=', '\x08', '\t', 'q',
    'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p', '[', ']', '\n', '\0', 'a', 's', 'd', 'f', 'g',
    'h', 'j', 'k', 'l', ';', '\'', '`', '\0', '\\', 'z', 'x', 'c', 'v', 'b', 'n', 'm', ',', '.',
    '/', '\0', '*', '\0', ' ',
];

/// Scancode translation lookup table for shifted US QWERTY keyboard (Shift key held).
static SCANCODE_TABLE_SHIFTED: [char; 58] = [
    '\0', '\x1B', '!', '@', '#', '$', '%', '^', '&', '*', '(', ')', '_', '+', '\x08', '\t', 'Q',
    'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P', '{', '}', '\n', '\0', 'A', 'S', 'D', 'F', 'G',
    'H', 'J', 'K', 'L', ':', '"', '~', '\0', '|', 'Z', 'X', 'C', 'V', 'B', 'N', 'M', '<', '>', '?',
    '\0', '*', '\0', ' ',
];

/// Controller managing PS/2 keyboard state and shift modifiers.
pub struct Ps2Keyboard {
    shift_pressed: bool,
}

impl Ps2Keyboard {
    /// Creates a new PS/2 keyboard handler instance.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            shift_pressed: false,
        }
    }

    /// Checks if a byte is waiting in the PS/2 controller output buffer.
    #[must_use]
    pub fn has_data(&self) -> bool {
        // Status port bit 0: Output Buffer Full (OBF).
        // SAFETY: Reading status port 0x64.
        unsafe { (inb(PS2_STATUS_PORT) & 1) != 0 }
    }

    /// Reads a raw scancode byte from port 0x60 if available.
    #[must_use]
    pub fn read_scancode(&self) -> Option<u8> {
        if self.has_data() {
            // SAFETY: Reading data port 0x60.
            Some(unsafe { inb(PS2_DATA_PORT) })
        } else {
            None
        }
    }

    /// Processes an incoming scancode byte and converts it into a printable ASCII character.
    pub fn process_scancode(&mut self, scancode: u8) -> Option<char> {
        // Left Shift = 0x2A (make) / 0xAA (break); Right Shift = 0x36 (make) / 0xB6 (break)
        if scancode == 0x2A || scancode == 0x36 {
            self.shift_pressed = true;
            return None;
        } else if scancode == 0xAA || scancode == 0xB6 {
            self.shift_pressed = false;
            return None;
        }

        // Ignore key releases (break codes have bit 7 set).
        if (scancode & 0x80) != 0 {
            return None;
        }

        let idx = scancode as usize;
        if idx < SCANCODE_TABLE.len() {
            let ch = if self.shift_pressed {
                SCANCODE_TABLE_SHIFTED[idx]
            } else {
                SCANCODE_TABLE[idx]
            };
            if ch != '\0' {
                return Some(ch);
            }
        }

        None
    }
}

impl Default for Ps2Keyboard {
    fn default() -> Self {
        Self::new()
    }
}
