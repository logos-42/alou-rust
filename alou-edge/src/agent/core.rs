use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::agent::context::AgentContext;
use crate::agent::session::{SessionManager, Message};
use crate::agent::claude_client::{ClaudeClient, ClaudeMessage, ClaudeTool, ToolUse};
use crate::agent::ai_client::{AiClient, AiMessage, AiTool};
use crate::agent::prompts::PromptMode;
use crate::mcp::executor::{McpExecutor, ToolCall};
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
}

impl AgentCore {
    /// Create AgentCore with Claude (legacy)
    #[allow(dead_code)]
    pub fn new(
        api_key: String,
        session_manager: SessionManager,
        mcp_executor: McpExecutor,
    ) -> Self {
        Self {
            claude_client: Some(ClaudeClient::new(api_key)),
            ai_client: None,
            provider_type: AiProviderType::Claude,
            session_manager,
            mcp_executor,
        }
    }
    
    /// Create AgentCore with configurable AI provider
    pub fn with_provider(
        provider: &str,
        api_key: String,
        model: Option<String>,
        session_manager: SessionManager,
        mcp_executor: McpExecutor,
    ) -> Result<Self> {
        let provider_type = match provider.to_lowercase().as_str() {
            "claude" => AiProviderType::Claude,
            "deepseek" => AiProviderType::DeepSeek,
            "qwen" => AiProviderType::Qwen,
            "openai" => AiProviderType::OpenAI,
            _ => return Err(AloudError::InvalidInput(format!("Unknown provider: {}", provider))),
        };
        
        let ai_client = AiClient::new(provider, api_key, model)?;
        
        Ok(Self {
            claude_client: None,
            ai_client: Some(ai_client),
            provider_type,
            session_manager,
            mcp_executor,
        })
    }
    
    /// Handle a user message with tool calling loop
    pub async fn handle_message(
        &self,
        session_id: &str,
        message: &str,
        wallet_address: Option<String>,
    ) -> Result<AgentResponse> {
        // Detect prompt mode from user message
        let prompt_mode = PromptMode::detect_from_message(message);
        let system_prompt = prompt_mode.system_prompt();
        
        // Add user message to session
        self.session_manager
            .add_message(session_id, "user", message)
            .await?;
        
        // Load conversation history
        let history = self.session_manager.get_history(session_id).await?;
        
        // Get session to extract chain info
        let session = self.session_manager.get_session(session_id).await?;
        
        // Create agent context
        let context = if let Some(wallet) = wallet_address.or(session.wallet_address) {
            AgentContext {
                session_id: session_id.to_string(),
                wallet_address: Some(wallet),
                chain: None, // TODO: Extract from session or JWT
            }
        } else {
            AgentContext::new(session_id.to_string())
        };
        
        // Convert history to Claude messages
        let mut messages = self.history_to_claude_messages(&history);
        
        // Prepend system message if this is the first message in the conversation
        if messages.is_empty() || !messages.iter().any(|m| m.role == "system") {
            messages.insert(0, ClaudeMessage::text("system", system_prompt.to_string()));
        }
        
        // Get available tools
        let tools = self.get_claude_tools();
        
        // Tool calling loop
        let mut iterations = 0;
        let mut tool_call_info = Vec::new();
        
        loop {
            iterations += 1;
            if iterations > MAX_TOOL_ITERATIONS {
                return Err(AloudError::AgentError(
                    "Maximum tool iterations exceeded".to_string()
                ));
            }
            
            // Call AI API (Claude or other provider)
            let response = if let Some(ai_client) = &self.ai_client {
                // Use new unified AI client
                let ai_messages: Vec<AiMessage> = messages
                    .iter()
                    .map(|m| {
                        // Extract text content and tool calls from ContentBlocks
                        let mut text_parts = Vec::new();
                        let mut tool_uses = Vec::new();
                        let mut tool_result_id = None;
                        
                        for block in &m.content {
                            match block {
                                crate::agent::claude_client::ContentBlock::Text { text } => {
                                    text_parts.push(text.clone());
                                },
                                crate::agent::claude_client::ContentBlock::ToolUse { id, name, input } => {
                                    tool_uses.push(crate::agent::ai_client::AiToolCall {
                                        id: id.clone(),
                                        name: name.clone(),
                                        arguments: input.clone(),
                                    });
                                },
                                crate::agent::claude_client::ContentBlock::ToolResult { tool_use_id, content } => {
                                    tool_result_id = Some(tool_use_id.clone());
                                    text_parts.push(content.clone());
                                },
                            }
                        }
                        
                        let content = text_parts.join("\n");
                        
                        // Build appropriate message type
                        if let Some(tool_id) = tool_result_id {
                            // This is a tool result message
                            AiMessage::tool_result(tool_id, content)
                        } else if !tool_uses.is_empty() {
                            // This is an assistant message with tool calls
                            AiMessage::assistant_with_tools(content, tool_uses)
                        } else {
                            // Regular text message
                            AiMessage::text(&m.role, content)
                        }
                    })
                    .collect();
                
                let ai_tools: Vec<AiTool> = tools
                    .iter()
                    .map(|t| AiTool {
                        name: t.name.clone(),
                        description: t.description.clone(),
                        parameters: t.input_schema.clone(),
                    })
                    .collect();
                
                let ai_response = ai_client
                    .send_message(ai_messages, Some(ai_tools))
                    .await?;
                
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
                return Err(AloudError::AgentError("No AI client configured".to_string()));
            };
            
            // If no tool calls, we're done
            if response.tool_calls.is_empty() {
                // Add assistant response to session
                self.session_manager
                    .add_message(session_id, "assistant", &response.content)
                    .await?;
                
                return Ok(AgentResponse {
                    content: response.content,
                    session_id: session_id.to_string(),
                    tool_calls: tool_call_info,
                });
            }
            
            // Execute tool calls
            let tool_results = self.execute_tool_calls(&response.tool_calls, &context).await;
            
            // Add assistant message with tool calls to history
            // For providers that need it (like DeepSeek), we need to include the tool calls in the assistant message
            if self.ai_client.is_some() {
                // For unified AI client (DeepSeek, etc.), add assistant message with tool calls
                let mut assistant_msg = ClaudeMessage::text("assistant", response.content.clone());
                // Add tool use blocks for Claude compatibility
                for tool_use in &response.tool_calls {
                    assistant_msg.content.push(crate::agent::claude_client::ContentBlock::ToolUse {
                        id: tool_use.id.clone(),
                        name: tool_use.name.clone(),
                        input: tool_use.input.clone(),
                    });
                }
                messages.push(assistant_msg);
            } else if !response.content.is_empty() {
                // For Claude client, just add text
                messages.push(ClaudeMessage::text("assistant", response.content.clone()));
            }
            
            // Add tool results to messages and session
            for (tool_use, result) in response.tool_calls.iter().zip(tool_results.iter()) {
                let result_str = serde_json::to_string(&result.result)
                    .unwrap_or_else(|_| "{}".to_string());
                
                // Add to Claude messages (using tool result format)
                messages.push(ClaudeMessage::with_tool_result(
                    "user",
                    tool_use.id.clone(),
                    result_str.clone(),
                ));
                
                // Add to session
                self.session_manager
                    .add_message_with_tool_call(
                        session_id,
                        "tool",
                        &result_str,
                        &tool_use.id,
                    )
                    .await?;
                
                // Track tool call info
                tool_call_info.push(ToolCallInfo {
                    id: tool_use.id.clone(),
                    name: tool_use.name.clone(),
                    result: result.result.clone(),
                });
            }
            
            // Continue loop to get next response from Claude
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
            .map(|tu| ToolCall {
                id: tu.id.clone(),
                name: tu.name.clone(),
                args: tu.input.clone(),
            })
            .collect();
        
        self.mcp_executor.execute_batch(tool_calls, context).await
    }
    
    /// Convert session history to Claude messages
    fn history_to_claude_messages(&self, history: &[Message]) -> Vec<ClaudeMessage> {
        history
            .iter()
            .map(|m| ClaudeMessage::text(&m.role, m.content.clone()))
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
}
