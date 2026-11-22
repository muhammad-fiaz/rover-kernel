//! # In-Memory RAM Filesystem (`RamFS`)
//!
//! Complete, in-memory hierarchical filesystem supporting files, directories,
//! dynamic reading, and writing.

extern crate alloc;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;

use super::vfs::{FileStat, FileSystem, Inode, NodeType};
use crate::sync::SpinLock;

/// An in-memory regular file containing byte data.
pub struct RamFile {
    data: SpinLock<Vec<u8>>,
}

impl RamFile {
    /// Creates a new empty RAM file.
    #[must_use]
    pub fn new() -> Self {
        Self {
            data: SpinLock::new(Vec::new()),
        }
    }

    /// Creates a new RAM file populated with initial byte data.
    #[must_use]
    pub fn with_data(initial: &[u8]) -> Self {
        let mut vec = Vec::with_capacity(initial.len());
        vec.extend_from_slice(initial);
        Self {
            data: SpinLock::new(vec),
        }
    }
}

impl Default for RamFile {
    fn default() -> Self {
        Self::new()
    }
}

impl Inode for RamFile {
    fn stat(&self) -> FileStat {
        let len = self.data.lock().len() as u64;
        FileStat {
            size: len,
            node_type: NodeType::File,
            mode: 0o644,
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    fn read(&self, offset: u64, buf: &mut [u8]) -> Result<usize, &'static str> {
        let lock = self.data.lock();
        let off = offset as usize;
        if off >= lock.len() {
            return Ok(0);
        }
        let available = lock.len() - off;
        let to_read = available.min(buf.len());
        buf[..to_read].copy_from_slice(&lock[off..off + to_read]);
        Ok(to_read)
    }

    #[allow(clippy::cast_possible_truncation)]
    fn write(&mut self, offset: u64, buf: &[u8]) -> Result<usize, &'static str> {
        let mut lock = self.data.lock();
        let off = offset as usize;
        if off + buf.len() > lock.len() {
            lock.resize(off + buf.len(), 0);
        }
        lock[off..off + buf.len()].copy_from_slice(buf);
        Ok(buf.len())
    }

    fn lookup(&self, _name: &str) -> Result<Arc<dyn Inode>, &'static str> {
        Err("Not a directory")
    }

    fn read_dir(&self) -> Result<Vec<String>, &'static str> {
        Err("Not a directory")
    }
}

/// An in-memory directory containing child nodes indexed by name.
pub struct RamDir {
    children: SpinLock<BTreeMap<String, Arc<dyn Inode>>>,
}

impl RamDir {
    /// Creates a new empty directory node.
    #[must_use]
    pub fn new() -> Self {
        Self {
            children: SpinLock::new(BTreeMap::new()),
        }
    }

    /// Adds a child inode under the specified name.
    pub fn add_child(&self, name: String, node: Arc<dyn Inode>) {
        self.children.lock().insert(name, node);
    }
}

impl Default for RamDir {
    fn default() -> Self {
        Self::new()
    }
}

impl Inode for RamDir {
    fn stat(&self) -> FileStat {
        let count = self.children.lock().len() as u64;
        FileStat {
            size: count,
            node_type: NodeType::Directory,
            mode: 0o755,
        }
    }

    fn read(&self, _offset: u64, _buf: &mut [u8]) -> Result<usize, &'static str> {
        Err("Is a directory")
    }

    fn write(&mut self, _offset: u64, _buf: &[u8]) -> Result<usize, &'static str> {
        Err("Is a directory")
    }

    fn lookup(&self, name: &str) -> Result<Arc<dyn Inode>, &'static str> {
        self.children
            .lock()
            .get(name)
            .cloned()
            .ok_or("File not found")
    }

    fn read_dir(&self) -> Result<Vec<String>, &'static str> {
        let lock = self.children.lock();
        let mut list = Vec::new();
        for key in lock.keys() {
            list.push(key.clone());
        }
        Ok(list)
    }
}

/// In-memory `RamFS` root filesystem.
pub struct RamFs {
    root: Arc<RamDir>,
}

impl RamFs {
    /// Creates a new initialized in-memory filesystem.
    #[must_use]
    pub fn new() -> Self {
        Self {
            root: Arc::new(RamDir::new()),
        }
    }

    /// Returns a reference to the root directory node.
    #[must_use]
    pub fn root_dir(&self) -> Arc<RamDir> {
        self.root.clone()
    }
}

impl Default for RamFs {
    fn default() -> Self {
        Self::new()
    }
}

impl FileSystem for RamFs {
    fn root(&self) -> Arc<dyn Inode> {
        self.root.clone()
    }
}
