//! # Kernel Mutex Primitive
//!
//! A mutual exclusion primitive that yields the processor when contested.

use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, Ordering};

/// A cooperative, yield-based mutual exclusion primitive.
pub struct Mutex<T> {
    locked: AtomicBool,
    data: UnsafeCell<T>,
}

// SAFETY: Mutex provides safe synchronization across threads.
unsafe impl<T: Send> Sync for Mutex<T> {}
unsafe impl<T: Send> Send for Mutex<T> {}

impl<T> Mutex<T> {
    /// Creates a new unlocked mutex wrapping `data`.
    #[must_use]
    pub const fn new(data: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }

    /// Acquires the mutex, yielding execution if contested.
    pub fn lock(&self) -> MutexGuard<'_, T> {
        while self
            .locked
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            crate::sched::sched_yield();
        }

        MutexGuard { lock: self }
    }

    /// Attempts to acquire the mutex without yielding.
    pub fn try_lock(&self) -> Option<MutexGuard<'_, T>> {
        if self
            .locked
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
        {
            Some(MutexGuard { lock: self })
        } else {
            None
        }
    }

    /// Returns `true` if the mutex is currently locked.
    #[must_use]
    pub fn is_locked(&self) -> bool {
        self.locked.load(Ordering::Relaxed)
    }

    fn unlock(&self) {
        self.locked.store(false, Ordering::Release);
    }
}

/// An RAII guard providing exclusive mutable access to the mutex's protected data.
pub struct MutexGuard<'a, T> {
    lock: &'a Mutex<T>,
}

impl<T> Deref for MutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // SAFETY: The mutex is held exclusively.
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: The mutex is held exclusively.
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.unlock();
    }
}
