//! Session 消息协议
//!
//! 定义所有 SessionActor 处理的消息类型

use serde::{Deserialize, Serialize};

/// Session 消息协议
///
/// 所有输入都转换为消息，由 SessionActor 顺序处理
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionMessage {
    /// 用户消息
    UserMessage {
        content: String,
        metadata: MessageMetadata,
    },

    /// 工具调用结果
    ToolResult {
        tool_call_id: String,
        result: ToolResult,
    },

    /// 工具调用请求
    ToolCall {
        tool_call_id: String,
        tool_name: String,
        arguments: serde_json::Value,
    },

    /// 工作流事件
    WorkflowStep {
        workflow_id: String,
        step_id: String,
        action: WorkflowStepAction,
    },

    /// 工作流执行
    WorkflowExecute {
        workflow_id: String,
        input: serde_json::Value,
    },

    /// 工作流事件（从 WorkflowEngine 发出）
    WorkflowEvent {
        workflow_id: String,
        event_type: String,
        data: serde_json::Value,
    },

    /// 系统事件
    SystemEvent {
        event_type: SystemEventType,
        data: serde_json::Value,
    },

    /// 群聊消息
    GroupChatMessage {
        group_id: String,
        from_agent: Option<String>,
        content: String,
    },

    /// 群聊事件
    GroupChatEvent {
        group_id: String,
        event_type: String,
    },

    /// Session 创建
    SessionCreated,

    /// Session 销毁
    SessionDestroy,

    /// 心跳
    Ping,

    /// 心跳响应
    Pong,

    // ========================================================================
    // Agent 控制命令（Hook + Command 系统）
    // ========================================================================
    /// Agent 命令 - 主动控制 Agent 行为
    Command {
        command: crate::runtime::command::AgentCommand,
    },
}

/// 消息元数据
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MessageMetadata {
    pub timestamp: Option<i64>,
    pub user_id: Option<String>,
    pub source: Option<String>,
}

/// 工具调用结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
}

/// 工具调用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

/// 工作流步骤动作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowStepAction {
    Execute,
    Pause,
    Resume,
    Cancel,
}

/// 系统事件类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemEventType {
    Initialized,
    Error,
    Warning,
    ResourceExhausted,
    Timeout,
}

/// Session 响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionResponse {
    pub id: String,
    pub content: String,
    pub metadata: ResponseMetadata,
}

impl SessionResponse {
    pub fn text(content: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            content,
            metadata: ResponseMetadata::default(),
        }
    }
}

/// 响应元数据
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResponseMetadata {
    pub timestamp: i64,
    pub latency_ms: u64,
}
