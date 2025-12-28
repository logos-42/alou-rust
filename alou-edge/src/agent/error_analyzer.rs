use crate::agent::error_response::{CorrectedArgs, ErrorType, ToolErrorResponse};
use crate::agent::session::Message;
use crate::utils::error::AloudError;
use serde_json::Value;
use std::collections::HashMap;

/// Error analysis result
#[derive(Debug, Clone)]
pub struct ErrorAnalysis {
    pub error_type: ErrorType,
    pub root_cause: String,
    pub can_auto_correct: bool,
    pub suggestion: Option<String>,
    pub corrected_args: Option<CorrectedArgs>,
}

/// Session context for error analysis
#[derive(Debug, Clone)]
pub struct SessionContext {
    pub recent_errors: Vec<String>,
    pub error_count: HashMap<String, usize>,
    pub tool_usage_history: Vec<String>,
}

/// Error analyzer for intelligent error handling
pub struct ErrorAnalyzer;

impl ErrorAnalyzer {
    /// Analyze an error and provide correction suggestions
    pub fn analyze_error(
        &self,
        error: &AloudError,
        tool_name: &str,
        args: &Value,
        session_context: &SessionContext,
    ) -> ErrorAnalysis {
        let error_type = self.classify_error_type(error);
        let root_cause = self.extract_root_cause(error);
        
        let can_auto_correct = self.determine_auto_correctability(&error_type, tool_name, args);
        
        let suggestion = self.generate_suggestion(
            &error_type,
            tool_name,
            args,
            session_context,
        );
        
        let corrected_args = if can_auto_correct {
            self.try_auto_correct(&error_type, tool_name, args)
        } else {
            None
        };
        
        ErrorAnalysis {
            error_type,
            root_cause,
            can_auto_correct,
            suggestion,
            corrected_args,
        }
    }
    
    /// Classify error into ErrorType
    fn classify_error_type(&self, error: &AloudError) -> ErrorType {
        use crate::utils::error::AloudError::*;
        
        match error {
            InvalidInput(_) | InvalidToolArgs(_) => ErrorType::ArgumentError,
            AuthError(_) | InvalidSignature | NonceExpired => ErrorType::AuthenticationError,
            RpcError(_) => ErrorType::NetworkError,
            TransactionFailed(_) => ErrorType::TransactionError,
            InsufficientBalance => ErrorType::ResourceError,
            ToolExecutionError(_) | McpError(_) => ErrorType::InternalError,
            McpConnectionError(_) | McpTimeout => ErrorType::NetworkError,
            DatabaseError(_) | CacheError(_) => ErrorType::InternalError,
            AgentError(_) | InternalError(_) => ErrorType::InternalError,
            ClaudeApiError(_) => ErrorType::NetworkError,
            WorkerError(_) => ErrorType::InternalError,
            ToolNotFound(_) => ErrorType::ArgumentError,
        }
    }
    
    /// Extract root cause from error
    fn extract_root_cause(&self, error: &AloudError) -> String {
        error.to_string()
    }
    
    /// Determine if error can be auto-corrected
    fn determine_auto_correctability(
        &self,
        error_type: &ErrorType,
        tool_name: &str,
        args: &Value,
    ) -> bool {
        // Only argument errors can often be auto-corrected
        if *error_type != ErrorType::ArgumentError {
            return false;
        }
        
        // Check specific tools that support auto-correction
        match tool_name {
            "query" | "broadcast_transaction" | "transaction" => {
                // These tools often have auto-correctable argument errors
                self.check_correctable_args(args)
            },
            _ => false,
        }
    }
    
    /// Check if arguments have common correctable issues
    fn check_correctable_args(&self, args: &Value) -> bool {
        if let Some(obj) = args.as_object() {
            // Check for common issues:
            // - Empty required strings
            // - Invalid addresses
            // - Missing required fields
            
            if let Some(address) = obj.get("address").and_then(|v| v.as_str()) {
                // Check if address looks valid (basic check)
                if address.is_empty() || address.len() < 10 {
                    return true;
                }
            }
            
            if let Some(amount) = obj.get("amount").and_then(|v| v.as_str()) {
                // Check if amount is valid
                if amount.is_empty() || amount.parse::<f64>().is_err() {
                    return true;
                }
            }
        }
        
        false
    }
    
    /// Try to auto-correct arguments
    fn try_auto_correct(
        &self,
        error_type: &ErrorType,
        tool_name: &str,
        args: &Value,
    ) -> Option<CorrectedArgs> {
        if *error_type != ErrorType::ArgumentError {
            return None;
        }
        
        match tool_name {
            "query" => self.correct_query_args(args),
            "broadcast_transaction" => self.correct_broadcast_args(args),
            "transaction" => self.correct_transaction_args(args),
            _ => None,
        }
    }
    
    /// Correct query tool arguments
    fn correct_query_args(&self, args: &Value) -> Option<CorrectedArgs> {
        let mut corrected = args.clone();
        let mut explanation = Vec::new();
        
        if let Some(obj) = corrected.as_object_mut() {
            // Fix empty or invalid addresses
            if let Some(address) = obj.get_mut("address") {
                if let Some(addr_str) = address.as_str() {
                    if addr_str.is_empty() || addr_str.len() < 10 {
                        explanation.push("地址格式无效，已移除".to_string());
                        *address = Value::Null;
                    }
                }
            }
            
            // Fix missing chain parameter
            if !obj.contains_key("chain") {
                obj.insert(
                    "chain".to_string(),
                    Value::String("ethereum".to_string())
                );
                explanation.push("添加默认 chain 参数: ethereum".to_string());
            }
        }
        
        if explanation.is_empty() {
            None
        } else {
            Some(CorrectedArgs {
                args: corrected,
                explanation: explanation.join("; "),
            })
        }
    }
    
    /// Correct broadcast tool arguments
    fn correct_broadcast_args(&self, args: &Value) -> Option<CorrectedArgs> {
        let mut corrected = args.clone();
        let mut explanation = Vec::new();
        
        if let Some(obj) = corrected.as_object_mut() {
            // Fix missing chain parameter
            if !obj.contains_key("chain") {
                obj.insert(
                    "chain".to_string(),
                    Value::String("eth".to_string())
                );
                explanation.push("添加默认 chain 参数: eth".to_string());
            }
            
            // Fix empty signed_tx
            if let Some(signed_tx) = obj.get("signed_tx") {
                if let Some(tx_str) = signed_tx.as_str() {
                    if tx_str.is_empty() || !tx_str.starts_with("0x") {
                        explanation.push("signed_tx 格式无效".to_string());
                    }
                }
            }
        }
        
        if explanation.is_empty() {
            None
        } else {
            Some(CorrectedArgs {
                args: corrected,
                explanation: explanation.join("; "),
            })
        }
    }
    
    /// Correct transaction tool arguments
    fn correct_transaction_args(&self, args: &Value) -> Option<CorrectedArgs> {
        let mut corrected = args.clone();
        let mut explanation = Vec::new();
        
        if let Some(obj) = corrected.as_object_mut() {
            // Fix invalid amount formats
            if let Some(amount) = obj.get("amount") {
                match amount {
                    Value::String(ref s) => {
                        if let Ok(value) = s.parse::<f64>() {
                            if value <= 0.0 {
                                explanation.push("金额必须大于 0".to_string());
                            } else if value.is_infinite() || value.is_nan() {
                                explanation.push("金额值无效".to_string());
                            }
                        } else {
                            explanation.push("金额格式错误，应为数字".to_string());
                        }
                    },
                    Value::Number(n) => {
                        if n.as_f64().map_or(true, |v| v <= 0.0) {
                            explanation.push("金额必须大于 0".to_string());
                        }
                    },
                    _ => {
                        explanation.push("金额类型错误".to_string());
                    }
                }
            }
            
            // Add missing required fields
            if !obj.contains_key("from") && !obj.contains_key("to") {
                explanation.push("缺少必需参数: from 和/或 to".to_string());
            }
        }
        
        if explanation.is_empty() {
            None
        } else {
            Some(CorrectedArgs {
                args: corrected,
                explanation: explanation.join("; "),
            })
        }
    }
    
    /// Generate suggestion for error
    fn generate_suggestion(
        &self,
        error_type: &ErrorType,
        tool_name: &str,
        args: &Value,
        session_context: &SessionContext,
    ) -> Option<String> {
        // Check for repeated errors
        let error_key = format!("{}:{}", error_type, tool_name);
        let error_count = session_context.error_count.get(&error_key).unwrap_or(&0);
        
        if *error_count > 2 {
            return Some(format!(
                "此错误已重复 {} 次。建议检查参数格式或联系支持。",
                error_count
            ));
        }
        
        // Tool-specific suggestions
        match tool_name {
            "query" => match error_type {
                ErrorType::NetworkError => Some(
                    "查询失败，可能是网络问题。建议：1) 检查网络连接；2) 稍后重试；3) 检查 RPC 端点是否可用。".to_string()
                ),
                ErrorType::ArgumentError => {
                    if args.get("address").is_none() {
                        Some("缺少地址参数。请提供有效的钱包地址或合约地址。".to_string())
                    } else {
                        Some("参数错误。请检查地址格式和必需参数。".to_string())
                    }
                },
                _ => None,
            },
            "broadcast_transaction" => match error_type {
                ErrorType::NetworkError => Some(
                    "广播失败。建议：1) 检查网络连接；2) 确认交易格式正确；3) 检查链上状态。".to_string()
                ),
                ErrorType::TransactionError => Some(
                    "交易失败。建议：1) 检查交易数据；2) 确认有足够的 gas 费用；3) 检查 nonce 是否正确。".to_string()
                ),
                _ => None,
            },
            "transaction" => match error_type {
                ErrorType::ResourceError => Some(
                    "余额不足。建议：1) 检查账户余额；2) 减少转账金额；3) 确保有足够的 gas 费用。".to_string()
                ),
                ErrorType::ArgumentError => Some(
                    "交易参数错误。请检查：1) 收款地址格式；2) 转账金额；3) Chain 参数。".to_string()
                ),
                _ => None,
            },
            _ => None,
        }
    }
    
    /// Create session context from message history
    pub fn create_session_context(messages: &[Message]) -> SessionContext {
        let mut recent_errors = Vec::new();
        let mut error_count = HashMap::new();
        let mut tool_usage_history = Vec::new();
        
        for msg in messages.iter().rev().take(10) {
            // Extract tool calls from assistant messages
            if msg.role == "assistant" || msg.role == "tool" {
                // Try to extract error information
                if msg.content.contains("error") || msg.content.contains("失败") {
                    recent_errors.push(msg.content.clone());
                }
            }
            
            // Extract tool usage (simplified)
            if msg.tool_call_id.is_some() {
                tool_usage_history.push(msg.tool_call_id.as_ref().unwrap().clone());
            }
        }
        
        // Count errors by type
        for error in &recent_errors {
            let error_key = error.split(':').next().unwrap_or("unknown").to_string();
            *error_count.entry(error_key).or_insert(0) += 1;
        }
        
        SessionContext {
            recent_errors,
            error_count,
            tool_usage_history,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_classify_error_type() {
        use crate::utils::error::AloudError;
        
        let arg_error = AloudError::InvalidInput("test".to_string());
        let analysis = ErrorAnalyzer::classify_error_type(&arg_error);
        assert_eq!(analysis, ErrorType::ArgumentError);
        
        let net_error = AloudError::RpcError("connection failed".to_string());
        let analysis = ErrorAnalyzer::classify_error_type(&net_error);
        assert_eq!(analysis, ErrorType::NetworkError);
    }
    
    #[test]
    fn test_determine_auto_correctability() {
        let args = json!({"address": "invalid"});
        
        let can_correct = ErrorAnalyzer::determine_auto_correctability(
            &ErrorType::ArgumentError,
            "query",
            &args
        );
        assert!(!can_correct);
        
        let args_empty = json!({"address": ""});
        let can_correct = ErrorAnalyzer::determine_auto_correctability(
            &ErrorType::ArgumentError,
            "query",
            &args_empty
        );
        assert!(can_correct);
    }
    
    #[test]
    fn test_correct_query_args() {
        let args = json!({});
        let corrected = ErrorAnalyzer::correct_query_args(&args);
        
        assert!(corrected.is_some());
        let corrected = corrected.unwrap();
        assert!(corrected.explanation.contains("chain"));
        assert!(corrected.args.get("chain").is_some());
    }
    
    #[test]
    fn test_correct_transaction_args() {
        let args = json!({"amount": "-1"});
        let corrected = ErrorAnalyzer::correct_transaction_args(&args);
        
        assert!(corrected.is_some());
        let corrected = corrected.unwrap();
        assert!(corrected.explanation.contains("金额"));
    }
    
    #[test]
    fn test_create_session_context() {
        use crate::agent::session::Message;
        
        let messages = vec![
            Message::new("user".to_string(), "hello".to_string()),
            Message::new("assistant".to_string(), "error: something went wrong".to_string()),
            Message::new("tool".to_string(), "tool result".to_string()),
        ];
        
        let context = ErrorAnalyzer::create_session_context(&messages);
        assert_eq!(context.recent_errors.len(), 1);
        assert!(context.recent_errors[0].contains("error"));
    }
}
