//! Scheduler 类型定义

use serde::{Deserialize, Serialize};

/// Agent 优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AgentPriority {
    /// 后台任务（最低优先级）
    Background = 0,
    /// 正常任务
    Normal = 1,
    /// 用户交互任务
    Interactive = 2,
    /// 关键任务（最高优先级）
    Critical = 3,
}

impl Default for AgentPriority {
    fn default() -> Self {
        AgentPriority::Normal
    }
}

/// Agent 状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentState {
    /// 空闲
    Idle,
    /// 正在执行任务
    Busy {
        task_id: String,
        started_at: i64,
    },
    /// 等待中（资源不足）
    Waiting {
        resource: String,
        since: i64,
    },
    /// 已暂停
    Paused,
    /// 错误状态
    Error {
        message: String,
        since: i64,
    },
}

/// Agent 任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTask {
    /// 任务 ID
    pub task_id: String,
    
    /// Session ID
    pub session_id: String,
    
    /// Agent ID
    pub agent_id: Option<String>,
    
    /// 任务类型
    pub task_type: TaskType,
    
    /// 优先级
    pub priority: AgentPriority,
    
    /// 创建时间
    pub created_at: i64,
    
    /// 截止时间（可选）
    pub deadline: Option<i64>,
    
    /// 重试次数
    pub retry_count: u32,
    
    /// 最大重试次数
    pub max_retries: u32,
    
    /// 任务负载（估计的执行时间，毫秒）
    pub estimated_load_ms: u64,
    
    /// 任务输入
    pub input: serde_json::Value,
}

impl AgentTask {
    pub fn new(
        session_id: String,
        task_type: TaskType,
        input: serde_json::Value,
    ) -> Self {
        Self {
            task_id: format!("task_{}", uuid::Uuid::new_v4()),
            session_id,
            agent_id: None,
            task_type,
            priority: AgentPriority::default(),
            created_at: chrono::Utc::now().timestamp(),
            deadline: None,
            retry_count: 0,
            max_retries: 3,
            estimated_load_ms: 5000,  // 默认 5 秒
            input,
        }
    }
    
    /// 设置优先级
    pub fn with_priority(mut self, priority: AgentPriority) -> Self {
        self.priority = priority;
        self
    }
    
    /// 设置截止时间
    pub fn with_deadline(mut self, deadline_secs: u64) -> Self {
        self.deadline = Some(chrono::Utc::now().timestamp() + deadline_secs as i64);
        self
    }
    
    /// 设置估计负载
    pub fn with_load(mut self, load_ms: u64) -> Self {
        self.estimated_load_ms = load_ms;
        self
    }
    
    /// 检查是否超时
    pub fn is_expired(&self) -> bool {
        if let Some(deadline) = self.deadline {
            chrono::Utc::now().timestamp() > deadline
        } else {
            false
        }
    }
    
    /// 检查是否可以重试
    pub fn can_retry(&self) -> bool {
        self.retry_count < self.max_retries
    }
}

/// 任务类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskType {
    /// Agent 对话（LLM 调用）
    AgentChat,
    
    /// 工具调用
    ToolCall {
        tool_name: String,
    },
    
    /// 工作流执行
    Workflow {
        workflow_id: String,
    },
    
    /// 群聊消息处理
    GroupChat {
        group_id: String,
    },
    
    /// 后台任务（记忆整理等）
    Background {
        task_name: String,
    },
    
    /// 系统任务
    System {
        task_name: String,
    },
}

/// 调度决策
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulingDecision {
    /// 任务 ID
    pub task_id: String,
    
    /// 决策类型
    pub decision: DecisionType,
    
    /// 决策时间
    pub decided_at: i64,
    
    /// 决策原因
    pub reason: String,
}

/// 决策类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DecisionType {
    /// 立即执行
    ExecuteNow,
    
    /// 加入队列
    Enqueue {
        position: usize,
    },
    
    /// 拒绝（资源不足）
    Reject {
        retry_after_secs: Option<u64>,
    },
    
    /// 抢占（高优先级）
    Preempt {
        preempted_task_id: String,
    },
}
