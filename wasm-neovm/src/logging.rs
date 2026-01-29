// Copyright (c) 2025 R3E Network
// Licensed under the MIT License

//! Standardized logging for wasm-neovm
//!
//! This module provides consistent log levels and formats across the codebase.

use std::fmt;
use std::str::FromStr;

/// Log levels for wasm-neovm operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LogLevel {
    /// Critical errors that prevent operation
    Error = 0,
    /// Warning conditions that don't prevent operation
    Warn = 1,
    /// General information about translation progress
    Info = 2,
    /// Detailed debugging information
    Debug = 3,
    /// Very detailed tracing information
    Trace = 4,
}

impl LogLevel {
    /// Get the string representation
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Error => "ERROR",
            Self::Warn => "WARN",
            Self::Info => "INFO",
            Self::Debug => "DEBUG",
            Self::Trace => "TRACE",
        }
    }

    /// Check if this level includes another level
    pub const fn includes(&self, other: LogLevel) -> bool {
        *self as i32 >= other as i32
    }
}

impl FromStr for LogLevel {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "error" => Ok(Self::Error),
            "warn" | "warning" => Ok(Self::Warn),
            "info" => Ok(Self::Info),
            "debug" => Ok(Self::Debug),
            "trace" => Ok(Self::Trace),
            _ => Err(()),
        }
    }
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl From<log::Level> for LogLevel {
    fn from(level: log::Level) -> Self {
        match level {
            log::Level::Error => Self::Error,
            log::Level::Warn => Self::Warn,
            log::Level::Info => Self::Info,
            log::Level::Debug => Self::Debug,
            log::Level::Trace => Self::Trace,
        }
    }
}

impl From<LogLevel> for log::Level {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Error => log::Level::Error,
            LogLevel::Warn => log::Level::Warn,
            LogLevel::Info => log::Level::Info,
            LogLevel::Debug => log::Level::Debug,
            LogLevel::Trace => log::Level::Trace,
        }
    }
}

/// Log event categories for consistent filtering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LogCategory {
    /// Translation process
    Translation,
    /// WASM parsing
    Parsing,
    /// Bytecode generation
    Codegen,
    /// Manifest operations
    Manifest,
    /// Metadata handling
    Metadata,
    /// Optimization passes
    Optimization,
    /// Runtime helpers
    Runtime,
    /// Cross-chain adapters
    Adapter,
    /// I/O operations
    Io,
    /// General operations
    General,
}

impl LogCategory {
    /// Get the string representation
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Translation => "TRANSLATE",
            Self::Parsing => "PARSE",
            Self::Codegen => "CODEGEN",
            Self::Manifest => "MANIFEST",
            Self::Metadata => "METADATA",
            Self::Optimization => "OPT",
            Self::Runtime => "RUNTIME",
            Self::Adapter => "ADAPTER",
            Self::Io => "IO",
            Self::General => "GENERAL",
        }
    }
}

impl fmt::Display for LogCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Standardized logging function
#[macro_export]
macro_rules! wlog {
    ($level:expr, $category:expr, $($arg:tt)*) => {
        log::log!(
            $crate::logging::LogLevel::from($level).into(),
            "[{}] {}",
            $crate::logging::LogCategory::from($category),
            format!($($arg)*)
        )
    };
}

/// Log a translation event at info level
#[macro_export]
macro_rules! log_translation {
    ($($arg:tt)*) => {
        $crate::wlog!($crate::logging::LogLevel::Info, $crate::logging::LogCategory::Translation, $($arg)*)
    };
}

/// Log a parsing event at debug level
#[macro_export]
macro_rules! log_parse {
    ($($arg:tt)*) => {
        $crate::wlog!($crate::logging::LogLevel::Debug, $crate::logging::LogCategory::Parsing, $($arg)*)
    };
}

/// Log a codegen event at debug level
#[macro_export]
macro_rules! log_codegen {
    ($($arg:tt)*) => {
        $crate::wlog!($crate::logging::LogLevel::Debug, $crate::logging::LogCategory::Codegen, $($arg)*)
    };
}

/// Log a manifest event at info level
#[macro_export]
macro_rules! log_manifest {
    ($($arg:tt)*) => {
        $crate::wlog!($crate::logging::LogLevel::Info, $crate::logging::LogCategory::Manifest, $($arg)*)
    };
}

/// Log an adapter event at debug level
#[macro_export]
macro_rules! log_adapter {
    ($($arg:tt)*) => {
        $crate::wlog!($crate::logging::LogLevel::Debug, $crate::logging::LogCategory::Adapter, $($arg)*)
    };
}

/// Log a runtime event at trace level
#[macro_export]
macro_rules! log_runtime {
    ($($arg:tt)*) => {
        $crate::wlog!($crate::logging::LogLevel::Trace, $crate::logging::LogCategory::Runtime, $($arg)*)
    };
}

/// Initialize logging with the given level
pub fn init_logging(level: LogLevel) {
    let filter = match level {
        LogLevel::Error => "error",
        LogLevel::Warn => "warn",
        LogLevel::Info => "info",
        LogLevel::Debug => "debug",
        LogLevel::Trace => "trace",
    };

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(filter))
        .format_timestamp_millis()
        .init();
}

/// Initialize logging from environment
pub fn init_logging_from_env() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_ordering() {
        assert!(LogLevel::Error < LogLevel::Warn);
        assert!(LogLevel::Warn < LogLevel::Info);
        assert!(LogLevel::Info < LogLevel::Debug);
        assert!(LogLevel::Debug < LogLevel::Trace);
    }

    #[test]
    fn test_log_level_includes() {
        let info = LogLevel::Info;
        assert!(info.includes(LogLevel::Error));
        assert!(info.includes(LogLevel::Warn));
        assert!(info.includes(LogLevel::Info));
        assert!(!info.includes(LogLevel::Debug));
    }

    #[test]
    fn test_log_level_from_str() {
        assert_eq!(LogLevel::from_str("error"), Ok(LogLevel::Error));
        assert_eq!(LogLevel::from_str("warn"), Ok(LogLevel::Warn));
        assert_eq!(LogLevel::from_str("INFO"), Ok(LogLevel::Info));
        assert_eq!(LogLevel::from_str("Debug"), Ok(LogLevel::Debug));
        assert_eq!(LogLevel::from_str("trace"), Ok(LogLevel::Trace));
        assert_eq!(LogLevel::from_str("invalid"), Err(()));
    }

    #[test]
    fn test_log_category_display() {
        assert_eq!(LogCategory::Translation.to_string(), "TRANSLATE");
        assert_eq!(LogCategory::Parsing.to_string(), "PARSE");
    }

    #[test]
    fn test_log_level_conversion() {
        let log_level: log::Level = LogLevel::Debug.into();
        assert_eq!(log_level, log::Level::Debug);

        let our_level: LogLevel = log::Level::Warn.into();
        assert_eq!(our_level, LogLevel::Warn);
    }
}
