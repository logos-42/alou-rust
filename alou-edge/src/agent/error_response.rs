use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Error type classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ErrorType {
    /// Argument validation or format errors
    ArgumentError,
    /// Network connectivity or timeout errors
    NetworkError,
    /// Authentication or authorization failures
    AuthenticationError,
    /// Insufficient resources (balance, gas, etc.)
    ResourceError,
    /// Permission or access denied errors
    PermissionError,
    /// Rate limiting errors
    RateLimitError,
    /// Transaction or blockchain operation errors
    TransactionError,
    /// Internal system errors
    InternalError,
    /// Unknown or uncategorized errors
    UnknownError,
}

/// Corrected arguments for retry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrectedArgs {
    /// The corrected argument values
    pub args: Value,
    /// Explanation of what was corrected
    pub explanation: String,
}

/// Structured error response for tools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolErrorResponse {
    /// Error code for programmatic handling
    pub error_code: String,
    
    /// Type of error
    #[serde(rename = "error_type")]
    pub error_type: ErrorType,
    
    /// User-friendly error message
    pub error_message: String,
    
    /// Technical details for debugging
    #[serde(rename = "technical_details")]
    pub technical_details: String,
    
    /// Suggestion for fixing the error
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
    
    /// Whether the error is retryable
    #[serde(rename = "can_retry")]
    pub can_retry: bool,
    
    /// Suggested delay before retry (in milliseconds)
    #[serde(rename = "retry_delay_ms")]
    pub retry_delay_ms: Option<u64>,
    
    /// Automatically corrected arguments (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub corrected_args: Option<Value>,
}

impl ToolErrorResponse {
    /// Create a new error response
    pub fn new(error_type: ErrorType, error_message: String) -> Self {
        let error_code = Self::generate_error_code(&error_type);
        
        Self {
            error_code,
            error_type,
            error_message,
            technical_details: String::new(),
            suggestion: None,
            can_retry: Self::default_can_retry(&error_type),
            retry_delay_ms: Self::default_retry_delay(&error_type),
            corrected_args: None,
        }
    }
    
    /// Add technical details
    pub fn with_technical_details(mut self, details: String) -> Self {
        self.technical_details = details;
        self
    }
    
    /// Add a suggestion
    pub fn with_suggestion(mut self, suggestion: String) -> Self {
        self.suggestion = Some(suggestion);
        self
    }
    
    /// Add corrected arguments
    pub fn with_corrected_args(mut self, args: Value) -> Self {
        self.corrected_args = Some(args);
        self
    }
    
    /// Set whether the error is retryable
    pub fn with_can_retry(mut self, can_retry: bool) -> Self {
        self.can_retry = can_retry;
        self
    }
    
    /// Set retry delay
    pub fn with_retry_delay(mut self, delay_ms: u64) -> Self {
        self.retry_delay_ms = Some(delay_ms);
        self
    }
    
    /// Generate a standardized error code
    fn generate_error_code(error_type: &ErrorType) -> String {
        match error_type {
            ErrorType::ArgumentError => "ERR_ARG_001".to_string(),
            ErrorType::NetworkError => "ERR_NET_001".to_string(),
            ErrorType::AuthenticationError => "ERR_AUTH_001".to_string(),
            ErrorType::ResourceError => "ERR_RES_001".to_string(),
            ErrorType::PermissionError => "ERR_PERM_001".to_string(),
            ErrorType::RateLimitError => "ERR_RATE_001".to_string(),
            ErrorType::TransactionError => "ERR_TX_001".to_string(),
            ErrorType::InternalError => "ERR_INT_001".to_string(),
            ErrorType::UnknownError => "ERR_UNK_001".to_string(),
        }
    }
    
    /// Determine if error is retryable by default
    fn default_can_retry(error_type: &ErrorType) -> bool {
        matches!(
            error_type,
            ErrorType::NetworkError | ErrorType::RateLimitError
        )
    }
    
    /// Get default retry delay for error type
    fn default_retry_delay(error_type: &ErrorType) -> Option<u64> {
        match error_type {
            ErrorType::RateLimitError => Some(60000), // 1 minute
            ErrorType::NetworkError => Some(2000),   // 2 seconds
            _ => None,
        }
    }
    
    /// Create from AloudError
    pub fn from_aloud_error(error: &crate::utils::error::AloudError) -> Self {
        let (error_type, message) = Self::classify_aloud_error(error);
        
        let mut response = Self::new(error_type, message);
        response.technical_details = format!("{:?}", error);
        
        // Add specific suggestions based on error type
        match &error_type {
            ErrorType::ArgumentError => {
                response = response.with_suggestion(
                    "请检查参数格式和值。确保所有必需参数都已提供，且值类型正确。"
                    .to_string()
                );
            },
            ErrorType::NetworkError => {
                response = response.with_suggestion(
                    "网络连接失败。请检查网络连接，稍后重试。"
                    .to_string()
                );
                response = response.with_can_retry(true);
            },
            ErrorType::AuthenticationError => {
                response = response.with_suggestion(
                    "认证失败。请检查 API 密钥或令牌是否有效。"
                    .to_string()
                );
            },
            ErrorType::ResourceError => {
                response = response.with_suggestion(
                    "资源不足。请检查账户余额或资源限制。"
                    .to_string()
                );
            },
            ErrorType::PermissionError => {
                response = response.with_suggestion(
                    "权限不足。请检查操作权限或授权范围。"
                    .to_string()
                );
            },
            ErrorType::RateLimitError => {
                response = response.with_suggestion(
                    "请求过于频繁。请稍后重试，或考虑升级订阅。"
                    .to_string()
                );
                response = response.with_retry_delay(60000);
            },
            ErrorType::TransactionError => {
                response = response.with_suggestion(
                    "交易失败。请检查参数和网络状态，确认后重试。"
                    .to_string()
                );
            },
            _ => {}
        }
        
        response
    }
    
    /// Classify AloudError into ErrorType
    fn classify_aloud_error(
        error: &crate::utils::error::AloudError,
    ) -> (ErrorType, String) {
        use crate::utils::error::AloudError;
        
        match error {
            AloudError::InvalidInput(msg) => {
                (ErrorType::ArgumentError, format!("参数错误: {}", msg))
            },
            AloudError::InvalidToolArgs(msg) => {
                (ErrorType::ArgumentError, format!("工具参数错误: {}", msg))
            },
            AloudError::ToolNotFound(tool) => {
                (ErrorType::ArgumentError, format!("工具不存在: {}", tool))
            },
            AloudError::ToolExecutionError(msg) => {
                (ErrorType::InternalError, format!("工具执行错误: {}", msg))
            },
            AloudError::AuthError(msg) => {
                (ErrorType::AuthenticationError, format!("认证错误: {}", msg))
            },
            AloudError::InvalidSignature => {
                (ErrorType::AuthenticationError, "签名无效".to_string())
            },
            AloudError::NonceExpired => {
                (ErrorType::AuthenticationError, "Nonce 已过期".to_string())
            },
            AloudError::RpcError(msg) => {
                (ErrorType::NetworkError, format!("RPC 错误: {}", msg))
            },
            AloudError::TransactionFailed(msg) => {
                (ErrorType::TransactionError, format!("交易失败: {}", msg))
            },
            AloudError::InsufficientBalance => {
                (ErrorType::ResourceError, "余额不足".to_string())
            },
            AloudError::AgentError(msg) => {
                (ErrorType::InternalError, format!("智能体错误: {}", msg))
            },
            AloudError::ClaudeApiError(msg) => {
                (ErrorType::NetworkError, format!("Claude API 错误: {}", msg))
            },
            AloudError::McpError(msg) => {
                (ErrorType::InternalError, format!("MCP 错误: {}", msg))
            },
            AloudError::McpConnectionError(msg) => {
                (ErrorType::NetworkError, format!("MCP 连接错误: {}", msg))
            },
            AloudError::McpTimeout => {
                (ErrorType::NetworkError, "MCP 超时".to_string())
            },
            AloudError::DatabaseError(msg) => {
                (ErrorType::InternalError, format!("数据库错误: {}", msg))
            },
            AloudError::CacheError(msg) => {
                (ErrorType::InternalError, format!("缓存错误: {}", msg))
            },
            AloudError::WorkerError(msg) => {
                (ErrorType::InternalError, format!("Worker 错误: {}", msg))
            },
            AloudError::InternalError(msg) => {
                (ErrorType::InternalError, format!("内部错误: {}", msg))
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_code_generation() {
        assert_eq!(
            ToolErrorResponse::generate_error_code(&ErrorType::ArgumentError),
            "ERR_ARG_001"
        );
        assert_eq!(
            ToolErrorResponse::generate_error_code(&ErrorType::NetworkError),
            "ERR_NET_001"
        );
    }
    
    #[test]
    fn test_default_can_retry() {
        assert!(ToolErrorResponse::default_can_retry(&ErrorType::NetworkError));
        assert!(ToolErrorResponse::default_can_retry(&ErrorType::RateLimitError));
        assert!(!ToolErrorResponse::default_can_retry(&ErrorType::ArgumentError));
        assert!(!ToolErrorResponse::default_can_retry(&ErrorType::AuthenticationError));
    }
    
    #[test]
    fn test_default_retry_delay() {
        assert_eq!(
            ToolErrorResponse::default_retry_delay(&ErrorType::RateLimitError),
            Some(60000)
        );
        assert_eq!(
            ToolErrorResponse::default_retry_delay(&ErrorType::NetworkError),
            Some(2000)
        );
        assert_eq!(
            ToolErrorResponse::default_retry_delay(&ErrorType::ArgumentError),
            None
        );
    }
    
    #[test]
    fn test_builder_pattern() {
        let response = ToolErrorResponse::new(
            ErrorType::ArgumentError,
            "Test error".to_string()
        )
        .with_suggestion("Try again".to_string())
        .with_can_retry(true)
        .with_retry_delay(1000)
        .with_technical_details("Technical details".to_string());
        
        assert_eq!(response.error_message, "Test error");
        assert_eq!(response.suggestion, Some("Try again".to_string()));
        assert_eq!(response.can_retry, true);
        assert_eq!(response.retry_delay_ms, Some(1000));
        assert_eq!(response.technical_details, "Technical details");
    }
}
