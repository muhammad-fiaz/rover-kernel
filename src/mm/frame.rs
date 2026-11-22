//! # Physical Memory Frame Allocator
//!
//! Page-frame allocation using a bitmap tracker over usable system RAM regions.

use super::address::{PAGE_SIZE, PhysAddr};
use crate::boot::{RoverBootInfo, RoverMemoryDescriptor, RoverMemoryType};
use crate::sync::SpinLock;

/// Maximum number of physical memory pages supported by the early bitmap (128 GiB RAM = 33,554,432 pages).
/// For a 4 GiB initial tracking buffer: 4 GiB / 4096 = 1,048,576 pages.
/// 1,048,576 bits = 131,072 bytes (16,384 `u64` words = 128 KiB).
pub const BITMAP_WORDS: usize = 16_384;

/// Bitmap-backed physical frame allocator.
pub struct BitmapFrameAllocator {
    bitmap: [u64; BITMAP_WORDS],
    total_frames: usize,
    used_frames: usize,
    highest_frame: usize,
}

impl BitmapFrameAllocator {
    /// Creates an uninitialized bitmap allocator where all frames are initially marked as used.
    #[must_use]
    #[allow(clippy::large_stack_arrays)]
    pub const fn new() -> Self {
        Self {
            bitmap: [!0u64; BITMAP_WORDS], // All 1s = used
            total_frames: 0,
            used_frames: 0,
            highest_frame: 0,
        }
    }

    /// Initializes the frame allocator from the provided [`RoverBootInfo`].
    ///
    /// # Safety
    ///
    /// The caller must ensure that `boot_info` contains valid physical memory map pointers.
    pub unsafe fn init(&mut self, boot_info: &RoverBootInfo) {
        // Start with all bits set to 1 (used/reserved).
        self.bitmap.fill(!0u64);
        self.total_frames = 0;
        self.used_frames = 0;

        // 1. Mark usable regions as available (0).
        let descriptors = unsafe { boot_info.memory_map() };
        for desc in descriptors {
            if desc.region_type == RoverMemoryType::Usable {
                self.mark_region_free(desc);
            }
        }

        // 2. Reserve low 1 MiB (real mode IVT, BDA, BIOS data).
        self.reserve_range(0, 0x10_0000);

        // 3. Reserve loaded kernel executable memory.
        let k_base = boot_info.kernel_image.physical_base;
        let k_size = boot_info.kernel_image.size;
        self.reserve_range(k_base, k_size);

        // 4. Reserve boot info block and memory descriptor map.
        self.reserve_range(
            boot_info.memory_map_addr,
            boot_info
                .memory_map_entries
                .saturating_mul(boot_info.memory_map_entry_size),
        );
        self.reserve_range(
            core::ptr::from_ref::<RoverBootInfo>(boot_info) as u64,
            core::mem::size_of::<RoverBootInfo>() as u64,
        );

        // Calculate used vs free frames count.
        let mut free = 0;
        for i in 0..self.highest_frame {
            let word_idx = i / 64;
            let bit_idx = i % 64;
            if word_idx < BITMAP_WORDS && (self.bitmap[word_idx] & (1 << bit_idx)) == 0 {
                free += 1;
            }
        }
        self.used_frames = self.total_frames.saturating_sub(free);
    }

    #[allow(clippy::cast_possible_truncation)]
    fn mark_region_free(&mut self, desc: &RoverMemoryDescriptor) {
        let start_page = (desc.physical_start / PAGE_SIZE) as usize;
        let page_count = desc.number_of_pages as usize;
        let end_page = start_page.saturating_add(page_count);

        for page in start_page..end_page {
            let word_idx = page / 64;
            let bit_idx = page % 64;
            if word_idx < BITMAP_WORDS {
                self.bitmap[word_idx] &= !(1 << bit_idx);
                self.total_frames += 1;
                if page > self.highest_frame {
                    self.highest_frame = page;
                }
            }
        }
    }

    /// Reserves a range of physical memory, marking all intersecting pages as used.
    #[allow(clippy::cast_possible_truncation)]
    pub fn reserve_range(&mut self, start_addr: u64, size: u64) {
        if size == 0 {
            return;
        }
        let start_page = (start_addr / PAGE_SIZE) as usize;
        let end_addr = start_addr.saturating_add(size);
        let end_page = end_addr.div_ceil(PAGE_SIZE) as usize;

        for page in start_page..end_page {
            let word_idx = page / 64;
            let bit_idx = page % 64;
            if word_idx < BITMAP_WORDS {
                self.bitmap[word_idx] |= 1 << bit_idx;
            }
        }
    }

    /// Allocates a single 4 KiB physical memory frame.
    ///
    /// Returns the physical address of the allocated frame, or `None` if out of memory.
    pub fn allocate_frame(&mut self) -> Option<PhysAddr> {
        for word_idx in 0..BITMAP_WORDS {
            if self.bitmap[word_idx] != !0u64 {
                // Find first zero bit in this 64-bit word.
                let zero_bit = (!self.bitmap[word_idx]).trailing_zeros() as usize;
                self.bitmap[word_idx] |= 1 << zero_bit;
                self.used_frames += 1;

                let page_idx = (word_idx * 64) + zero_bit;
                let phys_addr = (page_idx as u64) * PAGE_SIZE;
                return Some(PhysAddr::new(phys_addr));
            }
        }
        None
    }

    /// Deallocates a previously allocated physical memory frame.
    #[allow(clippy::cast_possible_truncation)]
    pub fn deallocate_frame(&mut self, frame: PhysAddr) {
        let page = (frame.as_u64() / PAGE_SIZE) as usize;
        let word_idx = page / 64;
        let bit_idx = page % 64;
        if word_idx < BITMAP_WORDS && (self.bitmap[word_idx] & (1 << bit_idx)) != 0 {
            self.bitmap[word_idx] &= !(1 << bit_idx);
            self.used_frames = self.used_frames.saturating_sub(1);
        }
    }

    /// Returns the total number of physical memory frames tracked.
    #[must_use]
    pub const fn total_frames(&self) -> usize {
        self.total_frames
    }

    /// Returns the number of physical memory frames currently marked as used.
    #[must_use]
    pub const fn used_frames(&self) -> usize {
        self.used_frames
    }

    /// Returns the number of physical memory frames available for allocation.
    #[must_use]
    pub const fn free_frames(&self) -> usize {
        self.total_frames.saturating_sub(self.used_frames)
    }
}

impl Default for BitmapFrameAllocator {
    fn default() -> Self {
        Self::new()
    }
}

/// Global physical frame allocator instance protected by a spinlock.
pub static FRAME_ALLOCATOR: SpinLock<BitmapFrameAllocator> =
    SpinLock::new(BitmapFrameAllocator::new());
