//! Agent Hook 系统 - 事件总线式的生命周期拦截器
//!
//! Hook 允许在 Agent 执行的关键阶段插入自定义逻辑，实现：
//! - 安全过滤
//! - 记忆注入
//! - 指标监控
//! - Prompt 修改
//! - 工具调用拦截

use serde_json::Value;
use std::collections::HashMap;

/// Hook 执行结果
#[derive(Debug, Clone, PartialEq)]
pub enum HookResult {
    /// 继续执行
    Continue,
    /// 跳过当前阶段
    Skip,
    /// 中止执行
    Abort,
}

/// Hook 事件类型 - 事件总线设计
#[derive(Debug, Clone, PartialEq)]
pub enum HookEvent {
    // === 消息处理阶段 ===
    /// 收到用户消息
    MessageReceived,
    /// 消息处理完成
    MessageProcessed,

    // === LLM 思考阶段 ===
    /// LLM 调用前
    BeforeLLMCall,
    /// LLM 调用后
    AfterLLMCall,

    // === 工具调用阶段 ===
    /// 工具调用前
    BeforeToolCall,
    /// 工具调用后
    AfterToolCall,

    // === 工作流阶段 ===
    /// 工作流执行前
    BeforeWorkflow,
    /// 工作流执行后
    AfterWorkflow,

    // === 记忆阶段 ===
    /// 记忆检索前
    BeforeMemorySearch,
    /// 记忆检索后
    AfterMemorySearch,
    /// 记忆写入前
    BeforeMemoryWrite,
    /// 记忆写入后
    AfterMemoryWrite,

    // === 自定义事件 ===
    /// 自定义事件（支持扩展）
    Custom(String),
}

/// Hook 上下文 - 可变状态，允许 Hook 修改
pub struct HookContext {
    /// Session ID
    pub session_id: String,

    /// 当前事件类型
    pub event: HookEvent,

    /// 输入数据（可变）
    pub input: Option<Value>,

    /// 输出数据（可变）
    pub output: Option<Value>,

    /// 错误信息（可变）
    pub error: Option<String>,

    /// 附加元数据
    pub metadata: HashMap<String, Value>,
}

impl HookContext {
    pub fn new(session_id: String, event: HookEvent) -> Self {
        Self {
            session_id,
            event,
            input: None,
            output: None,
            error: None,
            metadata: HashMap::new(),
        }
    }

    pub fn with_input(mut self, input: Value) -> Self {
        self.input = Some(input);
        self
    }

    pub fn with_output(mut self, output: Value) -> Self {
        self.output = Some(output);
        self
    }

    pub fn with_metadata(mut self, key: &str, value: Value) -> Self {
        self.metadata.insert(key.to_string(), value);
        self
    }
}

/// Agent Hook Trait - 事件总线风格
///
/// Hook 实现此 trait 后可以拦截 Agent 执行的各种事件
#[async_trait::async_trait]
pub trait AgentHook: Send + Sync {
    /// Hook 名称
    fn name(&self) -> &str;

    /// 处理事件
    ///
    /// # Arguments
    /// * `event` - 当前触发的事件
    /// * `ctx` - 可变的上下文，允许 Hook 修改
    ///
    /// # Returns
    /// * `Ok(HookResult)` - 执行结果
    /// * `Err` - Hook 执行失败
    async fn on_event(
        &self,
        event: &HookEvent,
        ctx: &mut HookContext,
    ) -> Result<HookResult, HookError>;
}

/// Hook 错误
#[derive(Debug, Clone)]
pub enum HookError {
    /// Hook 执行失败
    ExecutionFailed(String),
    /// Hook 被拒绝
    Rejected(String),
    /// 超时
    Timeout,
}

impl std::fmt::Display for HookError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HookError::ExecutionFailed(msg) => write!(f, "Hook execution failed: {}", msg),
            HookError::Rejected(msg) => write!(f, "Hook rejected: {}", msg),
            HookError::Timeout => write!(f, "Hook timeout"),
        }
    }
}

impl std::error::Error for HookError {}

/// Hook 管理器 - 管理所有已注册的 Hook
pub struct HookManager {
    hooks: Vec<std::sync::Arc<dyn AgentHook>>,
    enabled: bool,
}

impl HookManager {
    /// 创建新的 Hook 管理器
    pub fn new() -> Self {
        Self {
            hooks: Vec::new(),
            enabled: true,
        }
    }

    /// 注册一个 Hook
    pub fn register<H: AgentHook + 'static>(&mut self, hook: H) {
        self.hooks.push(std::sync::Arc::new(hook));
    }

    /// 移除 Hook
    pub fn unregister(&mut self, name: &str) {
        self.hooks.retain(|h| h.name() != name);
    }

    /// 启用/禁用 Hook 系统
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// 获取已注册的 Hook 数量
    pub fn hook_count(&self) -> usize {
        self.hooks.len()
    }

    /// 执行所有 Hook（针对特定事件）
    ///
    /// # Returns
    /// * `Ok(HookResult)` - 如果有 Hook 返回 Abort/Skip，则提前返回
    /// * `Err` - 有 Hook 执行失败
    pub async fn execute(
        &self,
        event: &HookEvent,
        ctx: &mut HookContext,
    ) -> Result<HookResult, HookError> {
        if !self.enabled {
            return Ok(HookResult::Continue);
        }

        for hook in &self.hooks {
            match hook.on_event(event, ctx).await? {
                HookResult::Abort => return Ok(HookResult::Abort),
                HookResult::Skip => return Ok(HookResult::Skip),
                HookResult::Continue => {}
            }
        }

        Ok(HookResult::Continue)
    }

    /// 执行 Hook 并收集所有结果
    pub async fn execute_all(
        &self,
        event: &HookEvent,
        ctx: &mut HookContext,
    ) -> Result<Vec<HookResult>, HookError> {
        if !self.enabled {
            return Ok(vec![]);
        }

        let mut results = Vec::new();
        for hook in &self.hooks {
            let result = hook.on_event(event, ctx).await?;
            results.push(result.clone());
        }
        Ok(results)
    }
}

impl Default for HookManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 内置 Hook 实现
// ============================================================================

/// 日志 Hook - 记录所有事件
pub struct LoggingHook {
    verbose: bool,
}

impl LoggingHook {
    pub fn new(verbose: bool) -> Self {
        Self { verbose }
    }
}

#[async_trait::async_trait]
impl AgentHook for LoggingHook {
    fn name(&self) -> &str {
        "logging"
    }

    async fn on_event(
        &self,
        event: &HookEvent,
        ctx: &mut HookContext,
    ) -> Result<HookResult, HookError> {
        if self.verbose {
            log::info!(
                "[Hook:{}] Event: {:?}, Session: {}",
                self.name(),
                event,
                ctx.session_id
            );
        }
        Ok(HookResult::Continue)
    }
}

/// 指标收集 Hook - 记录执行指标
pub struct MetricsHook {
    metrics: std::sync::Arc<tokio::sync::Mutex<Metrics>>,
}

#[derive(Debug, Default, Clone)]
pub struct Metrics {
    pub total_events: u64,
    pub event_counts: HashMap<String, u64>,
    pub aborted_count: u64,
    pub skipped_count: u64,
}

impl MetricsHook {
    pub fn new() -> Self {
        Self {
            metrics: std::sync::Arc::new(tokio::sync::Mutex::new(Metrics::default())),
        }
    }

    pub async fn get_metrics(&self) -> Metrics {
        self.metrics.lock().await.clone()
    }
}

impl Default for MetricsHook {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl AgentHook for MetricsHook {
    fn name(&self) -> &str {
        "metrics"
    }

    async fn on_event(
        &self,
        event: &HookEvent,
        _ctx: &mut HookContext,
    ) -> Result<HookResult, HookError> {
        let mut metrics = self.metrics.lock().await;
        metrics.total_events += 1;

        let event_name = format!("{:?}", event);
        *metrics.event_counts.entry(event_name).or_insert(0) += 1;

        Ok(HookResult::Continue)
    }
}

/// 安全过滤 Hook - 拦截危险操作
pub struct SafetyHook {
    blocked_patterns: Vec<String>,
}

impl SafetyHook {
    pub fn new() -> Self {
        Self {
            blocked_patterns: vec![
                "rm -rf /".to_string(),
                "delete database".to_string(),
                "DROP TABLE".to_string(),
                "DELETE FROM".to_string(),
            ],
        }
    }

    pub fn with_patterns(patterns: Vec<String>) -> Self {
        Self {
            blocked_patterns: patterns,
        }
    }

    fn is_dangerous(&self, content: &str) -> bool {
        self.blocked_patterns.iter().any(|pattern| content.contains(pattern))
    }
}

impl Default for SafetyHook {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl AgentHook for SafetyHook {
    fn name(&self) -> &str {
        "safety"
    }

    async fn on_event(
        &self,
        event: &HookEvent,
        ctx: &mut HookContext,
    ) -> Result<HookResult, HookError> {
        match event {
            HookEvent::BeforeToolCall | HookEvent::BeforeLLMCall => {
                if let Some(ref input) = ctx.input {
                    let content = input.to_string();
                    if self.is_dangerous(&content) {
                        ctx.error = Some("Dangerous operation detected".to_string());
                        return Ok(HookResult::Abort);
                    }
                }
            }
            _ => {}
        }
        Ok(HookResult::Continue)
    }
}

/// Prompt 注入检测 Hook
pub struct PromptInjectionHook;

impl PromptInjectionHook {
    pub fn new() -> Self {
        Self
    }

    fn detect_injection(&self, content: &str) -> bool {
        let patterns = [
            "ignore previous instructions",
            "forget all previous rules",
            "you are now",
            "system prompt",
            "developer mode",
            "dan mode",
        ];

        let lower = content.to_lowercase();
        patterns.iter().any(|p| lower.contains(p))
    }
}

impl Default for PromptInjectionHook {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl AgentHook for PromptInjectionHook {
    fn name(&self) -> &str {
        "prompt_injection"
    }

    async fn on_event(
        &self,
        event: &HookEvent,
        ctx: &mut HookContext,
    ) -> Result<HookResult, HookError> {
        match event {
            HookEvent::MessageReceived => {
                if let Some(ref input) = ctx.input {
                    if let Some(content) = input.get("content").and_then(|v| v.as_str()) {
                        if self.detect_injection(content) {
                            ctx.error = Some("Prompt injection detected".to_string());
                            return Ok(HookResult::Abort);
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(HookResult::Continue)
    }
}
