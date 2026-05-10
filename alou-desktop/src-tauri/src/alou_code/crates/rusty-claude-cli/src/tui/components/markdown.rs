//! Markdown Rendering Component
//!
//! Provides basic markdown rendering for assistant messages.

/// ANSI colors for markdown rendering
pub mod colors {
    pub const RESET: &str = "\x1b[0m";
    pub const BOLD: &str = "\x1b[1m";
    pub const DIM: &str = "\x1b[2m";
    pub const ITALIC: &str = "\x1b[3m";
    pub const UNDERLINE: &str = "\x1b[4m";

    pub const HEADER: &str = "\x1b[38;5;39m"; // Blue
    pub const CODE: &str = "\x1b[38;5;245m"; // Gray
    pub const CODE_BG: &str = "\x1b[48;5;236m"; // Dark gray bg
    pub const LINK: &str = "\x1b[38;5;75m"; // Blue link
    pub const QUOTE: &str = "\x1b[38;5;240m"; // Gray
    pub const LIST: &str = "\x1b[38;5;208m"; // Orange bullet
    pub const HR: &str = "\x1b[38;5;240m"; // Gray
    pub const TABLE_HEADER: &str = "\x1b[1m\x1b[38;5;39m";
    pub const TABLE_BORDER: &str = "\x1b[38;5;240m";
}

/// Markdown renderer configuration
#[derive(Debug, Clone)]
pub struct MarkdownConfig {
    pub max_width: usize,
    pub code_block_theme: CodeBlockTheme,
    pub link_style: LinkStyle,
}

impl Default for MarkdownConfig {
    fn default() -> Self {
        Self {
            max_width: 120,
            code_block_theme: CodeBlockTheme::Monokai,
            link_style: LinkStyle::Text,
        }
    }
}

/// Code block color theme
#[derive(Debug, Clone, Copy)]
pub enum CodeBlockTheme {
    Monokai,
    Dracula,
    OneDark,
    Plain,
}

impl CodeBlockTheme {
    fn get_colors(&self) -> (&str, &[(&str, &str)], &str) {
        match self {
            CodeBlockTheme::Monokai => (
                "\x1b[48;5;235m",
                &[
                    ("keyword", "\x1b[38;5;203m"), // pink
                    ("string", "\x1b[38;5;230m"),  // yellow
                    ("comment", "\x1b[38;5;245m"), // gray
                    ("number", "\x1b[38;5;197m"),  // magenta
                    ("function", "\x1b[38;5;81m"), // cyan
                    ("type", "\x1b[38;5;75m"),     // blue
                ],
                "\x1b[0m",
            ),
            CodeBlockTheme::Dracula => (
                "\x1b[48;5;234m",
                &[
                    ("keyword", "\x1b[38;5;199m"),
                    ("string", "\x1b[38;5;165m"),
                    ("comment", "\x1b[38;5;139m"),
                    ("number", "\x1b[38;5;189m"),
                    ("function", "\x1b[38;5;111m"),
                    ("type", "\x1b[38;5;117m"),
                ],
                "\x1b[0m",
            ),
            CodeBlockTheme::OneDark => (
                "\x1b[48;5;234m",
                &[
                    ("keyword", "\x1b[38;5;168m"),
                    ("string", "\x1b[38;5;142m"),
                    ("comment", "\x1b[38;5;245m"),
                    ("number", "\x1b[38;5;209m"),
                    ("function", "\x1b[38;5;81m"),
                    ("type", "\x1b[38;5;109m"),
                ],
                "\x1b[0m",
            ),
            CodeBlockTheme::Plain => ("\x1b[48;5;236m", &[], "\x1b[0m"),
        }
    }
}

/// Link display style
#[derive(Debug, Clone, Copy)]
pub enum LinkStyle {
    /// Show link text only
    Text,
    /// Show link text with URL
    TextAndUrl,
    /// Show URL only
    Url,
}

/// Markdown renderer
pub struct MarkdownRenderer {
    config: MarkdownConfig,
}

impl MarkdownRenderer {
    /// Create a new markdown renderer
    pub fn new() -> Self {
        Self {
            config: MarkdownConfig::default(),
        }
    }

    /// Create with custom config
    pub fn with_config(config: MarkdownConfig) -> Self {
        Self { config }
    }

    /// Render markdown text to ANSI-escaped strings
    pub fn render(&self, text: &str) -> Vec<String> {
        let mut lines = Vec::new();
        let mut in_code_block = false;
        let mut code_block_lang = String::new();
        let code_block_lines: Vec<String>;

        // Pre-process code blocks
        let processed: Vec<&str> = text.lines().collect();
        let mut i = 0;

        while i < processed.len() {
            let line = processed[i];

            // Code block detection
            if line.starts_with("```") {
                if !in_code_block {
                    in_code_block = true;
                    code_block_lang = line.trim_start_matches("```").trim().to_string();
                    lines.push(format!(
                        "{}┌─{}─{}",
                        colors::CODE_BG,
                        colors::CODE,
                        code_block_lang
                    ));
                } else {
                    lines.push(format!(
                        "{}└{}─{}",
                        colors::CODE_BG,
                        colors::RESET,
                        "─".repeat(40)
                    ));
                    in_code_block = false;
                    code_block_lang.clear();
                }
                i += 1;
                continue;
            }

            if in_code_block {
                lines.push(format!("{}{}{}", colors::CODE_BG, colors::CODE, line));
                i += 1;
                continue;
            }

            // Headers
            if line.starts_with("# ") {
                lines.push(self.render_header(line, 1));
            } else if line.starts_with("## ") {
                lines.push(self.render_header(line, 2));
            } else if line.starts_with("### ") {
                lines.push(self.render_header(line, 3));
            } else if line.starts_with("#### ") {
                lines.push(self.render_header(line, 4));
            }
            // Horizontal rule
            else if line == "---" || line == "***" || line == "___" {
                lines.push(format!(
                    "{}{}",
                    colors::HR,
                    "─".repeat(60.min(self.config.max_width))
                ));
            }
            // Blockquote
            else if line.starts_with("> ") {
                lines.push(self.render_quote(line));
            }
            // Unordered list
            else if line.starts_with("- ") || line.starts_with("* ") {
                lines.push(self.render_list_item(line));
            }
            // Ordered list
            else if line
                .chars()
                .next()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
                && line.contains(". ")
            {
                lines.push(self.render_list_item(line));
            }
            // Regular paragraph
            else if !line.is_empty() {
                lines.push(self.render_inline(line));
            }
            // Empty line
            else {
                lines.push(String::new());
            }

            i += 1;
        }

        // Close any unclosed code block
        if in_code_block {
            lines.push(format!(
                "{}└{}─{}",
                colors::CODE_BG,
                colors::RESET,
                "─".repeat(40)
            ));
        }

        lines.push(colors::RESET.to_string());
        lines
    }

    /// Render a header line
    fn render_header(&self, line: &str, level: usize) -> String {
        let content = line.trim_start_matches(|c| c == '#').trim();
        let prefix = match level {
            1 => "══",
            2 => "──",
            3 => "┄┄",
            _ => "  ",
        };
        format!(
            "{}{}{}{}{}{}",
            colors::HEADER,
            colors::BOLD,
            prefix,
            colors::RESET,
            colors::HEADER,
            content
        )
    }

    /// Render a blockquote
    fn render_quote(&self, line: &str) -> String {
        let content = line.trim_start_matches("> ").trim_start_matches(">");
        format!(
            "{}│{}{} {}",
            colors::QUOTE,
            colors::ITALIC,
            colors::DIM,
            content
        )
    }

    /// Render a list item
    fn render_list_item(&self, line: &str) -> String {
        let content = line.trim_start_matches(|c| c == '-' || c == '*').trim();
        format!(
            "{}•{} {}",
            colors::LIST,
            colors::RESET,
            self.render_inline(content)
        )
    }

    /// Render inline elements (bold, italic, code, links)
    fn render_inline(&self, text: &str) -> String {
        let mut result = text.to_string();

        // Bold: **text** or __text__
        while let Some(start) = result.find("**") {
            if let Some(end) = result[start + 2..].find("**") {
                let before = &result[..start];
                let content = &result[start + 2..start + 2 + end];
                let after = &result[start + 2 + end + 2..];
                result = format!(
                    "{}{}{}{}{}",
                    before,
                    colors::BOLD,
                    content,
                    colors::RESET,
                    after
                );
            } else {
                break;
            }
        }

        // Inline code: `code`
        while let Some(start) = result.find('`') {
            if let Some(end) = result[start + 1..].find('`') {
                let before = &result[..start];
                let content = &result[start + 1..start + 1 + end];
                let after = &result[start + 1 + end + 1..];
                result = format!(
                    "{}{}{}{}{}",
                    before,
                    colors::CODE,
                    content,
                    colors::RESET,
                    after
                );
            } else {
                break;
            }
        }

        // Italic: *text* or _text_ (not inside words)
        result = self.render_italic(&result);

        result
    }

    /// Render italic text
    fn render_italic(&self, text: &str) -> String {
        // Match *text* but not **text**
        let mut i = 0;
        let chars: Vec<char> = text.chars().collect();
        let mut output = String::new();
        let mut in_italic = false;

        while i < chars.len() {
            let remaining = chars.len() - i;

            // Check for ** (bold)
            if remaining >= 2 && chars[i] == '*' && chars[i + 1] == '*' {
                output.push_str("**");
                i += 2;
                continue;
            }

            // Check for * (italic)
            if chars[i] == '*' {
                if in_italic {
                    output.push_str(colors::RESET);
                } else {
                    output.push_str(colors::ITALIC);
                }
                in_italic = !in_italic;
                i += 1;
                continue;
            }

            output.push(chars[i]);
            i += 1;
        }

        output
    }
}

impl Default for MarkdownRenderer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_header() {
        let renderer = MarkdownRenderer::default();
        let lines = renderer.render("# Hello World");
        assert!(!lines.is_empty());
        assert!(lines[0].contains("Hello World"));
    }

    #[test]
    fn test_render_code_block() {
        let renderer = MarkdownRenderer::default();
        let md = "```rust\nfn main() {}\n```";
        let lines = renderer.render(md);
        assert!(lines.iter().any(|l| l.contains("fn main()")));
    }

    #[test]
    fn test_render_list() {
        let renderer = MarkdownRenderer::default();
        let md = "- Item 1\n- Item 2";
        let lines = renderer.render(md);
        assert!(lines.iter().any(|l| l.contains("Item 1")));
        assert!(lines.iter().any(|l| l.contains("Item 2")));
    }

    #[test]
    fn test_render_inline_code() {
        let renderer = MarkdownRenderer::default();
        let lines = renderer.render("Use `cargo run` to start");
        assert!(lines.iter().any(|l| l.contains("cargo run")));
    }
}
