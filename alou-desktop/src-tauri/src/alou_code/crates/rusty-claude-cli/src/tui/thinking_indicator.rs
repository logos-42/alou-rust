//! Thinking / Reasoning indicator for streaming assistant responses.
//!
//! Shows an animated indicator when the model is thinking/reasoning, including
//! support for extended thinking (thinking block with character count), redacted
//! thinking, and live text delta streaming.
//!
//! ## Persistence
//!
//! The `ThinkingState` struct captures the state of an active thinking session
//! and can be serialized to JSON for persistence across turns/sessions.

use std::fmt::Write;
use std::io;
use std::time::{Duration, Instant};

use crossterm::cursor::{MoveToColumn, RestorePosition, SavePosition};
use crossterm::style::{Color, Print, ResetColor, SetForegroundColor};
use crossterm::terminal::{Clear, ClearType};
use crossterm::{execute, queue};

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Thinking phase
// ---------------------------------------------------------------------------

/// Phases of the thinking/reasoning indicator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThinkingPhase {
    /// No thinking in progress — indicator is hidden.
    Idle,
    /// Extended thinking block detected (model has a thinking block).
    Thinking,
    /// Redacted thinking block — model collapsed the thinking.
    Redacted,
    /// Live text delta streaming (model is producing text).
    Streaming,
    /// Thinking completed and flushed.
    Done,
}

// ---------------------------------------------------------------------------
// Thinking state (persisted)
// ---------------------------------------------------------------------------

/// Captures the state of an active thinking session for persistence.
///
/// Serialized to JSON and stored in the session so that when a session is
/// resumed, the thinking state can be restored (showing "thinking" or "done"
/// based on whether the previous turn had thinking).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThinkingState {
    /// Current phase of the thinking indicator.
    pub phase: ThinkingPhase,
    /// Character count of the thinking block (if known).
    pub thinking_char_count: Option<usize>,
    /// Whether the thinking was redacted.
    pub is_redacted: bool,
    /// Whether the thinking was flushed/completed.
    pub is_flushed: bool,
    /// Timestamp (Unix ms) when thinking started.
    pub started_at_ms: u64,
    /// Timestamp (Unix ms) when thinking ended (if completed).
    pub ended_at_ms: Option<u64>,
    /// Number of thinking ticks/frames rendered.
    pub tick_count: u32,
}

impl ThinkingState {
    /// Create a new thinking state with the given phase.
    pub fn new(phase: ThinkingPhase) -> Self {
        Self {
            phase,
            thinking_char_count: None,
            is_redacted: false,
            is_flushed: false,
            started_at_ms: current_time_millis(),
            ended_at_ms: None,
            tick_count: 0,
        }
    }

    /// Mark thinking as redacted.
    pub fn with_redacted(mut self) -> Self {
        self.is_redacted = true;
        self
    }

    /// Mark thinking as flushed/done.
    pub fn finish(&mut self) {
        self.phase = ThinkingPhase::Done;
        self.is_flushed = true;
        self.ended_at_ms = Some(current_time_millis());
    }

    /// Serialize to JSON string for session persistence.
    pub fn to_json_string(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| r#"{"phase":"idle"}"#.to_string())
    }

    /// Deserialize from JSON string.
    pub fn from_json_string(json: &str) -> Option<Self> {
        serde_json::from_str(json).ok()
    }

    /// Returns a human-readable label for the current phase.
    pub fn label(&self) -> &'static str {
        match self.phase {
            ThinkingPhase::Idle => "",
            ThinkingPhase::Thinking => {
                if let Some(n) = self.thinking_char_count {
                    if n > 0 {
                        return "🧠 reasoning…";
                    }
                }
                "🧠 reasoning…"
            }
            ThinkingPhase::Redacted => "🧠 reasoning…",
            ThinkingPhase::Streaming => "🪼 generating…",
            ThinkingPhase::Done => "✨ done",
        }
    }
}

impl Default for ThinkingState {
    fn default() -> Self {
        Self::new(ThinkingPhase::Idle)
    }
}

// ---------------------------------------------------------------------------
// Thinking indicator
// ---------------------------------------------------------------------------

/// Animated thinking/reasoning indicator for streaming output.
///
/// Renders a cycling spinner with phase-appropriate labels, updating in place
/// without disrupting the streaming text below.
///
/// ## Usage
///
/// ```ignore
/// let mut indicator = ThinkingIndicator::new();
/// let theme = TerminalRenderer::new().color_theme();
///
/// // When extended thinking starts:
/// indicator.start_thinking(&theme, &mut stdout);
///
/// // As thinking chars stream in:
/// indicator.tick(&theme, &mut stdout);
///
/// // When thinking is redacted:
/// indicator.start_redacted(&theme, &mut stdout);
///
/// // When final text starts streaming:
/// indicator.start_streaming(&theme, &mut stdout);
///
/// // When complete:
/// indicator.finish(&theme, &mut stdout)?;
/// ```
#[derive(Debug)]
pub struct ThinkingIndicator {
    /// Current thinking state (phase, char count, etc).
    state: ThinkingState,
    /// Frame index for cycling through spinner dots.
    dot_frame: usize,
    /// Last time the spinner was ticked (for throttling animation).
    last_tick: Option<Instant>,
    /// Whether the indicator is currently visible on screen.
    visible: bool,
}

impl ThinkingIndicator {
    /// The animation frame characters for the thinking dots.
    const DOT_FRAMES: [&str; 4] = ["", ".", "..", "..."];

    /// Minimum duration between spinner ticks (for animation rate limiting).
    const TICK_INTERVAL: Duration = Duration::from_millis(200);

    /// Create a new thinking indicator.
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: ThinkingState::default(),
            dot_frame: 0,
            last_tick: None,
            visible: false,
        }
    }

    /// Returns the current thinking state for persistence.
    #[must_use]
    pub fn state(&self) -> &ThinkingState {
        &self.state
    }

    /// Restore state from a persisted `ThinkingState`.
    pub fn restore(&mut self, state: ThinkingState) {
        self.state = state;
        self.dot_frame = 0;
        self.last_tick = None;
        self.visible = false;
    }

    /// Start the extended thinking phase (model has a thinking block).
    pub fn start_thinking(&mut self) {
        self.state = ThinkingState::new(ThinkingPhase::Thinking);
    }

    /// Start the redacted thinking phase.
    pub fn start_redacted(&mut self) {
        self.state = ThinkingState::new(ThinkingPhase::Redacted).with_redacted();
    }

    /// Start the streaming phase (live text delta).
    pub fn start_streaming(&mut self) {
        self.state = ThinkingState::new(ThinkingPhase::Streaming);
    }

    /// Update the thinking character count as deltas arrive.
    pub fn update_thinking_chars(&mut self, delta_len: usize) {
        if matches!(self.state.phase, ThinkingPhase::Thinking | ThinkingPhase::Redacted) {
            let current = self.state.thinking_char_count.unwrap_or(0);
            self.state.thinking_char_count = Some(current.saturating_add(delta_len));
        }
    }

    /// Advance the indicator by one animation tick.
    ///
    /// Returns `Ok(true)` if the display was updated, `Ok(false)` if the
    /// tick was throttled (too soon since last update).
    pub fn tick(&mut self, out: &mut impl Write) -> io::Result<bool> {
        let now = Instant::now();
        if let Some(last) = self.last_tick {
            if now.duration_since(last) < Self::TICK_INTERVAL {
                return Ok(false);
            }
        }
        self.last_tick = Some(now);
        self.dot_frame = (self.dot_frame + 1) % Self::DOT_FRAMES.len();
        self.state.tick_count += 1;

        if !self.visible {
            return Ok(false);
        }

        self.render_indicator_line(out)
    }

    /// Show the indicator on screen (renders the initial line).
    pub fn show(&mut self, theme: &crate::render::ColorTheme, out: &mut impl Write) -> io::Result<()> {
        self.visible = true;
        self.dot_frame = 0;
        self.last_tick = Some(Instant::now());
        self.render_indicator_line(out)
    }

    /// Hide the indicator from screen.
    pub fn hide(&mut self, out: &mut impl Write) -> io::Result<()> {
        if !self.visible {
            return Ok(());
        }
        self.visible = false;
        queue!(
            out,
            SavePosition,
            MoveToColumn(0),
            Clear(ClearType::CurrentLine),
            RestorePosition,
        )?;
        out.flush()
    }

    /// Finish the indicator — render the final done/complete state.
    ///
    /// Returns the final `ThinkingState` for persistence.
    pub fn finish(&mut self, theme: &crate::render::ColorTheme, out: &mut impl Write) -> io::Result<ThinkingState> {
        self.state.finish();
        self.visible = true;

        if self.dot_frame > 0 || matches!(self.state.phase, ThinkingPhase::Streaming | ThinkingPhase::Thinking) {
            let label = self.state.label();
            execute!(
                out,
                SavePosition,
                MoveToColumn(0),
                Clear(ClearType::CurrentLine),
                SetForegroundColor(theme.spinner_done),
                Print(format!("✔ {label}")),
                ResetColor,
                RestorePosition,
            )?;
            out.flush()?;
        }

        let final_state = self.state.clone();
        self.visible = false;
        Ok(final_state)
    }

    /// Clear the indicator line (replace with blank space).
    pub fn clear(&mut self, out: &mut impl Write) -> io::Result<()> {
        self.visible = false;
        execute!(
            out,
            MoveToColumn(0),
            Clear(ClearType::CurrentLine),
        )?;
        out.flush()
    }

    /// Render the current indicator line to the given writer.
    fn render_indicator_line(&self, out: &mut impl Write) -> io::Result<bool> {
        let label = self.state.label();
        if label.is_empty() {
            return Ok(false);
        }

        let dots = Self::DOT_FRAMES[self.dot_frame];
        let text = format!("{label}{dots}");

        queue!(
            out,
            SavePosition,
            MoveToColumn(0),
            Clear(ClearType::CurrentLine),
            SetForegroundColor(Color::Cyan),
            Print(text),
            ResetColor,
            RestorePosition,
        )?;
        out.flush()
    }
}

impl Default for ThinkingIndicator {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn current_time_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thinking_state_default_is_idle() {
        let state = ThinkingState::default();
        assert_eq!(state.phase, ThinkingPhase::Idle);
        assert!(!state.is_flushed);
        assert!(!state.is_redacted);
    }

    #[test]
    fn thinking_state_finish_marks_done() {
        let mut state = ThinkingState::new(ThinkingPhase::Thinking);
        state.finish();
        assert_eq!(state.phase, ThinkingPhase::Done);
        assert!(state.is_flushed);
        assert!(state.ended_at_ms.is_some());
    }

    #[test]
    fn thinking_state_serialization_round_trip() {
        let state = ThinkingState::new(ThinkingPhase::Thinking);
        let json = state.to_json_string();
        let restored = ThinkingState::from_json_string(&json).unwrap();
        assert_eq!(restored.phase, ThinkingPhase::Thinking);
    }

    #[test]
    fn thinking_indicator_default() {
        let indicator = ThinkingIndicator::default();
        assert_eq!(indicator.state().phase, ThinkingPhase::Idle);
    }

    #[test]
    fn thinking_indicator_phase_labels() {
        let mut idle = ThinkingState::default();
        assert_eq!(idle.label(), "");

        let thinking = ThinkingState::new(ThinkingPhase::Thinking);
        assert_eq!(thinking.label(), "🧠 reasoning…");

        let streaming = ThinkingState::new(ThinkingPhase::Streaming);
        assert_eq!(streaming.label(), "🪼 generating…");

        let redacted = ThinkingState::new(ThinkingPhase::Redacted).with_redacted();
        assert_eq!(redacted.label(), "🧠 reasoning…");
    }

    #[test]
    fn thinking_indicator_update_char_count() {
        let mut indicator = ThinkingIndicator::new();
        indicator.start_thinking();
        indicator.update_thinking_chars(42);
        assert_eq!(indicator.state().thinking_char_count, Some(42));

        indicator.update_thinking_chars(10);
        assert_eq!(indicator.state().thinking_char_count, Some(52));
    }

    #[test]
    fn thinking_indicator_restore() {
        let mut indicator = ThinkingIndicator::new();
        let state = ThinkingState::new(ThinkingPhase::Streaming);
        indicator.restore(state.clone());
        assert_eq!(indicator.state().phase, ThinkingPhase::Streaming);
    }
}
