use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Error type for classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorType {
    Transient,
    Permanent,
    UserError,
    SystemError,
}

/// Tool error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolErrorResponse {
    pub error: String,
    pub error_type: ErrorType,
    pub retryable: bool,
    pub context: Option<Value>,
}

impl ToolErrorResponse {
    #[allow(dead_code)]
    pub fn new(error: impl Into<String>, error_type: ErrorType, retryable: bool) -> Self {
        Self {
            error: error.into(),
            error_type,
            retryable,
            context: None,
        }
    }

    #[allow(dead_code)]
    pub fn with_context(mut self, context: Value) -> Self {
        self.context = Some(context);
        self
    }

    pub fn from_aloud_error(error: &str) -> Self {
        // Simple error classification based on error message content
        let lower = error.to_lowercase();
        let (error_type, retryable) = if lower.contains("timeout") || lower.contains("connection") {
            (ErrorType::Transient, true)
        } else if lower.contains("auth") || lower.contains("unauthorized") || lower.contains("forbidden") {
            (ErrorType::Permanent, false)
        } else if lower.contains("invalid") || lower.contains("parse") || lower.contains("missing") {
            (ErrorType::UserError, false)
        } else if lower.contains("rate limit") || lower.contains("too many requests") {
            (ErrorType::Transient, true)
        } else {
            (ErrorType::SystemError, false)
        };

        Self {
            error: error.to_string(),
            error_type,
            retryable,
            context: None,
        }
    }
}

impl std::fmt::Display for ToolErrorResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} (retryable: {})", self.error, self.retryable)
    }
}

impl std::error::Error for ToolErrorResponse {}
