/**
 * types.rs - Logs Module Types
 *
 * Type definitions for log management.
 */

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

/// Log entry structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Local>,
    pub level: LogLevel,
    pub event: String,
    pub details: Option<String>,
}

/// Log level enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

impl LogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warning => "WARNING",
            LogLevel::Error => "ERROR",
            LogLevel::Critical => "CRITICAL",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "DEBUG" => LogLevel::Debug,
            "INFO" => LogLevel::Info,
            "WARNING" | "WARN" => LogLevel::Warning,
            "ERROR" => LogLevel::Error,
            "CRITICAL" | "FATAL" => LogLevel::Critical,
            _ => LogLevel::Info,
        }
    }

    pub fn to_emoji(&self) -> &'static str {
        match self {
            LogLevel::Debug => "🔍",
            LogLevel::Info => "ℹ️",
            LogLevel::Warning => "⚠️",
            LogLevel::Error => "❌",
            LogLevel::Critical => "🔥",
        }
    }
}

/// Log statistics structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogStats {
    pub total_lines: usize,
    pub debug_count: usize,
    pub info_count: usize,
    pub warning_count: usize,
    pub error_count: usize,
    pub critical_count: usize,
    pub file_size_bytes: usize,
}

impl LogStats {
    /// Get file size in KB
    pub fn file_size_kb(&self) -> f64 {
        self.file_size_bytes as f64 / 1024.0
    }

    /// Get file size in MB
    pub fn file_size_mb(&self) -> f64 {
        self.file_size_bytes as f64 / (1024.0 * 1024.0)
    }
}
