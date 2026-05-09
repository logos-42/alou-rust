//! 任务管理
//!
//! 提供本地内存任务管理，支持任务创建、更新、删除和事件订阅

use super::ai_client::AiMessage;
use super::error::{AgentError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

/// 任务状态
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TaskStatus {
    /// 等待中
    Pending,
    /// 运行中
    Running,
    /// 正在执行工具
    ProcessingTools,
    /// 已完成
    Completed,
    /// 失败
    Failed,
    /// 已取消
    Cancelled,
}

/// 任务元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskMetadata {
    pub agent_id: String,
    pub workflow_id: Option<String>,
    pub iteration_count: u32,
    pub max_iterations: u32,
    pub ralph_loop_enabled: bool,
}

impl Default for TaskMetadata {
    fn default() -> Self {
        Self {
            agent_id: String::new(),
            workflow_id: None,
            iteration_count: 0,
            max_iterations: 10,
            ralph_loop_enabled: false,
        }
    }
}

/// 任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub status: TaskStatus,
    pub messages: Vec<AiMessage>,
    pub pending_tools: Vec<ToolCall>,
    pub tool_results: Vec<ToolResult>,
    pub final_response: Option<String>,
    pub error: Option<String>,
    #[serde(skip, default = "Instant::now")]
    pub created_at: Instant,
    #[serde(skip, default = "Instant::now")]
    pub updated_at: Instant,
    pub metadata: TaskMetadata,
    /// 🔥 前端传入的系统提示（优先使用）
    #[serde(skip)]
    pub system_prompt: Option<String>,
}

impl Default for Task {
    fn default() -> Self {
        Self {
            id: String::new(),
            status: TaskStatus::Pending,
            messages: Vec::new(),
            pending_tools: Vec::new(),
            tool_results: Vec::new(),
            final_response: None,
            error: None,
            created_at: Instant::now(),
            updated_at: Instant::now(),
            metadata: TaskMetadata::default(),
            system_prompt: None,
        }
    }
}

/// 工具调用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

/// 工具结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool_call_id: String,
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
}

impl ToolResult {
    /// 获取摘要（用于前端展示）
    pub fn summary(&self) -> ToolResultSummary {
        ToolResultSummary {
            success: self.success,
            error: self.error.clone(),
            data_preview: self
                .data
                .as_ref()
                .and_then(|d| serde_json::to_string(d).ok())
                .map(|s| s.chars().take(100).collect()),
        }
    }
}

/// 工具结果摘要（不包含详细内容）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResultSummary {
    pub success: bool,
    pub error: Option<String>,
    pub data_preview: Option<String>,
}

/// 任务事件
#[derive(Debug, Clone, Serialize)]
pub enum TaskEvent {
    /// 任务创建
    TaskCreated { task_id: String },

    /// 任务开始
    TaskStarted { task_id: String },

    /// AI 响应（内容）
    AiResponse { task_id: String, content: String },

    /// 待处理工具调用
    ToolCallsPending { task_id: String, count: usize },

    /// 工具开始执行
    ToolExecuting { task_id: String, tool_name: String, arguments: Option<serde_json::Value> },

    /// 工具执行完成
    ToolCompleted {
        task_id: String,
        tool_name: String,
        result: ToolResultSummary,
    },

    /// 任务进度更新
    TaskProgress {
        task_id: String,
        progress: f32,
        message: String,
    },

    /// 任务完成
    TaskCompleted { task_id: String, result: String },

    /// 任务失败
    TaskFailed { task_id: String, error: String },
}

/// 任务管理器（内存存储）
pub struct TaskManager {
    tasks: Arc<RwLock<HashMap<String, Task>>>,
    event_sender: tokio::sync::broadcast::Sender<TaskEvent>,
}

impl TaskManager {
    pub fn new() -> Self {
        let (tx, _) = tokio::sync::broadcast::channel(100);
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            event_sender: tx,
        }
    }

    /// 创建任务（单条消息）
    pub async fn create_task(&self, agent_id: String, initial_message: String) -> String {
        let messages = vec![AiMessage {
            role: "user".to_string(),
            content: initial_message,
            tool_call_id: None,
            tool_calls: None,
        }];
        self.create_task_with_messages(agent_id, messages).await
    }

    /// 创建任务（完整消息数组，支持 system/user/assistant 角色）
    pub async fn create_task_with_messages(&self, agent_id: String, messages: Vec<AiMessage>) -> String {
        self.create_task_with_messages_and_prompt(agent_id, messages, None).await
    }

    /// 🔥 创建任务（完整消息数组 + 系统提示）
    pub async fn create_task_with_messages_and_prompt(
        &self, 
        agent_id: String, 
        messages: Vec<AiMessage>,
        system_prompt: Option<String>,
    ) -> String {
        let task_id = uuid::Uuid::new_v4().to_string();
        let task = Task {
            id: task_id.clone(),
            status: TaskStatus::Pending,
            messages,
            pending_tools: Vec::new(),
            tool_results: Vec::new(),
            final_response: None,
            error: None,
            created_at: Instant::now(),
            updated_at: Instant::now(),
            metadata: TaskMetadata {
                agent_id,
                workflow_id: None,
                iteration_count: 0,
                max_iterations: 20, // 合理的最大迭代次数
                ralph_loop_enabled: true,
            },
            system_prompt,
        };

        let mut tasks = self.tasks.write().await;
        tasks.insert(task_id.clone(), task);

        let _ = self.event_sender.send(TaskEvent::TaskCreated { task_id: task_id.clone() });

        task_id
    }

    /// 获取任务
    pub async fn get_task(&self, task_id: &str) -> Option<Task> {
        let tasks = self.tasks.read().await;
        tasks.get(task_id).cloned()
    }

    /// 更新任务
    pub async fn update_task<F>(&self, task_id: &str, f: F) -> Result<()>
    where
        F: FnOnce(&mut Task),
    {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(task_id) {
            f(task);
            task.updated_at = Instant::now();
            Ok(())
        } else {
            Err(AgentError::TaskError(format!("任务不存在: {}", task_id)))
        }
    }

    /// 删除任务
    pub async fn delete_task(&self, task_id: &str) {
        let mut tasks = self.tasks.write().await;
        tasks.remove(task_id);
    }

    /// 订阅事件
    pub fn subscribe_events(&self) -> tokio::sync::broadcast::Receiver<TaskEvent> {
        self.event_sender.subscribe()
    }

    /// 发送事件
    pub async fn emit_event(&self, event: TaskEvent) {
        let _ = self.event_sender.send(event);
    }

    /// 🔥 获取所有待处理的任务（用于 AgentTick 自主执行）
    /// 只返回真正需要执行的任务（Pending 或 需要继续迭代的 Running）
    pub async fn get_pending_tasks(&self) -> Vec<Task> {
        let tasks = self.tasks.read().await;
        tasks
            .values()
            .filter(|task| {
                // 只处理 Pending 状态的任务
                // Running 状态的任务已经在执行中，不需要重复触发
                task.status == TaskStatus::Pending
            })
            .cloned()
            .collect()
    }

    /// 🔥 获取指定 session 的待处理任务
    pub async fn get_pending_tasks_for_session(&self, session_id: &str) -> Vec<Task> {
        let tasks = self.tasks.read().await;
        tasks
            .values()
            .filter(|task| {
                task.status == TaskStatus::Pending
            })
            .cloned()
            .collect()
    }
}

/// 任务结果（最终返回）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskFinalResult {
    pub task_id: String,
    pub success: bool,
    pub result: String,
    pub error: Option<String>,
    pub iteration_count: u32,
}
