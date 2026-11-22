//! # Synchronization Subsystem
//!
//! Exposes spinlocks, interrupt-safe spinlocks, cooperative mutexes,
//! reader-writer locks, and bare-metal `OnceLock`.

pub mod mutex;
pub mod once;
pub mod rwlock;
pub mod spinlock;

pub use mutex::{Mutex, MutexGuard};
pub use once::OnceLock;
pub use rwlock::{RwLock, RwLockReadGuard, RwLockWriteGuard};
pub use spinlock::{IrqSafeSpinLock, IrqSafeSpinLockGuard, SpinLock, SpinLockGuard};
