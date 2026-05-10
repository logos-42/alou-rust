//! TUI State Management
//!
//! Manages the state for the terminal UI including:
//! - Conversation history
//! - Current input buffer
//! - Render mode (compact/expanded)
//! - Scroll position
//! - Tool execution state
//! - Streaming response state
//! - Token and cost tracking

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Token consumption rates (USD per 1K tokens)
#[derive(Debug, Clone, Copy)]
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

/// Token tracking for current session
#[derive(Debug, Clone, Default)]
pub struct TokenStats {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub total_cost: f64,
}

impl TokenStats {
    /// Update with new token counts
    pub fn update(&mut self, input: u32, output: u32, rates: TokenRates) {
        self.input_tokens = input;
        self.output_tokens = output;
        self.total_cost = rates.calculate_cost(input, output);
    }

    /// Add tokens (for streaming updates)
    pub fn add_output(&mut self, additional: u32, rates: TokenRates) {
        self.output_tokens += additional;
        self.total_cost = rates.calculate_cost(self.input_tokens, self.output_tokens);
    }

    /// Get total tokens
    pub fn total(&self) -> u32 {
        self.input_tokens + self.output_tokens
    }
}

/// Represents a single message in the conversation
#[derive(Debug, Clone)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
    pub timestamp: std::time::SystemTime,
    pub tool_calls: Vec<ToolCall>,
    pub tool_results: Vec<ToolResult>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageRole {
    User,
    Assistant,
    System,
    Tool,
}

/// A tool call made by the assistant
#[derive(Debug, Clone)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub input: String,
}

/// A result from a tool execution
#[derive(Debug, Clone)]
pub struct ToolResult {
    pub tool_call_id: String,
    pub output: String,
    pub is_error: bool,
}

/// The mode for rendering assistant messages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RenderMode {
    #[default]
    Compact,
    Expanded,
    SyntaxHighlighted,
}

/// Streaming state for real-time response display
#[derive(Debug, Clone)]
pub struct StreamingState {
    pub active: bool,
    pub content: String,
    pub frame: usize,
    pub start_time: Option<Instant>,
    pub chars_received: usize,
    pub chars_per_second: f64,
}

impl Default for StreamingState {
    fn default() -> Self {
        Self {
            active: false,
            content: String::new(),
            frame: 0,
            start_time: None,
            chars_received: 0,
            chars_per_second: 0.0,
        }
    }
}

impl StreamingState {
    /// Start streaming
    pub fn start(&mut self) {
        self.active = true;
        self.content.clear();
        self.frame = 0;
        self.start_time = Some(Instant::now());
        self.chars_received = 0;
        self.chars_per_second = 0.0;
    }

    /// Append content to streaming
    pub fn append(&mut self, chunk: &str) {
        if !self.active {
            self.start();
        }

        let chunk_len = chunk.chars().count();
        self.content.push_str(chunk);
        self.chars_received += chunk_len;

        // Calculate rate
        if let Some(start) = self.start_time {
            let elapsed = start.elapsed().as_secs_f64();
            if elapsed > 0.0 {
                self.chars_per_second = self.chars_received as f64 / elapsed;
            }
        }

        // Advance animation frame
        self.frame = (self.frame + 1) % 4;
    }

    /// Finish streaming
    pub fn finish(&mut self) {
        self.active = false;
    }

    /// Tick animation frame (called periodically)
    pub fn tick(&mut self) {
        self.frame = (self.frame + 1) % 4;
    }
}

/// Main TUI state
#[derive(Debug)]
pub struct TuiState {
    messages: VecDeque<Message>,
    input_buffer: String,
    cursor_position: usize,
    scroll_offset: usize,
    render_mode: RenderMode,
    is_processing: bool,
    thinking_visible: bool,
    thinking_content: Option<String>,
    current_turn_tool_calls: Vec<ToolCall>,
    // Streaming support
    streaming: StreamingState,
    // Token tracking
    token_stats: TokenStats,
    token_rates: TokenRates,
}

/// Thread-safe wrapper for TUI state
pub type SharedTuiState = Arc<Mutex<TuiState>>;

impl TuiState {
    /// Create a new TUI state
    pub fn new() -> Self {
        Self {
            messages: VecDeque::new(),
            input_buffer: String::new(),
            cursor_position: 0,
            scroll_offset: 0,
            render_mode: RenderMode::default(),
            is_processing: false,
            thinking_visible: false,
            thinking_content: None,
            current_turn_tool_calls: Vec::new(),
            streaming: StreamingState::default(),
            token_stats: TokenStats::default(),
            token_rates: TokenRates::default(),
        }
    }

    /// Add a user message to the conversation
    pub fn add_user_message(&mut self, content: String) {
        self.messages.push_back(Message {
            role: MessageRole::User,
            content,
            timestamp: std::time::SystemTime::now(),
            tool_calls: Vec::new(),
            tool_results: Vec::new(),
        });
    }

    /// Add an assistant message to the conversation
    pub fn add_assistant_message(&mut self, content: String) {
        self.messages.push_back(Message {
            role: MessageRole::Assistant,
            content,
            timestamp: std::time::SystemTime::now(),
            tool_calls: std::mem::take(&mut self.current_turn_tool_calls),
            tool_results: Vec::new(),
        });
    }

    /// Add a tool call to the current turn
    pub fn add_tool_call(&mut self, id: String, name: String, input: String) {
        self.current_turn_tool_calls
            .push(ToolCall { id, name, input });
    }

    /// Add a tool result
    pub fn add_tool_result(&mut self, tool_call_id: String, output: String, is_error: bool) {
        // Find the last assistant message and add the result to it
        if let Some(msg) = self.messages.back_mut() {
            if msg.role == MessageRole::Assistant {
                msg.tool_results.push(ToolResult {
                    tool_call_id,
                    output,
                    is_error,
                });
            }
        }
    }

    /// Set thinking content (for models with extended thinking)
    pub fn set_thinking(&mut self, visible: bool, content: Option<String>) {
        self.thinking_visible = visible;
        self.thinking_content = content;
    }

    /// Set processing state
    pub fn set_processing(&mut self, processing: bool) {
        self.is_processing = processing;
    }

    /// Update input buffer
    pub fn set_input(&mut self, input: String) {
        self.input_buffer = input;
        self.cursor_position = self.input_buffer.len();
    }

    /// Insert a character at the current cursor position
    pub fn insert_char(&mut self, c: char) {
        let mut chars: Vec<char> = self.input_buffer.chars().collect();
        chars.insert(self.cursor_position, c);
        self.input_buffer = chars.into_iter().collect();
        self.cursor_position += 1;
    }

    /// Delete character before cursor
    pub fn delete_before_cursor(&mut self) {
        if self.cursor_position > 0 {
            let mut chars: Vec<char> = self.input_buffer.chars().collect();
            chars.remove(self.cursor_position - 1);
            self.input_buffer = chars.into_iter().collect();
            self.cursor_position -= 1;
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
        if self.cursor_position < self.input_buffer.len() {
            self.cursor_position += 1;
        }
    }

    /// Move cursor to start of line
    pub fn move_cursor_to_start(&mut self) {
        self.cursor_position = 0;
    }

    /// Move cursor to end of line
    pub fn move_cursor_to_end(&mut self) {
        self.cursor_position = self.input_buffer.len();
    }

    /// Clear the input buffer
    pub fn clear_input(&mut self) {
        self.input_buffer.clear();
        self.cursor_position = 0;
    }

    /// Cycle through render modes
    pub fn cycle_render_mode(&mut self) {
        self.render_mode = match self.render_mode {
            RenderMode::Compact => RenderMode::Expanded,
            RenderMode::Expanded => RenderMode::SyntaxHighlighted,
            RenderMode::SyntaxHighlighted => RenderMode::Compact,
        };
    }

    /// Scroll up in the message history
    pub fn scroll_up(&mut self) {
        let max_scroll = self.messages.len().saturating_sub(1);
        self.scroll_offset = self.scroll_offset.saturating_add(1).min(max_scroll);
    }

    /// Scroll down in the message history
    pub fn scroll_down(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    /// Reset scroll to bottom
    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset = 0;
    }

    /// Get all messages for rendering
    pub fn messages(&self) -> &VecDeque<Message> {
        &self.messages
    }

    /// Get current input buffer
    pub fn input(&self) -> &str {
        &self.input_buffer
    }

    /// Get cursor position
    pub fn cursor_position(&self) -> usize {
        self.cursor_position
    }

    /// Get current render mode
    pub fn render_mode(&self) -> RenderMode {
        self.render_mode
    }

    /// Check if currently processing
    pub fn is_processing(&self) -> bool {
        self.is_processing
    }

    /// Check if thinking is visible
    pub fn is_thinking_visible(&self) -> bool {
        self.thinking_visible
    }

    /// Get thinking content
    pub fn thinking_content(&self) -> Option<&String> {
        self.thinking_content.as_ref()
    }

    /// Get scroll offset
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    /// Get current turn tool calls
    pub fn current_turn_tool_calls(&self) -> &[ToolCall] {
        &self.current_turn_tool_calls
    }

    /// Get total message count
    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    // ============ Streaming API ============

    /// Start streaming a response
    pub fn start_streaming(&mut self) {
        self.streaming.start();
        self.is_processing = true;
    }

    /// Append streaming content chunk
    pub fn append_streaming(&mut self, chunk: &str) {
        self.streaming.append(chunk);
    }

    /// Finish streaming
    pub fn finish_streaming(&mut self) {
        self.streaming.finish();
        self.is_processing = false;
    }

    /// Tick animation frame
    pub fn tick_streaming(&mut self) {
        self.streaming.tick();
    }

    /// Get streaming state
    pub fn streaming(&self) -> &StreamingState {
        &self.streaming
    }

    /// Is currently streaming
    pub fn is_streaming(&self) -> bool {
        self.streaming.active
    }

    /// Get streaming content
    pub fn streaming_content(&self) -> &str {
        &self.streaming.content
    }

    /// Get streaming animation frame
    pub fn streaming_frame(&self) -> usize {
        self.streaming.frame
    }

    /// Get streaming rate (chars per second)
    pub fn streaming_rate(&self) -> f64 {
        self.streaming.chars_per_second
    }

    // ============ Token Stats API ============

    /// Update token stats
    pub fn update_tokens(&mut self, input: u32, output: u32) {
        self.token_stats.update(input, output, self.token_rates);
    }

    /// Add output tokens
    pub fn add_output_tokens(&mut self, additional: u32) {
        self.token_stats.add_output(additional, self.token_rates);
    }

    /// Get token stats
    pub fn token_stats(&self) -> &TokenStats {
        &self.token_stats
    }

    /// Get input tokens
    pub fn input_tokens(&self) -> u32 {
        self.token_stats.input_tokens
    }

    /// Get output tokens
    pub fn output_tokens(&self) -> u32 {
        self.token_stats.output_tokens
    }

    /// Get total tokens
    pub fn total_tokens(&self) -> u32 {
        self.token_stats.total()
    }

    /// Get total cost
    pub fn total_cost(&self) -> f64 {
        self.token_stats.total_cost
    }

    /// Set token rates
    pub fn set_token_rates(&mut self, rates: TokenRates) {
        self.token_rates = rates;
        // Recalculate cost with new rates
        self.token_stats.total_cost = self.token_rates.calculate_cost(
            self.token_stats.input_tokens,
            self.token_stats.output_tokens,
        );
    }

    /// Get token rates
    pub fn token_rates(&self) -> TokenRates {
        self.token_rates
    }
}

impl Default for TuiState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_buffer_operations() {
        let mut state = TuiState::new();

        state.insert_char('h');
        state.insert_char('i');
        assert_eq!(state.input(), "hi");
        assert_eq!(state.cursor_position(), 2);

        state.move_cursor_left();
        assert_eq!(state.cursor_position(), 1);

        state.insert_char('e');
        assert_eq!(state.input(), "hei");

        state.delete_before_cursor();
        assert_eq!(state.input(), "hi");

        state.move_cursor_to_start();
        assert_eq!(state.cursor_position(), 0);

        state.move_cursor_to_end();
        assert_eq!(state.cursor_position(), 2);

        state.clear_input();
        assert_eq!(state.input(), "");
        assert_eq!(state.cursor_position(), 0);
    }

    #[test]
    fn test_message_adding() {
        let mut state = TuiState::new();

        state.add_user_message("Hello".to_string());
        assert_eq!(state.message_count(), 1);
        assert_eq!(state.messages().back().unwrap().role, MessageRole::User);

        state.add_assistant_message("Hi there!".to_string());
        assert_eq!(state.message_count(), 2);
        assert_eq!(
            state.messages().back().unwrap().role,
            MessageRole::Assistant
        );
    }

    #[test]
    fn test_tool_calls() {
        let mut state = TuiState::new();

        state.add_user_message("Read file".to_string());
        state.add_tool_call(
            "tool1".to_string(),
            "read_file".to_string(),
            r#"{"path":"test.rs"}"#.to_string(),
        );
        state.add_tool_result("tool1".to_string(), "file contents".to_string(), false);
        state.add_assistant_message("Here is the file".to_string());

        let msg = state.messages().back().unwrap();
        assert_eq!(msg.tool_calls.len(), 1);
        assert_eq!(msg.tool_results.len(), 1);
        assert!(!msg.tool_results[0].is_error);
    }

    #[test]
    fn test_render_mode_cycling() {
        let mut state = TuiState::new();

        assert_eq!(state.render_mode(), RenderMode::Compact);

        state.cycle_render_mode();
        assert_eq!(state.render_mode(), RenderMode::Expanded);

        state.cycle_render_mode();
        assert_eq!(state.render_mode(), RenderMode::SyntaxHighlighted);

        state.cycle_render_mode();
        assert_eq!(state.render_mode(), RenderMode::Compact);
    }

    #[test]
    fn test_thinking() {
        let mut state = TuiState::new();

        state.set_thinking(true, Some("Let me think...".to_string()));
        assert!(state.is_thinking_visible());
        assert_eq!(
            state.thinking_content(),
            Some(&"Let me think...".to_string())
        );

        state.set_thinking(false, None);
        assert!(!state.is_thinking_visible());
        assert_eq!(state.thinking_content(), None);
    }
}
