//! # x86-64 4-Level Paging Implementation
//!
//! Page tables (PML4, PDPT, PD, PT), entry flags, page table walker,
//! mapping creation, unmapping, address translation, and TLB invalidation.

use super::address::{PhysAddr, VirtAddr};
use super::frame::BitmapFrameAllocator;
use core::ops::{Index, IndexMut};

/// Flags for page table entries.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PageTableFlags(pub u64);

impl PageTableFlags {
    /// Page is present in physical memory.
    pub const PRESENT: Self = Self(1 << 0);
    /// Page is writable. If not set, page is read-only.
    pub const WRITABLE: Self = Self(1 << 1);
    /// Page is accessible from user space (ring 3).
    pub const USER_ACCESSIBLE: Self = Self(1 << 2);
    /// Write-through caching enabled.
    pub const WRITE_THROUGH: Self = Self(1 << 3);
    /// Cache disabled for this page.
    pub const NO_CACHE: Self = Self(1 << 4);
    /// Page was accessed by CPU.
    pub const ACCESSED: Self = Self(1 << 5);
    /// Page was written to by CPU (dirty).
    pub const DIRTY: Self = Self(1 << 6);
    /// Huge page (2 MiB in PD, 1 GiB in PDPT).
    pub const HUGE_PAGE: Self = Self(1 << 7);
    /// Global page (not flushed from TLB across CR3 reloads).
    pub const GLOBAL: Self = Self(1 << 8);
    /// No-Execute bit: instruction execution disabled from this page.
    pub const NO_EXECUTE: Self = Self(1 << 63);

    /// Combines two sets of flags.
    #[inline]
    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Returns `true` if all flags in `other` are contained in `self`.
    #[inline]
    #[must_use]
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }
}

/// A single entry in an x86-64 page table.
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct PageTableEntry(pub u64);

impl PageTableEntry {
    /// Creates an unused/cleared page table entry.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self(0)
    }

    /// Returns `true` if the entry is not present.
    #[inline]
    #[must_use]
    pub const fn is_unused(&self) -> bool {
        self.0 == 0
    }

    /// Clears the entry.
    #[inline]
    pub fn set_unused(&mut self) {
        self.0 = 0;
    }

    /// Returns the entry's flags.
    #[inline]
    #[must_use]
    pub const fn flags(&self) -> PageTableFlags {
        PageTableFlags(self.0 & 0x8000_0000_0000_01FF)
    }

    /// Returns the physical address referenced by this entry (bits 12..51).
    #[inline]
    #[must_use]
    pub const fn addr(&self) -> PhysAddr {
        PhysAddr::new(self.0 & 0x000F_FFFF_FFFF_F000)
    }

    /// Sets the physical address and flags for this entry.
    #[inline]
    pub fn set_addr_and_flags(&mut self, addr: PhysAddr, flags: PageTableFlags) {
        self.0 = (addr.as_u64() & 0x000F_FFFF_FFFF_F000) | flags.0;
    }
}

impl Default for PageTableEntry {
    fn default() -> Self {
        Self::new()
    }
}

/// A 4 KiB page table containing 512 entries.
#[repr(C, align(4096))]
pub struct PageTable {
    entries: [PageTableEntry; 512],
}

impl PageTable {
    /// Creates a new zeroed page table.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            entries: [PageTableEntry::new(); 512],
        }
    }

    /// Clears all 512 entries in the table.
    pub fn zero(&mut self) {
        for entry in &mut self.entries {
            entry.set_unused();
        }
    }
}

impl Default for PageTable {
    fn default() -> Self {
        Self::new()
    }
}

impl Index<usize> for PageTable {
    type Output = PageTableEntry;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.entries[index]
    }
}

impl IndexMut<usize> for PageTable {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.entries[index]
    }
}

/// Flushes the TLB entry for the specified virtual address (`invlpg`).
#[inline]
pub fn invalidate_page(addr: VirtAddr) {
    // SAFETY: invlpg flushes the translation cache for the specified linear address.
    unsafe {
        core::arch::asm!("invlpg [{0}]", in(reg) addr.as_u64(), options(nostack, preserves_flags));
    }
}

/// Translates a virtual address to its corresponding physical address by walking page tables.
///
/// # Safety
///
/// The caller must ensure `pml4` points to an active, valid PML4 table in identity-mapped memory.
#[must_use]
#[allow(clippy::similar_names)]
pub unsafe fn translate_virtual_address(
    pml4: *const PageTable,
    addr: VirtAddr,
) -> Option<PhysAddr> {
    if pml4.is_null() {
        return None;
    }

    // SAFETY: Verified by caller.
    let pml4_ref = unsafe { &*pml4 };
    let pml4e = pml4_ref[addr.pml4_index()];
    if !pml4e.flags().contains(PageTableFlags::PRESENT) {
        return None;
    }

    let pdpt = pml4e.addr().as_u64() as *const PageTable;
    // SAFETY: Dereferencing valid PDPT pointer in identity memory.
    let pdpt_ref = unsafe { &*pdpt };
    let pdpte = pdpt_ref[addr.pdpt_index()];
    if !pdpte.flags().contains(PageTableFlags::PRESENT) {
        return None;
    }

    // Check for 1 GiB huge page.
    if pdpte.flags().contains(PageTableFlags::HUGE_PAGE) {
        let base = pdpte.addr().as_u64();
        let offset = addr.as_u64() & 0x3FFF_FFFF;
        return Some(PhysAddr::new(base + offset));
    }

    let pd = pdpte.addr().as_u64() as *const PageTable;
    // SAFETY: Dereferencing valid PD pointer.
    let pd_ref = unsafe { &*pd };
    let pde = pd_ref[addr.pd_index()];
    if !pde.flags().contains(PageTableFlags::PRESENT) {
        return None;
    }

    // Check for 2 MiB huge page.
    if pde.flags().contains(PageTableFlags::HUGE_PAGE) {
        let base = pde.addr().as_u64();
        let offset = addr.as_u64() & 0x1F_FFFF;
        return Some(PhysAddr::new(base + offset));
    }

    let pt = pde.addr().as_u64() as *const PageTable;
    // SAFETY: Dereferencing valid PT pointer.
    let pt_ref = unsafe { &*pt };
    let pte = pt_ref[addr.pt_index()];
    if !pte.flags().contains(PageTableFlags::PRESENT) {
        return None;
    }

    Some(PhysAddr::new(pte.addr().as_u64() + addr.page_offset()))
}

/// Maps a 4 KiB virtual page to a physical frame in the specified PML4 table.
///
/// # Errors
///
/// Returns an error string if `pml4` is null or if frame allocation fails for an intermediate table.
///
/// # Safety
///
/// `pml4` must point to a valid mutable PML4 table. Frame allocator must be initialized.
#[allow(clippy::similar_names)]
pub unsafe fn map_page(
    pml4: *mut PageTable,
    virt: VirtAddr,
    phys: PhysAddr,
    flags: PageTableFlags,
    allocator: &mut BitmapFrameAllocator,
) -> Result<(), &'static str> {
    if pml4.is_null() {
        return Err("PML4 is null");
    }

    let pml4_ref = unsafe { &mut *pml4 };
    let pml4_idx = virt.pml4_index();

    // 1. Ensure PDPT is present.
    if !pml4_ref[pml4_idx].flags().contains(PageTableFlags::PRESENT) {
        let frame = allocator.allocate_frame().ok_or("Out of memory for PDPT")?;
        let table = frame.as_u64() as *mut PageTable;
        unsafe { (*table).zero() };
        pml4_ref[pml4_idx].set_addr_and_flags(
            frame,
            PageTableFlags::PRESENT.union(PageTableFlags::WRITABLE),
        );
    }

    let pdpt = pml4_ref[pml4_idx].addr().as_u64() as *mut PageTable;
    let pdpt_ref = unsafe { &mut *pdpt };
    let pdpt_idx = virt.pdpt_index();

    // 2. Ensure PD is present.
    if !pdpt_ref[pdpt_idx].flags().contains(PageTableFlags::PRESENT) {
        let frame = allocator.allocate_frame().ok_or("Out of memory for PD")?;
        let table = frame.as_u64() as *mut PageTable;
        unsafe { (*table).zero() };
        pdpt_ref[pdpt_idx].set_addr_and_flags(
            frame,
            PageTableFlags::PRESENT.union(PageTableFlags::WRITABLE),
        );
    }

    let pd = pdpt_ref[pdpt_idx].addr().as_u64() as *mut PageTable;
    let pd_ref = unsafe { &mut *pd };
    let pd_idx = virt.pd_index();

    // 3. Ensure PT is present.
    if !pd_ref[pd_idx].flags().contains(PageTableFlags::PRESENT) {
        let frame = allocator.allocate_frame().ok_or("Out of memory for PT")?;
        let table = frame.as_u64() as *mut PageTable;
        unsafe { (*table).zero() };
        pd_ref[pd_idx].set_addr_and_flags(
            frame,
            PageTableFlags::PRESENT.union(PageTableFlags::WRITABLE),
        );
    }

    let pt = pd_ref[pd_idx].addr().as_u64() as *mut PageTable;
    let pt_ref = unsafe { &mut *pt };
    let pt_idx = virt.pt_index();

    // 4. Map the physical page.
    pt_ref[pt_idx].set_addr_and_flags(phys, flags.union(PageTableFlags::PRESENT));
    invalidate_page(virt);

    Ok(())
}
