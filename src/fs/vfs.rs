//! # Virtual File System (VFS) Abstractions
//!
//! Node types, directory entries, filesystem traits, and node traversal interfaces.

extern crate alloc;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;

/// Classification of a virtual filesystem node.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeType {
    /// Regular binary or text file.
    File,
    /// Directory containing child nodes.
    Directory,
    /// Character device (e.g., serial console).
    CharDevice,
    /// Block storage device (e.g., `NVMe`, `SATA` disk).
    BlockDevice,
}

/// Metadata statistics describing an inode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FileStat {
    /// Size of the node in bytes.
    pub size: u64,
    /// Classification type of the node.
    pub node_type: NodeType,
    /// Permission mode bits.
    pub mode: u32,
}

/// Common trait implemented by all virtual filesystem inodes.
pub trait Inode: Send + Sync {
    /// Returns metadata statistics for this inode.
    fn stat(&self) -> FileStat;

    /// Reads data from the node into `buf` beginning at byte `offset`.
    ///
    /// # Errors
    ///
    /// Returns an error string if reading fails or if the node is not readable.
    fn read(&self, offset: u64, buf: &mut [u8]) -> Result<usize, &'static str>;

    /// Writes data from `buf` to the node beginning at byte `offset`.
    ///
    /// # Errors
    ///
    /// Returns an error string if writing fails or if the node is not writable.
    fn write(&mut self, offset: u64, buf: &[u8]) -> Result<usize, &'static str>;

    /// Searches for a child node by `name` within this directory inode.
    ///
    /// # Errors
    ///
    /// Returns an error string if this node is not a directory or if child is not found.
    fn lookup(&self, name: &str) -> Result<Arc<dyn Inode>, &'static str>;

    /// Lists directory entry names contained within this directory inode.
    ///
    /// # Errors
    ///
    /// Returns an error string if this node is not a directory.
    fn read_dir(&self) -> Result<Vec<String>, &'static str>;
}

/// Common trait implemented by all concrete filesystems.
pub trait FileSystem: Send + Sync {
    /// Returns the root inode of the filesystem.
    fn root(&self) -> Arc<dyn Inode>;
}
