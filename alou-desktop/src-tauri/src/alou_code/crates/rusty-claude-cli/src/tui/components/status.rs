//! Status Bar Component
//!
//! Displays status information at the bottom of the terminal with enhanced token dashboard.

use std::time::{Duration, SystemTime};

/// Token consumption rates (approximate USD per 1K tokens)
#[derive(Debug, Clone)]
pub struct TokenRates {
    pub input_per_1k: f64,
    pub output_per_1k: f64,
}

impl Default for TokenRates {
    fn default() -> Self {
        Self {
            input_per_1k: 0.003, // Claude 3.5 Sonnet rates
            output_per_1k: 0.015,
        }
    }
}

impl TokenRates {
    /// Calculate cost from token counts
    pub fn calculate_cost(&self, input_tokens: u32, output_tokens: u32) -> f64 {
        (input_tokens as f64 / 1000.0 * self.input_per_1k)
            + (output_tokens as f64 / 1000.0 * self.output_per_1k)
    }
}

/// Status bar configuration
#[derive(Debug, Clone)]
pub struct StatusConfig {
    pub show_model: bool,
    pub show_tokens: bool,
    pub show_cost: bool,
    pub show_mode: bool,
    pub show_input_output: bool,
    pub show_rate: bool,
    pub compact: bool,
}

impl Default for StatusConfig {
    fn default() -> Self {
        Self {
            show_model: true,
            show_tokens: true,
            show_cost: true,
            show_mode: true,
            show_input_output: true,
            show_rate: true,
            compact: false,
        }
    }
}

/// Status information to display
#[derive(Debug, Clone)]
pub struct StatusInfo {
    pub model: Option<String>,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub total_tokens: u32,
    pub cost_usd: Option<f64>,
    pub processing: bool,
    pub thinking: bool,
    pub render_mode: String,
    pub connection_status: ConnectionStatus,
    pub elapsed: Duration,
    /// Characters per second rate for streaming
    pub chars_per_second: Option<f64>,
    /// Streaming state
    pub streaming: bool,
    /// Current thinking content (for animation)
    pub thinking_content: Option<String>,
}

impl Default for StatusInfo {
    fn default() -> Self {
        Self {
            model: None,
            input_tokens: 0,
            output_tokens: 0,
            total_tokens: 0,
            cost_usd: None,
            processing: false,
            thinking: false,
            render_mode: "compact".to_string(),
            connection_status: ConnectionStatus::Connected,
            elapsed: Duration::from_secs(0),
            chars_per_second: None,
            streaming: false,
            thinking_content: None,
        }
    }
}

/// Connection status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
    Connecting,
    Error,
}

impl std::fmt::Display for ConnectionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectionStatus::Connected => write!(f, "connected"),
            ConnectionStatus::Disconnected => write!(f, "disconnected"),
            ConnectionStatus::Connecting => write!(f, "connecting"),
            ConnectionStatus::Error => write!(f, "error"),
        }
    }
}

/// ANSI colors for status bar
pub mod colors {
    pub const RESET: &str = "\x1b[0m";
    pub const BOLD: &str = "\x1b[1m";
    pub const DIM: &str = "\x1b[2m";

    // Primary colors
    pub const NORMAL: &str = "\x1b[38;5;240m"; // Gray
    pub const PROCESSING: &str = "\x1b[38;5;226m"; // Bright yellow
    pub const SUCCESS: &str = "\x1b[38;5;82m"; // Green
    pub const ERROR: &str = "\x1b[38;5;196m"; // Red
    pub const INFO: &str = "\x1b[38;5;51m"; // Cyan

    // Token colors
    pub const INPUT_TOKEN: &str = "\x1b[38;5;75m"; // Blue
    pub const OUTPUT_TOKEN: &str = "\x1b[38;5;82m"; // Green
    pub const TOTAL_TOKEN: &str = "\x1b[38;5;141m"; // Purple

    // Cost colors
    pub const COST: &str = "\x1b[38;5;214m"; // Orange
    pub const RATE: &str = "\x1b[38;5;45m"; // Teal

    // Thinking animation colors
    pub const THINKING: &str = "\x1b[38;5;201m"; // Magenta
    pub const THINKING_BG: &str = "\x1b[48;5;236m"; // Dark bg

    // Border colors
    pub const BORDER: &str = "\x1b[38;5;236m"; // Dark gray
}

/// Thinking animation frames
const THINKING_FRAMES: [&str; 4] = ["◐", "◓", "◑", "◒"];

/// Status bar renderer with enhanced dashboard
pub struct StatusBar {
    config: StatusConfig,
    width: u16,
    rates: TokenRates,
    animation_frame: usize,
}

impl StatusBar {
    /// Create a new status bar
    pub fn new(width: u16) -> Self {
        Self {
            config: StatusConfig::default(),
            width,
            rates: TokenRates::default(),
            animation_frame: 0,
        }
    }

    /// Set custom config
    pub fn with_config(mut self, config: StatusConfig) -> Self {
        self.config = config;
        self
    }

    /// Set width
    pub fn with_width(mut self, width: u16) -> Self {
        self.width = width;
        self
    }

    /// Set token rates
    pub fn with_rates(mut self, rates: TokenRates) -> Self {
        self.rates = rates;
        self
    }

    /// Advance animation frame
    pub fn tick(&mut self) {
        self.animation_frame = (self.animation_frame + 1) % THINKING_FRAMES.len();
    }

    /// Get current thinking frame
    pub fn thinking_frame(&self) -> &str {
        &THINKING_FRAMES[self.animation_frame]
    }

    /// Format token count with thousands separator
    fn format_tokens(&self, tokens: u32) -> String {
        let s = tokens.to_string();
        let mut result = String::new();
        for (i, c) in s.chars().rev().enumerate() {
            if i > 0 && i % 3 == 0 {
                result.push(',');
            }
            result.push(c);
        }
        result.chars().rev().collect()
    }

    /// Format cost
    fn format_cost(&self, cost: f64) -> String {
        if cost < 0.001 {
            format!("${:.4}", cost)
        } else if cost < 0.01 {
            format!("${:.3}", cost)
        } else {
            format!("${:.2}", cost)
        }
    }

    /// Format rate
    fn format_rate(&self, cps: f64) -> String {
        if cps >= 1000.0 {
            format!("{:.1}K c/s", cps / 1000.0)
        } else {
            format!("{:.0} c/s", cps)
        }
    }

    /// Render the enhanced status bar with token dashboard
    pub fn render(&self, info: &StatusInfo) -> String {
        if self.config.compact {
            return self.render_compact(info);
        }
        self.render_dashboard(info)
    }

    /// Render compact single-line status
    fn render_compact(&self, info: &StatusInfo) -> String {
        let mut parts = Vec::new();

        // Status indicator
        if info.processing {
            if info.thinking {
                parts.push(format!(
                    "{} {}thinking{}",
                    colors::THINKING,
                    self.thinking_frame(),
                    colors::RESET
                ));
            } else if info.streaming {
                parts.push(format!(
                    "{} {}streaming{}",
                    colors::SUCCESS,
                    self.thinking_frame(),
                    colors::RESET
                ));
            } else {
                parts.push(format!("{}{}{}", colors::PROCESSING, "●", colors::RESET));
            }
        }

        // Model
        if self.config.show_model {
            if let Some(model) = &info.model {
                parts.push(format!("{}{}{}", colors::INFO, model, colors::RESET));
            }
        }

        // Tokens
        if self.config.show_tokens {
            parts.push(format!(
                "{}tkn: {}{}",
                colors::NORMAL,
                self.format_tokens(info.total_tokens),
                colors::RESET
            ));
        }

        // Cost
        if self.config.show_cost {
            if let Some(cost) = info.cost_usd {
                parts.push(format!(
                    "{}{}{}",
                    colors::COST,
                    self.format_cost(cost),
                    colors::RESET
                ));
            }
        }

        // Rate
        if self.config.show_rate {
            if let Some(cps) = info.chars_per_second {
                parts.push(format!(
                    "{}@ {}{}",
                    colors::RATE,
                    self.format_rate(cps),
                    colors::RESET
                ));
            }
        }

        // Connection
        let conn = match info.connection_status {
            ConnectionStatus::Connected => format!("{}●{}", colors::SUCCESS, colors::RESET),
            ConnectionStatus::Connecting => format!("{}◐{}", colors::PROCESSING, colors::RESET),
            ConnectionStatus::Disconnected => format!("{}○{}", colors::NORMAL, colors::RESET),
            ConnectionStatus::Error => format!("{}✗{}", colors::ERROR, colors::RESET),
        };
        parts.push(conn);

        let status_line = parts.join(&format!(" {}│ ", colors::BORDER));
        format!(
            "{}{}{}{}",
            colors::DIM,
            status_line,
            colors::RESET,
            colors::BORDER
        )
    }

    /// Render full dashboard with token breakdown
    fn render_dashboard(&self, info: &StatusInfo) -> String {
        let mut lines = Vec::new();

        // Top border
        lines.push(self.render_top_border());

        // Main status line
        lines.push(self.render_main_status(info));

        // Token breakdown
        if self.config.show_input_output {
            lines.push(self.render_token_breakdown(info));
        }

        // Bottom border
        lines.push(self.render_bottom_border());

        lines.join("\n")
    }

    /// Render top border
    fn render_top_border(&self) -> String {
        let border_width = self.width as usize - 2;
        format!(
            "{}{}{}{}{}{}",
            colors::BORDER,
            "╭",
            "─".repeat(border_width.min(100)),
            "╮",
            colors::RESET,
            colors::BORDER
        )
    }

    /// Render bottom border
    fn render_bottom_border(&self) -> String {
        let border_width = self.width as usize - 2;
        format!(
            "{}{}{}{}{}{}",
            colors::BORDER,
            "╰",
            "─".repeat(border_width.min(100)),
            "╯",
            colors::RESET,
            colors::BORDER
        )
    }

    /// Render main status line
    fn render_main_status(&self, info: &StatusInfo) -> String {
        let mut left = Vec::new();
        let mut right = Vec::new();

        // Left side: Status + Model
        if info.processing {
            if info.thinking {
                left.push(format!(
                    "{}{}{}{}{}",
                    colors::THINKING,
                    self.thinking_frame(),
                    colors::RESET,
                    colors::THINKING,
                    " thinking"
                ));
            } else if info.streaming {
                left.push(format!(
                    "{}{}{}{}{}",
                    colors::SUCCESS,
                    self.thinking_frame(),
                    colors::RESET,
                    colors::SUCCESS,
                    " streaming"
                ));
            } else {
                left.push(format!(
                    "{}{}{}{}{}",
                    colors::PROCESSING,
                    "●",
                    colors::RESET,
                    colors::PROCESSING,
                    " processing"
                ));
            }
        } else {
            left.push(format!("{}● ready{}", colors::SUCCESS, colors::RESET));
        }

        if self.config.show_model {
            if let Some(model) = &info.model {
                left.push(format!("{}{}{}", colors::INFO, model, colors::RESET));
            }
        }

        // Right side: Time + Connection
        let elapsed_secs = info.elapsed.as_secs();
        let elapsed_str = if elapsed_secs < 60 {
            format!("{}s", elapsed_secs)
        } else {
            format!("{}m {}s", elapsed_secs / 60, elapsed_secs % 60)
        };
        right.push(format!("{}{}{}", colors::DIM, elapsed_str, colors::RESET));

        let conn_str = match info.connection_status {
            ConnectionStatus::Connected => format!("{}●{}", colors::SUCCESS, colors::RESET),
            ConnectionStatus::Connecting => format!("{}◐{}", colors::PROCESSING, colors::RESET),
            ConnectionStatus::Disconnected => format!("{}○{}", colors::NORMAL, colors::RESET),
            ConnectionStatus::Error => format!("{}✗{}", colors::ERROR, colors::RESET),
        };
        right.push(conn_str);

        // Build the line
        let left_str = left.join(&format!(" {}│ ", colors::BORDER));
        let right_str = right.join(&format!(" {}│ ", colors::BORDER));

        let separator = format!(" {}│ ", colors::BORDER);

        format!(
            "{}{}{}{}{}{}{}{}",
            colors::BORDER,
            "│ ",
            colors::RESET,
            left_str,
            separator,
            right_str,
            colors::BORDER,
            " │",
        )
    }

    /// Render token breakdown
    fn render_token_breakdown(&self, info: &StatusInfo) -> String {
        let mut parts = Vec::new();

        // Input tokens
        if self.config.show_input_output {
            parts.push(format!(
                "{}{}in:{}{} {}{}",
                colors::BORDER,
                "│",
                colors::RESET,
                colors::INPUT_TOKEN,
                self.format_tokens(info.input_tokens),
                colors::RESET
            ));
            parts.push(format!(
                "{}{}out:{}{} {}{}",
                colors::BORDER,
                "│",
                colors::RESET,
                colors::OUTPUT_TOKEN,
                self.format_tokens(info.output_tokens),
                colors::RESET
            ));
        }

        // Total tokens
        if self.config.show_tokens {
            parts.push(format!(
                "{}{}tot:{}{} {}{}",
                colors::BORDER,
                "│",
                colors::RESET,
                colors::TOTAL_TOKEN,
                self.format_tokens(info.total_tokens),
                colors::RESET
            ));
        }

        // Cost
        if self.config.show_cost {
            let cost = info.cost_usd.unwrap_or_else(|| {
                self.rates
                    .calculate_cost(info.input_tokens, info.output_tokens)
            });
            parts.push(format!(
                "{}{}cost:{}{} {}{}",
                colors::BORDER,
                "│",
                colors::RESET,
                colors::COST,
                self.format_cost(cost),
                colors::RESET
            ));
        }

        // Rate
        if self.config.show_rate {
            if let Some(cps) = info.chars_per_second {
                parts.push(format!(
                    "{}{}rate:{}{} {}{}",
                    colors::BORDER,
                    "│",
                    colors::RESET,
                    colors::RATE,
                    self.format_rate(cps),
                    colors::RESET
                ));
            }
        }

        // Mode
        if self.config.show_mode {
            parts.push(format!(
                "{}{}{}{}",
                colors::DIM,
                info.render_mode,
                colors::RESET,
                colors::BORDER
            ));
        }

        let content = parts.join(&format!(" {}│ ", colors::BORDER));
        format!("{}{}{}{}", colors::BORDER, "│ ", colors::RESET, content)
    }

    /// Render just the left portion (for compact mode)
    pub fn render_left(&self, info: &StatusInfo) -> String {
        let mut parts = Vec::new();

        if info.processing {
            if info.thinking {
                parts.push(format!(
                    "{}{} {}",
                    colors::THINKING,
                    self.thinking_frame(),
                    colors::RESET
                ));
            } else {
                parts.push(format!(
                    "{}{}...{}",
                    colors::PROCESSING,
                    colors::BOLD,
                    colors::RESET
                ));
            }
        }

        if let Some(model) = &info.model {
            parts.push(format!("{}{}{}", colors::INFO, model, colors::RESET));
        }

        parts.join(" ")
    }

    /// Render the divider line
    pub fn render_divider(&self) -> String {
        let width = (self.width as usize).saturating_sub(2);
        format!(
            "{}{}{}{}{}",
            colors::BORDER,
            "─".repeat(width.min(200)),
            colors::RESET,
            colors::BORDER,
            " "
        )
    }

    /// Render thinking animation indicator
    pub fn render_thinking(&self, content: Option<&str>) -> String {
        let frame = self.thinking_frame();

        if let Some(text) = content {
            // Show thinking content with animation
            let truncated = if text.len() > 50 {
                format!("{}...", &text[..50])
            } else {
                text.to_string()
            };
            format!(
                "{}{}{}{}{}",
                colors::THINKING,
                colors::BOLD,
                frame,
                colors::RESET,
                format!(" {} {}", colors::THINKING, truncated)
            )
        } else {
            format!(
                "{}{}{}{}",
                colors::THINKING,
                colors::BOLD,
                frame,
                colors::RESET
            )
        }
    }
}

impl Default for StatusBar {
    fn default() -> Self {
        Self::new(80)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_bar_render() {
        let bar = StatusBar::new(80);
        let info = StatusInfo {
            model: Some("claude-3-sonnet".to_string()),
            input_tokens: 100,
            output_tokens: 200,
            total_tokens: 300,
            cost_usd: Some(0.0125),
            processing: true,
            thinking: false,
            render_mode: "expanded".to_string(),
            connection_status: ConnectionStatus::Connected,
            elapsed: Duration::from_secs(42),
            chars_per_second: Some(45.5),
            streaming: true,
            thinking_content: None,
        };

        let rendered = bar.render(&info);
        assert!(rendered.contains("processing") || rendered.contains("●"));
        assert!(rendered.contains("300") || rendered.contains("tot"));
        assert!(rendered.contains("claude-3-sonnet"));
    }

    #[test]
    fn test_connection_status_display() {
        assert_eq!(ConnectionStatus::Connected.to_string(), "connected");
        assert_eq!(ConnectionStatus::Error.to_string(), "error");
    }

    #[test]
    fn test_token_rates() {
        let rates = TokenRates::default();
        let cost = rates.calculate_cost(1000, 1000);
        assert!((cost - 0.018).abs() < 0.001);
    }

    #[test]
    fn test_thinking_animation() {
        let mut bar = StatusBar::new(80);
        for i in 0..8 {
            let frame = bar.thinking_frame();
            assert!(THINKING_FRAMES.contains(&frame));
            bar.tick();
        }
    }
}
