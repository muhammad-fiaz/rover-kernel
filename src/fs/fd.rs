//! # File Descriptor Management
//!
//! File descriptor allocation, per-process descriptor tables, and offset tracking.

extern crate alloc;
use alloc::sync::Arc;
use alloc::vec::Vec;

use super::vfs::Inode;

/// Representation of an open file instance with an active seek offset.
pub struct OpenFile {
    /// Target inode.
    pub inode: Arc<dyn Inode>,
    /// Current read/write seek offset in bytes.
    pub offset: u64,
    /// Access flags (read, write).
    pub flags: u32,
}

/// Per-process file descriptor table.
pub struct FdTable {
    descriptors: Vec<Option<OpenFile>>,
}

impl FdTable {
    /// Creates an empty file descriptor table.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            descriptors: Vec::new(),
        }
    }

    /// Allocates the lowest available file descriptor for the specified inode.
    pub fn allocate(&mut self, inode: Arc<dyn Inode>, flags: u32) -> usize {
        let entry = OpenFile {
            inode,
            offset: 0,
            flags,
        };

        for (idx, slot) in self.descriptors.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = Some(entry);
                return idx;
            }
        }

        let idx = self.descriptors.len();
        self.descriptors.push(Some(entry));
        idx
    }

    /// Retrieves a mutable reference to an open file by file descriptor number.
    #[must_use]
    pub fn get_mut(&mut self, fd: usize) -> Option<&mut OpenFile> {
        self.descriptors.get_mut(fd)?.as_mut()
    }

    /// Closes a file descriptor, releasing its slot.
    pub fn close(&mut self, fd: usize) -> bool {
        if let Some(slot) = self.descriptors.get_mut(fd) {
            slot.take().is_some()
        } else {
            false
        }
    }
}

impl Default for FdTable {
    fn default() -> Self {
        Self::new()
    }
}
