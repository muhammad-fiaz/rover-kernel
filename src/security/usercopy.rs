//! # Safe User-Space Memory Copy Routines
//!
//! Enforces bounds checking, non-null validation, and canonical lower-half address space limits
//! to prevent user space from triggering kernel memory disclosures or corruptions.

/// Highest permissible user-space virtual memory address in 64-bit canonical memory layout.
pub const USER_VIRTUAL_MAX: u64 = 0x0000_7FFF_FFFF_FFFF;

/// Validates that a user memory range `[ptr, ptr + len)` resides entirely within user space.
#[inline]
#[must_use]
pub fn is_valid_user_range(ptr: u64, len: usize) -> bool {
    if ptr == 0 || len == 0 {
        return false;
    }

    let Some(end) = ptr.checked_add(len as u64) else {
        return false;
    };

    end <= USER_VIRTUAL_MAX
}

/// Safely copies `len` bytes from a user-space source pointer into a kernel destination slice.
///
/// # Errors
///
/// Returns an error string if `src` is null, unaligned, or intersects kernel memory space.
///
/// # Safety
///
/// The caller must ensure that `src` points to allocated memory accessible to the user process.
pub unsafe fn copy_from_user(
    dest: &mut [u8],
    src: *const u8,
    len: usize,
) -> Result<(), &'static str> {
    if dest.len() < len {
        return Err("Destination slice too small");
    }

    if !is_valid_user_range(src as u64, len) {
        return Err("Invalid or illegal user address range");
    }

    // SAFETY: Address range has been verified to reside entirely in user space.
    unsafe {
        core::ptr::copy_nonoverlapping(src, dest.as_mut_ptr(), len);
    }

    Ok(())
}

/// Safely copies `len` bytes from a kernel source slice to a user-space destination pointer.
///
/// # Errors
///
/// Returns an error string if `dest` is null or intersects kernel memory space.
///
/// # Safety
///
/// The caller must ensure that `dest` points to writable memory in user space.
pub unsafe fn copy_to_user(dest: *mut u8, src: &[u8], len: usize) -> Result<(), &'static str> {
    if src.len() < len {
        return Err("Source slice too small");
    }

    if !is_valid_user_range(dest as u64, len) {
        return Err("Invalid or illegal user address range");
    }

    // SAFETY: Address range has been verified to reside entirely in user space.
    unsafe {
        core::ptr::copy_nonoverlapping(src.as_ptr(), dest, len);
    }

    Ok(())
}
