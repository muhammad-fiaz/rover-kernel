//! # Diagnostics and Profiling Subsystem
//!
//! Subsystem diagnostics, memory tracking statistics, and CPU hardware specifications.

pub mod cpuinfo;
pub mod meminfo;

pub use cpuinfo::dump_cpu_diagnostics;
pub use meminfo::{MemoryInfo, dump_memory_diagnostics, get_memory_info};
