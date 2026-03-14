//! Session 消息协议

use serde::{Deserialize, Serialize};

/// Session 消息协议
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionMessage {
    UserMessage {
        content: String,
        metadata: MessageMetadata,
    },
    ToolResult {
        tool_call_id: String,
        result: ToolResult,
    },
    ToolCall {
        tool_call_id: String,
        tool_name: String,
        arguments: serde_json::Value,
    },
    WorkflowStep {
        workflow_id: String,
        step_id: String,
        action: WorkflowStepAction,
    },
    WorkflowExecute {
        workflow_id: String,
        input: serde_json::Value,
    },
    WorkflowEvent {
        workflow_id: String,
        event_type: String,
        data: serde_json::Value,
    },
    SystemEvent {
        event_type: SystemEventType,
        data: serde_json::Value,
    },
    GroupChatMessage {
        group_id: String,
        from_agent: Option<String>,
        content: String,
    },
    GroupChatEvent {
        group_id: String,
        event_type: String,
    },
    SessionCreated,
    SessionDestroy,
    Ping,
    Pong,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MessageMetadata {
    pub timestamp: Option<i64>,
    pub user_id: Option<String>,
    pub source: Option<String>,
    pub stream: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowStepAction {
    Execute,
    Pause,
    Resume,
    Cancel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemEventType {
    Initialized,
    Error,
    Warning,
    ResourceExhausted,
    Timeout,
}

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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResponseMetadata {
    pub timestamp: i64,
    pub latency_ms: u64,
}
