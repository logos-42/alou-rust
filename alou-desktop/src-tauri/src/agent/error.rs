//! Agent 模块错误定义

use super::executor::ExecutorError;
use thiserror::Error;

#[derive(Debug, Error, Clone)]
pub enum AgentError {
    /// AI API 错误
    #[error("AI API error: {0}")]
    AiError(String),

    /// Provider 错误
    #[error("Provider error: {0}")]
    ProviderError(String),

    /// 配置错误
    #[error("Config error: {0}")]
    ConfigError(String),

    /// 任务错误
    #[error("Task error: {0}")]
    TaskError(String),

    /// 工具执行错误
    #[error("Tool execution error: {0}")]
    ToolError(String),

    /// 网络/HTTP 错误
    #[error("Network error: {0}")]
    NetworkError(String),

    /// 序列化错误
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// 无效输入
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// 内部错误
    #[error("Internal error: {0}")]
    InternalError(String),

    /// 外部 API 错误（新增）
    #[error("External API error: {0}")]
    ExternalApiError(String),

    /// 通用 Agent 错误（别名，用于兼容旧代码）
    #[error("Agent error: {0}")]
    AgentError(String),
}

impl From<reqwest::Error> for AgentError {
    fn from(err: reqwest::Error) -> Self {
        AgentError::NetworkError(err.to_string())
    }
}

impl From<serde_json::Error> for AgentError {
    fn from(err: serde_json::Error) -> Self {
        AgentError::SerializationError(err.to_string())
    }
}

impl From<ExecutorError> for AgentError {
    fn from(err: ExecutorError) -> Self {
        match err {
            ExecutorError::TaskNotFound(msg) => AgentError::TaskError(msg),
            ExecutorError::AiError(msg) => AgentError::AiError(msg),
            ExecutorError::ToolError(msg) => AgentError::ToolError(msg),
            ExecutorError::InternalError(msg) => AgentError::InternalError(msg),
        }
    }
}

impl From<String> for AgentError {
    fn from(err: String) -> Self {
        AgentError::InternalError(err)
    }
}

pub type Result<T> = std::result::Result<T, AgentError>;
