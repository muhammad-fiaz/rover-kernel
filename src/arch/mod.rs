//! # Architecture Abstraction Layer
//!
//! Provides hardware-specific primitives, CPU control, descriptor tables,
//! and low-level execution context handling across supported processor architectures.

#[cfg(target_arch = "aarch64")]
pub mod aarch64;
#[cfg(target_arch = "arm")]
pub mod arm32;
#[cfg(not(any(
    target_arch = "x86_64",
    target_arch = "aarch64",
    target_arch = "riscv64",
    target_arch = "x86",
    target_arch = "arm"
)))]
pub mod generic;
#[cfg(target_arch = "x86")]
pub mod ia32;
#[cfg(target_arch = "riscv64")]
pub mod riscv64;
#[cfg(target_arch = "x86_64")]
pub mod x86_64;

#[cfg(target_arch = "aarch64")]
pub use aarch64 as current;
#[cfg(target_arch = "arm")]
pub use arm32 as current;
#[cfg(not(any(
    target_arch = "x86_64",
    target_arch = "aarch64",
    target_arch = "riscv64",
    target_arch = "x86",
    target_arch = "arm"
)))]
pub use generic as current;
#[cfg(target_arch = "x86")]
pub use ia32 as current;
#[cfg(target_arch = "riscv64")]
pub use riscv64 as current;
#[cfg(target_arch = "x86_64")]
pub use x86_64 as current;
