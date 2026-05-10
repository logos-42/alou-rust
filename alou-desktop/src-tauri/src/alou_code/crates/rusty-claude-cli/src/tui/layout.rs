//! Layout Components for TUI
//!
//! Provides layout utilities for positioning and sizing UI elements
//! within the terminal.

use std::cmp::{max, min};

/// Represents a rectangular region in the terminal
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl Rect {
    /// Create a new rect with the given dimensions
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Create a rect from absolute coordinates
    pub fn from_coords(x1: u16, y1: u16, x2: u16, y2: u16) -> Self {
        Self {
            x: x1,
            y: y1,
            width: x2.saturating_sub(x1),
            height: y2.saturating_sub(y1),
        }
    }

    /// Get the right edge x coordinate
    pub fn right(&self) -> u16 {
        self.x.saturating_add(self.width)
    }

    /// Get the bottom edge y coordinate
    pub fn bottom(&self) -> u16 {
        self.y.saturating_add(self.height)
    }

    /// Check if a point is inside the rect
    pub fn contains(&self, x: u16, y: u16) -> bool {
        x >= self.x && x < self.right() && y >= self.y && y < self.bottom()
    }

    /// Get the inner rect with borders subtracted
    pub fn inner(&self, margin: u16) -> Self {
        Self {
            x: self.x.saturating_add(margin),
            y: self.y.saturating_add(margin),
            width: self.width.saturating_sub(margin * 2),
            height: self.height.saturating_sub(margin * 2),
        }
    }

    /// Get the inner rect with asymmetric margins
    pub fn inner_with(&self, left: u16, right: u16, top: u16, bottom: u16) -> Self {
        Self {
            x: self.x.saturating_add(left),
            y: self.y.saturating_add(top),
            width: self.width.saturating_sub(left + right),
            height: self.height.saturating_sub(top + bottom),
        }
    }

    /// Split the rect horizontally into two parts
    pub fn split_horizontal(&self, first_height: u16) -> (Rect, Rect) {
        let first = Self {
            x: self.x,
            y: self.y,
            width: self.width,
            height: min(first_height, self.height),
        };
        let second = Self {
            x: self.x,
            y: self.y.saturating_add(first.height),
            width: self.width,
            height: self.height.saturating_sub(first.height),
        };
        (first, second)
    }

    /// Split the rect vertically into two parts
    pub fn split_vertical(&self, first_width: u16) -> (Rect, Rect) {
        let first = Self {
            x: self.x,
            y: self.y,
            width: min(first_width, self.width),
            height: self.height,
        };
        let second = Self {
            x: self.x.saturating_add(first.width),
            y: self.y,
            width: self.width.saturating_sub(first.width),
            height: self.height,
        };
        (first, second)
    }

    /// Split into multiple horizontal sections
    pub fn split_horizontal_equal(&self, count: usize) -> Vec<Rect> {
        if count == 0 {
            return Vec::new();
        }
        let height = max(1, self.height / count as u16);
        (0..count)
            .map(|i| Self {
                x: self.x,
                y: self.y.saturating_add(i as u16 * height),
                width: self.width,
                height: if i == count - 1 {
                    self.height.saturating_sub(i as u16 * height)
                } else {
                    height
                },
            })
            .collect()
    }

    /// Get intersection with another rect
    pub fn intersection(&self, other: &Rect) -> Option<Rect> {
        let x1 = max(self.x, other.x);
        let y1 = max(self.y, other.y);
        let x2 = min(self.right(), other.right());
        let y2 = min(self.bottom(), other.bottom());

        if x1 < x2 && y1 < y2 {
            Some(Self::from_coords(x1, y1, x2, y2))
        } else {
            None
        }
    }
}

impl Default for Rect {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            width: 80,
            height: 24,
        }
    }
}

/// Terminal size information
#[derive(Debug, Clone, Copy, Default)]
pub struct TerminalSize {
    pub width: u16,
    pub height: u16,
}

impl TerminalSize {
    /// Create a new terminal size
    pub fn new(width: u16, height: u16) -> Self {
        Self { width, height }
    }

    /// Get the full terminal as a rect
    pub fn to_rect(&self) -> Rect {
        Rect::new(0, 0, self.width, self.height)
    }
}

/// Layout chunk for positioning elements
#[derive(Debug, Clone)]
pub struct LayoutChunk {
    pub rect: Rect,
    pub id: String,
}

/// Simple vertical layout algorithm
pub struct VerticalLayout {
    chunks: Vec<LayoutChunk>,
    current_y: u16,
    width: u16,
    margin: u16,
}

impl VerticalLayout {
    /// Create a new vertical layout
    pub fn new(width: u16, margin: u16) -> Self {
        Self {
            chunks: Vec::new(),
            current_y: 0,
            width,
            margin,
        }
    }

    /// Add a fixed-height chunk
    pub fn add_fixed(&mut self, id: &str, height: u16) -> Rect {
        let rect = Rect::new(0, self.current_y, self.width, height);
        self.chunks.push(LayoutChunk {
            rect,
            id: id.to_string(),
        });
        self.current_y += height + self.margin;
        rect
    }

    /// Add a chunk that takes remaining space
    pub fn add_flexible(&mut self, id: &str) -> Rect {
        let rect = Rect::new(0, self.current_y, self.width, u16::MAX);
        self.chunks.push(LayoutChunk {
            rect,
            id: id.to_string(),
        });
        // Don't increment current_y for flexible - it takes remaining space
        rect
    }

    /// Add a proportional chunk
    pub fn add_proportional(&mut self, id: &str, total_height: u16, proportion: f32) -> Rect {
        let height = (total_height as f32 * proportion) as u16;
        let rect = Rect::new(0, self.current_y, self.width, height);
        self.chunks.push(LayoutChunk {
            rect,
            id: id.to_string(),
        });
        self.current_y += height + self.margin;
        rect
    }

    /// Get all chunks
    pub fn chunks(&self) -> &[LayoutChunk] {
        &self.chunks
    }

    /// Get a chunk by ID
    pub fn get(&self, id: &str) -> Option<&Rect> {
        self.chunks.iter().find(|c| c.id == id).map(|c| &c.rect)
    }
}

/// Helper to calculate visible lines with scrolling
pub fn calculate_visible_lines(
    total_lines: usize,
    visible_height: usize,
    scroll_offset: usize,
) -> (usize, usize) {
    let total = total_lines;
    let visible = visible_height;
    let offset = scroll_offset.min(total.saturating_sub(1));

    let start = offset.min(total.saturating_sub(visible));
    let end = (start + visible).min(total);

    (start, end)
}

/// ANSI escape codes for cursor positioning
pub mod ansi {
    /// Move cursor to position (1-indexed)
    pub fn cursor_position(x: u16, y: u16) -> String {
        format!("\x1b[{};{}H", y + 1, x + 1)
    }

    /// Clear screen
    pub fn clear_screen() -> String {
        "\x1b[2J".to_string()
    }

    /// Clear from cursor to end of screen
    pub fn clear_to_end() -> String {
        "\x1b[0J".to_string()
    }

    /// Clear from cursor to start of screen
    pub fn clear_to_start() -> String {
        "\x1b[1J".to_string()
    }

    /// Clear current line
    pub fn clear_line() -> String {
        "\x1b[2K".to_string()
    }

    /// Hide cursor
    pub fn hide_cursor() -> String {
        "\x1b[?25l".to_string()
    }

    /// Show cursor
    pub fn show_cursor() -> String {
        "\x1b[?25h".to_string()
    }

    /// Save cursor position
    pub fn save_cursor() -> String {
        "\x1b[s".to_string()
    }

    /// Restore cursor position
    pub fn restore_cursor() -> String {
        "\x1b[u".to_string()
    }

    /// Move cursor up
    pub fn cursor_up(n: u16) -> String {
        format!("\x1b[{}A", n)
    }

    /// Move cursor down
    pub fn cursor_down(n: u16) -> String {
        format!("\x1b[{}B", n)
    }

    /// Move cursor forward (right)
    pub fn cursor_forward(n: u16) -> String {
        format!("\x1b[{}C", n)
    }

    /// Move cursor back (left)
    pub fn cursor_back(n: u16) -> String {
        format!("\x1b[{}D", n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect_basic() {
        let rect = Rect::new(0, 0, 80, 24);
        assert_eq!(rect.right(), 80);
        assert_eq!(rect.bottom(), 24);
        assert!(rect.contains(0, 0));
        assert!(rect.contains(79, 23));
        assert!(!rect.contains(80, 0));
        assert!(!rect.contains(0, 24));
    }

    #[test]
    fn test_rect_split() {
        let rect = Rect::new(0, 0, 80, 24);
        let (top, bottom) = rect.split_horizontal(10);
        assert_eq!(top.height, 10);
        assert_eq!(bottom.height, 14);
        assert_eq!(top.y, 0);
        assert_eq!(bottom.y, 10);

        let (left, right) = rect.split_vertical(40);
        assert_eq!(left.width, 40);
        assert_eq!(right.width, 40);
        assert_eq!(left.x, 0);
        assert_eq!(right.x, 40);
    }

    #[test]
    fn test_rect_intersection() {
        let rect1 = Rect::new(0, 0, 80, 24);
        let rect2 = Rect::new(70, 20, 20, 10);
        let intersection = rect1.intersection(&rect2).unwrap();
        assert_eq!(intersection.x, 70);
        assert_eq!(intersection.y, 20);
        assert_eq!(intersection.width, 10);
        assert_eq!(intersection.height, 4);
    }

    #[test]
    fn test_vertical_layout() {
        let mut layout = VerticalLayout::new(80, 1);
        let header = layout.add_fixed("header", 3);
        let content = layout.add_flexible("content");
        let footer = layout.add_fixed("footer", 3);

        assert_eq!(header.height, 3);
        assert_eq!(footer.height, 3);

        assert_eq!(layout.get("header").unwrap().height, 3);
        assert_eq!(layout.get("footer").unwrap().height, 3);
        assert!(layout.get("content").is_some());
    }

    #[test]
    fn test_visible_lines() {
        let (start, end) = calculate_visible_lines(100, 20, 0);
        assert_eq!(start, 0);
        assert_eq!(end, 20);

        let (start, end) = calculate_visible_lines(100, 20, 50);
        assert_eq!(start, 50);
        assert_eq!(end, 70);

        // Scroll past content
        let (start, end) = calculate_visible_lines(100, 20, 90);
        assert_eq!(start, 80);
        assert_eq!(end, 100);
    }
}
