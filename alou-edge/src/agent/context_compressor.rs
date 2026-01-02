use crate::agent::session::Message;
use crate::utils::error::{AloudError, Result};
use serde::{Deserialize, Serialize};

/// Compression strategy
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompressionStrategy {
    KeepLastN(usize),
    KeepToolCallsAndResults,
    SummarizeAndKeepKey,
    SemanticImportance,
}

/// Compressed history with metadata
#[derive(Debug, Clone, Serialize)]
pub struct CompressedHistory {
    pub messages: Vec<Message>,
    pub compression_info: CompressionInfo,
}

impl CompressedHistory {
    /// Create a new compressed history
    pub fn new(messages: Vec<Message>, info: CompressionInfo) -> Self {
        Self { messages, compression_info: info }
    }
}

/// Compression information and statistics
#[derive(Debug, Clone, Serialize)]
pub struct CompressionInfo {
    pub original_count: usize,
    pub compressed_count: usize,
    pub reduction_percent: f64,
    pub strategy: CompressionStrategy,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    pub timestamp: i64,
}

impl CompressionInfo {
    /// Create new compression info
    pub fn new(
        original_count: usize,
        compressed_count: usize,
        strategy: CompressionStrategy,
        summary: Option<String>,
    ) -> Self {
        let reduction_percent = if original_count > 0 {
            ((original_count - compressed_count) as f64 / original_count as f64) * 100.0
        } else {
            0.0
        };

        Self {
            original_count,
            compressed_count,
            reduction_percent,
            strategy,
            summary,
            timestamp: crate::utils::time::now_timestamp(),
        }
    }
}

/// Context compressor for managing conversation history
pub struct ContextCompressor {
    strategy: CompressionStrategy,
    max_tokens: usize,
    compression_threshold: f64,
    max_messages_per_session: usize,
    max_summary_length: usize,
}

impl ContextCompressor {
    /// Create a new context compressor
    pub fn new(
        strategy: CompressionStrategy,
        max_tokens: usize,
        compression_threshold: f64,
    ) -> Self {
        Self {
            strategy,
            max_tokens,
            compression_threshold,
            max_messages_per_session: 50,
            max_summary_length: 800,
        }
    }
    
    /// Estimate token count for messages (rough estimate)
    pub fn estimate_tokens(&self, messages: &[Message]) -> usize {
        let mut total_tokens = 0;
        
        for message in messages {
            // Rough estimate: ~4 tokens per character (varies by language)
            let char_count = message.content.len();
            total_tokens += (char_count as f64 / 4.0) as usize;
        }
        
        total_tokens
    }
    
    /// Check if compression is needed
    pub fn should_compress(&self, messages: &[Message]) -> bool {
        if messages.is_empty() {
            return false;
        }
        
        let estimated_tokens = self.estimate_tokens(messages);
        let ratio = estimated_tokens as f64 / self.max_tokens as f64;
        
        worker::console_log!(
            "Context compression check: {} / {} tokens ({:.1}%)",
            estimated_tokens,
            self.max_tokens,
            ratio * 100.0
        );
        
        ratio > self.compression_threshold || messages.len() > self.max_messages_per_session
    }
    
    /// Compress conversation history
    pub async fn compress_history(
        &self,
        messages: &[Message],
        session_id: &str,
        ai_client: Option<&crate::agent::ai_client::AiClient>,
    ) -> Result<CompressedHistory> {
        match self.strategy {
            CompressionStrategy::KeepLastN(n) => Ok(self.compress_by_keep_last_n(messages, n)),
            CompressionStrategy::KeepToolCallsAndResults => Ok(self.compress_by_keep_tool_calls(messages)),
            CompressionStrategy::SummarizeAndKeepKey => {
                self.compress_by_summary(messages, session_id, ai_client).await
            },
            CompressionStrategy::SemanticImportance => {
                Ok(self.compress_by_semantic_importance(messages))
            },
        }
    }
    
    /// Strategy 1: Keep last N messages (simple, no AI call needed)
    fn compress_by_keep_last_n(&self, messages: &[Message], n: usize) -> CompressedHistory {
        if messages.len() <= n {
            let info = CompressionInfo::new(
                messages.len(),
                messages.len(),
                self.strategy,
                None,
            );
            return CompressedHistory::new(messages.to_vec(), info);
        }
        
        let compressed = messages[messages.len() - n..].to_vec();
        
        let summary = Some(format!(
            "保留了最近的 {} 条消息（共 {} 条）",
            n,
            messages.len()
        ));
        
        let info = CompressionInfo::new(
            messages.len(),
            n,
            self.strategy,
            summary,
        );
        
        CompressedHistory::new(compressed, info)
    }
    
    /// Strategy 2: Keep tool calls and important messages (medium complexity)
    fn compress_by_keep_tool_calls(&self, messages: &[Message]) -> CompressedHistory {
        let filtered: Vec<Message> = messages
            .iter()
            .filter(|m| {
                // Keep user messages
                m.role == "user" ||
                // Keep assistant messages
                m.role == "assistant" ||
                // Keep tool results (important)
                (m.role == "tool" && !m.content.is_empty())
            })
            .cloned()
            .collect();
        
        let summary = Some(format!(
            "保留了用户消息、助手回复和工具结果（共 {} 条，原 {} 条）",
            filtered.len(),
            messages.len()
        ));
        
        let info = CompressionInfo::new(
            messages.len(),
            filtered.len(),
            self.strategy,
            summary,
        );
        
        CompressedHistory::new(filtered, info)
    }
    
    /// Strategy 3: Generate summary and keep key steps (recommended, needs AI)
    async fn compress_by_summary(
        &self,
        messages: &[Message],
        _session_id: &str,
        ai_client: Option<&crate::agent::ai_client::AiClient>,
    ) -> Result<CompressedHistory> {
        // 1. Extract key messages (tool calls, errors, important decisions)
        let key_messages = self.extract_key_messages(messages);
        
        // 2. Generate summary if AI client available
        let summary = if let Some(client) = ai_client {
            match self.generate_summary_with_ai(client, messages).await {
                Ok(s) => Some(s),
                Err(e) => {
                    worker::console_log!("Failed to generate summary: {}", e);
                    Some("摘要生成失败，保留了关键消息".to_string())
                }
            }
        } else {
            Some("没有 AI 客户端，保留了关键消息".to_string())
        };
        
        // 3. Build compressed history
        let mut compressed = Vec::new();
        
        // Add summary as system message
        if let Some(ref summary_text) = summary {
            compressed.push(Message::new(
                "system".to_string(),
                format!("[对话摘要]\n{}", summary_text)
            ));
        }
        
        // Add key messages
        compressed.extend(key_messages);
        
        let info = CompressionInfo::new(
            messages.len(),
            compressed.len(),
            self.strategy,
            summary,
        );
        
        Ok(CompressedHistory::new(compressed, info))
    }
    
    /// Strategy 4: Semantic importance scoring (advanced, needs AI)
    fn compress_by_semantic_importance(&self, messages: &[Message]) -> CompressedHistory {
        // Fallback to strategy 2 if no AI client available
        self.compress_by_keep_tool_calls(messages)
    }
    
    /// Extract key messages from history
    fn extract_key_messages(&self, messages: &[Message]) -> Vec<Message> {
        let mut key_messages = Vec::new();
        
        // Always keep the last few messages (recent context)
        let recent_count = 5.min(messages.len());
        let start_index = messages.len().saturating_sub(recent_count);
        
        // Add recent messages
        for msg in &messages[start_index..] {
            key_messages.push(msg.clone());
        }
        
        // Look for important messages in earlier history
        for (_i, msg) in messages.iter().enumerate().take(start_index) {
            let is_important = self.is_message_important(msg);
            
            if is_important {
                key_messages.push(msg.clone());
            }
        }
        
        // Sort by timestamp to maintain order
        key_messages.sort_by_key(|m| m.timestamp);
        
        key_messages
    }
    
    /// Determine if a message is important
    fn is_message_important(&self, message: &Message) -> bool {
        // Tool results are important
        if message.role == "tool" && !message.content.is_empty() {
            return true;
        }
        
        // User messages with specific keywords
        if message.role == "user" {
            let content_lower = message.content.to_lowercase();
            let important_keywords = [
                "error", "错误", "fail", "失败", "problem", "问题",
                "important", "重要", "note", "注意", "remember", "记住",
            ];
            
            for keyword in &important_keywords {
                if content_lower.contains(keyword) {
                    return true;
                }
            }
        }
        
        // Assistant messages that are long (likely contain important info)
        if message.role == "assistant" && message.content.len() > 200 {
            return true;
        }
        
        false
    }
    
    /// Generate summary using AI
    async fn generate_summary_with_ai(
        &self,
        ai_client: &crate::agent::ai_client::AiClient,
        messages: &[Message],
    ) -> Result<String> {
        // Build summary prompt
        let conversation_text: String = messages
            .iter()
            .take(20)
            .map(|m| format!("[{}]: {}", m.role, m.content.chars().take(200).collect::<String>()))
            .collect::<Vec<_>>()
            .join("\n\n");
        
        let summary_prompt = format!(
            "请简洁地总结以下对话的关键信息（最多 {} 字符）：\n\n{}\n\n总结应包含：\n\
            1. 讨论的主要主题\n\
            2. 重要决定或结论\n\
            3. 执行的关键操作\n\
            4. 遇到的错误（如有）",
            self.max_summary_length,
            conversation_text
        );
        
        // Call AI to generate summary
        let ai_messages = vec![crate::agent::ai_client::AiMessage::text(
            "user",
            summary_prompt
        )];
        
        let response = ai_client
            .send_message(ai_messages, None)
            .await
            .map_err(|e| AloudError::InternalError(format!("Summary generation failed: {}", e)))?;
        
        Ok(response.content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::session::Message;
    
    #[test]
    fn test_estimate_tokens() {
        let compressor = ContextCompressor::new(
            CompressionStrategy::KeepLastN(20),
            8000,
            0.7,
        );
        
        let messages = vec![
            Message::new("user".to_string(), "Hello".to_string()),
            Message::new("assistant".to_string(), "Hi there!".to_string()),
        ];
        
        let tokens = compressor.estimate_tokens(&messages);
        assert!(tokens > 0);
    }
    
    #[test]
    fn test_should_compress() {
        let compressor = ContextCompressor::new(
            CompressionStrategy::KeepLastN(20),
            8000,
            0.7,
        );
        
        // Small number of messages
        let small = vec![Message::new("user".to_string(), "Hi".to_string())];
        assert!(!compressor.should_compress(&small));
        
        // Large number of messages
        let large: Vec<Message> = (0..60)
            .map(|i| Message::new("user".to_string(), format!("Message {}", i)))
            .collect();
        assert!(compressor.should_compress(&large));
    }
    
    #[test]
    fn test_compress_by_keep_last_n() {
        let compressor = ContextCompressor::new(
            CompressionStrategy::KeepLastN(10),
            8000,
            0.7,
        );
        
        let messages: Vec<Message> = (0..20)
            .map(|i| Message::new("user".to_string(), format!("Message {}", i)))
            .collect();
        
        let compressed = compressor.compress_by_keep_last_n(&messages, 10);
        
        assert_eq!(compressed.messages.len(), 10);
        assert_eq!(compressed.compression_info.original_count, 20);
        assert_eq!(compressed.compression_info.compressed_count, 10);
        assert!(compressed.compression_info.summary.is_some());
    }
    
    #[test]
    fn test_compress_by_keep_tool_calls() {
        let compressor = ContextCompressor::new(
            CompressionStrategy::KeepToolCallsAndResults,
            8000,
            0.7,
        );
        
        let messages = vec![
            Message::new("user".to_string(), "Hello".to_string()),
            Message::new("assistant".to_string(), "Hi".to_string()),
            Message::new(
                "tool".to_string(),
                "Tool result: success".to_string()
            ),
            Message::new("assistant".to_string(), "How can I help?".to_string()),
        ];
        
        let compressed = compressor.compress_by_keep_tool_calls(&messages);
        
        // Should keep user, assistant, and tool messages
        assert!(compressed.messages.len() >= 3);
    }
    
    #[test]
    fn test_extract_key_messages() {
        let compressor = ContextCompressor::new(
            CompressionStrategy::SummarizeAndKeepKey,
            8000,
            0.7,
        );
        
        let messages: Vec<Message> = (0..15)
            .map(|i| {
                let role = if i % 2 == 0 { "user" } else { "assistant" };
                Message::new(role.to_string(), format!("Message {}", i))
            })
            .collect();
        
        let key_messages = compressor.extract_key_messages(&messages);
        
        // Should include last 5 messages + any important ones
        assert!(key_messages.len() >= 5);
        assert!(key_messages.len() <= messages.len());
    }
    
    #[test]
    fn test_is_message_important() {
        let compressor = ContextCompressor::new(
            CompressionStrategy::SummarizeAndKeepKey,
            8000,
            0.7,
        );
        
        // Tool result should be important
        let tool_msg = Message::new(
            "tool".to_string(),
            "Result: success".to_string()
        );
        assert!(compressor.is_message_important(&tool_msg));
        
        // User message with error keyword should be important
        let error_msg = Message::new(
            "user".to_string(),
            "There was an error".to_string()
        );
        assert!(compressor.is_message_important(&error_msg));
        
        // Short assistant message should not be important
        let short_msg = Message::new(
            "assistant".to_string(),
            "OK".to_string()
        );
        assert!(!compressor.is_message_important(&short_msg));
    }
    
    #[test]
    fn test_compression_info() {
        let info = CompressionInfo::new(100, 30, CompressionStrategy::KeepLastN(30), Some("Test".to_string()));
        
        assert_eq!(info.original_count, 100);
        assert_eq!(info.compressed_count, 30);
        assert!((info.reduction_percent - 70.0).abs() < 0.1);
        assert_eq!(info.strategy, CompressionStrategy::KeepLastN(30));
        assert_eq!(info.summary, Some("Test".to_string()));
    }
}
