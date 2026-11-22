//! # Physical and Virtual Memory Address Types
//!
//! Provides type-safe wrappers for 64-bit physical and virtual memory addresses,
//! page alignment calculations, and page-table indexing.

use core::fmt;

/// Standard page size in bytes (4 KiB).
pub const PAGE_SIZE: u64 = 4096;

/// A 64-bit physical memory address.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct PhysAddr(pub u64);

impl PhysAddr {
    /// Constructs a new physical address.
    #[inline]
    #[must_use]
    pub const fn new(addr: u64) -> Self {
        Self(addr)
    }

    /// Returns the raw 64-bit address value.
    #[inline]
    #[must_use]
    pub const fn as_u64(self) -> u64 {
        self.0
    }

    /// Returns `true` if the physical address is aligned to the specified byte boundary.
    #[inline]
    #[must_use]
    pub const fn is_aligned(self, align: u64) -> bool {
        if align == 0 {
            return false;
        }
        (self.0 % align) == 0
    }

    /// Aligns the physical address downwards to the nearest multiple of `align`.
    #[inline]
    #[must_use]
    pub const fn align_down(self, align: u64) -> Self {
        if align == 0 {
            return self;
        }
        Self(self.0 - (self.0 % align))
    }

    /// Aligns the physical address upwards to the nearest multiple of `align`.
    #[inline]
    #[must_use]
    pub const fn align_up(self, align: u64) -> Self {
        if align == 0 {
            return self;
        }
        let remainder = self.0 % align;
        if remainder == 0 {
            self
        } else {
            Self(self.0 + (align - remainder))
        }
    }
}

impl fmt::Debug for PhysAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PhysAddr(0x{:016x})", self.0)
    }
}

impl fmt::Display for PhysAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:016x}", self.0)
    }
}

/// A 64-bit virtual memory address.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct VirtAddr(pub u64);

impl VirtAddr {
    /// Constructs a new virtual address, ensuring canonical form sign-extension.
    #[inline]
    #[must_use]
    pub const fn new(addr: u64) -> Self {
        // x86-64 canonical address sign-extension: bit 47 must be extended through bit 63.
        let sign_extended = if (addr & (1 << 47)) != 0 {
            addr | 0xFFFF_0000_0000_0000
        } else {
            addr & 0x0000_FFFF_FFFF_FFFF
        };
        Self(sign_extended)
    }

    /// Returns the raw 64-bit address value.
    #[inline]
    #[must_use]
    pub const fn as_u64(self) -> u64 {
        self.0
    }

    /// Converts the virtual address to a raw constant pointer.
    #[inline]
    #[must_use]
    pub const fn as_ptr<T>(self) -> *const T {
        self.0 as *const T
    }

    /// Converts the virtual address to a raw mutable pointer.
    #[inline]
    #[must_use]
    pub const fn as_mut_ptr<T>(self) -> *mut T {
        self.0 as *mut T
    }

    /// Returns `true` if the virtual address is aligned to the specified byte boundary.
    #[inline]
    #[must_use]
    pub const fn is_aligned(self, align: u64) -> bool {
        if align == 0 {
            return false;
        }
        (self.0 % align) == 0
    }

    /// Aligns the virtual address downwards to the nearest multiple of `align`.
    #[inline]
    #[must_use]
    pub const fn align_down(self, align: u64) -> Self {
        if align == 0 {
            return self;
        }
        Self(self.0 - (self.0 % align))
    }

    /// Aligns the virtual address upwards to the nearest multiple of `align`.
    #[inline]
    #[must_use]
    pub const fn align_up(self, align: u64) -> Self {
        if align == 0 {
            return self;
        }
        let remainder = self.0 % align;
        if remainder == 0 {
            self
        } else {
            Self(self.0 + (align - remainder))
        }
    }

    /// Returns the 9-bit Level 4 (PML4) page table index (bits 39..47).
    #[inline]
    #[must_use]
    pub const fn pml4_index(self) -> usize {
        ((self.0 >> 39) & 0x1FF) as usize
    }

    /// Returns the 9-bit Level 3 (PDPT) page directory pointer index (bits 30..38).
    #[inline]
    #[must_use]
    pub const fn pdpt_index(self) -> usize {
        ((self.0 >> 30) & 0x1FF) as usize
    }

    /// Returns the 9-bit Level 2 (PD) page directory index (bits 21..29).
    #[inline]
    #[must_use]
    pub const fn pd_index(self) -> usize {
        ((self.0 >> 21) & 0x1FF) as usize
    }

    /// Returns the 9-bit Level 1 (PT) page table index (bits 12..20).
    #[inline]
    #[must_use]
    pub const fn pt_index(self) -> usize {
        ((self.0 >> 12) & 0x1FF) as usize
    }

    /// Returns the 12-bit physical page offset (bits 0..11).
    #[inline]
    #[must_use]
    pub const fn page_offset(self) -> u64 {
        self.0 & 0xFFF
    }
}

impl fmt::Debug for VirtAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "VirtAddr(0x{:016x})", self.0)
    }
}

impl fmt::Display for VirtAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:016x}", self.0)
    }
}
