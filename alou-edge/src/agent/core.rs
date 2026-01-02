use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use worker::console_log;

use crate::agent::ai_client::{AiClient, AiMessage, AiTool};
use crate::agent::claude_client::{ClaudeClient, ClaudeMessage, ClaudeTool, ToolUse};
use crate::agent::context::AgentContext;
use crate::agent::context_compressor::{ContextCompressor, CompressionStrategy};
use crate::agent::error_analyzer::ErrorAnalyzer;
use crate::agent::error_response::ToolErrorResponse;
use crate::agent::prompts::{AgentMode, CustomAgentInfo, PromptMode};
use crate::agent::retry_policy::{ErrorType, RetryPolicy};
use crate::agent::session::{ContextEvent, Message, SessionManager};
use crate::agent::stream::{StreamEvent, StreamPublisher};
use crate::mcp::executor::{McpExecutor, ToolCall};
use crate::storage::kv::KvStore;
use crate::utils::error::{AloudError, Result};

const MAX_TOOL_ITERATIONS: u32 = 10;

/// AI Provider type
#[derive(Debug, Clone)]
pub enum AiProviderType {
    Claude,
    DeepSeek,
    Qwen,
    OpenAI,
}

/// Agent response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResponse {
    pub content: String,
    pub session_id: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tool_calls: Vec<ToolCallInfo>,
}

/// Information about a tool call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallInfo {
    pub id: String,
    pub name: String,
    pub result: Value,
}

/// Agent core for handling conversations
pub struct AgentCore {
    claude_client: Option<ClaudeClient>,
    ai_client: Option<AiClient>,
    #[allow(dead_code)]
    provider_type: AiProviderType,
    session_manager: SessionManager,
    mcp_executor: McpExecutor,
    kv: KvStore,
    retry_policy: RetryPolicy,
}

impl AgentCore {
    /// Create AgentCore with Claude (legacy)
    #[allow(dead_code)]
    pub fn new(
        api_key: String,
        session_manager: SessionManager,
        mcp_executor: McpExecutor,
        kv: KvStore,
    ) -> Self {
        Self {
            claude_client: Some(ClaudeClient::new(api_key)),
            ai_client: None,
            provider_type: AiProviderType::Claude,
            session_manager,
            mcp_executor,
            kv,
            retry_policy: RetryPolicy::default(),
        }
    }

    /// Create AgentCore with configurable AI provider
    pub fn with_provider(
        provider: &str,
        api_key: String,
        model: Option<String>,
        session_manager: SessionManager,
        mcp_executor: McpExecutor,
        kv: KvStore,
    ) -> Result<Self> {
        let provider_type = match provider.to_lowercase().as_str() {
            "claude" => AiProviderType::Claude,
            "deepseek" => AiProviderType::DeepSeek,
            "qwen" => AiProviderType::Qwen,
            "openai" => AiProviderType::OpenAI,
            _ => {
                return Err(AloudError::InvalidInput(format!(
                    "Unknown provider: {}",
                    provider
                )))
            }
        };

        let ai_client = AiClient::new(provider, api_key, model)?;

        Ok(Self {
            claude_client: None,
            ai_client: Some(ai_client),
            provider_type,
            session_manager,
            mcp_executor,
            kv,
            retry_policy: RetryPolicy::default(),
        })
    }

    /// Handle a user message with tool calling loop
    pub async fn handle_message(
        &self,
        session_id: &str,
        message: &str,
        wallet_address: Option<String>,
        chain: Option<String>,
        recent_events: Vec<ContextEvent>,
        event_summary: Option<String>,
        stream: Option<StreamPublisher>,
    ) -> Result<AgentResponse> {
        // Detect prompt mode from user message
        let prompt_mode = PromptMode::detect_from_message(message);

        // Add user message to session
        self.session_manager
            .add_message(session_id, "user", message)
            .await?;
        Self::emit_stream(
            &stream,
            StreamEvent::new(session_id, "message.recorded")
                .with_label("收到用户消息")
                .with_payload(json!({
                    "role": "user",
                    "content": message,
                })),
            &self.kv,
        )
        .await;

        // Load conversation history
        let history = self.session_manager.get_history(session_id).await?;
        
        // Check if context compression is needed
        let compressor = ContextCompressor::new(
            CompressionStrategy::SummarizeAndKeepKey,
            8000,
            0.7,
        );
        
        let final_history = if compressor.should_compress(&history) {
            worker::console_log!("Compressing context for session {}", session_id);
            
            match compressor.compress_history(&history, session_id, self.ai_client.as_ref()).await {
                Ok(compressed) => {
                    worker::console_log!(
                        "Context compressed: {} -> {} messages ({:.1}% reduction)",
                        compressed.compression_info.original_count,
                        compressed.compression_info.compressed_count,
                        compressed.compression_info.reduction_percent
                    );
                    
                    Self::emit_stream(
                        &stream,
                        StreamEvent::new(session_id, "context.compressed")
                            .with_label("上下文已压缩")
                            .with_payload(json!({
                                "original_count": compressed.compression_info.original_count,
                                "compressed_count": compressed.compression_info.compressed_count,
                                "reduction_percent": compressed.compression_info.reduction_percent,
                                "strategy": format!("{:?}", compressed.compression_info.strategy),
                                "summary": compressed.compression_info.summary,
                            })),
                        &self.kv,
                    ).await;
                    
                    compressed.messages
                },
                Err(e) => {
                    worker::console_log!("Compression failed: {}, using original history", e);
                    history
                }
            }
        } else {
            history
        };
        
        // Get session to extract chain info
        let session = self.session_manager.get_session(session_id).await?;
        
        // Get wallet address from parameter or session
        let wallet = wallet_address.or(session.wallet_address.clone());
        
        let chain = chain.or(session.chain.clone());
        
        // Create agent context
        let mut context = AgentContext::new(session_id.to_string());
        context.wallet_address = wallet.clone();
        context.chain = chain.clone();
        context.recent_events = recent_events.clone();
        context.event_summary = event_summary.clone();

        // 获取 agent_metadata 以便使用智能体特定的 prompt
        let agent_metadata = self
            .session_manager
            .get_agent_metadata(session_id)
            .await
            .ok()
            .flatten();

        // 确定系统 prompt：优先使用 CustomAgentInfo 结构体生成更完整的 Prompt
        let mut system_prompt = if let Some(ref metadata) = agent_metadata {
            // 从 metadata 中读取模式（默认为 Agent 模式）
            let mode = metadata
                .get("mode")
                .and_then(|v| v.as_str())
                .map(|m| {
                    if m.eq_ignore_ascii_case("alou") {
                        AgentMode::Alou
                    } else {
                        AgentMode::Agent
                    }
                })
                .unwrap_or(AgentMode::Agent);

            // 构建 CustomAgentInfo 结构体
            let custom_agent_info = CustomAgentInfo {
                name: metadata
                    .get("display_name")
                    .and_then(|v| v.as_str())
                    .or_else(|| metadata.get("name").and_then(|v| v.as_str()))
                    .unwrap_or("智能体")
                    .to_string(),
                role_description: metadata
                    .get("role_description")
                    .and_then(|v| v.as_str())
                    .unwrap_or(if mode == AgentMode::Alou {
                        "Alou 平台的 Web3 支付助手"
                    } else {
                        "一个去中心化的 Web3 智能体"
                    })
                    .to_string(),
                custom_instructions: metadata
                    .get("customPrompt")
                    .and_then(|v| v.as_str())
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string()),
                did: metadata.get("did").and_then(|v| v.as_str()).map(|s| s.to_string()),
                ipns: metadata.get("ipns").and_then(|v| v.as_str()).map(|s| s.to_string()),
                mode,
            };

            // 如果存在自定义指令或角色描述，使用 CustomAgentInfo 生成 Prompt
            if custom_agent_info.custom_instructions.is_some()
                || !custom_agent_info.role_description.is_empty()
                || custom_agent_info.did.is_some()
                || custom_agent_info.ipns.is_some()
                || custom_agent_info.mode == AgentMode::Alou
            {
                PromptMode::system_prompt_for_custom_agent_with_context(
                    &custom_agent_info,
                    wallet.as_deref(),
                    chain.as_deref(),
                )
            } else {
                // 否则使用默认 prompt
                prompt_mode.system_prompt_with_context(wallet.as_deref(), chain.as_deref())
            }
        } else {
            // 没有 agent_metadata，使用默认 prompt
            prompt_mode.system_prompt_with_context(wallet.as_deref(), chain.as_deref())
        };

        // 添加 UI 交互快照
        if let Some(summary) = event_summary.as_ref() {
            if !summary.is_empty() {
                system_prompt.push_str("\n\n[最近 UI 交互快照]\n");
                system_prompt.push_str(summary);
            }
        }

        // Convert history to Claude messages
        let mut messages = self.history_to_claude_messages(&final_history);

        // Prepend system message if this is the first message in the conversation
        if messages.is_empty() || !messages.iter().any(|m| m.role == "system") {
            messages.insert(0, ClaudeMessage::text("system", system_prompt));
        }

        // Get available tools
        let tools = self.get_claude_tools();

        // Tool calling loop
        let mut iterations = 0;
        let mut tool_call_info = Vec::new();

        let final_content = loop {
            iterations += 1;
            if iterations > MAX_TOOL_ITERATIONS {
                return Err(AloudError::AgentError(
                    "Maximum tool iterations exceeded".to_string(),
                ));
            }

            // Call AI API (Claude or other provider)
            Self::emit_stream(
                &stream,
                StreamEvent::new(session_id, "llm.request")
                    .with_label("请求模型响应")
                    .with_payload(json!({
                        "provider": format!("{:?}", self.provider_type),
                        "iteration": iterations,
                        "tool_count": tools.len(),
                    })),
                &self.kv,
            )
            .await;

            let response = if let Some(ai_client) = &self.ai_client {
                // Use new unified AI client
                let mut pending_tool_ids: Vec<String> = Vec::new();
                let mut ai_messages: Vec<AiMessage> = Vec::new();

                for m in messages.iter() {
                    let mut text_parts = Vec::new();
                    let mut tool_uses = Vec::new();
                    let mut tool_result_id = None;

                    for block in &m.content {
                        match block {
                            crate::agent::claude_client::ContentBlock::Text { text } => {
                                text_parts.push(text.clone());
                            }
                            crate::agent::claude_client::ContentBlock::ToolUse {
                                id,
                                name,
                                input,
                            } => {
                                tool_uses.push(crate::agent::ai_client::AiToolCall {
                                    id: id.clone(),
                                    name: name.clone(),
                                    arguments: input.clone(),
                                });
                            }
                            crate::agent::claude_client::ContentBlock::ToolResult {
                                tool_use_id,
                                content,
                            } => {
                                tool_result_id = Some(tool_use_id.clone());
                                text_parts.push(content.clone());
                            }
                        }
                    }

                    let content = text_parts.join("\n");

                    if let Some(tool_id) = tool_result_id {
                        if pending_tool_ids.contains(&tool_id) {
                            pending_tool_ids.retain(|id| id != &tool_id);
                            ai_messages.push(AiMessage::tool_result(tool_id, content));
                        } else {
                            // Skip orphaned tool result that lacks preceding tool call
                            continue;
                        }
                    } else if !tool_uses.is_empty() {
                        pending_tool_ids = tool_uses.iter().map(|tc| tc.id.clone()).collect();
                        ai_messages.push(AiMessage::assistant_with_tools(content, tool_uses));
                    } else {
                        pending_tool_ids.clear();
                        ai_messages.push(AiMessage::text(&m.role, content));
                    }
                }

                let ai_tools: Vec<AiTool> = tools
                    .iter()
                    .map(|t| AiTool {
                        name: t.name.clone(),
                        description: t.description.clone(),
                        parameters: t.input_schema.clone(),
                    })
                    .collect();

                let ai_response = ai_client.send_message(ai_messages, Some(ai_tools)).await?;

                // Convert back to Claude format for compatibility
                crate::agent::claude_client::ClaudeResponse {
                    content: ai_response.content,
                    tool_calls: ai_response
                        .tool_calls
                        .into_iter()
                        .map(|tc| ToolUse {
                            id: tc.id,
                            name: tc.name,
                            input: tc.arguments,
                        })
                        .collect(),
                    stop_reason: ai_response.finish_reason,
                }
            } else if let Some(claude_client) = &self.claude_client {
                // Use legacy Claude client
                claude_client
                    .send_message(messages.clone(), Some(tools.clone()))
                    .await?
            } else {
                return Err(AloudError::AgentError(
                    "No AI client configured".to_string(),
                ));
            };

            // If no tool calls, we're done
            if response.tool_calls.is_empty() {
                console_log!(
                    "AgentCore: Final response, content length: {}",
                    response.content.len()
                );
                let preview = response.content.chars().take(100).collect::<String>();
                console_log!("AgentCore: Content preview: {}", preview);
                Self::emit_stream(
                    &stream,
                    StreamEvent::new(session_id, "llm.response")
                        .with_label("模型生成最终回复")
                        .with_payload(json!({
                            "preview": preview,
                            "length": response.content.len(),
                        })),
                    &self.kv,
                )
                .await;
                break response.content.clone();
            }

            // Execute tool calls
            Self::emit_stream(
                &stream,
                StreamEvent::new(session_id, "tool.calls")
                    .with_label("执行工具调用")
                    .with_payload(json!({
                        "count": response.tool_calls.len(),
                        "tools": response
                            .tool_calls
                            .iter()
                            .map(|t| json!({
                                "id": t.id,
                                "name": t.name,
                            }))
                            .collect::<Vec<_>>(),
                    })),
                &self.kv,
            )
            .await;

            let tool_results = self
                .execute_tool_calls(&response.tool_calls, &context)
                .await;

            // Add assistant message with tool calls to history
            // For providers that need it (like DeepSeek), we need to include the tool calls in the assistant message
            if self.ai_client.is_some() {
                // For unified AI client (DeepSeek, etc.), add assistant message with tool calls
                let mut assistant_msg = ClaudeMessage::text("assistant", response.content.clone());
                // Add tool use blocks for Claude compatibility
                for tool_use in &response.tool_calls {
                    assistant_msg.content.push(
                        crate::agent::claude_client::ContentBlock::ToolUse {
                            id: tool_use.id.clone(),
                            name: tool_use.name.clone(),
                            input: tool_use.input.clone(),
                        },
                    );
                }
                messages.push(assistant_msg);
            } else if !response.content.is_empty() {
                // For Claude client, just add text
                messages.push(ClaudeMessage::text("assistant", response.content.clone()));
            }

            // Add tool results to messages and session
            for (tool_use, result) in response.tool_calls.iter().zip(tool_results.iter()) {
                let result_str =
                    serde_json::to_string(&result.result).unwrap_or_else(|_| "{}".to_string());

                // Add to Claude messages (using tool result format)
                messages.push(ClaudeMessage::with_tool_result(
                    "tool",
                    tool_use.id.clone(),
                    result_str.clone(),
                ));

                // Add to session
                self.session_manager
                    .add_message_with_tool_call(session_id, "tool", &result_str, &tool_use.id)
                    .await?;

                // Track tool call info
                tool_call_info.push(ToolCallInfo {
                    id: tool_use.id.clone(),
                    name: tool_use.name.clone(),
                    result: result.result.clone(),
                });
                Self::emit_stream(
                    &stream,
                    StreamEvent::new(session_id, "tool.result").with_payload(json!({
                        "tool_call_id": tool_use.id,
                        "name": tool_use.name,
                    })),
                    &self.kv,
                )
                .await;
            }

            // Continue loop to get next response from Claude
        };

        self.session_manager
            .add_message(session_id, "assistant", &final_content)
            .await?;
        Self::emit_stream(
            &stream,
            StreamEvent::new(session_id, "assistant.persisted")
                .with_label("助手回复已记录")
                .with_payload(json!({
                    "length": final_content.len(),
                })),
            &self.kv,
        )
        .await;

        Ok(AgentResponse {
            content: final_content,
            session_id: session_id.to_string(),
            tool_calls: tool_call_info,
        })
    }

    async fn emit_stream(stream: &Option<StreamPublisher>, event: StreamEvent, kv: &KvStore) {
        if let Some(publisher) = stream {
            publisher.publish(event, kv).await;
        }
    }

    /// Execute tool calls using MCP executor
    async fn execute_tool_calls(
        &self,
        tool_uses: &[ToolUse],
        context: &AgentContext,
    ) -> Vec<crate::mcp::executor::ToolResult> {
        let tool_calls: Vec<ToolCall> = tool_uses
            .iter()
            .map(|tu| {
                let mut args = match tu.input.clone() {
                    Value::String(s) => serde_json::from_str(&s).unwrap_or(Value::Null),
                    other => other,
                };

                if let Some(chain) = context.chain.as_ref() {
                    if let Value::Object(ref mut map) = args {
                        let overwrite = match map.get("chain") {
                            None => true,
                            Some(Value::Null) => true,
                            Some(Value::String(existing)) => existing.trim().is_empty(),
                            _ => false,
                        };

                        if overwrite {
                            map.insert("chain".to_string(), Value::String(chain.clone()));
                        }
                        map.entry("chain_hint".to_string())
                            .or_insert_with(|| Value::String(chain.clone()));
                    }
                }

                ToolCall {
                    id: tu.id.clone(),
                    name: tu.name.clone(),
                    args,
                }
            })
            .collect();

        let results = self.mcp_executor.execute_batch(tool_calls, context).await;

        let error_analyzer = ErrorAnalyzer::new();
        let mut enhanced_results = Vec::new();

        for (tool_use, result) in tool_uses.iter().zip(results.iter()) {
            if let Some(ref error) = result.error {
                console_log!("Tool {} failed: {}", tool_use.name, error);

                let history = match self.session_manager.get_history(&context.session_id).await {
                    Ok(h) => h,
                    Err(_) => vec![],
                };

                let session_context = ErrorAnalyzer::create_session_context(&history);

                let analysis = error_analyzer.analyze_error(
                    error,
                    &tool_use.name,
                    &tool_use.input,
                    &session_context,
                );

                let error_response = ToolErrorResponse::from_aloud_error(error);

                if analysis.can_auto_correct {
                    if let Some(corrected) = &analysis.corrected_args {
                        console_log!("Auto-correcting args for tool {}: {:?}", tool_use.name, corrected.explanation);

                        let _corrected_tool_call = ToolCall {
                            id: tool_use.id.clone(),
                            name: tool_use.name.clone(),
                            args: corrected.corrected_args.clone(),
                        };

                        let corrected_tool_call = ToolCall {
                            id: tool_use.id.clone(),
                            name: tool_use.name.clone(),
                            args: corrected.corrected_args.clone(),
                        };
                        
                        let retry_result = self.execute_tool_with_retry(&corrected_tool_call, context, 0).await;

                        match retry_result {
                            Ok(tool_result) => {
                                console_log!("Auto-correction succeeded for tool {}", tool_use.name);
                                enhanced_results.push(tool_result);
                                continue;
                            },
                            Err(e2) => {
                                console_log!("Auto-correction failed: {}", e2);
                                enhanced_results.push(crate::mcp::executor::ToolResult {
                                    id: tool_use.id.clone(),
                                    name: tool_use.name.clone(),
                                    result: Value::Null,
                                    error: Some(serde_json::to_string(&error_response)
                                        .unwrap_or_else(|_| error.clone())),
                                });
                                continue;
                            }
                        }
                    }
                }

                enhanced_results.push(crate::mcp::executor::ToolResult {
                    id: tool_use.id.clone(),
                    name: tool_use.name.clone(),
                    result: result.result.clone(),
                    error: Some(serde_json::to_string(&error_response)
                        .unwrap_or_else(|_| error.clone())),
                });
            } else {
                enhanced_results.push(result.clone());
            }
        }

        enhanced_results
    }

    /// Convert session history to Claude messages
    fn history_to_claude_messages(&self, history: &[Message]) -> Vec<ClaudeMessage> {
        history
            .iter()
            .map(|m| {
                if let Some(tool_id) = &m.tool_call_id {
                    ClaudeMessage::with_tool_result(&m.role, tool_id.clone(), m.content.clone())
                } else {
                    ClaudeMessage::text(&m.role, m.content.clone())
                }
            })
            .collect()
    }

    /// Get available tools in Claude format
    fn get_claude_tools(&self) -> Vec<ClaudeTool> {
        self.mcp_executor
            .list_tools()
            .into_iter()
            .map(|tool_info| ClaudeTool {
                name: tool_info.name,
                description: tool_info.description,
                input_schema: tool_info.input_schema,
            })
            .collect()
    }

    /// Execute a tool call with retry logic
    async fn execute_tool_with_retry(
        &self,
        tool_call: &ToolCall,
        context: &AgentContext,
        attempt: u32,
    ) -> std::result::Result<crate::mcp::executor::ToolResult, String> {
        let result = self.mcp_executor.execute(&tool_call.name, tool_call.args.clone(), context).await;
        
        match result {
            Ok(value) => Ok(crate::mcp::executor::ToolResult {
                id: tool_call.id.clone(),
                name: tool_call.name.clone(),
                result: value,
                error: None,
            }),
            Err(error) => {
                // Determine error type for retry decision
                let error_str = error.to_string();
                let error_type = self.classify_error(&error_str);
                let decision = self.retry_policy.should_retry(&error_type, attempt);
                
                match decision {
                    crate::agent::retry_policy::RetryDecision::Retry { delay_ms, .. } => {
                        console_log!("Will retry tool {} after {}ms (attempt {})", tool_call.name, delay_ms, attempt + 1);
                        Err(format!("Retry scheduled after {}ms", delay_ms))
                    },
                    crate::agent::retry_policy::RetryDecision::NoRetry { reason } => {
                        console_log!("Will not retry tool {}: {}", tool_call.name, reason);
                        Err(error_str)
                    },
                    _ => Err(error_str),
                }
            }
        }
    }
    
    /// Classify error for retry decision
    fn classify_error(&self, error: &str) -> ErrorType {
        let error_lower = error.to_lowercase();
        
        if error_lower.contains("timeout") || error_lower.contains("network") || error_lower.contains("connection") {
            ErrorType::NetworkError
        } else if error_lower.contains("rate limit") || error_lower.contains("too many requests") {
            ErrorType::RateLimitError
        } else if error_lower.contains("invalid") || error_lower.contains("argument") {
            ErrorType::ArgumentError
        } else if error_lower.contains("auth") || error_lower.contains("permission") {
            ErrorType::AuthenticationError
        } else {
            ErrorType::TransactionError
        }
    }

    /// Get MCP executor reference (for tool execution)
    pub fn get_executor(&self) -> &McpExecutor {
        &self.mcp_executor
    }
}
