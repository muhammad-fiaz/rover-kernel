//! # Preemptive Kernel Scheduler
//!
//! Round-robin task scheduling, task creation, context switching, and yield operations.

extern crate alloc;
use alloc::boxed::Box;
use alloc::vec::Vec;

use super::context::switch_context;
use super::task::{Task, TaskId, TaskState};
use crate::sync::SpinLock;

/// Stack size allocated for each kernel task (64 KiB).
pub const TASK_STACK_SIZE: usize = 65_536;

/// Round-robin kernel scheduler managing the task run-queue.
pub struct Scheduler {
    tasks: Vec<Task>,
    current_idx: usize,
    initialized: bool,
}

impl Scheduler {
    /// Creates an empty, uninitialized scheduler.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            tasks: Vec::new(),
            current_idx: 0,
            initialized: false,
        }
    }

    /// Initializes the scheduler, registering the currently executing kernel thread as Task 0.
    pub fn init(&mut self) {
        if self.initialized {
            return;
        }

        let main_task = Task {
            id: TaskId(0),
            state: TaskState::Running,
            saved_rsp: core::ptr::null_mut(),
            stack_base: core::ptr::null_mut(),
            stack_size: 0,
            name: *b"kernel_main\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
        };

        self.tasks.push(main_task);
        self.current_idx = 0;
        self.initialized = true;
    }

    /// Spawns a new kernel thread with its own dedicated 64 KiB stack.
    pub fn spawn(&mut self, name: &str, entry: extern "C" fn() -> !) -> TaskId {
        let task_id = TaskId::generate();
        let stack_box = alloc::vec![0u8; TASK_STACK_SIZE].into_boxed_slice();
        let stack_ptr = Box::into_raw(stack_box).cast::<u8>();

        let task = unsafe { Task::new(task_id, name, stack_ptr, TASK_STACK_SIZE, entry) };

        self.tasks.push(task);
        task_id
    }

    /// Yields CPU execution to the next ready task in the run-queue.
    pub fn yield_now(&mut self) {
        if self.tasks.len() <= 1 {
            return;
        }

        let total_tasks = self.tasks.len();
        let current_task_idx = self.current_idx;

        // Find next task in Ready state.
        let mut next_task_idx = (current_task_idx + 1) % total_tasks;
        while next_task_idx != current_task_idx {
            if self.tasks[next_task_idx].state == TaskState::Ready {
                break;
            }
            next_task_idx = (next_task_idx + 1) % total_tasks;
        }

        if next_task_idx == current_task_idx || self.tasks[next_task_idx].state != TaskState::Ready
        {
            return;
        }

        self.tasks[current_task_idx].state = TaskState::Ready;
        self.tasks[next_task_idx].state = TaskState::Running;
        self.current_idx = next_task_idx;

        let prev_rsp_ptr = core::ptr::addr_of_mut!(self.tasks[current_task_idx].saved_rsp);
        let next_rsp = self.tasks[next_task_idx].saved_rsp;

        // SAFETY: Both stack frames are valid and mapped.
        unsafe {
            switch_context(prev_rsp_ptr, next_rsp);
        }
    }

    /// Returns the ID of the currently executing task, or `None` if uninitialized.
    #[must_use]
    pub fn current_task_id(&self) -> Option<TaskId> {
        self.tasks.get(self.current_idx).map(|t| t.id)
    }

    /// Returns the total number of tasks currently registered in the scheduler.
    #[must_use]
    pub fn task_count(&self) -> usize {
        self.tasks.len()
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

/// Global thread-safe kernel scheduler instance.
pub static SCHEDULER: SpinLock<Scheduler> = SpinLock::new(Scheduler::new());

/// Spawns a new kernel thread via the global [`SCHEDULER`].
pub fn spawn_task(name: &str, entry: extern "C" fn() -> !) -> TaskId {
    SCHEDULER.lock().spawn(name, entry)
}

/// Relinquishes the CPU and schedules the next ready task.
pub fn sched_yield() {
    SCHEDULER.lock().yield_now();
}
