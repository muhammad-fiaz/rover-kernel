//! # Rover System Call ABI Definitions
//!
//! Numeric identifiers and return codes for the versioned Rover System Call ABI.

/// Relinquishes remaining CPU quantum.
pub const SYS_YIELD: u64 = 1;
/// Terminates calling task or process.
pub const SYS_EXIT: u64 = 2;
/// Writes bytes to a file descriptor.
pub const SYS_WRITE: u64 = 3;
/// Reads bytes from a file descriptor.
pub const SYS_READ: u64 = 4;
/// Returns current process ID.
pub const SYS_GETPID: u64 = 5;
/// Returns monotonic system uptime in milliseconds.
pub const SYS_TIME: u64 = 6;
/// Maps memory into process address space.
pub const SYS_MMAP: u64 = 7;

/// System call success return code.
pub const SYS_SUCCESS: u64 = 0;
/// Invalid system call number.
pub const ENOSYS: u64 = 38;
/// Invalid user pointer or out-of-bounds argument.
pub const EFAULT: u64 = 14;
/// Bad file descriptor.
pub const EBADF: u64 = 9;
