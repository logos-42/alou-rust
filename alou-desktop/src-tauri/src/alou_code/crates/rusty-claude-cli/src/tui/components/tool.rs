//! Tool Call Component
//!
//! Renders tool calls and their results with formatting.

use crate::tui::state::{ToolCall, ToolResult};

/// ANSI colors for tool rendering
mod colors {
    pub const RESET: &str = "\x1b[0m";
    pub const BOLD: &str = "\x1b[1m";
    pub const DIM: &str = "\x1b[2m";

    pub const TOOL_NAME: &str = "\x1b[94m"; // Blue
    pub const TOOL_ID: &str = "\x1b[90m"; // Dark gray
    pub const TOOL_INPUT: &str = "\x1b[38;5;245m"; // Light gray
    pub const TOOL_OUTPUT: &str = "\x1b[38;5;250m"; // Light gray
    pub const SUCCESS: &str = "\x1b[32m"; // Green
    pub const ERROR: &str = "\x1b[31m"; // Red
    pub const HEADER: &str = "\x1b[36m"; // Cyan
}

/// Tool call renderer
pub struct ToolRenderer {
    pub max_output_lines: usize,
    pub truncate_output: bool,
}

impl Default for ToolRenderer {
    fn default() -> Self {
        Self {
            max_output_lines: 50,
            truncate_output: true,
        }
    }
}

impl ToolRenderer {
    /// Create a new tool renderer
    pub fn new() -> Self {
        Self::default()
    }

    /// Render a single tool call
    pub fn render_call(&self, tool_call: &ToolCall) -> Vec<String> {
        let mut lines = Vec::new();

        // Header line
        lines.push(format!(
            "{}{}[{}] {}{} {}({}",
            colors::TOOL_ID,
            "╭",
            tool_call.id,
            colors::BOLD,
            colors::TOOL_NAME,
            tool_call.name,
            colors::RESET
        ));

        // Input
        if !tool_call.input.is_empty() {
            let input_lines = self.format_json(&tool_call.input);
            for line in input_lines {
                lines.push(format!("{}  {}{}", colors::TOOL_INPUT, line, colors::RESET));
            }
        }

        // Footer
        lines.push(format!(
            "{}{}{}",
            colors::TOOL_ID,
            "╰─────────────────────────────────",
            colors::RESET
        ));

        lines
    }

    /// Render a single tool result
    pub fn render_result(&self, result: &ToolResult) -> Vec<String> {
        let mut lines = Vec::new();

        // Header with status
        let (status_color, status_text) = if result.is_error {
            (colors::ERROR, "error")
        } else {
            (colors::SUCCESS, "ok")
        };

        lines.push(format!(
            "{}{}[{}] {}{} ({}){}",
            colors::TOOL_ID,
            "╭",
            result.tool_call_id,
            status_color,
            status_text,
            self.truncate_string(&result.output, 50),
            colors::RESET
        ));

        // Output
        let output_lines: Vec<&str> = result.output.lines().collect();
        let display_lines = if self.truncate_output && output_lines.len() > self.max_output_lines {
            let half = self.max_output_lines / 2;
            let first: Vec<&str> = output_lines.iter().take(half).cloned().collect();
            let last: Vec<&str> = output_lines
                .iter()
                .skip(output_lines.len() - half)
                .cloned()
                .collect();
            let mut result = first;
            result.push(&"[...truncated...]");
            result.extend(last);
            result
        } else {
            output_lines
        };

        for line in display_lines {
            let line_color = if result.is_error {
                colors::ERROR
            } else {
                colors::TOOL_OUTPUT
            };
            lines.push(format!("{}  {}{}", line_color, line, colors::RESET));
        }

        // Footer
        lines.push(format!(
            "{}{}{}",
            colors::TOOL_ID,
            "╰─────────────────────────────────",
            colors::RESET
        ));

        lines
    }

    /// Render tool call and result together
    pub fn render_call_with_result(
        &self,
        tool_call: &ToolCall,
        result: &ToolResult,
    ) -> Vec<String> {
        let mut lines = self.render_call(tool_call);
        lines.push(String::new());
        lines.extend(self.render_result(result));
        lines
    }

    /// Format JSON with basic indentation
    fn format_json(&self, input: &str) -> Vec<String> {
        // Try to pretty-print JSON
        let mut lines = Vec::new();

        // Simple JSON formatting
        let mut indent = 0;
        let mut in_string = false;
        let mut result = String::new();

        for ch in input.chars() {
            match ch {
                '"' => {
                    in_string = !in_string;
                    result.push(ch);
                }
                '{' | '[' if !in_string => {
                    result.push(ch);
                    indent += 1;
                    lines.push(result);
                    result = "  ".repeat(indent);
                }
                '}' | ']' if !in_string => {
                    indent = indent.saturating_sub(1);
                    lines.push(result);
                    result = "  ".repeat(indent);
                    result.push(ch);
                }
                ',' if !in_string => {
                    result.push(ch);
                    lines.push(result);
                    result = "  ".repeat(indent);
                }
                ':' if !in_string => {
                    result.push_str(": ");
                }
                _ => {
                    result.push(ch);
                }
            }
        }

        if !result.trim().is_empty() {
            lines.push(result);
        }

        if lines.is_empty() {
            vec![input.to_string()]
        } else {
            lines
        }
    }

    /// Truncate a string with ellipsis
    fn truncate_string(&self, s: &str, max_len: usize) -> String {
        if s.len() <= max_len {
            s.to_string()
        } else {
            format!("{}...", &s[..max_len.saturating_sub(3)])
        }
    }

    /// Render multiple tool calls with results
    pub fn render_tool_stream(
        &self,
        calls_and_results: Vec<(ToolCall, ToolResult)>,
    ) -> Vec<String> {
        let mut lines = Vec::new();

        lines.push(format!(
            "{}{}Tool Execution{}{}",
            colors::HEADER,
            colors::BOLD,
            colors::RESET,
            colors::DIM
        ));
        lines.push(format!("{}{}", "─".repeat(40), colors::RESET));
        lines.push(String::new());

        for (i, (call, result)) in calls_and_results.into_iter().enumerate() {
            if i > 0 {
                lines.push(String::new());
            }
            lines.extend(self.render_call_with_result(&call, &result));
        }

        lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_tool_call() {
        let renderer = ToolRenderer::default();
        let tool_call = ToolCall {
            id: "tool_001".to_string(),
            name: "read_file".to_string(),
            input: r#"{"path":"/tmp/test.txt"}"#.to_string(),
        };

        let rendered = renderer.render_call(&tool_call);
        assert!(!rendered.is_empty());
        assert!(rendered[0].contains("tool_001"));
        assert!(rendered[0].contains("read_file"));
    }

    #[test]
    fn test_render_tool_result() {
        let renderer = ToolRenderer::default();
        let result = ToolResult {
            tool_call_id: "tool_001".to_string(),
            output: "Hello, World!".to_string(),
            is_error: false,
        };

        let rendered = renderer.render_result(&result);
        assert!(!rendered.is_empty());
        assert!(rendered[0].contains("ok"));
    }

    #[test]
    fn test_render_error_result() {
        let renderer = ToolRenderer::default();
        let result = ToolResult {
            tool_call_id: "tool_001".to_string(),
            output: "File not found".to_string(),
            is_error: true,
        };

        let rendered = renderer.render_result(&result);
        assert!(rendered[0].contains("error"));
    }

    #[test]
    fn test_truncate() {
        let renderer = ToolRenderer::default();
        assert_eq!(renderer.truncate_string("hello", 10), "hello");
        assert_eq!(renderer.truncate_string("hello world", 8), "hello...");
    }
}
