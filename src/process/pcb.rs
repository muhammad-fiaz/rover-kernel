//! # Process Model and Address Space Isolation
//!
//! Process descriptors, process IDs, user address space mappings, and thread associations.

extern crate alloc;
use crate::mm::PageTable;
use crate::sched::TaskId;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};

/// Monotonically increasing process identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProcessId(pub u64);

static NEXT_PID: AtomicU64 = AtomicU64::new(1);

impl ProcessId {
    /// Generates a new unique process identifier.
    #[must_use]
    pub fn generate() -> Self {
        Self(NEXT_PID.fetch_add(1, Ordering::Relaxed))
    }
}

/// Process Control Block representing an isolated execution unit.
pub struct Process {
    /// Unique process ID.
    pub pid: ProcessId,
    /// Root Level-4 Page Table for this process's address space.
    pub pml4: *mut PageTable,
    /// Collection of threads executing within this process.
    pub threads: Vec<TaskId>,
    /// Human-readable process name.
    pub name: [u8; 32],
}

// SAFETY: Processes are synchronized via kernel locks.
unsafe impl Send for Process {}
unsafe impl Sync for Process {}

impl Process {
    /// Creates a new process structure.
    #[must_use]
    pub fn new(pid: ProcessId, name: &str, pml4: *mut PageTable) -> Self {
        let mut name_buf = [0u8; 32];
        let bytes = name.as_bytes();
        let copy_len = bytes.len().min(31);
        name_buf[..copy_len].copy_from_slice(&bytes[..copy_len]);

        Self {
            pid,
            pml4,
            threads: Vec::new(),
            name: name_buf,
        }
    }

    /// Attaches an existing task to this process.
    pub fn add_thread(&mut self, task_id: TaskId) {
        self.threads.push(task_id);
    }

    /// Returns the human-readable process name.
    #[must_use]
    pub fn name_str(&self) -> &str {
        core::str::from_utf8(&self.name)
            .unwrap_or("unknown")
            .trim_matches('\0')
    }
}
