/**
 * mod.rs - Logs Module
 *
 * Execution log recording and rotation for Alou.
 */

pub mod types;
pub mod manager;

pub use types::{LogLevel, LogStats};
pub use manager::LogsManager;
