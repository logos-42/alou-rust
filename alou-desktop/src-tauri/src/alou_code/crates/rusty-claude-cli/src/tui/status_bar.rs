//! Bottom-of-screen status bar showing model, tokens, session, and turn info.
//!
//! The status bar is always visible at the bottom of the terminal during a
//! REPL session, updating in real-time as streaming events arrive.
//!
//! ## Design
//!
//! ```
//! ┌──────────────────────────────────────────────────────────────────────────┐
//! │  ... conversation output ...                                             │
//! │                                                                           │
//! │  [input area]                                                            │
//! └──────────────────────────────────────────────────────────────────────────┘
//!  🧠 sonnet  │  in 1,234  out 5,678  cache: 234R / 12W  │  session:abc123  │  💰$0.0234
//! ```
//!
//! The bar is split into three sections:
//! - **Left** — model name + thinking indicator icon
//! - **Center** — token counts (input, output, cache read/write)
//! - **Right** — session ID + estimated cost

use std::fmt::Write;
use std::io::{self, Write as IoWrite};

use crossterm::cursor::{MoveTo, MoveToColumn};
use crossterm::style::{Color, Print, ResetColor, SetForegroundColor};
use crossterm::terminal::{Clear, ClearType, DisableLineWrap, EnableLineWrap};
use crossterm::{execute, queue};

use crate::render::ColorTheme;
use crate::runtime::TokenUsage;

// ---------------------------------------------------------------------------
// Token dashboard (center section)
// ---------------------------------------------------------------------------

/// Live token dashboard for the status bar.
///
/// Tracks input/output/cache token counts in real-time as `Usage` events
/// arrive during streaming. Renders compact summaries suitable for the
/// status bar line.
#[derive(Debug, Clone)]
pub struct TokenDashboard {
    /// Latest per-turn token usage.
    latest: TokenUsage,
    /// Cumulative token usage across all turns.
    cumulative: TokenUsage,
    /// Number of turns in this session.
    turn_count: u32,
    /// Max budget tokens (for warning when approaching limit).
    max_tokens: u32,
}

impl TokenDashboard {
    /// Create a new token dashboard.
    #[must_use]
    pub fn new() -> Self {
        Self {
            latest: TokenUsage::default(),
            cumulative: TokenUsage::default(),
            turn_count: 0,
            max_tokens: 64_000,
        }
    }

    /// Set the maximum token budget (triggers warning colors when approaching).
    pub fn with_max_tokens(mut self, max: u32) -> Self {
        self.max_tokens = max;
        self
    }

    /// Record a new usage event from the streaming response.
    pub fn record_usage(&mut self, usage: TokenUsage) {
        self.latest = usage;
        self.cumulative.input_tokens += usage.input_tokens;
        self.cumulative.output_tokens += usage.output_tokens;
        self.cumulative.cache_creation_input_tokens += usage.cache_creation_input_tokens;
        self.cumulative.cache_read_input_tokens += usage.cache_read_input_tokens;
        self.turn_count += 1;
    }

    /// Returns the current turn count.
    #[must_use]
    pub fn turns(&self) -> u32 {
        self.turn_count
    }

    /// Returns the cumulative token usage.
    #[must_use]
    pub fn cumulative(&self) -> TokenUsage {
        self.cumulative
    }

    /// Returns whether we are approaching the token budget.
    #[must_use]
    pub fn approaching_limit(&self) -> bool {
        self.cumulative.total_tokens() > self.max_tokens.saturating_sub(2_000)
    }

    /// Render the token summary to a string.
    ///
    /// Returns a compact form like: `"in 1,234  out 5,678  cache: 234R / 12W"`
    #[must_use]
    pub fn render(&self) -> String {
        let in_tokens = Self::fmt_number(self.cumulative.input_tokens);
        let out_tokens = Self::fmt_number(self.cumulative.output_tokens);
        let cache_read = Self::fmt_number(self.cumulative.cache_read_input_tokens);
        let cache_write = Self::fmt_number(self.cumulative.cache_creation_input_tokens);

        format!(
            "in {in_tokens}  out {out_tokens}  cache: {cache_read}R / {cache_write}W"
        )
    }

    /// Render the estimated cost in USD.
    #[must_use]
    pub fn render_cost(&self) -> String {
        let cost = self.cumulative.estimate_cost_usd();
        format!("${:.4}", cost.total_cost_usd())
    }

    /// Format a number with thousands separators.
    fn fmt_number(n: u32) -> String {
        let s = n.to_string();
        let mut result = String::with_capacity(s.len() + s.len() / 3);
        for (i, ch) in s.chars().rev().enumerate() {
            if i > 0 && i % 3 == 0 {
                result.push(',');
            }
            result.push(ch);
        }
        result.chars().rev().collect()
    }
}

impl Default for TokenDashboard {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Status bar
// ---------------------------------------------------------------------------

/// The bottom-pinned status bar.
///
/// Renders at the bottom of the terminal and shows:
/// - Model name (left)
/// - Token dashboard (center)
/// - Session ID + cost (right)
///
/// The bar updates in-place as events arrive, without disrupting the
/// conversation output above it.
#[derive(Debug)]
pub struct StatusBar {
    /// Token dashboard with live token counts.
    dashboard: TokenDashboard,
    /// Current model name.
    model: String,
    /// Current session ID (shortened to 8 chars).
    session_id: String,
    /// Permission mode string.
    permission_mode: String,
    /// Terminal height (used for bottom positioning).
    height: u16,
    /// Whether the status bar is currently rendered.
    rendered: bool,
}

impl StatusBar {
    /// Create a new status bar.
    #[must_use]
    pub fn new(model: &str, session_id: &str, permission_mode: &str) -> Self {
        let short_session = session_id.chars().take(8).collect();
        Self {
            dashboard: TokenDashboard::new(),
            model: model.to_string(),
            session_id: short_session,
            permission_mode: permission_mode.to_string(),
            height: 24,
            rendered: false,
        }
    }

    /// Update the terminal height (call on resize events).
    pub fn set_height(&mut self, height: u16) {
        self.height = height;
    }

    /// Update the token usage from a streaming usage event.
    pub fn record_usage(&mut self, usage: TokenUsage) {
        self.dashboard.record_usage(usage);
    }

    /// Update the model name.
    pub fn set_model(&mut self, model: &str) {
        self.model = model.to_string();
    }

    /// Update the session ID.
    pub fn set_session_id(&mut self, session_id: &str) {
        self.session_id = session_id.chars().take(8).collect();
    }

    /// Render the status bar at the bottom of the terminal.
    ///
    /// Overwrites the last line with the current status information.
    /// Call `hide()` first to clear the bar when exiting.
    pub fn render(&mut self, theme: &ColorTheme, out: &mut impl IoWrite) -> io::Result<()> {
        let token_text = self.dashboard.render();
        let cost_text = self.dashboard.render_cost();
        let approaching = self.dashboard.approaching_limit();

        // Left: model + permission
        let left = format!(" {}  {}", self.model, self.permission_mode);

        // Center: token counts
        let center = token_text;

        // Right: session + cost
        let right = format!("  session:{}  💰{}", self.session_id, cost_text);

        // Build the full bar line
        let bar = format!("{}{}{}", left, center, right);

        // Color: warning if approaching token limit
        let color = if approaching {
            Color::Yellow
        } else {
            theme.table_border
        };

        queue!(
            out,
            MoveTo(0, self.height.saturating_sub(1)),
            Clear(ClearType::CurrentLine),
            SetForegroundColor(color),
            Print(format!("─")),
            Print(&bar),
            Print(format!("─")),
            ResetColor,
        )?;

        // Disable line wrap so long status lines don't cause scroll issues
        // Note: we don't call DisableLineWrap here since it persists after
        // the bar — the caller should manage this globally
        execute!(out, MoveToColumn(0))?;

        self.rendered = true;
        out.flush()
    }

    /// Update the status bar in-place (call this on each Usage event).
    pub fn update(&mut self, theme: &ColorTheme, out: &mut impl IoWrite) -> io::Result<()> {
        if !self.rendered {
            return self.render(theme, out);
        }

        // For in-place update, we only re-render the center (token) portion
        // since the left/right sections rarely change.
        let token_text = self.dashboard.render();
        let cost_text = self.dashboard.render_cost();
        let approaching = self.dashboard.approaching_limit();
        let color = if approaching { Color::Yellow } else { theme.table_border };

        // Calculate cursor position for center section
        let left_width = self.model.len() + self.permission_mode.len() + 3;
        let center_start = left_width;

        queue!(
            out,
            MoveTo(center_start as u16, self.height.saturating_sub(1)),
            Clear(ClearType::ToEndOfLine),
            SetForegroundColor(color),
            Print(&token_text),
        )?;

        // Also update cost on the right
        let right_width = format!("  session:{}  💰", self.session_id).len();
        let cost_start = (self.height as usize * 80).saturating_sub(right_width + cost_text.len());
        queue!(
            out,
            MoveTo(self.height.saturating_sub(1), (self.height as u16).saturating_sub(1)),
        )?;

        // Reset and flush
        let total_width = left_width + token_text.len() + right_width + cost_text.len();
        execute!(
            out,
            MoveTo(total_width as u16, self.height.saturating_sub(1)),
            ResetColor,
        )?;
        out.flush()
    }

    /// Hide the status bar (clear the bottom line).
    pub fn hide(&mut self, out: &mut impl IoWrite) -> io::Result<()> {
        if !self.rendered {
            return Ok(());
        }
        queue!(
            out,
            MoveTo(0, self.height.saturating_sub(1)),
            Clear(ClearType::CurrentLine),
        )?;
        out.flush()?;
        self.rendered = false;
        Ok(())
    }

    /// Return the dashboard for reading token stats.
    #[must_use]
    pub fn dashboard(&self) -> &TokenDashboard {
        &self.dashboard
    }

    /// Serialize the dashboard state to JSON for session persistence.
    #[must_use]
    pub fn dashboard_json(&self) -> String {
        serde_json::json!({
            "latest": {
                "input_tokens": self.dashboard.latest.input_tokens,
                "output_tokens": self.dashboard.latest.output_tokens,
                "cache_creation_input_tokens": self.dashboard.latest.cache_creation_input_tokens,
                "cache_read_input_tokens": self.dashboard.latest.cache_read_input_tokens,
            },
            "cumulative": {
                "input_tokens": self.dashboard.cumulative.input_tokens,
                "output_tokens": self.dashboard.cumulative.output_tokens,
                "cache_creation_input_tokens": self.dashboard.cumulative.cache_creation_input_tokens,
                "cache_read_input_tokens": self.dashboard.cumulative.cache_read_input_tokens,
            },
            "turn_count": self.dashboard.turn_count,
            "max_tokens": self.dashboard.max_tokens,
        })
        .to_string()
    }

    /// Restore dashboard state from JSON (on session resume).
    pub fn restore_dashboard_from_json(&mut self, json: &str) {
        if let Ok(data) = serde_json::from_str::<serde_json::Value>(json) {
            if let (Some(in_latest), Some(out_latest), Some(cr), Some(cw), Some(cumulative), Some(turns)) = (
                data.get("latest")
                    .and_then(|v| v.get("input_tokens"))
                    .and_then(|v| v.as_u64()),
                data.get("latest")
                    .and_then(|v| v.get("output_tokens"))
                    .and_then(|v| v.as_u64()),
                data.get("latest")
                    .and_then(|v| v.get("cache_creation_input_tokens"))
                    .and_then(|v| v.as_u64()),
                data.get("latest")
                    .and_then(|v| v.get("cache_read_input_tokens"))
                    .and_then(|v| v.as_u64()),
                data.get("cumulative").and_then(|v| serde_json::from_value(v.clone()).ok()),
                data.get("turn_count").and_then(|v| v.as_u64()),
            ) {
                self.dashboard.latest = TokenUsage {
                    input_tokens: in_latest as u32,
                    output_tokens: out_latest as u32,
                    cache_creation_input_tokens: cr as u32,
                    cache_read_input_tokens: cw as u32,
                };
                if let Some(cum) = cumulative {
                    self.dashboard.cumulative = cum;
                }
                self.dashboard.turn_count = turns as u32;
            }
        }
    }
}

impl Default for StatusBar {
    fn default() -> Self {
        Self {
            dashboard: TokenDashboard::default(),
            model: String::new(),
            session_id: String::new(),
            permission_mode: String::new(),
            height: 24,
            rendered: false,
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_dashboard_records_and_renders() {
        let mut dash = TokenDashboard::new();
        dash.record_usage(TokenUsage {
            input_tokens: 1234,
            output_tokens: 5678,
            cache_creation_input_tokens: 100,
            cache_read_input_tokens: 200,
        });

        let rendered = dash.render();
        assert!(rendered.contains("1,234"));
        assert!(rendered.contains("5,678"));
        assert!(rendered.contains("200R"));
        assert!(rendered.contains("100W"));
    }

    #[test]
    fn token_dashboard_cumulative_across_turns() {
        let mut dash = TokenDashboard::new();
        dash.record_usage(TokenUsage {
            input_tokens: 1000,
            output_tokens: 100,
            cache_creation_input_tokens: 0,
            cache_read_input_tokens: 0,
        });
        dash.record_usage(TokenUsage {
            input_tokens: 2000,
            output_tokens: 200,
            cache_creation_input_tokens: 50,
            cache_read_input_tokens: 0,
        });

        assert_eq!(dash.turns(), 2);
        assert_eq!(dash.cumulative.input_tokens, 3000);
        assert_eq!(dash.cumulative.output_tokens, 300);
        assert_eq!(dash.cumulative.cache_creation_input_tokens, 50);
    }

    #[test]
    fn token_dashboard_approaching_limit() {
        let mut dash = TokenDashboard::new().with_max_tokens(5000);
        assert!(!dash.approaching_limit());

        dash.record_usage(TokenUsage {
            input_tokens: 3000,
            output_tokens: 0,
            cache_creation_input_tokens: 0,
            cache_read_input_tokens: 0,
        });
        assert!(!dash.approaching_limit());

        dash.record_usage(TokenUsage {
            input_tokens: 3000,
            output_tokens: 0,
            cache_creation_input_tokens: 0,
            cache_read_input_tokens: 0,
        });
        // Now we have ~6000 cumulative with max 5000
        assert!(dash.approaching_limit());
    }

    #[test]
    fn status_bar_new() {
        let bar = StatusBar::new("claude-sonnet", "abc123def456", "ask");
        assert_eq!(bar.session_id, "abc123de");
        assert_eq!(bar.model, "claude-sonnet");
    }

    #[test]
    fn status_bar_dashboard_json_round_trip() {
        let mut bar = StatusBar::new("sonnet", "sess1", "ask");
        bar.record_usage(TokenUsage {
            input_tokens: 100,
            output_tokens: 50,
            cache_creation_input_tokens: 10,
            cache_read_input_tokens: 5,
        });

        let json = bar.dashboard_json();
        let mut bar2 = StatusBar::new("sonnet", "sess2", "ask");
        bar2.restore_dashboard_from_json(&json);

        assert_eq!(bar2.dashboard.turn_count, bar.dashboard.turn_count);
    }
}
