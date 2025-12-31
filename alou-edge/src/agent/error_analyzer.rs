use serde_json::Value;

/// Session context for error analysis
#[derive(Debug, Clone)]
pub struct SessionContext {
    pub recent_history: Vec<String>,
    pub tool_usage_patterns: Vec<String>,
}

impl Default for SessionContext {
    fn default() -> Self {
        Self {
            recent_history: Vec::new(),
            tool_usage_patterns: Vec::new(),
        }
    }
}

/// Placeholder for AgentMessage enum
#[derive(Debug, Clone)]
pub enum AgentMessage {
    User { content: String },
    Assistant { content: String },
    ToolCall { tool_name: String },
    ToolResult { tool_name: String },
}

/// Error analyzer for agent errors
pub struct ErrorAnalyzer {
    _private: (),
}

impl ErrorAnalyzer {
    pub fn new() -> Self {
        Self { _private: () }
    }

    pub fn create_session_context(history: &[crate::agent::session::Message]) -> SessionContext {
        let recent_history = history
            .iter()
            .rev()
            .take(10)
            .map(|msg| format!("{}: {}", msg.role, msg.content))
            .collect();

        // Simple pattern extraction - could be enhanced
        let tool_usage_patterns = history
            .iter()
            .filter_map(|msg| {
                if msg.role == "tool" {
                    Some(msg.content.clone())
                } else {
                    None
                }
            })
            .collect();

        SessionContext {
            recent_history,
            tool_usage_patterns,
        }
    }

    pub fn analyze_error(&self, error: &str, tool_name: &str, tool_args: &Value, session_context: &SessionContext) -> ErrorAnalysis {
        let error_type = self.classify_error(error);
        let can_retry = self.should_retry(error);
        let suggested_action = self.get_suggested_action(error);

        // Check if auto-correction is possible based on error type and context
        let (can_auto_correct, corrected_args) = self.attempt_auto_correction(error, tool_name, tool_args, &error_type, session_context);

        ErrorAnalysis {
            error_type,
            can_retry,
            suggested_action,
            can_auto_correct,
            corrected_args,
        }
    }

    fn classify_error(&self, error: &str) -> ErrorType {
        let lower = error.to_lowercase();
        
        if lower.contains("timeout") || lower.contains("timed out") {
            ErrorType::Timeout
        } else if lower.contains("rate limit") || lower.contains("too many requests") {
            ErrorType::RateLimit
        } else if lower.contains("network") || lower.contains("connection") {
            ErrorType::Network
        } else if lower.contains("auth") || lower.contains("unauthorized") {
            ErrorType::Auth
        } else if lower.contains("invalid") || lower.contains("parse") {
            ErrorType::InvalidInput
        } else {
            ErrorType::Unknown
        }
    }

    fn should_retry(&self, error: &str) -> bool {
        let lower = error.to_lowercase();
        lower.contains("timeout") 
            || lower.contains("network")
            || lower.contains("rate limit")
    }

    fn get_suggested_action(&self, error: &str) -> String {
        let lower = error.to_lowercase();

        if lower.contains("timeout") {
            "Increase timeout and retry".to_string()
        } else if lower.contains("rate limit") {
            "Implement exponential backoff and retry".to_string()
        } else if lower.contains("network") {
            "Check network connectivity and retry".to_string()
        } else {
            "Review error and adjust input".to_string()
        }
    }

    fn attempt_auto_correction(&self, error: &str, tool_name: &str, tool_args: &Value, error_type: &ErrorType, _session_context: &SessionContext) -> (bool, Option<CorrectedArgs>) {
        match error_type {
            ErrorType::InvalidInput => {
                // Simple auto-correction for common issues
                if let Some(obj) = tool_args.as_object() {
                    let mut corrected = obj.clone();

                    // Example: if missing required field
                    if tool_name == "search" && !corrected.contains_key("query") {
                        corrected.insert("query".to_string(), Value::String("default query".to_string()));
                        return (true, Some(CorrectedArgs {
                            corrected_args: Value::Object(corrected),
                            explanation: "Added missing query parameter".to_string(),
                        }));
                    }
                }
                (false, None)
            },
            _ => (false, None),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ErrorType {
    Timeout,
    RateLimit,
    Network,
    Auth,
    InvalidInput,
    Unknown,
}

/// Corrected arguments suggestion
#[derive(Debug, Clone)]
pub struct CorrectedArgs {
    pub corrected_args: Value,
    pub explanation: String,
}

#[derive(Debug, Clone)]
pub struct ErrorAnalysis {
    pub error_type: ErrorType,
    pub can_retry: bool,
    pub suggested_action: String,
    pub can_auto_correct: bool,
    pub corrected_args: Option<CorrectedArgs>,
}

impl Default for ErrorAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
