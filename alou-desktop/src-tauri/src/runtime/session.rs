//! Session 运行时状态
//!
//! # 架构原则
//!
//! **Agent 决策，Workflow 执行**
//!
//! SessionRuntime 只包含 Agent 认知状态，不包含 Workflow 执行状态。
//! Workflow 由独立的 WorkflowEngine 管理，Session 只通过 WorkflowClient 调用。
//!
//! ## Agent Runtime v1 组件
//! - HookManager: 事件总线式生命周期拦截
//! - CommandQueue: 主动控制命令
//! - EventLog: 事件日志（Debug/Replay/Metrics）

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::runtime::message::SessionMessage;

/// Session 运行时状态
///
/// 被 SessionActor 独占拥有，**不实现 Clone**
pub struct SessionRuntime {
    /// Session 唯一标识
    pub session_id: String,

    /// 关联的 Agent ID（可选）
    pub agent_id: Option<String>,

    /// Agent 状态（认知循环）
    pub agent_state: AgentState,

    /// 记忆状态
    pub memory_state: MemoryState,

    /// 消息缓冲区（带容量限制）
    pub message_buffer: MessageBuffer,

    /// Workflow 客户端（纯接口，无状态）
    pub workflow_client: WorkflowClient,

    // ========================================================================
    // Agent Runtime v1 核心组件
    // ========================================================================
    /// Hook 管理器 - 事件总线式生命周期拦截
    pub hook_manager: crate::runtime::hook::HookManager,
    /// Command 队列 - 主动控制命令
    pub command_queue: crate::runtime::command::CommandQueue,
    /// 事件日志 - 记录所有关键事件（Debug/Replay/Metrics）
    pub event_log: crate::runtime::event_log::EventLog,
}

impl SessionRuntime {
    /// 创建新的 SessionRuntime
    pub fn new(session_id: String) -> Self {
        Self {
            session_id: session_id.clone(),
            agent_id: None,
            agent_state: AgentState::new(),
            memory_state: MemoryState::new(),
            message_buffer: MessageBuffer::new(100),
            workflow_client: WorkflowClient::new(),
            hook_manager: crate::runtime::hook::HookManager::new(),
            command_queue: crate::runtime::command::CommandQueue::new(),
            event_log: crate::runtime::event_log::EventLog::with_in_memory(),
        }
    }
    
    /// 设置关联的 Agent ID
    pub fn set_agent_id(&mut self, agent_id: String) {
        self.agent_id = Some(agent_id);
    }
    
    /// 清理资源
    pub fn cleanup(&mut self) {
        self.agent_state.clear();
        self.memory_state.clear();
        self.message_buffer.clear();
    }
}

// 注意：SessionRuntime **不实现 Clone**，保持 Actor 独占语义

/// 消息缓冲区（带容量限制）
pub struct MessageBuffer {
    messages: Vec<SessionMessage>,
    max_size: usize,
}

impl MessageBuffer {
    pub fn new(max_size: usize) -> Self {
        Self {
            messages: Vec::new(),
            max_size,
        }
    }
    
    pub fn push(&mut self, msg: SessionMessage) {
        self.messages.push(msg);
        if self.messages.len() > self.max_size {
            self.messages.remove(0);
        }
    }
    
    pub fn clear(&mut self) {
        self.messages.clear();
    }
}

impl Default for MessageBuffer {
    fn default() -> Self {
        Self::new(100)
    }
}

/// Agent 状态
pub struct AgentState {
    pub message_history: Vec<AiMessage>,
    pub pending_tools: Vec<ToolCall>,
    pub context_events: Vec<ContextEvent>,
}

impl AgentState {
    pub fn new() -> Self {
        Self {
            message_history: Vec::new(),
            pending_tools: Vec::new(),
            context_events: Vec::new(),
        }
    }
    
    pub fn add_message(&mut self, role: &str, content: String) {
        self.message_history.push(AiMessage {
            role: role.to_string(),
            content,
            tool_call_id: None,
            tool_calls: None,
        });
        
        if self.message_history.len() > 200 {
            self.message_history.remove(0);
        }
    }
    
    pub fn clear(&mut self) {
        self.message_history.clear();
        self.pending_tools.clear();
        self.context_events.clear();
    }
}

impl Default for AgentState {
    fn default() -> Self {
        Self::new()
    }
}

/// AI 消息结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiMessage {
    pub role: String,
    pub content: String,
    pub tool_call_id: Option<String>,
    pub tool_calls: Option<Vec<ToolCall>>,
}

/// 工具调用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

/// 上下文事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextEvent {
    pub action: String,
    pub detail: serde_json::Value,
    pub timestamp: i64,
}

impl ContextEvent {
    pub fn new(action: String, detail: serde_json::Value) -> Self {
        Self {
            action,
            detail,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }
}

/// 记忆状态
pub struct MemoryState {
    pub short_term: Vec<MemoryEntry>,
    pub long_term: HashMap<String, MemoryEntry>,
}

impl MemoryState {
    pub fn new() -> Self {
        Self {
            short_term: Vec::new(),
            long_term: HashMap::new(),
        }
    }
    
    pub fn add_short_term(&mut self, entry: MemoryEntry) {
        self.short_term.push(entry);
        if self.short_term.len() > 100 {
            self.short_term.remove(0);
        }
    }
    
    pub fn add_long_term(&mut self, key: String, entry: MemoryEntry) {
        self.long_term.insert(key, entry);
    }
    
    pub fn clear(&mut self) {
        self.short_term.clear();
        self.long_term.clear();
    }
}

impl Default for MemoryState {
    fn default() -> Self {
        Self::new()
    }
}

/// 记忆条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub key: String,
    pub value: serde_json::Value,
    pub created_at: i64,
}

/// Workflow 客户端（纯接口，无状态）
pub struct WorkflowClient;

impl WorkflowClient {
    pub fn new() -> Self {
        Self
    }
}

impl Default for WorkflowClient {
    fn default() -> Self {
        Self::new()
    }
}
