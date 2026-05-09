//! Event Log - 事件日志系统
//!
//! 记录 Agent 执行的所有关键事件，支持：
//! - Debug/Replay：逐步回放 Agent 执行过程
//! - Metrics：统计 LLM 延迟、工具延迟、循环次数
//! - AI Training：导出会话数据为训练集
//! - Audit：审计 Agent 行为

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// 事件类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EventType {
    // === 消息事件 ===
    UserMessage,
    AssistantMessage,
    SystemMessage,
    ToolMessage,

    // === LLM 事件 ===
    LLMRequest,
    LLMResponse,
    LLMError,

    // === 工具事件 ===
    ToolCallStart,
    ToolCallEnd,
    ToolError,

    // === 工作流事件 ===
    WorkflowStart,
    WorkflowStep,
    WorkflowEnd,
    WorkflowError,

    // === Command 事件 ===
    CommandReceived,
    CommandProcessed,
    CommandError,

    // === Hook 事件 ===
    HookExecuted,
    HookAborted,
    HookError,

    // === Agent 状态事件 ===
    AgentStarted,
    AgentPaused,
    AgentResumed,
    AgentStopped,
    AgentError,

    // === 自定义事件 ===
    Custom(String),
}

/// 事件严重级别
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum EventLevel {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

/// 事件条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEntry {
    /// 事件 ID（UUID）
    pub id: String,

    /// Session ID
    pub session_id: String,

    /// 事件类型
    pub event_type: EventType,

    /// 严重级别
    pub level: EventLevel,

    /// 时间戳（毫秒）
    pub timestamp: i64,

    /// 事件数据
    pub data: Value,

    /// 元数据（用于索引）
    pub metadata: HashMap<String, String>,

    /// 父事件 ID（用于追踪因果关系）
    pub parent_id: Option<String>,

    /// 耗时（毫秒，用于性能分析）
    pub duration_ms: Option<u64>,
}

impl EventEntry {
    pub fn new(session_id: String, event_type: EventType, level: EventLevel, data: Value) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            session_id,
            event_type,
            level,
            timestamp: chrono::Utc::now().timestamp_millis(),
            data,
            metadata: HashMap::new(),
            parent_id: None,
            duration_ms: None,
        }
    }

    pub fn with_metadata(mut self, key: &str, value: String) -> Self {
        self.metadata.insert(key.to_string(), value);
        self
    }

    pub fn with_parent(mut self, parent_id: String) -> Self {
        self.parent_id = Some(parent_id);
        self
    }

    pub fn with_duration(mut self, duration_ms: u64) -> Self {
        self.duration_ms = Some(duration_ms);
        self
    }

    /// 创建用户消息事件
    pub fn user_message(session_id: String, content: String) -> Self {
        Self::new(
            session_id,
            EventType::UserMessage,
            EventLevel::Info,
            serde_json::json!({
                "content": content,
            }),
        )
    }

    /// 创建助手消息事件
    pub fn assistant_message(session_id: String, content: String) -> Self {
        Self::new(
            session_id,
            EventType::AssistantMessage,
            EventLevel::Info,
            serde_json::json!({
                "content": content,
            }),
        )
    }

    /// 创建 LLM 请求事件
    pub fn llm_request(session_id: String, model: String, prompt_length: usize) -> Self {
        Self::new(
            session_id,
            EventType::LLMRequest,
            EventLevel::Debug,
            serde_json::json!({
                "model": model,
                "prompt_length": prompt_length,
            }),
        )
    }

    /// 创建 LLM 响应事件
    pub fn llm_response(
        session_id: String,
        content_length: usize,
        tool_calls_count: usize,
        latency_ms: u64,
    ) -> Self {
        Self::new(
            session_id,
            EventType::LLMResponse,
            EventLevel::Info,
            serde_json::json!({
                "content_length": content_length,
                "tool_calls_count": tool_calls_count,
                "latency_ms": latency_ms,
            }),
        )
        .with_duration(latency_ms)
    }

    /// 创建工具调用开始事件
    pub fn tool_call_start(session_id: String, tool_name: String, args: Value) -> Self {
        Self::new(
            session_id,
            EventType::ToolCallStart,
            EventLevel::Debug,
            serde_json::json!({
                "tool_name": tool_name,
                "arguments": args,
            }),
        )
    }

    /// 创建工具调用结束事件
    pub fn tool_call_end(
        session_id: String,
        tool_name: String,
        result: Value,
        latency_ms: u64,
    ) -> Self {
        Self::new(
            session_id,
            EventType::ToolCallEnd,
            EventLevel::Info,
            serde_json::json!({
                "tool_name": tool_name,
                "success": !result.is_null(),
                "latency_ms": latency_ms,
            }),
        )
        .with_duration(latency_ms)
    }

    /// 创建 Command 事件
    pub fn command_received(session_id: String, command: String) -> Self {
        Self::new(
            session_id,
            EventType::CommandReceived,
            EventLevel::Info,
            serde_json::json!({
                "command": command,
            }),
        )
    }

    /// 创建 Hook 事件
    pub fn hook_executed(
        session_id: String,
        hook_name: String,
        event_type: String,
        result: String,
    ) -> Self {
        Self::new(
            session_id,
            EventType::HookExecuted,
            EventLevel::Debug,
            serde_json::json!({
                "hook_name": hook_name,
                "event_type": event_type,
                "result": result,
            }),
        )
    }
}

/// 事件查询条件
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EventQuery {
    /// Session ID 过滤
    pub session_id: Option<String>,

    /// 事件类型过滤
    pub event_types: Option<Vec<EventType>>,

    /// 严重级别过滤
    pub min_level: Option<EventLevel>,

    /// 时间范围（开始）
    pub start_time: Option<i64>,

    /// 时间范围（结束）
    pub end_time: Option<i64>,

    /// 元数据过滤
    pub metadata: Option<HashMap<String, String>>,

    /// 限制返回数量
    pub limit: Option<usize>,

    /// 偏移量
    pub offset: Option<usize>,
}

/// 事件存储 Trait
#[async_trait::async_trait]
pub trait EventStore: Send + Sync {
    /// 追加事件
    async fn append(&self, event: EventEntry) -> Result<(), EventLogError>;

    /// 查询事件
    async fn query(&self, query: EventQuery) -> Result<Vec<EventEntry>, EventLogError>;

    /// 获取 Session 的所有事件
    async fn get_session_events(
        &self,
        session_id: &str,
    ) -> Result<Vec<EventEntry>, EventLogError>;

    /// 删除 Session 的事件
    async fn delete_session(&self, session_id: &str) -> Result<(), EventLogError>;

    /// 清空所有事件
    async fn clear(&self) -> Result<(), EventLogError>;
}

/// 内存事件存储（默认实现）
pub struct InMemoryEventStore {
    events: tokio::sync::RwLock<Vec<EventEntry>>,
}

impl InMemoryEventStore {
    pub fn new() -> Self {
        Self {
            events: tokio::sync::RwLock::new(Vec::new()),
        }
    }
}

impl Default for InMemoryEventStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl EventStore for InMemoryEventStore {
    async fn append(&self, event: EventEntry) -> Result<(), EventLogError> {
        let mut events = self.events.write().await;
        events.push(event);
        Ok(())
    }

    async fn query(&self, query: EventQuery) -> Result<Vec<EventEntry>, EventLogError> {
        let events = self.events.read().await;

        let mut filtered: Vec<&EventEntry> = events.iter().collect();

        // Session ID 过滤
        if let Some(ref session_id) = query.session_id {
            filtered.retain(|e| &e.session_id == session_id);
        }

        // 事件类型过滤
        if let Some(ref event_types) = query.event_types {
            filtered.retain(|e| event_types.contains(&e.event_type));
        }

        // 严重级别过滤
        if let Some(min_level) = query.min_level {
            filtered.retain(|e| e.level >= min_level);
        }

        // 时间范围过滤
        if let Some(start_time) = query.start_time {
            filtered.retain(|e| e.timestamp >= start_time);
        }
        if let Some(end_time) = query.end_time {
            filtered.retain(|e| e.timestamp <= end_time);
        }

        // 元数据过滤
        if let Some(ref metadata) = query.metadata {
            filtered.retain(|e| {
                metadata
                    .iter()
                    .all(|(k, v)| e.metadata.get(k).map(|s| s == v).unwrap_or(false))
            });
        }

        // 分页
        let offset = query.offset.unwrap_or(0);
        let limit = query.limit.unwrap_or(usize::MAX);
        let result: Vec<EventEntry> = filtered
            .into_iter()
            .skip(offset)
            .take(limit)
            .cloned()
            .collect();

        Ok(result)
    }

    async fn get_session_events(
        &self,
        session_id: &str,
    ) -> Result<Vec<EventEntry>, EventLogError> {
        let query = EventQuery {
            session_id: Some(session_id.to_string()),
            ..Default::default()
        };
        self.query(query).await
    }

    async fn delete_session(&self, session_id: &str) -> Result<(), EventLogError> {
        let mut events = self.events.write().await;
        events.retain(|e| e.session_id != session_id);
        Ok(())
    }

    async fn clear(&self) -> Result<(), EventLogError> {
        let mut events = self.events.write().await;
        events.clear();
        Ok(())
    }
}

/// 事件日志管理器
pub struct EventLog {
    store: std::sync::Arc<dyn EventStore>,
    enabled: bool,
}

impl EventLog {
    pub fn new(store: std::sync::Arc<dyn EventStore>) -> Self {
        Self {
            store,
            enabled: true,
        }
    }

    pub fn with_in_memory() -> Self {
        Self::new(std::sync::Arc::new(InMemoryEventStore::new()))
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// 追加事件
    pub async fn append(&self, event: EventEntry) -> Result<(), EventLogError> {
        if !self.enabled {
            return Ok(());
        }
        self.store.append(event).await
    }

    /// 记录用户消息
    pub async fn log_user_message(
        &self,
        session_id: &str,
        content: &str,
    ) -> Result<(), EventLogError> {
        let event = EventEntry::user_message(session_id.to_string(), content.to_string());
        self.append(event).await
    }

    /// 记录助手消息
    pub async fn log_assistant_message(
        &self,
        session_id: &str,
        content: &str,
    ) -> Result<(), EventLogError> {
        let event = EventEntry::assistant_message(session_id.to_string(), content.to_string());
        self.append(event).await
    }

    /// 记录 LLM 请求
    pub async fn log_llm_request(
        &self,
        session_id: &str,
        model: &str,
        prompt_length: usize,
    ) -> Result<(), EventLogError> {
        let event = EventEntry::llm_request(session_id.to_string(), model.to_string(), prompt_length);
        self.append(event).await
    }

    /// 记录 LLM 响应
    pub async fn log_llm_response(
        &self,
        session_id: &str,
        content_length: usize,
        tool_calls_count: usize,
        latency_ms: u64,
    ) -> Result<(), EventLogError> {
        let event = EventEntry::llm_response(
            session_id.to_string(),
            content_length,
            tool_calls_count,
            latency_ms,
        );
        self.append(event).await
    }

    /// 记录工具调用开始
    pub async fn log_tool_call_start(
        &self,
        session_id: &str,
        tool_name: &str,
        args: &Value,
    ) -> Result<(), EventLogError> {
        let event =
            EventEntry::tool_call_start(session_id.to_string(), tool_name.to_string(), args.clone());
        self.append(event).await
    }

    /// 记录工具调用结束
    pub async fn log_tool_call_end(
        &self,
        session_id: &str,
        tool_name: &str,
        result: &Value,
        latency_ms: u64,
    ) -> Result<(), EventLogError> {
        let event = EventEntry::tool_call_end(
            session_id.to_string(),
            tool_name.to_string(),
            result.clone(),
            latency_ms,
        );
        self.append(event).await
    }

    /// 记录 Command
    pub async fn log_command(&self, session_id: &str, command: &str) -> Result<(), EventLogError> {
        let event = EventEntry::command_received(session_id.to_string(), command.to_string());
        self.append(event).await
    }

    /// 记录 Hook 执行
    pub async fn log_hook(
        &self,
        session_id: &str,
        hook_name: &str,
        event_type: &str,
        result: &str,
    ) -> Result<(), EventLogError> {
        let event = EventEntry::hook_executed(
            session_id.to_string(),
            hook_name.to_string(),
            event_type.to_string(),
            result.to_string(),
        );
        self.append(event).await
    }

    /// 查询事件
    pub async fn query(&self, query: EventQuery) -> Result<Vec<EventEntry>, EventLogError> {
        self.store.query(query).await
    }

    /// 获取 Session 的所有事件
    pub async fn get_session_events(
        &self,
        session_id: &str,
    ) -> Result<Vec<EventEntry>, EventLogError> {
        self.store.get_session_events(session_id).await
    }

    /// 导出 Session 为数据集（用于 AI 训练）
    pub async fn export_session(
        &self,
        session_id: &str,
    ) -> Result<SessionDataset, EventLogError> {
        let events = self.get_session_events(session_id).await?;

        // 提取对话对
        let mut conversations = Vec::new();
        let mut last_user_message = None;

        for event in &events {
            match &event.event_type {
                EventType::UserMessage => {
                    if let Some(content) = event.data.get("content").and_then(|v| v.as_str()) {
                        last_user_message = Some(content.to_string());
                    }
                }
                EventType::AssistantMessage => {
                    if let Some(content) = event.data.get("content").and_then(|v| v.as_str()) {
                        if let Some(user_msg) = last_user_message.take() {
                            conversations.push(ConversationPair {
                                user: user_msg,
                                assistant: content.to_string(),
                            });
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(SessionDataset {
            session_id: session_id.to_string(),
            total_events: events.len(),
            conversations,
            created_at: events.first().map(|e| e.timestamp).unwrap_or(0),
            updated_at: events.last().map(|e| e.timestamp).unwrap_or(0),
        })
    }

    /// 生成 Session 指标
    pub async fn generate_metrics(
        &self,
        session_id: &str,
    ) -> Result<SessionMetrics, EventLogError> {
        let events = self.get_session_events(session_id).await?;

        let mut metrics = SessionMetrics::default();

        for event in &events {
            match &event.event_type {
                EventType::UserMessage => metrics.user_messages += 1,
                EventType::AssistantMessage => metrics.assistant_messages += 1,
                EventType::ToolCallStart => metrics.tool_calls += 1,
                EventType::LLMRequest => metrics.llm_requests += 1,
                EventType::CommandReceived => metrics.commands += 1,
                EventType::HookExecuted => metrics.hook_executions += 1,
                _ => {}
            }

            // 计算延迟
            if let Some(duration) = event.duration_ms {
                match &event.event_type {
                    EventType::LLMResponse => {
                        metrics.total_llm_latency_ms += duration;
                        metrics.llm_response_count += 1;
                    }
                    EventType::ToolCallEnd => {
                        metrics.total_tool_latency_ms += duration;
                        metrics.tool_call_count += 1;
                    }
                    _ => {}
                }
            }
        }

        // 计算平均值
        if metrics.llm_response_count > 0 {
            metrics.avg_llm_latency_ms =
                metrics.total_llm_latency_ms / metrics.llm_response_count;
        }
        if metrics.tool_call_count > 0 {
            metrics.avg_tool_latency_ms =
                metrics.total_tool_latency_ms / metrics.tool_call_count;
        }

        metrics.session_id = session_id.to_string();
        metrics.total_events = events.len();

        Ok(metrics)
    }
}

impl Default for EventLog {
    fn default() -> Self {
        Self::with_in_memory()
    }
}

/// 会话数据集（用于 AI 训练）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionDataset {
    pub session_id: String,
    pub total_events: usize,
    pub conversations: Vec<ConversationPair>,
    pub created_at: i64,
    pub updated_at: i64,
}

/// 对话对（用于训练）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationPair {
    pub user: String,
    pub assistant: String,
}

/// Session 指标
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionMetrics {
    pub session_id: String,
    pub total_events: usize,
    pub user_messages: usize,
    pub assistant_messages: usize,
    pub tool_calls: usize,
    pub llm_requests: usize,
    pub commands: usize,
    pub hook_executions: usize,
    pub total_llm_latency_ms: u64,
    pub llm_response_count: u64,
    pub avg_llm_latency_ms: u64,
    pub total_tool_latency_ms: u64,
    pub tool_call_count: u64,
    pub avg_tool_latency_ms: u64,
}

/// 事件日志错误
#[derive(Debug, Clone)]
pub enum EventLogError {
    StorageError(String),
    NotFound(String),
    InvalidQuery(String),
}

impl std::fmt::Display for EventLogError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventLogError::StorageError(msg) => write!(f, "Storage error: {}", msg),
            EventLogError::NotFound(msg) => write!(f, "Not found: {}", msg),
            EventLogError::InvalidQuery(msg) => write!(f, "Invalid query: {}", msg),
        }
    }
}

impl std::error::Error for EventLogError {}
