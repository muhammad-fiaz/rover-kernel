//! # Memory Diagnostics Reporting
//!
//! Queries physical frame allocator and heap statistics to generate memory diagnostics.

use crate::mm::{FRAME_ALLOCATOR, PAGE_SIZE, heap_stats};

/// Memory usage statistics snapshot across physical and heap memory.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MemoryInfo {
    /// Total physical RAM tracked by the frame allocator in bytes.
    pub total_ram_bytes: u64,
    /// Physical RAM currently allocated in bytes.
    pub used_ram_bytes: u64,
    /// Physical RAM currently free in bytes.
    pub free_ram_bytes: u64,
    /// Total kernel heap size in bytes.
    pub heap_total_bytes: usize,
    /// Heap memory currently allocated in bytes.
    pub heap_used_bytes: usize,
    /// Heap memory currently free in bytes.
    pub heap_free_bytes: usize,
}

/// Retrieves a snapshot of current memory allocations across physical frames and kernel heap.
#[must_use]
pub fn get_memory_info() -> MemoryInfo {
    let frame_alloc = FRAME_ALLOCATOR.lock();
    let total_ram_bytes = (frame_alloc.total_frames() as u64) * PAGE_SIZE;
    let used_ram_bytes = (frame_alloc.used_frames() as u64) * PAGE_SIZE;
    let free_ram_bytes = (frame_alloc.free_frames() as u64) * PAGE_SIZE;
    drop(frame_alloc);

    let (heap_total, heap_used, heap_free) = heap_stats();

    MemoryInfo {
        total_ram_bytes,
        used_ram_bytes,
        free_ram_bytes,
        heap_total_bytes: heap_total,
        heap_used_bytes: heap_used,
        heap_free_bytes: heap_free,
    }
}

/// Emits formatted memory diagnostics to the kernel log.
pub fn dump_memory_diagnostics() {
    let info = get_memory_info();
    crate::kinfo!("--- Memory Diagnostics ---");
    crate::kinfo!(
        "Physical RAM: Total: {} MiB, Used: {} KiB, Free: {} MiB",
        info.total_ram_bytes / (1024 * 1024),
        info.used_ram_bytes / 1024,
        info.free_ram_bytes / (1024 * 1024)
    );
    crate::kinfo!(
        "Kernel Heap:  Total: {} KiB, Used: {} KiB, Free: {} KiB",
        info.heap_total_bytes / 1024,
        info.heap_used_bytes / 1024,
        info.heap_free_bytes / 1024
    );
    crate::kinfo!("--------------------------");
}
