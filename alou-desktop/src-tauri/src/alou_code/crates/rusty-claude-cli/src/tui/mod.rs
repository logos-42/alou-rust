//! Terminal UI components for alou-code
//!
//! This module provides the TUI infrastructure for the interactive REPL,
//! including line editing, rendering, and session management.

pub mod app;
pub mod components;
pub mod layout;
pub mod state;
pub mod streaming_reporter;

// Re-export commonly used types
pub use app::TuiApp;
pub use components::status::StatusBar;
pub use state::{SharedTuiState, StreamingState, TokenRates, TokenStats, TuiState};
