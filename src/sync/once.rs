//! # Bare-Metal `OnceLock` Cell
//!
//! One-time initialization primitive for global and static kernel structures.

use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicBool, Ordering};

/// A synchronization primitive which can be written to only once.
pub struct OnceLock<T> {
    initialized: AtomicBool,
    data: UnsafeCell<Option<T>>,
}

// SAFETY: Reading initialized OnceLock is safe across threads.
unsafe impl<T: Sync + Send> Sync for OnceLock<T> {}
unsafe impl<T: Send> Send for OnceLock<T> {}

impl<T> OnceLock<T> {
    /// Creates a new uninitialized [`OnceLock`].
    #[must_use]
    pub const fn new() -> Self {
        Self {
            initialized: AtomicBool::new(false),
            data: UnsafeCell::new(None),
        }
    }

    /// Sets the value of the cell if uninitialized.
    ///
    /// # Errors
    ///
    /// Returns the provided `value` if the cell was already initialized.
    pub fn set(&self, value: T) -> Result<(), T> {
        if self
            .initialized
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
        {
            // SAFETY: Verified exclusive one-time initialization.
            unsafe {
                *self.data.get() = Some(value);
            }
            Ok(())
        } else {
            Err(value)
        }
    }

    /// Returns a reference to the contained value if initialized.
    #[must_use]
    pub fn get(&self) -> Option<&T> {
        if self.initialized.load(Ordering::Acquire) {
            // SAFETY: Initialized and immutable.
            unsafe { (*self.data.get()).as_ref() }
        } else {
            None
        }
    }

    /// Gets the value or initializes it using the provided closure.
    pub fn get_or_init<F: FnOnce() -> T>(&self, init: F) -> &T {
        if let Some(val) = self.get() {
            return val;
        }

        let val = init();
        let _ = self.set(val);
        // SAFETY: Ensured initialized by set() or prior winner.
        self.get().unwrap_or_else(|| unreachable!())
    }
}

impl<T> Default for OnceLock<T> {
    fn default() -> Self {
        Self::new()
    }
}
