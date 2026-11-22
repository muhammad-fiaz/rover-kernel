//! # Scheduling and Multitasking Subsystem
//!
//! Kernel threads, Task Control Blocks (TCB), context switching, and round-robin scheduler.

pub mod context;
pub mod scheduler;
pub mod task;

pub use context::switch_context;
pub use scheduler::{SCHEDULER, Scheduler, TASK_STACK_SIZE, sched_yield, spawn_task};
pub use task::{CpuContext, Task, TaskId, TaskState};
