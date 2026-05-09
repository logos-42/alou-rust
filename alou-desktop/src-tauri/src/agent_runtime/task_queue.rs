//! Task Queue - 全局任务队列

use tokio::sync::mpsc;
use serde::{Deserialize, Serialize};

/// 任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub agent_id: String,
    pub action: TaskAction,
    pub priority: u8,
    pub created_at: i64,
}

/// 任务动作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskAction {
    SendMessage {
        group_id: String,
        content: String,
    },
    CheckGroupActivity {
        group_id: String,
    },
    Custom {
        script_path: String,
        args: serde_json::Value,
    },
}

/// 任务队列
pub struct TaskQueue {
    tx: mpsc::Sender<Task>,
}

impl TaskQueue {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(1000);

        // 启动 worker 池（4 个并发 worker）
        // Note: mpsc::Receiver doesn't support clone, so we use a single worker for now
        tokio::spawn(async move {
            Self::worker(0, rx).await;
        });

        Self { tx }
    }
    
    async fn worker(id: usize, mut rx: mpsc::Receiver<Task>) {
        log::info!("Task Worker {} 启动", id);

        while let Some(task) = rx.recv().await {
            log::info!("Worker {} 执行任务：{}", id, task.id);
            Self::execute_task(task).await;
        }
    }

    #[allow(dead_code)]
    async fn execute_task(task: Task) {
        match task.action {
            TaskAction::SendMessage { group_id, content } => {
                log::info!("发送消息到群聊 {}: {}", group_id, content);
            }
            TaskAction::CheckGroupActivity { group_id } => {
                log::info!("检查群聊活跃度：{}", group_id);
            }
            TaskAction::Custom { script_path, args } => {
                log::info!("执行自定义脚本：{} ({})", script_path, args);
            }
        }
    }
    
    pub async fn submit(&self, task: Task) -> Result<(), String> {
        self.tx.send(task).await
            .map_err(|_| "任务队列已满".to_string())
    }

    pub async fn get_queue_size(&self) -> usize {
        // Simple implementation - in production could use a counter
        0
    }

    pub fn create_task(agent_id: String, action: TaskAction, priority: u8) -> Task {
        Task {
            id: format!("task_{}", uuid::Uuid::new_v4()),
            agent_id,
            action,
            priority,
            created_at: chrono::Utc::now().timestamp_millis(),
        }
    }
}

impl Default for TaskQueue {
    fn default() -> Self {
        Self::new()
    }
}
