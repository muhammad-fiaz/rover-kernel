//! # Logging Subsystem
//!
//! Exposes structured logging levels, ANSI terminal formatting, and diagnostic print macros.

pub mod logger;

pub use logger::{ANSI_RESET, LogLevel, log_message};

/// Formatted print without an automatic newline.
#[macro_export]
macro_rules! kprint {
    ($($arg:tt)*) => {
        $crate::drivers::serial::early_write_fmt(format_args!($($arg)*))
    };
}

/// Formatted print with an automatic trailing newline.
#[macro_export]
macro_rules! kprintln {
    () => {
        $crate::kprint!("\n")
    };
    ($($arg:tt)*) => {
        $crate::drivers::serial::early_write_fmt(format_args!("{}\n", format_args!($($arg)*)))
    };
}

/// Emits an ERROR-level log message.
#[macro_export]
macro_rules! kerror {
    ($($arg:tt)*) => {
        $crate::logging::log_message($crate::logging::LogLevel::Error, format_args!($($arg)*))
    };
}

/// Emits a WARN-level log message.
#[macro_export]
macro_rules! kwarn {
    ($($arg:tt)*) => {
        $crate::logging::log_message($crate::logging::LogLevel::Warn, format_args!($($arg)*))
    };
}

/// Emits an INFO-level log message.
#[macro_export]
macro_rules! kinfo {
    ($($arg:tt)*) => {
        $crate::logging::log_message($crate::logging::LogLevel::Info, format_args!($($arg)*))
    };
}

/// Emits a DEBUG-level log message.
#[macro_export]
macro_rules! kdebug {
    ($($arg:tt)*) => {
        $crate::logging::log_message($crate::logging::LogLevel::Debug, format_args!($($arg)*))
    };
}

/// Emits a TRACE-level log message.
#[macro_export]
macro_rules! ktrace {
    ($($arg:tt)*) => {
        $crate::logging::log_message($crate::logging::LogLevel::Trace, format_args!($($arg)*))
    };
}
