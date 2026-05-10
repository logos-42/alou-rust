//! Streaming event reporter for real-time TUI updates.
//!
//! This module provides the `StreamingEventReporter` trait and `TuiStreamingReporter`
//! implementation that allows the TUI components to be updated in real-time as
//! streaming events arrive from the API.

use std::sync::Arc;

use crate::tui::state::{SharedTuiState, TokenRates};
use runtime::TokenUsage;

/// Trait for receiving streaming events in real-time.
///
/// Implement this trait to receive callbacks as streaming events arrive
/// during a turn. This allows TUI components to be updated immediately
/// rather than waiting for the turn to complete.
pub trait StreamingEventReporter: Send + Sync {
    /// Called when a thinking delta is received.
    fn on_thinking_delta(&self, delta: &str);

    /// Called when thinking block is complete.
    fn on_thinking_complete(&self);

    /// Called when a thinking block is redacted by the provider.
    fn on_thinking_redacted(&self);

    /// Called when a text delta is received.
    fn on_text_delta(&self, text: &str);

    /// Called when a usage update is received.
    fn on_usage_update(&self, usage: TokenUsage);

    /// Called when content block starts (e.g., "tool_use", "text", "thinking").
    fn on_content_block_start(&self, block_type: &str);
}

/// A streaming reporter that updates TuiState via SharedTuiState.
///
/// This implementation updates the TuiState as streaming events arrive,
/// providing real-time feedback to the user via the TUI.
pub struct TuiStreamingReporter {
    state: SharedTuiState,
    token_rates: TokenRates,
}

impl TuiStreamingReporter {
    /// Create a new TuiStreamingReporter.
    pub fn new(state: SharedTuiState) -> Self {
        Self {
            state,
            token_rates: TokenRates::default(),
        }
    }

    /// Get a clone of the inner state (for rendering).
    pub fn state(&self) -> SharedTuiState {
        Arc::clone(&self.state)
    }
}

impl StreamingEventReporter for TuiStreamingReporter {
    fn on_thinking_delta(&self, delta: &str) {
        if let Ok(mut state) = self.state.lock() {
            state.append_streaming(delta);
            let content = state.streaming_content().to_string();
            if !content.is_empty() {
                state.set_thinking(true, Some(content));
            }
        }
    }

    fn on_thinking_complete(&self) {
        if let Ok(mut state) = self.state.lock() {
            state.finish_streaming();
            state.set_thinking(false, None);
        }
    }

    fn on_thinking_redacted(&self) {
        if let Ok(mut state) = self.state.lock() {
            state.finish_streaming();
            state.set_thinking(false, None);
        }
    }

    fn on_text_delta(&self, text: &str) {
        if let Ok(mut state) = self.state.lock() {
            state.append_streaming(text);
        }
    }

    fn on_usage_update(&self, usage: TokenUsage) {
        if let Ok(mut state) = self.state.lock() {
            state.update_tokens(usage.input_tokens, usage.output_tokens);
        }
    }

    fn on_content_block_start(&self, block_type: &str) {
        if let Ok(mut state) = self.state.lock() {
            match block_type {
                "thinking" => {
                    state.set_thinking(true, None);
                    state.start_streaming();
                }
                "text" => {
                    if state.is_thinking_visible() {
                        state.set_thinking(false, None);
                    }
                    if !state.is_streaming() {
                        state.start_streaming();
                    }
                }
                _ => {}
            }
        }
    }
}

/// A no-op reporter for non-TUI modes.
pub struct NoopReporter;

impl StreamingEventReporter for NoopReporter {
    fn on_thinking_delta(&self, _delta: &str) {}
    fn on_thinking_complete(&self) {}
    fn on_thinking_redacted(&self) {}
    fn on_text_delta(&self, _text: &str) {}
    fn on_usage_update(&self, _usage: TokenUsage) {}
    fn on_content_block_start(&self, _block_type: &str) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noop_reporter_works() {
        let reporter = NoopReporter;
        reporter.on_thinking_delta("test");
        reporter.on_thinking_complete();
        reporter.on_usage_update(TokenUsage {
            input_tokens: 100,
            output_tokens: 50,
            cache_creation_input_tokens: 0,
            cache_read_input_tokens: 0,
        });
    }

    #[test]
    fn tui_streaming_reporter_creation() {
        let state = Arc::new(Mutex::new(crate::tui::TuiState::new()));
        let reporter = TuiStreamingReporter::new(state);
        assert!(reporter.state().lock().is_ok());
    }
}
