/**
 * types.rs - Tasks Module Types
 *
 * Type definitions for task management.
 */

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

/// Task status enumeration
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Stuck,
    Cancelled,
}

impl TaskStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskStatus::Pending => "pending",
            TaskStatus::InProgress => "in_progress",
            TaskStatus::Completed => "completed",
            TaskStatus::Stuck => "stuck",
            TaskStatus::Cancelled => "cancelled",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "pending" => TaskStatus::Pending,
            "in_progress" => TaskStatus::InProgress,
            "completed" => TaskStatus::Completed,
            "stuck" => TaskStatus::Stuck,
            "cancelled" => TaskStatus::Cancelled,
            _ => TaskStatus::Pending,
        }
    }

    pub fn to_markdown(&self) -> &'static str {
        match self {
            TaskStatus::Pending => "[ ]",
            TaskStatus::InProgress => "[~]",
            TaskStatus::Completed => "[x]",
            TaskStatus::Stuck => "[!]",
            TaskStatus::Cancelled => "[-]",
        }
    }
}

/// Task structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: TaskStatus,
    pub priority: u8, // 1-5
    pub created_at: DateTime<Local>,
    pub started_at: Option<DateTime<Local>>,
    pub completed_at: Option<DateTime<Local>>,
    pub stuck_reason: Option<String>,
    pub tags: Vec<String>,
    pub parent_id: Option<String>,
}

impl Task {
    /// Create a new task
    pub fn new(title: &str, description: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            title: title.to_string(),
            description: description.to_string(),
            status: TaskStatus::Pending,
            priority: 3,
            created_at: Local::now(),
            started_at: None,
            completed_at: None,
            stuck_reason: None,
            tags: Vec::new(),
            parent_id: None,
        }
    }

    /// Create a task with custom ID
    pub fn with_id(id: &str, title: &str, description: &str) -> Self {
        Self {
            id: id.to_string(),
            title: title.to_string(),
            description: description.to_string(),
            status: TaskStatus::Pending,
            priority: 3,
            created_at: Local::now(),
            started_at: None,
            completed_at: None,
            stuck_reason: None,
            tags: Vec::new(),
            parent_id: None,
        }
    }

    /// Start the task
    pub fn start(&mut self) {
        self.status = TaskStatus::InProgress;
        self.started_at = Some(Local::now());
    }

    /// Complete the task
    pub fn complete(&mut self) {
        self.status = TaskStatus::Completed;
        self.completed_at = Some(Local::now());
    }

    /// Mark task as stuck
    pub fn mark_stuck(&mut self, reason: &str) {
        self.status = TaskStatus::Stuck;
        self.stuck_reason = Some(reason.to_string());
    }

    /// Add a tag to the task
    pub fn add_tag(&mut self, tag: &str) {
        if !self.tags.contains(&tag.to_string()) {
            self.tags.push(tag.to_string());
        }
    }
}

/// Task statistics structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStats {
    pub total: usize,
    pub pending: usize,
    pub in_progress: usize,
    pub completed: usize,
    pub stuck: usize,
    pub cancelled: usize,
}
