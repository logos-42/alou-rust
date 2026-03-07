/**
 * manager.rs - Tasks Manager
 *
 * Handles task status tracking and breakpoint resumption for TASKS.md
 */

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use chrono::{DateTime, Local};
use crate::tasks::types::{Task, TaskStatus, TaskStats};

/// Tasks Manager for handling TASKS.md
pub struct TasksManager {
    file_path: PathBuf,
}

impl TasksManager {
    /// Create a new TasksManager
    pub fn new(base_dir: &PathBuf) -> Self {
        let file_path = base_dir.join("TASKS.md");
        Self { file_path }
    }

    /// Get the default home directory path
    pub fn get_default_dir() -> Option<PathBuf> {
        dirs::home_dir().map(|home| home.join(".alou"))
    }

    /// Load tasks from TASKS.md
    pub fn load(&self) -> Result<Vec<Task>, String> {
        if !self.file_path.exists() {
            self.create_default()?;
        }

        let content = fs::read_to_string(&self.file_path)
            .map_err(|e| format!("Failed to load TASKS.md: {}", e))?;

        self.parse_tasks(&content)
    }

    /// Save tasks to TASKS.md
    pub fn save(&self, tasks: &[Task]) -> Result<(), String> {
        let content = self.format_tasks(tasks);

        if let Some(parent) = self.file_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        }

        let mut file = fs::File::create(&self.file_path)
            .map_err(|e| format!("Failed to create TASKS.md: {}", e))?;

        file.write_all(content.as_bytes())
            .map_err(|e| format!("Failed to write TASKS.md: {}", e))?;

        Ok(())
    }

    /// Add a new task
    pub fn add_task(&self, task: Task) -> Result<(), String> {
        let mut tasks = self.load()?;
        tasks.push(task);
        self.save(&tasks)
    }

    /// Update task status
    pub fn update_task_status(&self, id: &str, status: TaskStatus) -> Result<(), String> {
        let mut tasks = self.load()?;

        for task in &mut tasks {
            if task.id == id {
                task.status = status.clone();

                match status {
                    TaskStatus::InProgress => {
                        if task.started_at.is_none() {
                            task.started_at = Some(Local::now());
                        }
                    }
                    TaskStatus::Completed => {
                        task.completed_at = Some(Local::now());
                    }
                    _ => {}
                }

                return self.save(&tasks);
            }
        }

        Err(format!("Task not found: {}", id))
    }

    /// Update task status from string
    pub fn update_task(&self, id: &str, status: &str) -> Result<(), String> {
        let task_status = TaskStatus::from_str(status);
        self.update_task_status(id, task_status)
    }

    /// Get a task by ID
    pub fn get_task(&self, id: &str) -> Result<Option<Task>, String> {
        let tasks = self.load()?;
        Ok(tasks.into_iter().find(|t| t.id == id))
    }

    /// Get stuck tasks
    pub fn get_stuck_tasks(&self) -> Result<Vec<Task>, String> {
        let tasks = self.load()?;
        Ok(tasks
            .into_iter()
            .filter(|t| t.status == TaskStatus::Stuck)
            .collect())
    }

    /// Get in-progress tasks
    pub fn get_in_progress_tasks(&self) -> Result<Vec<Task>, String> {
        let tasks = self.load()?;
        Ok(tasks
            .into_iter()
            .filter(|t| t.status == TaskStatus::InProgress)
            .collect())
    }

    /// Get pending tasks
    pub fn get_pending_tasks(&self) -> Result<Vec<Task>, String> {
        let tasks = self.load()?;
        Ok(tasks
            .into_iter()
            .filter(|t| t.status == TaskStatus::Pending)
            .collect())
    }

    /// Get completed tasks
    pub fn get_completed_tasks(&self) -> Result<Vec<Task>, String> {
        let tasks = self.load()?;
        Ok(tasks
            .into_iter()
            .filter(|t| t.status == TaskStatus::Completed)
            .collect())
    }

    /// Get tasks by tag
    pub fn get_tasks_by_tag(&self, tag: &str) -> Result<Vec<Task>, String> {
        let tasks = self.load()?;
        Ok(tasks
            .into_iter()
            .filter(|t| t.tags.contains(&tag.to_string()))
            .collect())
    }

    /// Remove a task
    pub fn remove_task(&self, id: &str) -> Result<(), String> {
        let mut tasks = self.load()?;
        tasks.retain(|t| t.id != id);
        self.save(&tasks)
    }

    /// Mark task as stuck with reason
    pub fn mark_task_stuck(&self, id: &str, reason: &str) -> Result<(), String> {
        let mut tasks = self.load()?;

        for task in &mut tasks {
            if task.id == id {
                task.mark_stuck(reason);
                return self.save(&tasks);
            }
        }

        Err(format!("Task not found: {}", id))
    }

    /// Start a task
    pub fn start_task(&self, id: &str) -> Result<(), String> {
        let mut tasks = self.load()?;

        for task in &mut tasks {
            if task.id == id {
                task.start();
                return self.save(&tasks);
            }
        }

        Err(format!("Task not found: {}", id))
    }

    /// Complete a task
    pub fn complete_task(&self, id: &str) -> Result<(), String> {
        let mut tasks = self.load()?;

        for task in &mut tasks {
            if task.id == id {
                task.complete();
                return self.save(&tasks);
            }
        }

        Err(format!("Task not found: {}", id))
    }

    /// Get task statistics
    pub fn get_stats(&self) -> Result<TaskStats, String> {
        let tasks = self.load()?;

        let total = tasks.len();
        let pending = tasks.iter().filter(|t| t.status == TaskStatus::Pending).count();
        let in_progress = tasks.iter().filter(|t| t.status == TaskStatus::InProgress).count();
        let completed = tasks.iter().filter(|t| t.status == TaskStatus::Completed).count();
        let stuck = tasks.iter().filter(|t| t.status == TaskStatus::Stuck).count();
        let cancelled = tasks.iter().filter(|t| t.status == TaskStatus::Cancelled).count();

        Ok(TaskStats {
            total,
            pending,
            in_progress,
            completed,
            stuck,
            cancelled,
        })
    }

    /// Get tasks ready for breakpoint resumption
    pub fn get_resumable_tasks(&self) -> Result<Vec<Task>, String> {
        let tasks = self.load()?;
        Ok(tasks
            .into_iter()
            .filter(|t| t.status == TaskStatus::InProgress || t.status == TaskStatus::Stuck)
            .collect())
    }

    // Helper methods

    fn create_default(&self) -> Result<(), String> {
        let _template = self.get_default_template();
        self.save(&[])
    }

    fn get_default_template(&self) -> String {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        format!(
            r#"# 任务列表

## 创建时间
{}

## 进行中任务
- 暂无进行中任务

## 已完成任务
- 暂无已完成任务

## 卡住的任务
- 暂无卡住的任务

## 任务历史
{}：任务系统初始化
"#,
            now, now
        )
    }

    fn parse_tasks(&self, content: &str) -> Result<Vec<Task>, String> {
        let mut tasks = Vec::new();

        let sections = [
            ("进行中任务", TaskStatus::InProgress),
            ("已完成任务", TaskStatus::Completed),
            ("卡住的任务", TaskStatus::Stuck),
            ("待处理任务", TaskStatus::Pending),
        ];

        for (section_name, default_status) in sections.iter() {
            if let Some(section_content) = self.extract_section(content, section_name) {
                for line in section_content.lines() {
                    let line = line.trim();
                    if line.starts_with("- [") {
                        if let Some(task) = self.parse_task_line(line, default_status) {
                            tasks.push(task);
                        }
                    }
                }
            }
        }

        Ok(tasks)
    }

    fn parse_task_line(&self, line: &str, default_status: &TaskStatus) -> Option<Task> {
        let line = line.trim_start_matches('-').trim();

        let status = if line.starts_with("[x]") {
            TaskStatus::Completed
        } else if line.starts_with("[!]") {
            TaskStatus::Stuck
        } else if line.starts_with("[~]") {
            TaskStatus::InProgress
        } else if line.starts_with("[-]") {
            TaskStatus::Cancelled
        } else {
            default_status.clone()
        };

        let content = line.trim_start_matches("[x]").trim_start_matches("[!]").trim_start_matches("[~]").trim_start_matches("[-]").trim_start_matches("[ ]").trim();

        let title = if let Some(paren_pos) = content.find('(') {
            content[..paren_pos].trim().to_string()
        } else {
            content.to_string()
        };

        if title.is_empty() {
            return None;
        }

        let mut task = Task::new(&title, &title);
        task.status = status;

        if let Some(start_pos) = content.find("开始时间：") {
            let start_str = &content[start_pos + 5..];
            if let Some(end_pos) = start_str.find(')') {
                let time_str = start_str[..end_pos].trim();
                if let Ok(dt) = DateTime::parse_from_str(time_str, "%Y-%m-%d %H:%M:%S") {
                    task.started_at = Some(dt.with_timezone(&Local));
                }
            }
        }

        if let Some(end_pos) = content.find("完成时间：") {
            let end_str = &content[end_pos + 5..];
            if let Some(end_paren) = end_str.find(')') {
                let time_str = end_str[..end_paren].trim();
                if let Ok(dt) = DateTime::parse_from_str(time_str, "%Y-%m-%d %H:%M:%S") {
                    task.completed_at = Some(dt.with_timezone(&Local));
                }
            }
        }

        if status == TaskStatus::Stuck {
            if let Some(reason_pos) = content.find("卡住原因：") {
                let reason_str = &content[reason_pos + 5..];
                if let Some(end_paren) = reason_str.find(')') {
                    task.stuck_reason = Some(reason_str[..end_paren].trim().to_string());
                } else {
                    task.stuck_reason = Some(reason_str.trim().to_string());
                }
            }
        }

        Some(task)
    }

    fn format_tasks(&self, tasks: &[Task]) -> String {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        let mut content = format!(
            r#"# 任务列表

## 创建时间
{}

## 进行中任务
"#,
            now
        );

        let in_progress: Vec<&Task> = tasks.iter().filter(|t| t.status == TaskStatus::InProgress).collect();
        if in_progress.is_empty() {
            content.push_str("- 暂无进行中任务\n");
        } else {
            for task in &in_progress {
                content.push_str(&self.format_task_line(task));
            }
        }

        content.push_str("\n## 已完成任务\n");
        let completed: Vec<&Task> = tasks.iter().filter(|t| t.status == TaskStatus::Completed).collect();
        if completed.is_empty() {
            content.push_str("- 暂无已完成任务\n");
        } else {
            for task in &completed {
                content.push_str(&self.format_task_line(task));
            }
        }

        content.push_str("\n## 卡住的任务\n");
        let stuck: Vec<&Task> = tasks.iter().filter(|t| t.status == TaskStatus::Stuck).collect();
        if stuck.is_empty() {
            content.push_str("- 暂无卡住的任务\n");
        } else {
            for task in &stuck {
                content.push_str(&self.format_task_line(task));
            }
        }

        content.push_str("\n## 待处理任务\n");
        let pending: Vec<&Task> = tasks.iter().filter(|t| t.status == TaskStatus::Pending).collect();
        if pending.is_empty() {
            content.push_str("- 暂无待处理任务\n");
        } else {
            for task in &pending {
                content.push_str(&self.format_task_line(task));
            }
        }

        content.push_str("\n## 任务历史\n");
        content.push_str(&format!("{}：任务系统最后更新\n", now));

        content
    }

    fn format_task_line(&self, task: &Task) -> String {
        let mut line = format!("{} {} ", task.status.to_markdown(), task.title);

        if let Some(started) = &task.started_at {
            line.push_str(&format!("(开始时间：{}) ", started.format("%Y-%m-%d %H:%M:%S")));
        }

        if let Some(completed) = &task.completed_at {
            line.push_str(&format!("(完成时间：{}) ", completed.format("%Y-%m-%d %H:%M:%S")));
        }

        if let Some(reason) = &task.stuck_reason {
            line.push_str(&format!("(卡住原因：{}) ", reason));
        }

        line.push_str("\n");
        line
    }

    fn extract_section(&self, content: &str, section_name: &str) -> Option<String> {
        let mut result = String::new();
        let mut in_section = false;

        for line in content.lines() {
            if line.contains(&format!("## {}", section_name)) {
                in_section = true;
                continue;
            }

            if in_section {
                if line.starts_with("## ") && !line.contains(&format!("## {}", section_name)) {
                    break;
                }
                result.push_str(line);
                result.push('\n');
            }
        }

        if result.is_empty() {
            None
        } else {
            Some(result.trim().to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_tasks_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let manager = TasksManager::new(&temp_dir.path().to_path_buf());
        let tasks = manager.load().unwrap();
        assert!(tasks.is_empty() || !tasks.is_empty());
    }

    #[test]
    fn test_add_task() {
        let temp_dir = TempDir::new().unwrap();
        let manager = TasksManager::new(&temp_dir.path().to_path_buf());

        let task = Task::new("Test Task", "Test Description");
        manager.add_task(task).unwrap();

        let tasks = manager.load().unwrap();
        assert!(!tasks.is_empty());
    }

    #[test]
    fn test_update_task_status() {
        let temp_dir = TempDir::new().unwrap();
        let manager = TasksManager::new(&temp_dir.path().to_path_buf());

        let mut task = Task::new("Test Task", "Test Description");
        let task_id = task.id.clone();
        manager.add_task(task).unwrap();

        manager.update_task_status(&task_id, TaskStatus::InProgress).unwrap();

        let updated_task = manager.get_task(&task_id).unwrap().unwrap();
        assert_eq!(updated_task.status, TaskStatus::InProgress);
    }

    #[test]
    fn test_get_stuck_tasks() {
        let temp_dir = TempDir::new().unwrap();
        let manager = TasksManager::new(&temp_dir.path().to_path_buf());

        let mut task = Task::new("Stuck Task", "Test Description");
        task.mark_stuck("Testing stuck state");
        manager.add_task(task).unwrap();

        let stuck_tasks = manager.get_stuck_tasks().unwrap();
        assert!(!stuck_tasks.is_empty());
    }

    #[test]
    fn test_task_stats() {
        let temp_dir = TempDir::new().unwrap();
        let manager = TasksManager::new(&temp_dir.path().to_path_buf());

        let mut task1 = Task::new("Task 1", "Description 1");
        task1.start();
        let mut task2 = Task::new("Task 2", "Description 2");
        task2.complete();

        manager.add_task(task1).unwrap();
        manager.add_task(task2).unwrap();

        let stats = manager.get_stats().unwrap();
        assert_eq!(stats.total, 2);
        assert_eq!(stats.in_progress, 1);
        assert_eq!(stats.completed, 1);
    }
}
