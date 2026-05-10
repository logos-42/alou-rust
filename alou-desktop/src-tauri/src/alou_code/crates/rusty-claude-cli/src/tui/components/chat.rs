//! Chat Message Component
//!
//! Renders individual chat messages with role-based styling and streaming support.

use crate::tui::state::{Message, MessageRole, RenderMode};

/// ANSI color codes for chat roles
pub mod colors {
    pub const RESET: &str = "\x1b[0m";
    pub const BOLD: &str = "\x1b[1m";
    pub const DIM: &str = "\x1b[2m";
    pub const BLINK: &str = "\x1b[5m";

    // User message colors (cyan)
    pub const USER_PREFIX: &str = "\x1b[96m";
    pub const USER_TEXT: &str = "\x1b[36m";

    // Assistant message colors (green)
    pub const ASSISTANT_PREFIX: &str = "\x1b[92m";
    pub const ASSISTANT_TEXT: &str = "\x1b[32m";

    // System message colors (yellow)
    pub const SYSTEM_PREFIX: &str = "\x1b[93m";
    pub const SYSTEM_TEXT: &str = "\x1b[33m";

    // Tool message colors (magenta)
    pub const TOOL_PREFIX: &str = "\x1b[95m";
    pub const TOOL_TEXT: &str = "\x1b[35m";

    // Error color (red)
    pub const ERROR: &str = "\x1b[91m";

    // Tool call color (blue)
    pub const TOOL_CALL: &str = "\x1b[94m";

    // Streaming colors
    pub const STREAMING: &str = "\x1b[38;5;82m"; // Green
    pub const STREAMING_CURSOR: &str = "\x1b[38;5;208m"; // Orange
    pub const THINKING: &str = "\x1b[38;5;201m"; // Magenta
    pub const THINKING_BLOCK: &str = "\x1b[48;5;236m"; // Dark bg

    // Thinking section colors
    pub const THINKING_BG: &str = "\x1b[48;5;235m";
    pub const THINKING_LABEL: &str = "\x1b[38;5;201m";
    pub const THINKING_TEXT: &str = "\x1b[38;5;247m";
}

/// Animation frames for streaming cursor
const STREAMING_FRAMES: [&str; 4] = ["▏", "▎", "▍", "▌"];

/// Configuration for chat renderer
#[derive(Debug, Clone)]
pub struct ChatRenderer {
    pub show_timestamps: bool,
    pub show_tool_calls: bool,
    pub max_width: usize,
    pub indent_size: usize,
    pub streaming_enabled: bool,
}

impl Default for ChatRenderer {
    fn default() -> Self {
        Self {
            show_timestamps: false,
            show_tool_calls: true,
            max_width: 120,
            indent_size: 2,
            streaming_enabled: true,
        }
    }
}

impl ChatRenderer {
    /// Create a new chat renderer with custom config
    pub fn new(max_width: usize) -> Self {
        Self {
            max_width,
            ..Default::default()
        }
    }

    /// Render a message with optional streaming content
    pub fn render_message_streaming(
        &self,
        message: &Message,
        mode: RenderMode,
        streaming_content: Option<&str>,
        cursor_frame: usize,
    ) -> Vec<String> {
        let mut lines = Vec::new();

        // Role prefix with enhanced styling
        let (prefix, text_color) = match message.role {
            MessageRole::User => (
                format!("{}┌─ user ─{}", colors::USER_PREFIX, colors::RESET),
                colors::USER_TEXT,
            ),
            MessageRole::Assistant => (
                format!(
                    "{}┌─ assistant ─{}",
                    colors::ASSISTANT_PREFIX,
                    colors::RESET
                ),
                colors::ASSISTANT_TEXT,
            ),
            MessageRole::System => (
                format!("{}┌─ system ─{}", colors::SYSTEM_PREFIX, colors::RESET),
                colors::SYSTEM_TEXT,
            ),
            MessageRole::Tool => (
                format!("{}┌─ tool ─{}", colors::TOOL_PREFIX, colors::RESET),
                colors::TOOL_TEXT,
            ),
        };

        lines.push(prefix);

        // Content based on render mode
        let content_lines = if let Some(streaming) = streaming_content {
            self.render_streaming_content(streaming, cursor_frame)
        } else {
            match mode {
                RenderMode::Compact => self.render_compact(&message.content),
                RenderMode::Expanded => self.render_expanded(&message.content),
                RenderMode::SyntaxHighlighted => self.render_with_highlight(&message.content),
            }
        };

        for line in content_lines {
            lines.push(format!("{}{}{}", text_color, line, colors::RESET));
        }

        // Tool calls
        if self.show_tool_calls && !message.tool_calls.is_empty() {
            lines.push(String::new());
            for tc in &message.tool_calls {
                lines.push(self.render_tool_call(&tc.id, &tc.name, &tc.input));
            }
        }

        // Tool results
        if !message.tool_results.is_empty() {
            lines.push(String::new());
            for tr in &message.tool_results {
                lines.push(self.render_tool_result(tr));
            }
        }

        // Bottom border
        let border = match message.role {
            MessageRole::User => format!("{}└", colors::USER_PREFIX),
            MessageRole::Assistant => format!("{}└", colors::ASSISTANT_PREFIX),
            MessageRole::System => format!("{}└", colors::SYSTEM_PREFIX),
            MessageRole::Tool => format!("{}└", colors::TOOL_PREFIX),
        };
        lines.push(format!("{}{}{}", border, "─".repeat(10), colors::RESET));

        lines
    }

    /// Render streaming content with animated cursor
    fn render_streaming_content(&self, content: &str, frame: usize) -> Vec<String> {
        let cursor = STREAMING_FRAMES[frame % STREAMING_FRAMES.len()];

        if content.is_empty() {
            return vec![format!(
                "{}{}{}{}{}",
                colors::STREAMING,
                colors::BOLD,
                cursor,
                colors::RESET,
                colors::DIM
            )];
        }

        let mut lines = Vec::new();
        let wrapped = self.render_expanded(content);

        for (i, line) in wrapped.iter().enumerate() {
            if i == wrapped.len() - 1 {
                // Last line gets the streaming cursor
                lines.push(format!(
                    "{}{}{}{}{}",
                    colors::STREAMING,
                    line,
                    colors::STREAMING_CURSOR,
                    cursor,
                    colors::RESET
                ));
            } else {
                lines.push(format!("{}{}{}", colors::STREAMING, line, colors::RESET));
            }
        }

        lines
    }

    /// Render a thinking section with block display
    pub fn render_thinking_section(&self, content: &str) -> Vec<String> {
        let mut lines = Vec::new();

        // Header
        lines.push(format!(
            "{}{}{} {}thinking{}",
            colors::THINKING_BLOCK,
            colors::THINKING_BG,
            colors::THINKING_LABEL,
            colors::BOLD,
            colors::RESET
        ));

        // Content with dim styling
        let wrapped = self.word_wrap(content, self.max_width - 4);
        for line in wrapped {
            lines.push(format!(
                "{}{}{}{} {}",
                colors::THINKING_BLOCK,
                colors::THINKING_TEXT,
                line,
                colors::RESET,
                colors::THINKING_BLOCK
            ));
        }

        // Footer
        lines.push(format!(
            "{}{}{}",
            colors::THINKING_BLOCK,
            " ".repeat(12),
            colors::RESET
        ));

        lines
    }

    /// Render a single message
    pub fn render_message(&self, message: &Message, mode: RenderMode) -> Vec<String> {
        self.render_message_streaming(message, mode, None, 0)
    }

    /// Render content in compact mode (truncated if long)
    fn render_compact(&self, content: &str) -> Vec<String> {
        let first_line = content.lines().next().unwrap_or("");
        let trimmed = first_line.chars().take(self.max_width).collect::<String>();
        if trimmed.len() < first_line.len() {
            vec![format!("{}...", trimmed)]
        } else {
            vec![trimmed.to_string()]
        }
    }

    /// Render content in expanded mode (full content, word wrapped)
    fn render_expanded(&self, content: &str) -> Vec<String> {
        let mut lines = Vec::new();
        let indent = " ".repeat(self.indent_size);

        for line in content.lines() {
            if line.is_empty() {
                lines.push(String::new());
            } else {
                // Word wrap
                let wrapped = self.word_wrap(line, self.max_width - self.indent_size);
                for wline in wrapped {
                    lines.push(format!("{}{}", indent, wline));
                }
            }
        }

        lines
    }

    /// Render content with syntax highlighting hints
    fn render_with_highlight(&self, content: &str) -> Vec<String> {
        let expanded = self.render_expanded(content);

        expanded
            .into_iter()
            .map(|line| {
                // Basic code block detection
                if line.starts_with("```") || line.starts_with("    ") {
                    format!("{}{}{}", colors::DIM, line, colors::RESET)
                } else {
                    line
                }
            })
            .collect()
    }

    /// Render a tool call
    fn render_tool_call(&self, id: &str, name: &str, input: &str) -> String {
        format!(
            "{}{}[{}] {}{}({}{}{})",
            colors::TOOL_CALL,
            colors::BOLD,
            id,
            name,
            colors::RESET,
            colors::DIM,
            input,
            colors::RESET
        )
    }

    /// Render a tool result
    fn render_tool_result(&self, result: &crate::tui::state::ToolResult) -> String {
        let output_preview: String = result.output.chars().take(100).collect();

        let suffix = if result.output.len() > 100 { "..." } else { "" };
        let color = if result.is_error {
            colors::ERROR
        } else {
            colors::DIM
        };

        format!(
            "{}{}[{}] {}{}{}{}{}",
            colors::TOOL_CALL,
            if result.is_error { colors::ERROR } else { "" },
            "result",
            result.tool_call_id,
            color,
            output_preview,
            suffix,
            colors::RESET
        )
    }

    /// Word wrap text to fit within width
    fn word_wrap(&self, text: &str, width: usize) -> Vec<String> {
        let mut lines = Vec::new();
        let mut current_line = String::new();
        let mut current_len = 0;

        for word in text.split_whitespace() {
            let word_len = word.chars().count();

            if current_len + word_len + 1 > width {
                if !current_line.is_empty() {
                    lines.push(current_line.clone());
                    current_line.clear();
                }
                // If single word is longer than width, break it
                if word_len > width {
                    for chunk in word.chars().collect::<Vec<_>>().chunks(width) {
                        lines.push(chunk.iter().collect());
                    }
                    current_len = 0;
                } else {
                    current_line = word.to_string();
                    current_len = word_len;
                }
            } else {
                if !current_line.is_empty() {
                    current_line.push(' ');
                    current_len += 1;
                }
                current_line.push_str(word);
                current_len += word_len;
            }
        }

        if !current_line.is_empty() {
            lines.push(current_line);
        }

        lines
    }

    /// Render multiple messages
    pub fn render_messages(&self, messages: &[Message], mode: RenderMode) -> Vec<String> {
        let mut output = Vec::new();

        for msg in messages {
            let rendered = self.render_message(msg, mode);
            output.extend(rendered);
            output.push(String::new()); // Empty line between messages
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::state::{Message, MessageRole};

    #[test]
    fn test_word_wrap() {
        let renderer = ChatRenderer::default();
        let lines = renderer.word_wrap("hello world this is a test", 10);
        assert!(lines.len() > 1);
        for line in &lines {
            assert!(line.chars().count() <= 10);
        }
    }

    fn create_test_message(role: MessageRole, content: &str) -> Message {
        Message {
            role,
            content: content.to_string(),
            timestamp: std::time::SystemTime::now(),
            tool_calls: Vec::new(),
            tool_results: Vec::new(),
        }
    }

    #[test]
    fn test_render_compact() {
        let renderer = ChatRenderer::default();
        let msg = create_test_message(MessageRole::User, "Hello\nWorld\n!");
        let lines = renderer.render_message(&msg, RenderMode::Compact);
        assert_eq!(lines.len(), 2); // Prefix + one line
        assert!(lines[1].contains("Hello"));
    }

    #[test]
    fn test_render_expanded() {
        let renderer = ChatRenderer::default();
        let msg = create_test_message(MessageRole::Assistant, "Line 1\nLine 2\nLine 3");
        let lines = renderer.render_message(&msg, RenderMode::Expanded);
        assert!(lines.len() > 3); // Prefix + 3 lines + empty
    }
}
