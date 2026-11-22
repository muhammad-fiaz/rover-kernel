//! # Virtual File System Subsystem
//!
//! Exposes VFS traits (`Inode`, `FileSystem`), in-memory `RamFS`, and file descriptor tables.

pub mod fd;
pub mod ramfs;
pub mod vfs;

pub use fd::{FdTable, OpenFile};
pub use ramfs::{RamDir, RamFile, RamFs};
pub use vfs::{FileStat, FileSystem, Inode, NodeType};
