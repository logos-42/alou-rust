//! Prompt Input Component
//!
//! Handles the input prompt with enhanced line editing and streaming display.

use crate::tui::layout::ansi;

/// Prompt style variants
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PromptStyle {
    /// Minimal style with just prompt text
    Minimal,
    /// Classic bordered style
    Classic,
    /// Modern with rounded corners (Unicode box drawing)
    Modern,
    /// Futuristic with ASCII art styling
    Futuristic,
}

impl Default for PromptStyle {
    fn default() -> Self {
        Self::Modern
    }
}

/// ANSI color codes for enhanced prompt
pub mod colors {
    pub const RESET: &str = "\x1b[0m";
    pub const BOLD: &str = "\x1b[1m";
    pub const DIM: &str = "\x1b[2m";
    pub const REVERSE: &str = "\x1b[7m";
    pub const BLINK: &str = "\x1b[5m";

    // Primary prompt colors
    pub const PROMPT_BG: &str = "\x1b[48;5;236m"; // Dark gray background
    pub const PROMPT_FG: &str = "\x1b[38;5;51m"; // Cyan text
    pub const INPUT_FG: &str = "\x1b[38;5;255m"; // White text
    pub const CURSOR: &str = "\x1b[38;5;208m"; // Orange cursor (flashing)
    pub const CURSOR_BLOCK: &str = "\x1b[48;5;208m"; // Orange block cursor

    // Accent colors
    pub const ACCENT: &str = "\x1b[38;5;141m"; // Purple accent
    pub const SUCCESS: &str = "\x1b[38;5;82m"; // Green
    pub const WARNING: &str = "\x1b[38;5;214m"; // Orange
    pub const ERROR: &str = "\x1b[38;5;196m"; // Red

    // Thinking animation colors
    pub const THINKING_1: &str = "\x1b[38;5;39m"; // Blue
    pub const THINKING_2: &str = "\x1b[38;5;45m"; // Cyan
    pub const THINKING_3: &str = "\x1b[38;5;82m"; // Green
}

/// Represents a prompt with input state
#[derive(Debug)]
pub struct Prompt {
    prompt_text: String,
    input: String,
    cursor_position: usize,
    history: Vec<String>,
    history_index: usize,
    max_history: usize,
    style: PromptStyle,
    multiline_mode: bool,
}

impl Prompt {
    /// Create a new prompt
    pub fn new(prompt_text: &str) -> Self {
        Self {
            prompt_text: prompt_text.to_string(),
            input: String::new(),
            cursor_position: 0,
            history: Vec::new(),
            history_index: 0,
            max_history: 100,
            style: PromptStyle::default(),
            multiline_mode: false,
        }
    }

    /// Create prompt with specific style
    pub fn with_style(prompt_text: &str, style: PromptStyle) -> Self {
        Self {
            prompt_text: prompt_text.to_string(),
            input: String::new(),
            cursor_position: 0,
            history: Vec::new(),
            history_index: 0,
            max_history: 100,
            style,
            multiline_mode: false,
        }
    }

    /// Get the current input
    pub fn input(&self) -> &str {
        &self.input
    }

    /// Get cursor position within input
    pub fn cursor_position(&self) -> usize {
        self.cursor_position
    }

    /// Get the prompt text
    pub fn prompt_text(&self) -> &str {
        &self.prompt_text
    }

    /// Get the style
    pub fn style(&self) -> PromptStyle {
        self.style
    }

    /// Set the input content
    pub fn set_input(&mut self, input: String) {
        self.input = input;
        self.cursor_position = self.input.len();
        self.history_index = self.history.len();
    }

    /// Toggle multiline mode
    pub fn toggle_multiline(&mut self) {
        self.multiline_mode = !self.multiline_mode;
    }

    /// Set style
    pub fn set_style(&mut self, style: PromptStyle) {
        self.style = style;
    }

    /// Insert a character at cursor
    pub fn insert_char(&mut self, c: char) {
        let mut chars: Vec<char> = self.input.chars().collect();
        chars.insert(self.cursor_position, c);
        self.input = chars.into_iter().collect();
        self.cursor_position += 1;
    }

    /// Insert a string at cursor
    pub fn insert_str(&mut self, s: &str) {
        for c in s.chars() {
            self.insert_char(c);
        }
    }

    /// Delete character before cursor
    pub fn delete_before_cursor(&mut self) -> Option<char> {
        if self.cursor_position > 0 {
            let mut chars: Vec<char> = self.input.chars().collect();
            let removed = chars.remove(self.cursor_position - 1);
            self.input = chars.into_iter().collect();
            self.cursor_position -= 1;
            Some(removed)
        } else {
            None
        }
    }

    /// Delete character after cursor
    pub fn delete_after_cursor(&mut self) -> Option<char> {
        if self.cursor_position < self.input.len() {
            let mut chars: Vec<char> = self.input.chars().collect();
            let removed = chars.remove(self.cursor_position);
            self.input = chars.into_iter().collect();
            Some(removed)
        } else {
            None
        }
    }

    /// Move cursor left
    pub fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    /// Move cursor right
    pub fn move_cursor_right(&mut self) {
        if self.cursor_position < self.input.len() {
            self.cursor_position += 1;
        }
    }

    /// Move cursor to start
    pub fn move_cursor_to_start(&mut self) {
        self.cursor_position = 0;
    }

    /// Move cursor to end
    pub fn move_cursor_to_end(&mut self) {
        self.cursor_position = self.input.len();
    }

    /// Move cursor by word
    pub fn move_word_left(&mut self) {
        let mut found_non_space = false;
        while self.cursor_position > 0 {
            self.cursor_position -= 1;
            let c = self.input.chars().nth(self.cursor_position).unwrap();
            if c.is_whitespace() {
                if found_non_space {
                    self.cursor_position += 1;
                    break;
                }
            } else {
                found_non_space = true;
            }
        }
    }

    /// Move cursor forward by word
    pub fn move_word_right(&mut self) {
        let mut found_non_space = false;
        while self.cursor_position < self.input.len() {
            let c = self.input.chars().nth(self.cursor_position).unwrap();
            if c.is_whitespace() {
                if found_non_space {
                    break;
                }
            } else {
                found_non_space = true;
            }
            self.cursor_position += 1;
        }
    }

    /// Delete word before cursor
    pub fn delete_word_before_cursor(&mut self) -> String {
        let start = self.cursor_position;
        self.move_word_left();
        let end = self.cursor_position;
        let deleted: String = self.input.chars().skip(end).take(start - end).collect();
        self.input = format!(
            "{}{}",
            self.input.chars().take(end).collect::<String>(),
            self.input.chars().skip(start).collect::<String>()
        );
        self.cursor_position = end;
        deleted
    }

    /// Clear the input
    pub fn clear(&mut self) {
        self.input.clear();
        self.cursor_position = 0;
    }

    /// Submit the current input (adds to history and returns it)
    pub fn submit(&mut self) -> Option<String> {
        if !self.input.trim().is_empty() {
            let submitted = self.input.clone();
            self.history.push(submitted.clone());
            if self.history.len() > self.max_history {
                self.history.remove(0);
            }
            self.clear();
            self.history_index = self.history.len();
            Some(submitted)
        } else {
            None
        }
    }

    /// Navigate history up
    pub fn history_up(&mut self) {
        if !self.history.is_empty() && self.history_index > 0 {
            self.history_index -= 1;
            self.input = self.history[self.history_index].clone();
            self.cursor_position = self.input.len();
        }
    }

    /// Navigate history down
    pub fn history_down(&mut self) {
        if self.history_index < self.history.len() {
            self.history_index += 1;
            if self.history_index == self.history.len() {
                self.input.clear();
            } else {
                self.input = self.history[self.history_index].clone();
            }
            self.cursor_position = self.input.len();
        }
    }

    /// Get history
    pub fn history(&self) -> &[String] {
        &self.history
    }

    /// Transpose characters (swap chars before and after cursor)
    pub fn transpose_chars(&mut self) {
        if self.cursor_position > 0 && self.cursor_position < self.input.len() {
            let chars: Vec<char> = self.input.chars().collect();
            let c1 = chars[self.cursor_position - 1];
            let c2 = chars[self.cursor_position];
            let mut new_chars = chars.clone();
            new_chars[self.cursor_position - 1] = c2;
            new_chars[self.cursor_position] = c1;
            self.input = new_chars.into_iter().collect();
        } else if self.cursor_position == self.input.len() && self.cursor_position > 0 {
            let chars: Vec<char> = self.input.chars().collect();
            let len = chars.len();
            let c1 = chars[len - 1];
            let c2 = chars[len - 2];
            let mut new_chars = chars.clone();
            new_chars[len - 2] = c1;
            new_chars[len - 1] = c2;
            self.input = new_chars.into_iter().collect();
        }
    }

    /// Render the prompt with styled borders based on style
    fn render_styled(&self, width: u16) -> String {
        match self.style {
            PromptStyle::Minimal => self.render_minimal(width),
            PromptStyle::Classic => self.render_classic(width),
            PromptStyle::Modern => self.render_modern(width),
            PromptStyle::Futuristic => self.render_futuristic(width),
        }
    }

    /// Render minimal style
    fn render_minimal(&self, width: u16) -> String {
        let prompt = format!("{}{}>{} ", colors::PROMPT_FG, colors::BOLD, colors::RESET);
        self.render_input_line(width, &prompt)
    }

    /// Render classic bordered style
    fn render_classic(&self, width: u16) -> String {
        let prompt = format!(
            "{}[{}]>{}{} ",
            colors::ACCENT,
            colors::PROMPT_FG,
            colors::ACCENT,
            colors::RESET
        );

        let border_char = "─";
        let available = width as usize - 4; // Space for borders

        let mut output = String::new();
        output.push_str(&ansi::clear_line());
        output.push_str(&format!(
            "{}{}{}{}{}\n",
            colors::DIM,
            border_char,
            colors::RESET,
            colors::DIM,
            border_char
        ));

        output.push_str(&self.render_input_line(width, &prompt));
        output
    }

    /// Render modern style with Unicode box drawing
    fn render_modern(&self, width: u16) -> String {
        let left_corner = "╭─";
        let right_corner = "─╮";
        let bottom_left = "╰─";
        let bottom_right = "─╯";

        let prompt_text = format!(" {} ", self.prompt_text);
        let input_placeholder = if self.input.is_empty() {
            format!("{}type...{}", colors::DIM, colors::RESET)
        } else {
            self.input.clone()
        };

        let content = format!(
            "{}{}{}{}",
            colors::PROMPT_FG,
            colors::BOLD,
            prompt_text,
            colors::RESET
        );

        // Calculate positions
        let prompt_len = content.len();
        let available = width as usize - prompt_len - 4;

        let visible_input: String = input_placeholder.chars().take(available.max(1)).collect();

        let input_len = visible_input.len();
        let padding = if input_len < available {
            available - input_len
        } else {
            0
        };

        // Build line
        let line = format!(
            "{}{}{}{}{}{}{}",
            colors::PROMPT_FG,
            left_corner,
            colors::RESET,
            content,
            colors::INPUT_FG,
            visible_input,
            colors::RESET
        );

        // Bottom with cursor line
        let cursor_prefix = format!("{}{}{}", colors::PROMPT_FG, bottom_left, colors::RESET);
        let cursor_space = " ".repeat(prompt_len + 1);

        let mut output = String::new();
        output.push_str(&ansi::clear_line());
        output.push_str(&line);
        output.push_str(&format!(
            "{} {}{}{}{}\n",
            colors::PROMPT_FG,
            " ".repeat(padding),
            right_corner,
            colors::RESET,
            colors::DIM
        ));

        // Cursor line with blinking cursor
        output.push_str(&cursor_prefix);
        output.push_str(&colors::DIM);
        output.push_str(&cursor_space);
        output.push_str(&colors::RESET);

        // Show cursor position
        let cursor_x = prompt_len + 2 + self.cursor_position.min(input_len);
        output.push_str(&ansi::cursor_position(cursor_x as u16, 1));
        output.push_str(&format!(
            "{}{} {}{}{}",
            colors::CURSOR_BLOCK,
            colors::RESET,
            colors::DIM,
            bottom_right,
            colors::RESET
        ));

        output
    }

    /// Render futuristic ASCII art style
    fn render_futuristic(&self, width: u16) -> String {
        let prompt_indicator = format!(
            "{}/\\{}[{}]{}/\\{} ",
            colors::ACCENT,
            colors::PROMPT_FG,
            colors::BOLD,
            colors::ACCENT,
            colors::RESET
        );

        let prompt_text = format!("{}alou{}", colors::PROMPT_FG, colors::BOLD);
        let cursor_char = "▋"; // Block cursor

        let mut output = String::new();
        output.push_str(&ansi::clear_line());

        // Top decoration
        let top_line = format!(
            "{}{}{}{}{}\n",
            colors::DIM,
            "═".repeat(width as usize - 20),
            colors::RESET,
            colors::ACCENT,
            " alou "
        );
        output.push_str(&top_line);

        // Input line
        let input_display = if self.input.is_empty() {
            format!(
                "{}{}type something...{}",
                colors::DIM,
                cursor_char,
                colors::RESET
            )
        } else {
            let (before, after) = self.input.split_at(self.cursor_position);
            format!(
                "{}{}{}{}{}{}{}",
                colors::INPUT_FG,
                before,
                colors::CURSOR,
                cursor_char,
                colors::RESET,
                colors::INPUT_FG,
                after
            )
        };

        let line = format!("{}│ {} │{}\n", colors::DIM, prompt_text, colors::RESET);
        output.push_str(&line);

        // Bottom decoration
        output.push_str(&format!(
            "{}{}{}{}{}\n",
            colors::DIM,
            "═".repeat(width as usize - 20),
            colors::RESET,
            colors::ACCENT,
            "══════"
        ));

        // Return cursor to input line
        output.push_str(&ansi::cursor_position(4, 1));
        output.push_str(&input_display);

        output
    }

    /// Render input line helper
    fn render_input_line(&self, width: u16, prompt: &str) -> String {
        let prompt_len = prompt.len();
        let input_len = self.input.len();

        // Calculate visible portion based on cursor position
        let cursor_col = prompt_len + self.cursor_position;

        // If cursor is beyond visible width, we need to scroll
        let display_offset = if cursor_col > width as usize {
            cursor_col - (width as usize - 1)
        } else {
            0
        };

        let visible_input: String = self
            .input
            .chars()
            .skip(display_offset)
            .take(width as usize - prompt_len - 1)
            .collect();

        // Calculate cursor visual position
        let visual_cursor_pos = if cursor_col > width as usize {
            (width as usize - 1) - prompt_len
        } else {
            self.cursor_position - display_offset
        };

        let line = format!("{}{}", prompt, visible_input);

        // Pad to width
        let padding = if line.len() < width as usize {
            " ".repeat(width as usize - line.len())
        } else {
            String::new()
        };

        // Build output with cursor
        let mut output = String::new();
        output.push_str(&line);
        output.push_str(&padding);

        // Position cursor
        let final_x = prompt_len + visual_cursor_pos;
        output.push_str(&ansi::cursor_position(final_x as u16, 0));

        output
    }

    /// Render the prompt with input to a string
    pub fn render(&self, width: u16) -> String {
        self.render_styled(width)
    }

    /// Get a preview of the prompt without cursor positioning
    /// Useful for multiline editing
    pub fn render_preview(&self, max_lines: usize) -> String {
        let lines: Vec<&str> = self.input.lines().take(max_lines).collect();
        lines.join("\n")
    }
}

impl Default for Prompt {
    fn default() -> Self {
        Self::new("alou")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_input() {
        let mut prompt = Prompt::new("test");
        prompt.insert_char('h');
        prompt.insert_char('i');
        assert_eq!(prompt.input(), "hi");
        assert_eq!(prompt.cursor_position(), 2);
    }

    #[test]
    fn test_delete() {
        let mut prompt = Prompt::new("test");
        prompt.set_input("hello".to_string());
        prompt.move_cursor_left();
        prompt.move_cursor_left();
        prompt.delete_before_cursor();
        assert_eq!(prompt.input(), "helo");
    }

    #[test]
    fn test_history() {
        let mut prompt = Prompt::new("test");
        prompt.set_input("first".to_string());
        prompt.submit();
        prompt.set_input("second".to_string());
        prompt.submit();

        assert_eq!(prompt.history().len(), 2);

        prompt.history_up();
        assert_eq!(prompt.input(), "second");
        prompt.history_up();
        assert_eq!(prompt.input(), "first");
        prompt.history_down();
        assert_eq!(prompt.input(), "second");
        prompt.history_down();
        assert_eq!(prompt.input(), "");
    }

    #[test]
    fn test_transpose() {
        let mut prompt = Prompt::new("test");
        prompt.set_input("ab".to_string());
        prompt.move_cursor_to_end();
        prompt.transpose_chars();
        assert_eq!(prompt.input(), "ba");
    }
}
