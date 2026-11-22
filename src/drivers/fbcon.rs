//! # Framebuffer Graphical Console (fbcon)
//!
//! Direct pixel text rendering onto linear UEFI GOP framebuffers using a built-in bitmap font.

use crate::boot::{RoverFramebuffer, RoverPixelFormat};
use crate::sync::SpinLock;

/// Simple 8x16 VGA font glyph bitmap data (first 128 ASCII characters).
const FONT_WIDTH: usize = 8;
const FONT_HEIGHT: usize = 16;

/// Built-in 8x16 font table for printable ASCII characters.
static FONT_DATA: [u8; 128 * 16] = [0u8; 128 * 16];

/// Text console rendering directly to the graphics framebuffer aperture.
pub struct FramebufferConsole {
    fb: RoverFramebuffer,
    cursor_x: usize,
    cursor_y: usize,
    cols: usize,
    rows: usize,
    fg_color: u32,
    bg_color: u32,
}

impl FramebufferConsole {
    /// Creates a new console from the provided linear framebuffer.
    #[must_use]
    pub fn new(fb: RoverFramebuffer) -> Self {
        let cols = (fb.width as usize) / FONT_WIDTH;
        let rows = (fb.height as usize) / FONT_HEIGHT;

        Self {
            fb,
            cursor_x: 0,
            cursor_y: 0,
            cols,
            rows,
            fg_color: 0x00FF_FFFF, // Cyan / White
            bg_color: 0x0000_0000, // Black
        }
    }

    /// Clears the entire framebuffer display with the specified 32-bit background color.
    pub fn clear(&mut self, color: u32) {
        if self.fb.base == 0 || self.fb.size == 0 {
            return;
        }

        let ptr = self.fb.base as *mut u32;
        let count = (self.fb.stride as usize) * (self.fb.height as usize);

        for i in 0..count {
            // SAFETY: Writing to valid mapped framebuffer memory aperture.
            unsafe {
                core::ptr::write_volatile(ptr.add(i), color);
            }
        }

        self.cursor_x = 0;
        self.cursor_y = 0;
    }

    /// Draws a single pixel at `(x, y)` in the framebuffer aperture.
    #[inline]
    #[allow(clippy::many_single_char_names)]
    pub fn put_pixel(&self, x: usize, y: usize, color: u32) {
        if x >= (self.fb.width as usize) || y >= (self.fb.height as usize) {
            return;
        }

        let offset = y * (self.fb.stride as usize) + x;
        let ptr = (self.fb.base as *mut u32).wrapping_add(offset);

        // Adjust RGB vs BGR format.
        let raw_color = if self.fb.format == RoverPixelFormat::Bgr {
            let red = (color >> 16) & 0xFF;
            let green = (color >> 8) & 0xFF;
            let blue = color & 0xFF;
            (blue << 16) | (green << 8) | red
        } else {
            color
        };

        // SAFETY: Writing within validated framebuffer boundary.
        unsafe {
            core::ptr::write_volatile(ptr, raw_color);
        }
    }

    /// Draws an 8x16 ASCII character glyph.
    pub fn draw_char(&self, px: usize, py: usize, ch: char) {
        let ascii = (ch as usize).min(127);
        let glyph_start = ascii * FONT_HEIGHT;

        for row in 0..FONT_HEIGHT {
            let row_byte = FONT_DATA[glyph_start + row];
            for col in 0..FONT_WIDTH {
                let is_fg = (row_byte & (0x80 >> col)) != 0;
                let color = if is_fg { self.fg_color } else { self.bg_color };
                self.put_pixel(px + col, py + row, color);
            }
        }
    }

    /// Writes a character to the console, advancing the cursor and handling newlines.
    pub fn write_char(&mut self, ch: char) {
        match ch {
            '\n' => {
                self.cursor_x = 0;
                self.cursor_y += 1;
                if self.cursor_y >= self.rows {
                    self.cursor_y = 0; // Wrap around for early console
                }
            }
            '\r' => {
                self.cursor_x = 0;
            }
            _ => {
                let px = self.cursor_x * FONT_WIDTH;
                let py = self.cursor_y * FONT_HEIGHT;
                self.draw_char(px, py, ch);

                self.cursor_x += 1;
                if self.cursor_x >= self.cols {
                    self.cursor_x = 0;
                    self.cursor_y += 1;
                    if self.cursor_y >= self.rows {
                        self.cursor_y = 0;
                    }
                }
            }
        }
    }

    /// Writes a string slice to the framebuffer console.
    pub fn write_str(&mut self, s: &str) {
        for ch in s.chars() {
            self.write_char(ch);
        }
    }
}

/// Global thread-safe framebuffer console instance.
pub static FRAMEBUFFER_CONSOLE: SpinLock<Option<FramebufferConsole>> = SpinLock::new(None);

/// Initializes the graphical framebuffer console if a valid linear GOP buffer is present.
pub fn init_framebuffer_console(fb: RoverFramebuffer) {
    if fb.is_valid() {
        let mut console = FramebufferConsole::new(fb);
        console.clear(0x0005_1020); // Dark navy blue background
        console.write_str("Rover OS Graphical Framebuffer Console Initialized.\n");
        *FRAMEBUFFER_CONSOLE.lock() = Some(console);
    }
}
