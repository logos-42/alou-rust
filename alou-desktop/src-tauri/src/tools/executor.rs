//! 工具执行器核心
//!
//! 提供统一的工具执行接口，支持异步执行、超时控制、错误处理等功能

use super::{ToolMetadata, ExecutionContext, ToolConfig, ToolStatus, ToolPriority, ToolCategory};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{timeout, Duration};

/// 工具执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// 执行是否成功
    pub success: bool,
    /// 结果数据
    pub data: serde_json::Value,
    /// 错误信息（如果有）
    pub error: Option<String>,
    /// 执行时间（毫秒）
    pub execution_time_ms: u64,
    /// 输出消息
    pub output: Option<String>,
    /// 警告信息
    pub warnings: Vec<String>,
    /// 执行上下文
    pub context: Option<ExecutionContext>,
}

/// 工具错误类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolError {
    /// 权限不足
    PermissionDenied(String),
    /// 工具不可用
    ToolUnavailable(String),
    /// 执行超时
    Timeout(String),
    /// 参数错误
    InvalidArguments(String),
    /// 执行失败
    ExecutionFailed(String),
    /// 内部错误
    InternalError(String),
    /// 取消执行
    Cancelled(String),
}

impl std::fmt::Display for ToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToolError::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
            ToolError::ToolUnavailable(msg) => write!(f, "Tool unavailable: {}", msg),
            ToolError::Timeout(msg) => write!(f, "Timeout: {}", msg),
            ToolError::InvalidArguments(msg) => write!(f, "Invalid arguments: {}", msg),
            ToolError::ExecutionFailed(msg) => write!(f, "Execution failed: {}", msg),
            ToolError::InternalError(msg) => write!(f, "Internal error: {}", msg),
            ToolError::Cancelled(msg) => write!(f, "Cancelled: {}", msg),
        }
    }
}

impl std::error::Error for ToolError {}

/// 工具执行上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolContext {
    /// 会话ID
    pub session_id: String,
    /// 工具ID
    pub tool_id: String,
    /// 执行ID（唯一标识一次执行）
    pub execution_id: String,
    /// 开始时间
    pub start_time: i64,
    /// 超时时间
    pub timeout_seconds: Option<u64>,
    /// 进度（0.0-1.0）
    pub progress: f64,
    /// 状态
    pub status: ExecutionStatus,
    /// 中间结果
    pub intermediate_results: Vec<serde_json::Value>,
}

/// 执行状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionStatus {
    /// 等待执行
    Pending,
    /// 正在执行
    Running,
    /// 已完成
    Completed,
    /// 已失败
    Failed,
    /// 已取消
    Cancelled,
}

/// 工具执行器 trait
#[async_trait]
pub trait ToolExecutor: Send + Sync {
    /// 获取工具元信息
    fn metadata(&self) -> &ToolMetadata;

    /// 执行工具
    async fn execute(
        &self,
        args: serde_json::Value,
        context: &ExecutionContext,
    ) -> Result<ToolResult, ToolError>;

    /// 验证参数
    async fn validate_args(
        &self,
        args: &serde_json::Value,
    ) -> Result<(), ToolError>;

    /// 获取工具帮助信息
    fn help(&self) -> String;

    /// 检查工具是否可用
    async fn is_available(&self) -> bool {
        true
    }

    /// 获取工具优先级
    fn priority(&self) -> ToolPriority {
        ToolPriority::Medium
    }

    /// 获取支持的平台
    fn supported_platforms(&self) -> Vec<String> {
        vec!["windows".to_string(), "macos".to_string(), "linux".to_string()]
    }
}

/// 工具执行器管理器
pub struct ToolExecutionManager {
    /// 工具执行器映射
    executors: HashMap<String, Arc<dyn ToolExecutor>>,
    /// 执行上下文映射
    contexts: Arc<Mutex<HashMap<String, ToolContext>>>,
    /// 配置
    config: ToolConfig,
    /// 活跃执行计数器
    active_executions: Arc<Mutex<usize>>,
}

impl ToolExecutionManager {
    /// 创建新的执行管理器
    pub fn new(config: ToolConfig) -> Self {
        Self {
            executors: HashMap::new(),
            contexts: Arc::new(Mutex::new(HashMap::new())),
            config,
            active_executions: Arc::new(Mutex::new(0)),
        }
    }

    /// 注册工具执行器
    pub fn register_executor(&mut self, tool_id: String, executor: Arc<dyn ToolExecutor>) {
        self.executors.insert(tool_id, executor);
    }

    /// 获取工具执行器
    pub fn get_executor(&self, tool_id: &str) -> Option<&Arc<dyn ToolExecutor>> {
        self.executors.get(tool_id)
    }

    /// 执行工具（带超时和并发控制）
    pub async fn execute_tool(
        &self,
        tool_id: &str,
        args: serde_json::Value,
        context: ExecutionContext,
    ) -> Result<ToolResult, ToolError> {
        // 检查并发限制
        {
            let mut active = self.active_executions.lock().await;
            if *active >= self.config.max_concurrent {
                return Err(ToolError::ExecutionFailed(
                    "Too many concurrent executions".to_string()
                ));
            }
            *active += 1;
        }

        // 获取执行器
        let executor = self.executors.get(tool_id)
            .ok_or_else(|| ToolError::ToolUnavailable(
                format!("Tool '{}' not found", tool_id)
            ))?
            .clone();

        // 检查工具是否可用
        if !executor.is_available().await {
            let mut active = self.active_executions.lock().await;
            *active -= 1;
            return Err(ToolError::ToolUnavailable(
                format!("Tool '{}' is not available", tool_id)
            ));
        }

        // 验证参数
        executor.validate_args(&args).await?;

        // 创建执行上下文
        let execution_id = format!("exec_{}_{}", tool_id, uuid::Uuid::new_v4().to_string());
        let tool_context = ToolContext {
            session_id: context.session_id.clone(),
            tool_id: tool_id.to_string(),
            execution_id: execution_id.clone(),
            start_time: chrono::Utc::now().timestamp(),
            timeout_seconds: context.timeout_seconds,
            progress: 0.0,
            status: ExecutionStatus::Pending,
            intermediate_results: Vec::new(),
        };

        // 存储执行上下文
        {
            let mut contexts = self.contexts.lock().await;
            contexts.insert(execution_id.clone(), tool_context);
        }

        // 执行工具（带超时控制）
        let start_time = std::time::Instant::now();
        let timeout_duration = context.timeout_seconds
            .unwrap_or(self.config.default_timeout);

        let result = match timeout(
            Duration::from_secs(timeout_duration),
            self.execute_with_context(&executor, args, &context, execution_id.clone())
        ).await {
            Ok(result) => result,
            Err(_) => {
                // 更新状态为失败
                self.update_context_status(&execution_id, ExecutionStatus::Failed).await;
                let mut active = self.active_executions.lock().await;
                *active -= 1;
                return Err(ToolError::Timeout(
                    format!("Tool execution timed out after {} seconds", timeout_duration)
                ));
            }
        };

        let execution_time = start_time.elapsed().as_millis() as u64;

        // 更新最终状态
        let final_status = if result.success {
            ExecutionStatus::Completed
        } else {
            ExecutionStatus::Failed
        };
        self.update_context_status(&execution_id, final_status).await;

        // 减少活跃执行计数
        let mut active = self.active_executions.lock().await;
        *active -= 1;

        // 返回结果
        Ok(ToolResult {
            success: result.success,
            data: result.data,
            error: result.error,
            execution_time_ms: execution_time,
            output: result.output,
            warnings: result.warnings,
            context: Some(context),
        })
    }

    /// 执行工具（内部方法）
    async fn execute_with_context(
        &self,
        executor: &Arc<dyn ToolExecutor>,
        args: serde_json::Value,
        context: &ExecutionContext,
        execution_id: String,
    ) -> ToolResult {
        // 更新状态为运行中
        self.update_context_status(&execution_id, ExecutionStatus::Running).await;

        match executor.execute(args, context).await {
            Ok(result) => result,
            Err(e) => ToolResult {
                success: false,
                data: serde_json::Value::Null,
                error: Some(format!("{:?}", e)),
                execution_time_ms: 0,
                output: None,
                warnings: vec![],
                context: Some(context.clone()),
            }
        }
    }

    /// 更新执行上下文状态
    async fn update_context_status(&self, execution_id: &str, status: ExecutionStatus) {
        let mut contexts = self.contexts.lock().await;
        if let Some(context) = contexts.get_mut(execution_id) {
            context.status = status;
        }
    }

    /// 获取执行上下文
    pub async fn get_execution_context(&self, execution_id: &str) -> Option<ToolContext> {
        let contexts = self.contexts.lock().await;
        contexts.get(execution_id).cloned()
    }

    /// 取消执行
    pub async fn cancel_execution(&self, execution_id: &str) -> Result<(), ToolError> {
        let mut contexts = self.contexts.lock().await;
        if let Some(context) = contexts.get_mut(execution_id) {
            if context.status == ExecutionStatus::Running {
                context.status = ExecutionStatus::Cancelled;
                Ok(())
            } else {
                Err(ToolError::ExecutionFailed(
                    "Cannot cancel execution that is not running".to_string()
                ))
            }
        } else {
            Err(ToolError::ExecutionFailed(
                "Execution context not found".to_string()
            ))
        }
    }

    /// 获取活跃执行数量
    pub async fn active_execution_count(&self) -> usize {
        *self.active_executions.lock().await
    }

    /// 获取所有注册的工具
    pub fn list_tools(&self) -> Vec<&ToolMetadata> {
        self.executors.values()
            .map(|executor| executor.metadata())
            .collect()
    }

    /// 检查工具是否存在
    pub fn has_tool(&self, tool_id: &str) -> bool {
        self.executors.contains_key(tool_id)
    }
}

/// 工具执行器工厂
pub struct ToolExecutorFactory;

impl ToolExecutorFactory {
    /// 创建工具执行器管理器
    pub fn create_manager(config: Option<ToolConfig>) -> ToolExecutionManager {
        let config = config.unwrap_or_default();
        ToolExecutionManager::new(config)
    }

    /// 创建默认配置的执行器管理器
    pub fn create_default_manager() -> ToolExecutionManager {
        Self::create_manager(None)
    }
}

/// 工具执行器构建器
pub struct ToolExecutorBuilder {
    config: ToolConfig,
    executors: Vec<(String, Arc<dyn ToolExecutor>)>,
}

impl ToolExecutorBuilder {
    /// 创建新的构建器
    pub fn new() -> Self {
        Self {
            config: ToolConfig::default(),
            executors: Vec::new(),
        }
    }

    /// 设置配置
    pub fn with_config(mut self, config: ToolConfig) -> Self {
        self.config = config;
        self
    }

    /// 添加执行器
    pub fn with_executor(mut self, tool_id: String, executor: Arc<dyn ToolExecutor>) -> Self {
        self.executors.push((tool_id, executor));
        self
    }

    /// 构建执行器管理器
    pub fn build(self) -> ToolExecutionManager {
        let mut manager = ToolExecutionManager::new(self.config);
        for (tool_id, executor) in self.executors {
            manager.register_executor(tool_id, executor);
        }
        manager
    }
}

impl Default for ToolExecutorBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    // Mock 工具执行器用于测试
    struct MockTool {
        metadata: ToolMetadata,
    }

    #[async_trait]
    impl ToolExecutor for MockTool {
        fn metadata(&self) -> &ToolMetadata {
            &self.metadata
        }

        async fn execute(
            &self,
            _args: serde_json::Value,
            _context: &ExecutionContext,
        ) -> Result<ToolResult, ToolError> {
            Ok(ToolResult {
                success: true,
                data: serde_json::json!({"result": "mock success"}),
                error: None,
                execution_time_ms: 100,
                output: Some("Mock output".to_string()),
                warnings: vec![],
                context: None,
            })
        }

        async fn validate_args(&self, _args: &serde_json::Value) -> Result<(), ToolError> {
            Ok(())
        }

        fn help(&self) -> String {
            "Mock tool for testing".to_string()
        }
    }

    #[tokio::test]
    async fn test_tool_execution_manager() {
        let mut manager = ToolExecutionManager::new(ToolConfig::default());

        let mock_tool = Arc::new(MockTool {
            metadata: ToolMetadata {
                id: "mock_tool".to_string(),
                name: "Mock Tool".to_string(),
                description: "A mock tool for testing".to_string(),
                category: super::ToolCategory::Other,
                priority: ToolPriority::Low,
                status: ToolStatus::Available,
                version: "1.0.0".to_string(),
                author: "Test".to_string(),
                created_at: chrono::Utc::now().timestamp(),
                updated_at: chrono::Utc::now().timestamp(),
                dependencies: vec![],
                platforms: vec!["test".to_string()],
                permissions: vec![],
            },
        });

        manager.register_executor("mock_tool".to_string(), mock_tool);

        let context = ExecutionContext {
            session_id: "test_session".to_string(),
            user_id: None,
            working_directory: None,
            environment: HashMap::new(),
            timeout_seconds: Some(10),
            permissions: vec![],
            timestamp: chrono::Utc::now().timestamp(),
        };

        let result = manager.execute_tool(
            "mock_tool",
            serde_json::json!({"test": "data"}),
            context,
        ).await;

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.success);
        assert_eq!(result.data["result"], "mock success");
    }
}