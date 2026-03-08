/**
 * manager.rs - Logs Manager
 *
 * Handles execution log recording and log rotation for LOGS.md
 */

use std::fs::{self, OpenOptions};
use std::io::{Write, BufRead};
use std::path::PathBuf;
use chrono::{Local, Duration};
use crate::logs::types::{LogLevel, LogStats};

/// Logs Manager for handling LOGS.md
pub struct LogsManager {
    file_path: PathBuf,
    max_file_size_mb: usize,
    max_log_files: usize,
}

impl LogsManager {
    /// Create a new LogsManager
    pub fn new(base_dir: &PathBuf) -> Self {
        let file_path = base_dir.join("LOGS.md");
        Self {
            file_path,
            max_file_size_mb: 10,
            max_log_files: 5,
        }
    }

    /// Create a new LogsManager with custom settings
    pub fn with_settings(base_dir: &PathBuf, max_file_size_mb: usize, max_log_files: usize) -> Self {
        let file_path = base_dir.join("LOGS.md");
        Self {
            file_path,
            max_file_size_mb,
            max_log_files,
        }
    }

    /// Get the default home directory path
    pub fn get_default_dir() -> Option<PathBuf> {
        dirs::home_dir().map(|home| home.join(".alou"))
    }

    /// Append a log entry
    pub fn append_log(&self, entry: &str) -> Result<(), String> {
        self.append_log_with_level(entry, LogLevel::Info)
    }

    /// Append a log entry with specific level
    pub fn append_log_with_level(&self, entry: &str, level: LogLevel) -> Result<(), String> {
        if let Some(parent) = self.file_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        }

        self.rotate_if_needed()?;

        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let log_line = format!(
            "- [{}] [{}] {}\n",
            timestamp,
            level.as_str(),
            entry
        );

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.file_path)
            .map_err(|e| format!("Failed to open LOGS.md: {}", e))?;

        file.write_all(log_line.as_bytes())
            .map_err(|e| format!("Failed to write LOGS.md: {}", e))?;

        Ok(())
    }

    /// Append a debug log entry
    pub fn debug(&self, entry: &str) -> Result<(), String> {
        self.append_log_with_level(entry, LogLevel::Debug)
    }

    /// Append an info log entry
    pub fn info(&self, entry: &str) -> Result<(), String> {
        self.append_log_with_level(entry, LogLevel::Info)
    }

    /// Append a warning log entry
    pub fn warning(&self, entry: &str) -> Result<(), String> {
        self.append_log_with_level(entry, LogLevel::Warning)
    }

    /// Append an error log entry
    pub fn error(&self, entry: &str) -> Result<(), String> {
        self.append_log_with_level(entry, LogLevel::Error)
    }

    /// Append a critical log entry
    pub fn critical(&self, entry: &str) -> Result<(), String> {
        self.append_log_with_level(entry, LogLevel::Critical)
    }

    /// Get recent logs
    pub fn get_recent_logs(&self, lines: usize) -> Result<Vec<String>, String> {
        if !self.file_path.exists() {
            return Ok(Vec::new());
        }

        let all_logs = self.get_all_logs()?;
        let mut result: Vec<String> = all_logs.into_iter().rev().take(lines).collect();
        result.reverse();
        Ok(result)
    }

    /// Get all logs
    pub fn get_all_logs(&self) -> Result<Vec<String>, String> {
        if !self.file_path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&self.file_path)
            .map_err(|e| format!("Failed to read LOGS.md: {}", e))?;

        Ok(content.lines().map(|s| s.to_string()).collect())
    }

    /// Get logs by level
    pub fn get_logs_by_level(&self, level: LogLevel) -> Result<Vec<String>, String> {
        let all_logs = self.get_all_logs()?;
        let level_str = format!("[{}]", level.as_str());

        Ok(all_logs
            .into_iter()
            .filter(|l| l.contains(&level_str))
            .collect())
    }

    /// Get error logs
    pub fn get_error_logs(&self) -> Result<Vec<String>, String> {
        let all_logs = self.get_all_logs()?;

        Ok(all_logs
            .into_iter()
            .filter(|l| l.contains("[ERROR]") || l.contains("[CRITICAL]"))
            .collect())
    }

    /// Get logs from a specific date
    pub fn get_logs_by_date(&self, date: &str) -> Result<Vec<String>, String> {
        let all_logs = self.get_all_logs()?;

        Ok(all_logs
            .into_iter()
            .filter(|l| l.contains(date))
            .collect())
    }

    /// Get today's logs
    pub fn get_today_logs(&self) -> Result<Vec<String>, String> {
        let today = Local::now().format("%Y-%m-%d").to_string();
        self.get_logs_by_date(&today)
    }

    /// Search logs by keyword
    pub fn search_logs(&self, keyword: &str) -> Result<Vec<String>, String> {
        let all_logs = self.get_all_logs()?;

        Ok(all_logs
            .into_iter()
            .filter(|l| l.contains(keyword))
            .collect())
    }

    /// Cleanup old logs (older than specified days)
    pub fn cleanup_old_logs(&self, days: u32) -> Result<(), String> {
        if !self.file_path.exists() {
            return Ok(());
        }

        let cutoff = Local::now() - Duration::days(days as i64);
        let content = fs::read_to_string(&self.file_path)
            .map_err(|e| format!("Failed to read LOGS.md: {}", e))?;

        let mut new_content = String::new();

        for line in content.lines() {
            let keep = if let Some(date_str) = self.extract_date(line) {
                if let Ok(log_date) = chrono::NaiveDate::parse_from_str(&date_str, "%Y-%m-%d") {
                    let log_date = log_date.and_hms_opt(0, 0, 0).unwrap();
                    let cutoff_date = cutoff.naive_local();
                    log_date >= cutoff_date
                } else {
                    true
                }
            } else {
                true
            };

            if keep {
                new_content.push_str(line);
                new_content.push('\n');
            }
        }

        fs::write(&self.file_path, new_content)
            .map_err(|e| format!("Failed to write LOGS.md: {}", e))?;

        Ok(())
    }

    /// Clear all logs
    pub fn clear_logs(&self) -> Result<(), String> {
        let template = self.get_default_template();
        fs::write(&self.file_path, template)
            .map_err(|e| format!("Failed to clear LOGS.md: {}", e))?;
        Ok(())
    }

    /// Get log statistics
    pub fn get_stats(&self) -> Result<LogStats, String> {
        if !self.file_path.exists() {
            return Ok(LogStats {
                total_lines: 0,
                debug_count: 0,
                info_count: 0,
                warning_count: 0,
                error_count: 0,
                critical_count: 0,
                file_size_bytes: 0,
            });
        }

        let content = fs::read_to_string(&self.file_path)
            .map_err(|e| format!("Failed to read LOGS.md: {}", e))?;

        let total_lines = content.lines().count();
        let debug_count = content.matches("[DEBUG]").count();
        let info_count = content.matches("[INFO]").count();
        let warning_count = content.matches("[WARNING]").count();
        let error_count = content.matches("[ERROR]").count();
        let critical_count = content.matches("[CRITICAL]").count();

        let file_size_bytes = fs::metadata(&self.file_path)
            .map(|m| m.len() as usize)
            .unwrap_or(0);

        Ok(LogStats {
            total_lines,
            debug_count,
            info_count,
            warning_count,
            error_count,
            critical_count,
            file_size_bytes,
        })
    }

    /// Rotate logs if file size exceeds limit
    pub fn rotate_if_needed(&self) -> Result<(), String> {
        if !self.file_path.exists() {
            return Ok(());
        }

        let metadata = fs::metadata(&self.file_path)
            .map_err(|e| format!("Failed to get file metadata: {}", e))?;

        let file_size_mb = metadata.len() as f64 / (1024.0 * 1024.0);

        if file_size_mb > self.max_file_size_mb as f64 {
            self.rotate_logs()?;
        }

        Ok(())
    }

    /// Rotate logs (archive current and start fresh)
    pub fn rotate_logs(&self) -> Result<(), String> {
        if !self.file_path.exists() {
            return Ok(());
        }

        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        let rotated_path = self.file_path.with_extension(format!("md.{}", timestamp));

        fs::rename(&self.file_path, &rotated_path)
            .map_err(|e| format!("Failed to rotate logs: {}", e))?;

        self.cleanup_old_rotated_logs()?;

        let template = self.get_default_template();
        fs::write(&self.file_path, template)
            .map_err(|e| format!("Failed to create new LOGS.md: {}", e))?;

        Ok(())
    }

    /// Cleanup old rotated log files
    fn cleanup_old_rotated_logs(&self) -> Result<(), String> {
        if let Some(parent) = self.file_path.parent() {
            let file_name_str = self.file_path.file_name().unwrap().to_string_lossy().to_string();
            let mut rotated_files: Vec<_> = fs::read_dir(parent)
                .map_err(|e| format!("Failed to read directory: {}", e))?
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.file_name()
                        .to_string_lossy()
                        .starts_with(file_name_str.as_str())
                })
                .filter(|e| {
                    e.file_name()
                        .to_string_lossy()
                        .contains(".md.20")
                })
                .map(|e| {
                    let path = e.path();
                    let mtime = e.metadata().ok().map(|m| m.modified().unwrap()).unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                    (path, mtime)
                })
                .collect();

            rotated_files.sort_by(|a, b| b.1.cmp(&a.1));

            if rotated_files.len() > self.max_log_files {
                for (path, _) in rotated_files.iter().skip(self.max_log_files) {
                    let _ = fs::remove_file(path);
                }
            }
        }

        Ok(())
    }

    fn get_default_template(&self) -> String {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        format!(
            r#"# 执行日志

## 创建时间
{}

## 日志内容
{}：日志系统初始化
"#,
            now, now
        )
    }

    fn extract_date(&self, line: &str) -> Option<String> {
        if let Some(start) = line.find('[') {
            if let Some(end) = line[start..].find(']') {
                let date_part = &line[start + 1..start + end];
                if date_part.len() >= 10 {
                    return Some(date_part[..10].to_string());
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_logs_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let manager = LogsManager::new(&temp_dir.path().to_path_buf());
        let stats = manager.get_stats().unwrap();
        assert_eq!(stats.total_lines, 0);
    }

    #[test]
    fn test_append_log() {
        let temp_dir = TempDir::new().unwrap();
        let manager = LogsManager::new(&temp_dir.path().to_path_buf());

        manager.info("Test info message").unwrap();
        let logs = manager.get_all_logs().unwrap();
        assert!(!logs.is_empty());
        assert!(logs.iter().any(|l| l.contains("Test info message")));
    }

    #[test]
    fn test_get_error_logs() {
        let temp_dir = TempDir::new().unwrap();
        let manager = LogsManager::new(&temp_dir.path().to_path_buf());

        manager.info("Info message").unwrap();
        manager.error("Error message").unwrap();
        manager.critical("Critical message").unwrap();

        let error_logs = manager.get_error_logs().unwrap();
        assert_eq!(error_logs.len(), 2);
    }

    #[test]
    fn test_log_stats() {
        let temp_dir = TempDir::new().unwrap();
        let manager = LogsManager::new(&temp_dir.path().to_path_buf());

        manager.debug("Debug 1").unwrap();
        manager.info("Info 1").unwrap();
        manager.info("Info 2").unwrap();
        manager.error("Error 1").unwrap();

        let stats = manager.get_stats().unwrap();
        assert_eq!(stats.debug_count, 1);
        assert_eq!(stats.info_count, 2);
        assert_eq!(stats.error_count, 1);
    }
}
