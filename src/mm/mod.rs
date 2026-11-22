//! # Memory Management Subsystem
//!
//! Exposes physical and virtual memory address primitives, bitmap frame allocation,
//! x86-64 4-level paging, and the kernel heap allocator.

pub mod address;
pub mod frame;
pub mod heap;
pub mod paging;

pub use address::{PAGE_SIZE, PhysAddr, VirtAddr};
pub use frame::{BitmapFrameAllocator, FRAME_ALLOCATOR};
pub use heap::{HeapAllocator, LockedHeap, heap_stats, init_kernel_heap};
pub use paging::{
    PageTable, PageTableEntry, PageTableFlags, invalidate_page, map_page, translate_virtual_address,
};
