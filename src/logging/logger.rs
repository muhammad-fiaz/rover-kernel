//! # Kernel Logger Implementation
//!
//! Formatted console output, severity levels, and ANSI color codes.

use core::fmt;

/// Severity level of a kernel log message.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    /// Severe errors that may indicate subsystem failure or imminent panic.
    Error,
    /// Warning conditions that require administrator attention.
    Warn,
    /// Informational notices regarding normal kernel operations.
    Info,
    /// Detailed diagnostic information for kernel development.
    Debug,
    /// Extremely verbose execution traces.
    Trace,
}

impl LogLevel {
    /// Returns the short uppercase text representation of the log level.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Error => "ERROR",
            Self::Warn => "WARN ",
            Self::Info => "INFO ",
            Self::Debug => "DEBUG",
            Self::Trace => "TRACE",
        }
    }

    /// Returns the ANSI escape code for terminal color formatting.
    #[must_use]
    pub const fn ansi_color(self) -> &'static str {
        match self {
            Self::Error => "\x1b[1;31m", // Bright Red
            Self::Warn => "\x1b[1;33m",  // Bright Yellow
            Self::Info => "\x1b[1;32m",  // Bright Green
            Self::Debug => "\x1b[1;36m", // Bright Cyan
            Self::Trace => "\x1b[1;90m", // Bright Gray
        }
    }
}

/// ANSI escape sequence to reset terminal text formatting.
pub const ANSI_RESET: &str = "\x1b[0m";

/// Dispatches a formatted log entry to active kernel log outputs.
pub fn log_message(level: LogLevel, args: fmt::Arguments<'_>) {
    crate::drivers::serial::early_write_fmt(format_args!(
        "{}[{}] {}{}\n",
        level.ansi_color(),
        level.as_str(),
        args,
        ANSI_RESET
    ));
}
