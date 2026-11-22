//! # Kernel Heap Allocator
//!
//! Provides dynamic memory allocation for `#![no_std]` Rust using a coalescing
//! free-list block allocator implementing [`core::alloc::GlobalAlloc`].

use crate::sync::SpinLock;
use core::alloc::{GlobalAlloc, Layout};

/// Minimum block alignment (16 bytes for x86-64 SSE/AVX compatibility).
pub const MIN_ALIGNMENT: usize = 16;

/// Header preceding each allocated or free chunk on the heap.
#[repr(C, align(16))]
struct BlockHeader {
    size: usize,
    is_free: bool,
    next: *mut BlockHeader,
}

impl BlockHeader {
    const HEADER_SIZE: usize = core::mem::size_of::<Self>();
}

/// A thread-safe, coalescing free-list kernel heap allocator.
pub struct HeapAllocator {
    head: *mut BlockHeader,
    total_size: usize,
    allocated_bytes: usize,
}

// SAFETY: Synchronization is managed by SpinLock.
unsafe impl Send for HeapAllocator {}

impl HeapAllocator {
    /// Creates an uninitialized heap allocator.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            head: core::ptr::null_mut(),
            total_size: 0,
            allocated_bytes: 0,
        }
    }

    /// Initializes the heap with the specified memory region.
    ///
    /// # Safety
    ///
    /// `start` must point to valid, mapped, writable memory of at least `size` bytes.
    pub unsafe fn init(&mut self, start: usize, size: usize) {
        if size <= BlockHeader::HEADER_SIZE {
            return;
        }

        let aligned_start = (start + MIN_ALIGNMENT - 1) & !(MIN_ALIGNMENT - 1);
        let usable_size = size.saturating_sub(aligned_start - start);

        let initial_block = aligned_start as *mut BlockHeader;
        unsafe {
            (*initial_block).size = usable_size - BlockHeader::HEADER_SIZE;
            (*initial_block).is_free = true;
            (*initial_block).next = core::ptr::null_mut();
        }

        self.head = initial_block;
        self.total_size = usable_size;
        self.allocated_bytes = 0;
    }

    /// Allocates a chunk of memory satisfying the specified layout.
    pub fn alloc(&mut self, layout: Layout) -> *mut u8 {
        if self.head.is_null() {
            return core::ptr::null_mut();
        }

        let align = layout.align().max(MIN_ALIGNMENT);
        let req_size = (layout.size() + align - 1) & !(align - 1);

        let mut current = self.head;

        while !current.is_null() {
            let block = unsafe { &mut *current };

            if block.is_free && block.size >= req_size {
                let excess = block.size - req_size;

                // Split block if excess space is large enough for another header + payload.
                if excess >= BlockHeader::HEADER_SIZE + MIN_ALIGNMENT {
                    let next_addr = (current as usize) + BlockHeader::HEADER_SIZE + req_size;
                    let next_block = next_addr as *mut BlockHeader;

                    unsafe {
                        (*next_block).size = excess - BlockHeader::HEADER_SIZE;
                        (*next_block).is_free = true;
                        (*next_block).next = block.next;
                    }

                    block.size = req_size;
                    block.next = next_block;
                }

                block.is_free = false;
                self.allocated_bytes += block.size;

                let data_ptr = (current as usize) + BlockHeader::HEADER_SIZE;
                return data_ptr as *mut u8;
            }

            current = block.next;
        }

        core::ptr::null_mut()
    }

    /// Deallocates a chunk of memory previously allocated by [`Self::alloc`].
    pub fn dealloc(&mut self, ptr: *mut u8) {
        if ptr.is_null() || self.head.is_null() {
            return;
        }

        let header_ptr = ((ptr as usize) - BlockHeader::HEADER_SIZE) as *mut BlockHeader;
        let block = unsafe { &mut *header_ptr };
        block.is_free = true;
        self.allocated_bytes = self.allocated_bytes.saturating_sub(block.size);

        // Coalesce adjacent free blocks.
        self.coalesce();
    }

    /// Merges contiguous adjacent free blocks in the list.
    fn coalesce(&mut self) {
        let mut current = self.head;

        while !current.is_null() {
            let block = unsafe { &mut *current };
            let next_ptr = block.next;

            if !next_ptr.is_null() {
                let next_block = unsafe { &mut *next_ptr };
                let current_end = (current as usize) + BlockHeader::HEADER_SIZE + block.size;

                if block.is_free && next_block.is_free && current_end == (next_ptr as usize) {
                    block.size += BlockHeader::HEADER_SIZE + next_block.size;
                    block.next = next_block.next;
                    continue; // Re-check this block with the new next
                }
            }

            current = block.next;
        }
    }

    /// Returns the total bytes managed by the heap.
    #[must_use]
    pub const fn total_size(&self) -> usize {
        self.total_size
    }

    /// Returns the number of currently allocated bytes.
    #[must_use]
    pub const fn allocated_bytes(&self) -> usize {
        self.allocated_bytes
    }

    /// Returns the number of free bytes available.
    #[must_use]
    pub const fn free_bytes(&self) -> usize {
        self.total_size.saturating_sub(self.allocated_bytes)
    }
}

impl Default for HeapAllocator {
    fn default() -> Self {
        Self::new()
    }
}

/// Global thread-safe wrapper implementing [`GlobalAlloc`].
pub struct LockedHeap(SpinLock<HeapAllocator>);

impl LockedHeap {
    /// Creates a new locked heap wrapper.
    #[must_use]
    pub const fn new() -> Self {
        Self(SpinLock::new(HeapAllocator::new()))
    }

    /// Initializes the heap with the specified memory region.
    ///
    /// # Safety
    ///
    /// `start` must point to valid writable memory of at least `size` bytes.
    pub unsafe fn init(&self, start: usize, size: usize) {
        unsafe {
            self.0.lock().init(start, size);
        }
    }

    /// Returns heap memory usage statistics `(total, allocated, free)`.
    #[must_use]
    pub fn stats(&self) -> (usize, usize, usize) {
        let heap = self.0.lock();
        (heap.total_size(), heap.allocated_bytes(), heap.free_bytes())
    }
}

impl Default for LockedHeap {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl GlobalAlloc for LockedHeap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.0.lock().alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        self.0.lock().dealloc(ptr);
    }
}

#[cfg_attr(target_os = "none", global_allocator)]
static GLOBAL_HEAP_ALLOCATOR: LockedHeap = LockedHeap::new();

/// Initializes the global kernel heap at the specified memory buffer.
///
/// # Safety
///
/// Caller must ensure `start` is a valid mapped virtual memory region.
pub unsafe fn init_kernel_heap(start: usize, size: usize) {
    unsafe {
        GLOBAL_HEAP_ALLOCATOR.init(start, size);
    }
}

/// Returns global heap usage statistics: `(total_bytes, allocated_bytes, free_bytes)`.
#[must_use]
pub fn heap_stats() -> (usize, usize, usize) {
    GLOBAL_HEAP_ALLOCATOR.stats()
}
