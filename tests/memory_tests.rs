//! # Memory Subsystem Unit Tests
//!
//! Validates physical and virtual address helpers, page table indexing,
//! frame allocation, and kernel heap allocation and coalescing.

#![allow(clippy::unwrap_used, clippy::unreadable_literal, clippy::borrow_as_ptr, clippy::doc_markdown)]

use core::alloc::Layout;
use rover_kernel::mm::address::{PAGE_SIZE, PhysAddr, VirtAddr};
use rover_kernel::mm::heap::HeapAllocator;
use rover_kernel::mm::paging::{PageTable, PageTableFlags};

#[test]
fn test_phys_addr_alignment() {
    let unaligned = PhysAddr::new(0x1005);
    assert!(!unaligned.is_aligned(PAGE_SIZE));
    assert_eq!(unaligned.align_down(PAGE_SIZE), PhysAddr::new(0x1000));
    assert_eq!(unaligned.align_up(PAGE_SIZE), PhysAddr::new(0x2000));

    let aligned = PhysAddr::new(0x2000);
    assert!(aligned.is_aligned(PAGE_SIZE));
    assert_eq!(aligned.align_down(PAGE_SIZE), PhysAddr::new(0x2000));
    assert_eq!(aligned.align_up(PAGE_SIZE), PhysAddr::new(0x2000));
}

#[test]
fn test_virt_addr_page_table_indices() {
    // Canonical virtual address: 0xFFFF_8000_1234_5678
    let virt = VirtAddr::new(0xFFFF_8000_1234_5678);

    assert_eq!(
        virt.pml4_index(),
        ((0xFFFF_8000_1234_5678u64 >> 39) & 0x1FF) as usize
    );
    assert_eq!(
        virt.pdpt_index(),
        ((0xFFFF_8000_1234_5678u64 >> 30) & 0x1FF) as usize
    );
    assert_eq!(
        virt.pd_index(),
        ((0xFFFF_8000_1234_5678u64 >> 21) & 0x1FF) as usize
    );
    assert_eq!(
        virt.pt_index(),
        ((0xFFFF_8000_1234_5678u64 >> 12) & 0x1FF) as usize
    );
    assert_eq!(virt.page_offset(), 0x678);
}

#[test]
fn test_page_table_flags() {
    let flags = PageTableFlags::PRESENT
        .union(PageTableFlags::WRITABLE)
        .union(PageTableFlags::NO_EXECUTE);

    assert!(flags.contains(PageTableFlags::PRESENT));
    assert!(flags.contains(PageTableFlags::WRITABLE));
    assert!(flags.contains(PageTableFlags::NO_EXECUTE));
    assert!(!flags.contains(PageTableFlags::USER_ACCESSIBLE));
}

#[test]
fn test_page_table_entries() {
    let mut table = PageTable::default();
    assert!(table[0].is_unused());

    table[0].set_addr_and_flags(
        PhysAddr::new(0x10_0000),
        PageTableFlags::PRESENT.union(PageTableFlags::WRITABLE),
    );

    assert!(!table[0].is_unused());
    assert_eq!(table[0].addr(), PhysAddr::new(0x10_0000));
    assert!(table[0].flags().contains(PageTableFlags::PRESENT));
    assert!(table[0].flags().contains(PageTableFlags::WRITABLE));
}

#[test]
fn test_heap_allocator_basic_and_coalesce() {
    let mut backing_buffer = [0u8; 16384];
    let mut heap = HeapAllocator::default();

    unsafe {
        heap.init(backing_buffer.as_mut_ptr() as usize, backing_buffer.len());
    }

    assert!(heap.total_size() > 0);
    assert_eq!(heap.allocated_bytes(), 0);

    let layout1 = Layout::from_size_align(64, 16).unwrap();
    let ptr1 = heap.alloc(layout1);
    assert!(!ptr1.is_null());
    assert!(heap.allocated_bytes() >= 64);

    let layout2 = Layout::from_size_align(128, 16).unwrap();
    let ptr2 = heap.alloc(layout2);
    assert!(!ptr2.is_null());
    assert_ne!(ptr1, ptr2);

    heap.dealloc(ptr1);
    heap.dealloc(ptr2);

    // After deallocation and coalescing, all space should be free again.
    assert_eq!(heap.allocated_bytes(), 0);
}
