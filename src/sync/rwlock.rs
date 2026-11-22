//! # Reader-Writer Lock (`RwLock`)
//!
//! Allows multiple concurrent readers or an exclusive single writer.

use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicIsize, Ordering};

const WRITER_LOCKED: isize = -1;

/// A reader-writer lock synchronization primitive.
pub struct RwLock<T> {
    state: AtomicIsize, // > 0: reader count, 0: unlocked, -1: writer locked
    data: UnsafeCell<T>,
}

// SAFETY: Synchronization ensures valid borrow semantics across threads.
unsafe impl<T: Send + Sync> Sync for RwLock<T> {}
unsafe impl<T: Send> Send for RwLock<T> {}

impl<T> RwLock<T> {
    /// Creates a new reader-writer lock wrapping `data`.
    #[must_use]
    pub const fn new(data: T) -> Self {
        Self {
            state: AtomicIsize::new(0),
            data: UnsafeCell::new(data),
        }
    }

    /// Acquires shared read access, spinning until available.
    pub fn read(&self) -> RwLockReadGuard<'_, T> {
        loop {
            let current = self.state.load(Ordering::Relaxed);
            if current >= 0
                && self
                    .state
                    .compare_exchange_weak(
                        current,
                        current + 1,
                        Ordering::Acquire,
                        Ordering::Relaxed,
                    )
                    .is_ok()
            {
                return RwLockReadGuard { lock: self };
            }
            core::hint::spin_loop();
        }
    }

    /// Acquires exclusive write access, spinning until available.
    pub fn write(&self) -> RwLockWriteGuard<'_, T> {
        while self
            .state
            .compare_exchange_weak(0, WRITER_LOCKED, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            core::hint::spin_loop();
        }

        RwLockWriteGuard { lock: self }
    }

    fn release_read(&self) {
        self.state.fetch_sub(1, Ordering::Release);
    }

    fn release_write(&self) {
        self.state.store(0, Ordering::Release);
    }
}

/// RAII guard providing shared read access.
pub struct RwLockReadGuard<'a, T> {
    lock: &'a RwLock<T>,
}

impl<T> Deref for RwLockReadGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // SAFETY: Held shared read access.
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> Drop for RwLockReadGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.release_read();
    }
}

/// RAII guard providing exclusive write access.
pub struct RwLockWriteGuard<'a, T> {
    lock: &'a RwLock<T>,
}

impl<T> Deref for RwLockWriteGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // SAFETY: Held exclusive write access.
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> DerefMut for RwLockWriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: Held exclusive write access.
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T> Drop for RwLockWriteGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.release_write();
    }
}
