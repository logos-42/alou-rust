//! TUI Application
//!
//! Main TUI application that orchestrates all components.

use std::io::{self, Write};
use std::sync::{Arc, Mutex};

use crate::tui::components::{
    chat::{colors as chat_colors, ChatRenderer},
    markdown::{colors as md_colors, MarkdownRenderer},
    prompt::Prompt,
    status::{colors as status_colors, ConnectionStatus, StatusBar, StatusInfo},
    tool::ToolRenderer,
};
use crate::tui::layout::{Rect, TerminalSize};
use crate::tui::state::{Message, MessageRole, RenderMode, SharedTuiState, TuiState};

/// ANSI escape codes for TUI operations
pub mod ansi {
    pub use crate::tui::layout::ansi::*;
}

/// Main TUI Application
pub struct TuiApp {
    state: SharedTuiState,
    terminal_size: TerminalSize,
    prompt: Prompt,
    chat_renderer: ChatRenderer,
    status_bar: StatusBar,
    tool_renderer: ToolRenderer,
    md_renderer: MarkdownRenderer,
    scroll_offset: usize,
}

impl TuiApp {
    /// Create a new TUI application
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(TuiState::new())),
            terminal_size: TerminalSize::default(),
            prompt: Prompt::new("alou"),
            chat_renderer: ChatRenderer::default(),
            status_bar: StatusBar::new(80),
            tool_renderer: ToolRenderer::default(),
            md_renderer: MarkdownRenderer::default(),
            scroll_offset: 0,
        }
    }

    /// Create with custom state
    pub fn with_state(state: SharedTuiState) -> Self {
        Self {
            state,
            terminal_size: TerminalSize::default(),
            prompt: Prompt::new("alou"),
            chat_renderer: ChatRenderer::default(),
            status_bar: StatusBar::new(80),
            tool_renderer: ToolRenderer::default(),
            md_renderer: MarkdownRenderer::default(),
            scroll_offset: 0,
        }
    }

    /// Get shared state
    pub fn state(&self) -> &SharedTuiState {
        &self.state
    }

    /// Get terminal size
    pub fn terminal_size(&self) -> TerminalSize {
        self.terminal_size
    }

    /// Set terminal size
    pub fn set_terminal_size(&mut self, width: u16, height: u16) {
        self.terminal_size = TerminalSize::new(width, height);
        self.status_bar = StatusBar::new(width);
    }

    /// Add a user message
    pub fn add_user_message(&self, content: String) {
        if let Ok(mut state) = self.state.lock() {
            state.add_user_message(content);
        }
    }

    /// Add an assistant message
    pub fn add_assistant_message(&self, content: String) {
        if let Ok(mut state) = self.state.lock() {
            state.add_assistant_message(content);
        }
    }

    /// Add a tool call
    pub fn add_tool_call(&self, id: String, name: String, input: String) {
        if let Ok(mut state) = self.state.lock() {
            state.add_tool_call(id, name, input);
        }
    }

    /// Add a tool result
    pub fn add_tool_result(&self, tool_call_id: String, output: String, is_error: bool) {
        if let Ok(mut state) = self.state.lock() {
            state.add_tool_result(tool_call_id, output, is_error);
        }
    }

    /// Set processing state
    pub fn set_processing(&self, processing: bool) {
        if let Ok(mut state) = self.state.lock() {
            state.set_processing(processing);
        }
    }

    /// Set thinking state
    pub fn set_thinking(&self, visible: bool, content: Option<String>) {
        if let Ok(mut state) = self.state.lock() {
            state.set_thinking(visible, content);
        }
    }

    /// Cycle render mode
    pub fn cycle_render_mode(&self) {
        if let Ok(mut state) = self.state.lock() {
            state.cycle_render_mode();
        }
    }

    /// Clear the conversation
    pub fn clear_conversation(&self) {
        if let Ok(mut state) = self.state.lock() {
            *state = TuiState::new();
        }
    }

    /// Get current input
    pub fn get_input(&self) -> String {
        self.prompt.input().to_string()
    }

    /// Handle a key press
    pub fn handle_key(&mut self, key: KeyEvent) -> Option<String> {
        match key {
            KeyEvent::Char(c) => {
                self.prompt.insert_char(c);
                None
            }
            KeyEvent::Enter => self.prompt.submit(),
            KeyEvent::Backspace => {
                self.prompt.delete_before_cursor();
                None
            }
            KeyEvent::Delete => {
                self.prompt.delete_after_cursor();
                None
            }
            KeyEvent::Left => {
                self.prompt.move_cursor_left();
                None
            }
            KeyEvent::Right => {
                self.prompt.move_cursor_right();
                None
            }
            KeyEvent::Home => {
                self.prompt.move_cursor_to_start();
                None
            }
            KeyEvent::End => {
                self.prompt.move_cursor_to_end();
                None
            }
            KeyEvent::Ctrl('c') => {
                self.prompt.clear();
                None
            }
            KeyEvent::Ctrl('l') => {
                self.scroll_offset = 0;
                None
            }
            KeyEvent::Ctrl('r') => {
                self.cycle_render_mode();
                None
            }
            KeyEvent::Up => {
                self.prompt.history_up();
                None
            }
            KeyEvent::Down => {
                self.prompt.history_down();
                None
            }
            KeyEvent::PageUp => {
                if let Ok(mut state) = self.state.lock() {
                    state.scroll_up();
                }
                None
            }
            KeyEvent::PageDown => {
                if let Ok(mut state) = self.state.lock() {
                    state.scroll_down();
                }
                None
            }
            KeyEvent::Tab => None,
            KeyEvent::Ctrl('k') => {
                self.prompt.move_cursor_to_end();
                None
            }
            KeyEvent::Ctrl('u') => {
                let cursor = self.prompt.cursor_position();
                let input = self.prompt.input().to_string();
                self.prompt.set_input(input.chars().skip(cursor).collect());
                None
            }
            KeyEvent::Ctrl('w') => {
                self.prompt.delete_word_before_cursor();
                None
            }
            KeyEvent::Alt('b') => {
                self.prompt.move_word_left();
                None
            }
            KeyEvent::Alt('f') => {
                self.prompt.move_word_right();
                None
            }
            _ => None,
        }
    }

    /// Render the entire TUI
    pub fn render(&self) -> String {
        let mut output = String::new();
        // Use unwrap_or_else to gracefully recover if the mutex was poisoned by a
        // previous panic. This prevents the render thread from crashing on PoisonError.
        let state = match self.state.lock() {
            Ok(s) => s,
            Err(poisoned) => {
                // Recover from poison: the data is still accessible, just potentially
                // in a partially-updated state. Log and return empty render to avoid
                // cascading panics.
                eprintln!("[WARN] TuiState mutex was poisoned, recovering");
                poisoned.into_inner()
            }
        };

        // Calculate layout
        let full_rect = self.terminal_size.to_rect();
        let (chat_area, status_area) =
            full_rect.split_horizontal(full_rect.height.saturating_sub(3));
        let (messages_area, input_area) =
            chat_area.split_horizontal(chat_area.height.saturating_sub(2));

        // Clear screen
        output.push_str(&ansi::clear_screen());
        output.push_str(&ansi::hide_cursor());

        // Render header
        output.push_str(&self.render_header());
        output.push('\n');

        // Render messages
        let messages = state.messages();
        let render_mode = state.render_mode();
        let visible_height = messages_area.height as usize;
        let total_messages = messages.len();
        let scroll = state.scroll_offset();

        if total_messages > visible_height {
            output.push_str(&ansi::cursor_position(0, messages_area.y));
            output.push_str(&format!(
                "{}({}/{}) scroll{}",
                status_colors::NORMAL,
                scroll,
                total_messages.saturating_sub(visible_height),
                status_colors::RESET
            ));
        }

        // Render visible messages
        let start_idx = scroll.min(total_messages.saturating_sub(1));
        let end_idx = (start_idx + visible_height).min(total_messages);

        output.push_str(&ansi::cursor_position(0, messages_area.y));

        let visible_messages: Vec<Message> = messages
            .iter()
            .skip(start_idx)
            .take(end_idx - start_idx)
            .cloned()
            .collect();
        let rendered = self
            .chat_renderer
            .render_messages(&visible_messages, render_mode);

        for (i, line) in rendered.iter().take(visible_height).enumerate() {
            let y = messages_area.y + i as u16;
            output.push_str(&ansi::cursor_position(0, y));
            output.push_str(&ansi::clear_line());

            let max_width = self.terminal_size.width as usize;
            let display_line: String = line.chars().take(max_width).collect();
            output.push_str(&display_line);
        }

        // Render input separator
        let sep_y = input_area.y.saturating_sub(1);
        output.push_str(&ansi::cursor_position(0, sep_y));
        output.push_str(&ansi::clear_line());
        let sep_len = (self.terminal_size.width as usize).min(60);
        output.push_str(&format!(
            "{}{}{}",
            status_colors::NORMAL,
            "─".repeat(sep_len),
            status_colors::RESET
        ));

        // Render input prompt
        output.push_str(&ansi::cursor_position(0, input_area.y));
        output.push_str(&ansi::clear_line());
        output.push_str(&self.prompt.render(self.terminal_size.width));

        // Render status bar (fixed at bottom)
        output.push_str(&ansi::cursor_position(0, status_area.y));
        output.push_str(&ansi::clear_line());
        let status_info = StatusInfo {
            model: None,
            total_tokens: 0,
            processing: state.is_processing(),
            thinking: state.is_thinking_visible(),
            render_mode: format!("{:?}", render_mode),
            connection_status: ConnectionStatus::Connected,
            elapsed: std::time::Duration::from_secs(0),
            ..Default::default()
        };
        output.push_str(&self.status_bar.render(&status_info));

        output.push_str(&ansi::show_cursor());

        output
    }

    fn render_header(&self) -> String {
        format!(
            "{}{}alou{} - Terminal Interface\n{}",
            chat_colors::BOLD,
            chat_colors::ASSISTANT_PREFIX,
            chat_colors::RESET,
            chat_colors::RESET
        )
    }

    fn render_thinking(&self, content: &str) -> String {
        let lines: Vec<&str> = content.lines().take(5).collect();
        let joined = lines.join(" ");
        let display = if joined.len() > 100 {
            format!("{}...", &joined[..100])
        } else {
            joined
        };

        format!(
            "{}thinking{}: {}{}{}",
            chat_colors::DIM,
            chat_colors::RESET,
            md_colors::ITALIC,
            display,
            md_colors::RESET
        )
    }

    /// Render to stdout and flush
    pub fn render_to_stdout(&self) {
        let output = self.render();
        print!("{}", output);
        io::stdout().flush().unwrap();
    }

    /// Get layout
    pub fn get_layout(&self) -> (Rect, Rect, Rect) {
        let full = self.terminal_size.to_rect();
        let (top, bottom) = full.split_horizontal(full.height.saturating_sub(3));
        let (chat, input) = top.split_horizontal(top.height.saturating_sub(2));
        (top, chat, input)
    }
}

impl Default for TuiApp {
    fn default() -> Self {
        Self::new()
    }
}

/// Key event representation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyEvent {
    Char(char),
    Enter,
    Backspace,
    Delete,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    Tab,
    Esc,
    Ctrl(char),
    Alt(char),
    Unknown,
}

impl KeyEvent {
    /// Parse a key sequence
    pub fn parse(sequence: &str) -> Self {
        match sequence {
            "\r" | "\n" => KeyEvent::Enter,
            "\x7f" | "\x08" => KeyEvent::Backspace,
            "\x1b[D" => KeyEvent::Left,
            "\x1b[C" => KeyEvent::Right,
            "\x1b[A" => KeyEvent::Up,
            "\x1b[B" => KeyEvent::Down,
            "\x1b[H" | "\x1b[1~" => KeyEvent::Home,
            "\x1b[F" | "\x1b[4~" => KeyEvent::End,
            "\x1b[5~" => KeyEvent::PageUp,
            "\x1b[6~" => KeyEvent::PageDown,
            "\t" => KeyEvent::Tab,
            "\x1b" => KeyEvent::Esc,
            _ => {
                if sequence.starts_with("\x1b[") {
                    KeyEvent::Unknown
                } else if sequence.len() == 1 {
                    let c = sequence.chars().next().unwrap();
                    if c.is_control() {
                        KeyEvent::Unknown
                    } else {
                        KeyEvent::Char(c)
                    }
                } else {
                    KeyEvent::Unknown
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_parsing() {
        assert_eq!(KeyEvent::parse("\r"), KeyEvent::Enter);
        assert_eq!(KeyEvent::parse("\n"), KeyEvent::Enter);
        assert_eq!(KeyEvent::parse("\x7f"), KeyEvent::Backspace);
        assert_eq!(KeyEvent::parse("a"), KeyEvent::Char('a'));
        assert_eq!(KeyEvent::parse("\x1b[D"), KeyEvent::Left);
        assert_eq!(KeyEvent::parse("\x1b[C"), KeyEvent::Right);
    }

    #[test]
    fn test_tui_app_creation() {
        let app = TuiApp::new();
        assert_eq!(app.state().lock().unwrap().message_count(), 0);
    }

    #[test]
    fn test_tui_app_messages() {
        let app = TuiApp::new();

        app.add_user_message("Hello".to_string());
        assert_eq!(app.state().lock().unwrap().message_count(), 1);

        app.add_assistant_message("Hi there!".to_string());
        assert_eq!(app.state().lock().unwrap().message_count(), 2);
    }
}
