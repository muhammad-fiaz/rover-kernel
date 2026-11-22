//! # System Call Dispatcher
//!
//! Validates user parameters and dispatches requests to appropriate kernel subsystems.

use super::abi::{
    EBADF, EFAULT, ENOSYS, SYS_EXIT, SYS_GETPID, SYS_SUCCESS, SYS_TIME, SYS_WRITE, SYS_YIELD,
};
use crate::sched::sched_yield;
use crate::time::uptime_ms;

/// Main entry point for dispatching all system calls.
#[unsafe(no_mangle)]
#[allow(clippy::cast_possible_truncation)]
pub extern "C" fn handle_syscall_dispatch(num: u64, a1: u64, a2: u64, a3: u64) -> u64 {
    match num {
        SYS_YIELD => {
            sched_yield();
            SYS_SUCCESS
        }
        SYS_TIME => uptime_ms(),
        SYS_GETPID => 1,
        SYS_WRITE => {
            let fd = a1;
            let buf_ptr = a2 as *const u8;
            let len = a3 as usize;

            if fd != 1 && fd != 2 {
                return !EBADF + 1;
            }

            if buf_ptr.is_null() || len == 0 {
                return !EFAULT + 1;
            }

            // SAFETY: In early kernel, printing bytes to serial output.
            let slice = unsafe { core::slice::from_raw_parts(buf_ptr, len) };
            if let Ok(s) = core::str::from_utf8(slice) {
                crate::kprint!("{}", s);
                len as u64
            } else {
                !EFAULT + 1
            }
        }
        SYS_EXIT => {
            crate::kinfo!("Task requested exit (status code {})", a1);
            sched_yield();
            SYS_SUCCESS
        }
        _ => !ENOSYS + 1,
    }
}
