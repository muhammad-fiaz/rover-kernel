//! # Task Control Block (TCB)
//!
//! Kernel thread representations, task identifiers, states, and execution contexts.

use core::sync::atomic::{AtomicU64, Ordering};

/// Monotonically increasing task identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TaskId(pub u64);

static NEXT_TASK_ID: AtomicU64 = AtomicU64::new(1);

impl TaskId {
    /// Generates a new unique task identifier.
    #[must_use]
    pub fn generate() -> Self {
        Self(NEXT_TASK_ID.fetch_add(1, Ordering::Relaxed))
    }
}

/// Lifecycle states of a kernel task.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskState {
    /// Task is ready to be scheduled and run on a CPU.
    Ready,
    /// Task is currently actively executing on a CPU.
    Running,
    /// Task is blocked waiting for an event, timer, or lock.
    Blocked,
    /// Task has completed execution and its resources can be reclaimed.
    Terminated,
}

/// Callee-saved register context preserved across context switches.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct CpuContext {
    /// RFLAGS register value.
    pub rflags: u64,
    /// R15 register.
    pub r15: u64,
    /// R14 register.
    pub r14: u64,
    /// R13 register.
    pub r13: u64,
    /// R12 register.
    pub r12: u64,
    /// RBP frame pointer.
    pub rbp: u64,
    /// RBX general register.
    pub rbx: u64,
    /// Return address / Instruction pointer (RIP).
    pub rip: u64,
}

/// A kernel thread / task control block.
pub struct Task {
    /// Unique task ID.
    pub id: TaskId,
    /// Current execution state.
    pub state: TaskState,
    /// Current saved stack pointer pointing to the top of the suspended stack frame.
    pub saved_rsp: *mut u8,
    /// Stack memory buffer base pointer.
    pub stack_base: *mut u8,
    /// Stack size in bytes.
    pub stack_size: usize,
    /// Human-readable task name.
    pub name: [u8; 32],
}

// SAFETY: Task stacks and execution contexts are transferred exclusively between cores.
unsafe impl Send for Task {}

impl Task {
    /// Creates a new kernel task with an allocated stack and sets up the initial stack frame.
    ///
    /// # Safety
    ///
    /// `stack_base` must point to valid writable memory of at least `stack_size` bytes.
    pub unsafe fn new(
        id: TaskId,
        name: &str,
        stack_base: *mut u8,
        stack_size: usize,
        entry: extern "C" fn() -> !,
    ) -> Self {
        let mut name_buf = [0u8; 32];
        let bytes = name.as_bytes();
        let copy_len = bytes.len().min(31);
        name_buf[..copy_len].copy_from_slice(&bytes[..copy_len]);

        // Stack grows downwards on x86-64. Align top to 16 bytes.
        let stack_top = ((stack_base as usize) + stack_size) & !15;

        // Place initial CpuContext at the top of the stack.
        let context_ptr = (stack_top - core::mem::size_of::<CpuContext>()) as *mut CpuContext;
        unsafe {
            core::ptr::write(
                context_ptr,
                CpuContext {
                    rflags: 0x202, // IF (Interrupt Flag) set, reserved bit 1 set
                    r15: 0,
                    r14: 0,
                    r13: 0,
                    r12: 0,
                    rbp: 0,
                    rbx: 0,
                    rip: entry as usize as u64,
                },
            );
        }

        Self {
            id,
            state: TaskState::Ready,
            saved_rsp: context_ptr.cast::<u8>(),
            stack_base,
            stack_size,
            name: name_buf,
        }
    }

    /// Returns the human-readable task name as a string slice.
    #[must_use]
    pub fn name_str(&self) -> &str {
        core::str::from_utf8(&self.name)
            .unwrap_or("unknown")
            .trim_matches('\0')
    }
}
