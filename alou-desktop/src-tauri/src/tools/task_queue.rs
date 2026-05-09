//! 任务队列系统
//!
//! 实现 Alou AI 的任务队列管理
//! 支持任务添加、查询、更新、优先级排序和持久化

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{interval, Duration};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;
use chrono::Utc;

/// 任务优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskPriority {
    Critical = 0,  // 最高优先级
    High = 1,
    Medium = 2,
    Low = 3,
}

/// 任务状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Pending = 0,      // 待执行
    InProgress = 1,   // 执行中
    Completed = 2,    // 已完成
    Failed = 3,       // 失败
    Cancelled = 4,    // 取消
}

/// 任务结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub success: bool,
    pub output: Option<Value>,
    pub error: Option<String>,
    pub executed_at: i64,
    pub execution_time_ms: u64,
}

/// 任务结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: String,
    pub priority: TaskPriority,
    pub status: TaskStatus,
    #[serde(default)]
    pub created_at: i64,
    #[serde(default)]
    pub updated_at: i64,
    pub executor: Option<String>,
    pub result: Option<TaskResult>,
    #[serde(default)]
    pub metadata: HashMap<String, Value>,
}

impl Task {
    /// 创建新任务
    pub fn new(
        title: String,
        description: String,
        priority: TaskPriority,
        executor: Option<String>,
    ) -> Self {
        let now = Utc::now().timestamp();
        Self {
            id: format!("task_{}", Uuid::new_v4().to_string().replace("-", "")[..12].to_string()),
            title,
            description,
            priority,
            status: TaskStatus::Pending,
            created_at: now,
            updated_at: now,
            executor,
            result: None,
            metadata: HashMap::new(),
        }
    }

    /// 更新状态
    pub fn update_status(&mut self, status: TaskStatus) {
        self.status = status;
        self.updated_at = Utc::now().timestamp();
    }

    /// 设置执行结果
    pub fn set_result(&mut self, result: TaskResult) {
        self.result = Some(result);
        self.updated_at = Utc::now().timestamp();
    }
}

/// 任务统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStats {
    pub total: usize,
    pub pending: usize,
    pub in_progress: usize,
    pub completed: usize,
    pub failed: usize,
    pub cancelled: usize,
}

/// 任务队列管理器
pub struct TaskQueueManager {
    queue: Vec<Task>,           // 任务队列
    history: Vec<Task>,         // 历史记录
    by_status: HashMap<TaskStatus, Vec<String>>,  // 按状态索引
    storage_path: PathBuf,      // 存储路径
}

impl TaskQueueManager {
    /// 创建新的任务队列管理器
    pub fn new(storage_path: Option<PathBuf>) -> Result<Self, Box<dyn std::error::Error>> {
        let path = storage_path.unwrap_or_else(|| {
            dirs::home_dir()
                .unwrap_or(PathBuf::from("."))
                .join(".alou")
                .join("tasks")
        });
        
        // 确保目录存在
        if !path.exists() {
            fs::create_dir_all(&path)?;
        }

        // 加载已有任务
        let mut manager = Self {
            queue: Vec::new(),
            history: Vec::new(),
            by_status: HashMap::new(),
            storage_path: path,
        };
        
        manager.load_from_disk()?;
        Ok(manager)
    }

    /// 添加任务
    pub async fn add_task(&mut self, task: Task) -> String {
        let task_id = task.id.clone();
        
        // 添加到队列
        self.queue.push(task.clone());
        
        // 更新索引
        self.by_status
            .entry(TaskStatus::Pending)
            .or_insert_with(Vec::new)
            .push(task_id.clone());
        
        // 按优先级排序
        self.sort_by_priority();
        
        // 保存到磁盘
        self.save_to_disk().await;
        
        task_id
    }

    /// 获取下一个任务（按优先级）
    pub async fn next_task(&mut self, executor: Option<&str>) -> Option<Task> {
        // 过滤出待执行的任务
        let mut candidates: Vec<usize> = self.queue
            .iter()
            .enumerate()
            .filter(|(_, task)| task.status == TaskStatus::Pending)
            .filter(|(_, task)| {
                // 如果指定了执行者，检查是否匹配
                match executor {
                    Some(e) => task.executor.as_deref() == Some(e),
                    None => true,
                }
            })
            .map(|(idx, _)| idx)
            .collect();
        
        if candidates.is_empty() {
            return None;
        }
        
        // 选择优先级最高的任务
        candidates.sort_by(|&a, &b| {
            self.queue[a].priority.cmp(&self.queue[b].priority)
        });
        
        let idx = candidates[0];
        let task = self.queue[idx].clone();
        
        // 更新状态为执行中
        self.queue[idx].update_status(TaskStatus::InProgress);
        
        // 更新索引
        self.update_index(idx).await;
        
        Some(task)
    }

    /// 更新任务状态
    pub async fn update_status(&mut self, task_id: &str, status: TaskStatus) -> bool {
        if let Some(task) = self.queue.iter_mut().find(|t| t.id == task_id) {
            task.update_status(status);
            
            // 找到索引并更新
            if let Some(idx) = self.queue.iter().position(|t| t.id == task_id) {
                self.update_index(idx).await;
            }
            
            // 如果完成或失败，移动到历史
            if status == TaskStatus::Completed || status == TaskStatus::Failed {
                if let Some(pos) = self.queue.iter().position(|t| t.id == task_id) {
                    let completed_task = self.queue.remove(pos);
                    self.history.push(completed_task);
                }
            }
            
            self.save_to_disk().await;
            return true;
        }
        false
    }

    /// 设置任务结果
    pub async fn set_result(&mut self, task_id: &str, result: TaskResult) -> bool {
        if let Some(task) = self.queue.iter_mut().find(|t| t.id == task_id) {
            task.set_result(result);
            
            // 如果完成或失败，移动到历史
            if task.status == TaskStatus::InProgress {
                if task.result.as_ref().map(|r| r.success).unwrap_or(false) {
                    task.update_status(TaskStatus::Completed);
                } else {
                    task.update_status(TaskStatus::Failed);
                }
            }
            
            if let Some(idx) = self.queue.iter().position(|t| t.id == task_id) {
                self.update_index(idx).await;
            }
            
            self.save_to_disk().await;
            return true;
        }
        false
    }

    /// 获取任务列表
    pub fn list_tasks(&self, filter: Option<TaskStatus>) -> Vec<Task> {
        match filter {
            Some(status) => {
                self.queue
                    .iter()
                    .filter(|t| t.status == status)
                    .cloned()
                    .collect()
            }
            None => self.queue.clone(),
        }
    }

    /// 获取任务统计
    pub fn stats(&self) -> TaskStats {
        let queue_stats = self.queue.iter().fold(TaskStats::default(), |mut stats, task| {
            match task.status {
                TaskStatus::Pending => stats.pending += 1,
                TaskStatus::InProgress => stats.in_progress += 1,
                _ => {}
            }
            stats.total += 1;
            stats
        });
        
        let history_stats = self.history.iter().fold(queue_stats, |mut stats, task| {
            match task.status {
                TaskStatus::Completed => stats.completed += 1,
                TaskStatus::Failed => stats.failed += 1,
                TaskStatus::Cancelled => stats.cancelled += 1,
                _ => {}
            }
            stats.total += 1;
            stats
        });
        
        history_stats
    }

    /// 按优先级排序
    fn sort_by_priority(&mut self) {
        self.queue.sort_by(|a, b| a.priority.cmp(&b.priority));
    }

    /// 更新状态索引
    async fn update_index(&mut self, idx: usize) {
        let task = &self.queue[idx];
        
        // 移除旧的索引
        let pending_status = TaskStatus::Pending;
        let in_progress_status = TaskStatus::InProgress;
        
        if let Some(ids) = self.by_status.get_mut(&pending_status) {
            ids.retain(|id| id != &task.id);
        }
        if let Some(ids) = self.by_status.get_mut(&in_progress_status) {
            ids.retain(|id| id != &task.id);
        }
        
        // 添加新的索引
        self.by_status
            .entry(task.status)
            .or_insert_with(Vec::new)
            .push(task.id.clone());
    }

    /// 保存到磁盘
    async fn save_to_disk(&self) {
        // 保存队列
        let queue_path = self.storage_path.join("queue.json");
        if let Ok(content) = serde_json::to_string_pretty(&self.queue) {
            let _ = fs::write(&queue_path, content);
        }
        
        // 保存历史
        let history_path = self.storage_path.join("history.json");
        if let Ok(content) = serde_json::to_string_pretty(&self.history) {
            let _ = fs::write(&history_path, content);
        }
    }

    /// 从磁盘加载
    fn load_from_disk(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let queue_path = self.storage_path.join("queue.json");
        if queue_path.exists() {
            let content = fs::read_to_string(&queue_path)?;
            self.queue = serde_json::from_str(&content)?;
        }
        
        let history_path = self.storage_path.join("history.json");
        if history_path.exists() {
            let content = fs::read_to_string(&history_path)?;
            self.history = serde_json::from_str(&content)?;
        }
        
        Ok(())
    }
}

impl Default for TaskStats {
    fn default() -> Self {
        Self {
            total: 0,
            pending: 0,
            in_progress: 0,
            completed: 0,
            failed: 0,
            cancelled: 0,
        }
    }
}

/// 心跳检测任务
pub struct HeartbeatTask {
    manager: Arc<Mutex<TaskQueueManager>>,
    interval_seconds: u64,
}

impl HeartbeatTask {
    /// 创建心跳任务
    pub fn new(manager: Arc<Mutex<TaskQueueManager>>, interval_seconds: u64) -> Self {
        Self {
            manager,
            interval_seconds,
        }
    }

    /// 启动心跳循环
    pub async fn start(&self) {
        let mut interval = interval(Duration::from_secs(self.interval_seconds));
        
        loop {
            interval.tick().await;
            
            let manager = self.manager.lock().await;
            
            // 检查是否有卡住的任务（执行中超过1小时）
            let stuck_tasks: Vec<String> = manager.queue
                .iter()
                .filter(|t| {
                    t.status == TaskStatus::InProgress && 
                    Utc::now().timestamp() - t.updated_at > 3600
                })
                .map(|t| t.id.clone())
                .collect();
            
            drop(manager);
            
            // 重置卡住的任务
            for task_id in stuck_tasks {
                let mut manager = self.manager.lock().await;
                let _ = manager.update_status(&task_id, TaskStatus::Pending).await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_and_get_task() {
        let manager = TaskQueueManager::new(None).unwrap();
        let task = Task::new(
            "测试任务".to_string(),
            "这是一个测试任务".to_string(),
            TaskPriority::Medium,
            None,
        );
        
        let task_id = manager.add_task(task).await;
        assert!(!task_id.is_empty());
        
        let tasks = manager.list_tasks(None);
        assert_eq!(tasks.len(), 1);
    }

    #[tokio::test]
    async fn test_priority_ordering() {
        let manager = TaskQueueManager::new(None).unwrap();
        
        // 添加不同优先级的任务
        manager.add_task(Task::new(
            "低优先级".to_string(),
            "".to_string(),
            TaskPriority::Low,
            None,
        )).await;
        
        manager.add_task(Task::new(
            "高优先级".to_string(),
            "".to_string(),
            TaskPriority::High,
            None,
        )).await;
        
        manager.add_task(Task::new(
            "关键优先级".to_string(),
            "".to_string(),
            TaskPriority::Critical,
            None,
        )).await;
        
        let next = manager.next_task(None).await;
        assert!(next.is_some());
        assert_eq!(next.unwrap().priority, TaskPriority::Critical);
    }

    #[tokio::test]
    async fn test_status_transition() {
        let manager = TaskQueueManager::new(None).unwrap();
        let task = Task::new(
            "测试任务".to_string(),
            "".to_string(),
            TaskPriority::Medium,
            None,
        );
        
        let task_id = manager.add_task(task).await;
        assert!(manager.update_status(&task_id, TaskStatus::InProgress).await);
        
        let tasks = manager.list_tasks(Some(TaskStatus::InProgress));
        assert_eq!(tasks.len(), 1);
        
        let stats = manager.stats();
        assert_eq!(stats.in_progress, 1);
    }
}
