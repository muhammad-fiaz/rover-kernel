//! # Block Device Driver Interfaces
//!
//! Standard traits and in-memory block device implementations for storage subsystems.

extern crate alloc;
use crate::sync::SpinLock;
use alloc::vec;

/// Common trait implemented by all block storage device drivers.
pub trait BlockDevice: Send + Sync {
    /// Returns the logical block size in bytes (typically 512 or 4096).
    fn block_size(&self) -> usize;

    /// Returns the total number of addressable logical blocks.
    fn total_blocks(&self) -> u64;

    /// Reads `count` logical blocks beginning at `lba` into `buf`.
    ///
    /// # Errors
    ///
    /// Returns an error string if `buf` is too small or if `lba` exceeds device capacity.
    fn read_blocks(&self, lba: u64, count: usize, buf: &mut [u8]) -> Result<(), &'static str>;

    /// Writes `count` logical blocks from `buf` beginning at `lba`.
    ///
    /// # Errors
    ///
    /// Returns an error string if `buf` is too small or if `lba` exceeds device capacity.
    fn write_blocks(&mut self, lba: u64, count: usize, buf: &[u8]) -> Result<(), &'static str>;
}

/// An in-memory block device backed by a contiguous byte vector.
pub struct RamBlockDevice {
    data: SpinLock<alloc::vec::Vec<u8>>,
    block_size: usize,
    total_blocks: u64,
}

impl RamBlockDevice {
    /// Creates a new RAM block device with the specified block size and total block count.
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn new(block_size: usize, total_blocks: u64) -> Self {
        let total_bytes = (block_size as u64).saturating_mul(total_blocks) as usize;
        let data = vec![0u8; total_bytes];

        Self {
            data: SpinLock::new(data),
            block_size,
            total_blocks,
        }
    }
}

impl BlockDevice for RamBlockDevice {
    fn block_size(&self) -> usize {
        self.block_size
    }

    fn total_blocks(&self) -> u64 {
        self.total_blocks
    }

    #[allow(clippy::cast_possible_truncation)]
    fn read_blocks(&self, lba: u64, count: usize, buf: &mut [u8]) -> Result<(), &'static str> {
        let block_sz = self.block_size;
        let req_bytes = count.saturating_mul(block_sz);
        if buf.len() < req_bytes {
            return Err("Buffer too small for requested block read");
        }

        let start_offset = (lba as usize).saturating_mul(block_sz);
        let end_offset = start_offset.saturating_add(req_bytes);

        let lock = self.data.lock();
        if end_offset > lock.len() {
            return Err("Read request exceeds device capacity");
        }

        buf[..req_bytes].copy_from_slice(&lock[start_offset..end_offset]);
        Ok(())
    }

    #[allow(clippy::cast_possible_truncation)]
    fn write_blocks(&mut self, lba: u64, count: usize, buf: &[u8]) -> Result<(), &'static str> {
        let block_sz = self.block_size;
        let req_bytes = count.saturating_mul(block_sz);
        if buf.len() < req_bytes {
            return Err("Buffer too small for requested block write");
        }

        let start_offset = (lba as usize).saturating_mul(block_sz);
        let end_offset = start_offset.saturating_add(req_bytes);

        let mut lock = self.data.lock();
        if end_offset > lock.len() {
            return Err("Write request exceeds device capacity");
        }

        lock[start_offset..end_offset].copy_from_slice(&buf[..req_bytes]);
        Ok(())
    }
}
