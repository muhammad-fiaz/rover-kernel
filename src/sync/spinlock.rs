//! # Kernel spinlock implementations
//!
//! Provides a standard test-and-set spinlock and an interrupt-safe variant
//! that disables hardware interrupts while the critical section is active.

use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, Ordering};

/// A low-level mutual exclusion primitive that spins in a busy-wait loop.
pub struct SpinLock<T> {
    locked: AtomicBool,
    data: UnsafeCell<T>,
}

// SAFETY: SpinLock synchronizes access to T across threads/cores.
unsafe impl<T: Send> Sync for SpinLock<T> {}
unsafe impl<T: Send> Send for SpinLock<T> {}

impl<T> SpinLock<T> {
    /// Creates a new unlocked spinlock wrapping the supplied value.
    #[must_use]
    pub const fn new(data: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }

    /// Acquires the spinlock, spinning until the lock becomes available.
    pub fn lock(&self) -> SpinLockGuard<'_, T> {
        while self
            .locked
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            while self.locked.load(Ordering::Relaxed) {
                core::hint::spin_loop();
            }
        }

        SpinLockGuard { lock: self }
    }

    /// Attempts to acquire the spinlock without blocking.
    pub fn try_lock(&self) -> Option<SpinLockGuard<'_, T>> {
        if self
            .locked
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
        {
            Some(SpinLockGuard { lock: self })
        } else {
            None
        }
    }

    /// Returns `true` if the spinlock is currently held.
    #[must_use]
    pub fn is_locked(&self) -> bool {
        self.locked.load(Ordering::Relaxed)
    }

    /// Releases the spinlock.
    fn unlock(&self) {
        self.locked.store(false, Ordering::Release);
    }
}

/// An RAII guard providing exclusive mutable access to the protected value.
pub struct SpinLockGuard<'a, T> {
    lock: &'a SpinLock<T>,
}

impl<T> Deref for SpinLockGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // SAFETY: The lock is held exclusively by this guard.
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> DerefMut for SpinLockGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: The lock is held exclusively by this guard.
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T> Drop for SpinLockGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.unlock();
    }
}

/// An interrupt-safe spinlock that disables hardware interrupts while held.
pub struct IrqSafeSpinLock<T> {
    inner: SpinLock<T>,
}

// SAFETY: IrqSafeSpinLock delegates synchronization to SpinLock.
unsafe impl<T: Send> Sync for IrqSafeSpinLock<T> {}
unsafe impl<T: Send> Send for IrqSafeSpinLock<T> {}

impl<T> IrqSafeSpinLock<T> {
    /// Creates a new interrupt-safe spinlock wrapping the supplied value.
    #[must_use]
    pub const fn new(data: T) -> Self {
        Self {
            inner: SpinLock::new(data),
        }
    }

    /// Acquires the lock while disabling hardware interrupts.
    pub fn lock(&self) -> IrqSafeSpinLockGuard<'_, T> {
        let rflags = save_and_disable_interrupts();
        let guard = self.inner.lock();
        IrqSafeSpinLockGuard {
            guard,
            saved_rflags: rflags,
        }
    }
}

/// RAII guard for [`IrqSafeSpinLock`].
pub struct IrqSafeSpinLockGuard<'a, T> {
    guard: SpinLockGuard<'a, T>,
    saved_rflags: u64,
}

impl<T> Deref for IrqSafeSpinLockGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.guard
    }
}

impl<T> DerefMut for IrqSafeSpinLockGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.guard
    }
}

impl<T> Drop for IrqSafeSpinLockGuard<'_, T> {
    fn drop(&mut self) {
        restore_interrupts(self.saved_rflags);
    }
}

#[inline]
fn save_and_disable_interrupts() -> u64 {
    #[cfg(target_arch = "x86_64")]
    {
        let rflags: u64;
        // SAFETY: Reading RFLAGS and disabling interrupts.
        unsafe {
            core::arch::asm!(
                "pushfq",
                "pop {0}",
                "cli",
                out(reg) rflags,
                options(nomem, preserves_flags)
            );
        }
        rflags
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        0
    }
}

#[inline]
fn restore_interrupts(saved_rflags: u64) {
    #[cfg(target_arch = "x86_64")]
    {
        // Bit 9 in RFLAGS is the IF (Interrupt Flag).
        if (saved_rflags & (1 << 9)) != 0 {
            // SAFETY: Re-enabling interrupts because they were previously enabled.
            unsafe {
                core::arch::asm!("sti", options(nomem, preserves_flags));
            }
        }
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        let _ = saved_rflags;
    }
}
