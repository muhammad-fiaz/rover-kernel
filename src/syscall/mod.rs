//! # System Call Subsystem
//!
//! Numeric ABI constants, x86-64 `syscall`/`sysretq` hardware registration,
//! parameter validation, and user/kernel dispatching.

pub mod abi;
pub mod dispatch;
pub mod entry;

pub use abi::{
    EBADF, EFAULT, ENOSYS, SYS_EXIT, SYS_GETPID, SYS_MMAP, SYS_READ, SYS_SUCCESS, SYS_TIME,
    SYS_WRITE, SYS_YIELD,
};
pub use dispatch::handle_syscall_dispatch;
pub use entry::init_syscall_architecture;
