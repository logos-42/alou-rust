//! Claude Agent SDK 兼容性模块
//! 
//! 处理 Claude Agent SDK 格式的请求和响应转换

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::agent::ai_client::{AiMessage, AiTool, AiResponse};
use crate::compatibility::models::{CompatibleRequest, HistoryMessage, Tool};
use crate::utils::error::AloudError;

/// Claude Agent SDK 请求格式（简化版）
/// 参考: claude-agent.js 中的输入格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeSdkRequest {
    #[serde(rename = "apiKey")]
    pub api_key: Option<String>,
    pub prompt: String,
    #[serde(rename = "systemPrompt")]
    pub system_prompt: Option<String>,
    #[serde(default)]
    pub history: Vec<HistoryMessage>,
    #[serde(rename = "agentInfo")]
    pub agent_info: Option<Value>,
    #[serde(default)]
    pub tools: Vec<Tool>,
    pub model: String,
    #[serde(rename = "maxTokens")]
    pub max_tokens: u32,
    #[serde(default = "default_temperature")]
    pub temperature: f32,
}

fn default_temperature() -> f32 {
    0.7
}

/// Claude Agent SDK 响应格式
/// 参考: claude-agent.js 中的输出格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeSdkResponse {
    pub success: bool,
    pub response: String,
    #[serde(rename = "toolCalls")]
    pub tool_calls: Vec<ClaudeSdkToolCall>,
    pub usage: ClaudeSdkUsage,
    pub metadata: ClaudeSdkMetadata,
}

/// Claude Agent SDK 工具调用格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeSdkToolCall {
    pub tool: String,
    pub arguments: Value,
}

/// Claude Agent SDK 使用统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeSdkUsage {
    #[serde(rename = "input_tokens")]
    pub input_tokens: u32,
    #[serde(rename = "output_tokens")]
    pub output_tokens: u32,
}

/// Claude Agent SDK 元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeSdkMetadata {
    #[serde(rename = "backend_url")]
    pub backend_url: Option<String>,
    #[serde(rename = "routed_to")]
    pub routed_to: String,
    #[serde(rename = "model_used")]
    pub model_used: String,
    #[serde(rename = "model_provider")]
    pub model_provider: String,
    #[serde(rename = "supports_tools")]
    pub supports_tools: bool,
    #[serde(rename = "supports_system_prompt")]
    pub supports_system_prompt: bool,
    #[serde(rename = "request_adjusted")]
    pub request_adjusted: bool,
    pub timestamp: String,
}

/// 转换 Claude Agent SDK 请求到兼容性请求
pub fn convert_from_claude_sdk(claude_req: &ClaudeSdkRequest) -> CompatibleRequest {
    CompatibleRequest {
        api_key: claude_req.api_key.clone(),
        prompt: claude_req.prompt.clone(),
        system_prompt: claude_req.system_prompt.clone(),
        history: claude_req.history.clone(),
        agent_info: claude_req.agent_info.clone(),
        tools: claude_req.tools.clone(),
        model: claude_req.model.clone(),
        max_tokens: Some(claude_req.max_tokens as i32),
        temperature: claude_req.temperature,
        task_type: Some("sync".to_string()), // Claude SDK 默认使用同步
        timeout: None,
        callback_url: None,
        session_id: None,
        wallet_address: None,
        chain: None,
        context_events: Vec::new(),
    }
}

/// 转换兼容性请求到 AI 客户端消息
pub fn convert_to_ai_messages(compat_req: &CompatibleRequest) -> Result<Vec<AiMessage>, AloudError> {
    let mut messages = Vec::new();

    // 添加系统提示词
    if let Some(system_prompt) = &compat_req.system_prompt {
        messages.push(AiMessage {
            role: "system".to_string(),
            content: system_prompt.clone(),
            tool_call_id: None,
            tool_calls: None,
        });
    }

    // 添加历史消息
    for history_msg in &compat_req.history {
        messages.push(AiMessage {
            role: history_msg.role.clone(),
            content: history_msg.content.clone(),
            tool_call_id: None,
            tool_calls: None,
        });
    }

    // 添加当前提示
    messages.push(AiMessage {
        role: "user".to_string(),
        content: compat_req.prompt.clone(),
        tool_call_id: None,
        tool_calls: None,
    });

    Ok(messages)
}

/// 转换兼容性工具到 AI 客户端工具
pub fn convert_to_ai_tools(tools: &[Tool]) -> Vec<AiTool> {
    tools.iter().map(|tool| {
        AiTool {
            name: tool.name.clone(),
            description: tool.description.clone().unwrap_or_default(),
            parameters: tool.parameters.clone().unwrap_or_default(),
        }
    }).collect()
}

/// 转换 AI 响应到 Claude Agent SDK 响应
pub fn convert_to_claude_sdk(
    ai_response: AiResponse,
    _compat_req: &CompatibleRequest,
    metadata: ClaudeSdkMetadata,
) -> Result<ClaudeSdkResponse, AloudError> {
    // 转换工具调用
    let tool_calls: Vec<ClaudeSdkToolCall> = ai_response.tool_calls.iter()
        .map(|tc| {
            ClaudeSdkToolCall {
                tool: tc.name.clone(),
                arguments: tc.arguments.clone(),
            }
        })
        .collect();

    // 创建使用统计（目前使用模拟值，实际应从 AI 响应中获取）
    let usage = ClaudeSdkUsage {
        input_tokens: 0, // 需要从 AI 响应中获取
        output_tokens: 0, // 需要从 AI 响应中获取
    };

    Ok(ClaudeSdkResponse {
        success: true,
        response: ai_response.content,
        tool_calls,
        usage,
        metadata,
    })
}

/// 创建错误响应
pub fn create_error_response(error: &str, metadata: ClaudeSdkMetadata) -> ClaudeSdkResponse {
    ClaudeSdkResponse {
        success: false,
        response: error.to_string(),
        tool_calls: Vec::new(),
        usage: ClaudeSdkUsage {
            input_tokens: 0,
            output_tokens: 0,
        },
        metadata,
    }
}

/// 创建默认元数据
pub fn create_default_metadata(model: &str, provider: &str) -> ClaudeSdkMetadata {
    let now = chrono::Utc::now();
    
    // 确定模型特性
    let (supports_tools, supports_system_prompt) = match provider {
        "deepseek" => (true, true),
        "openai" => (true, true),
        "claude" => (true, true),
        "qwen" => (true, true),
        "kimi" => (true, true),
        _ => (false, false),
    };

    ClaudeSdkMetadata {
        backend_url: None,
        routed_to: provider.to_string(),
        model_used: model.to_string(),
        model_provider: provider.to_string(),
        supports_tools,
        supports_system_prompt,
        request_adjusted: false,
        timestamp: now.to_rfc3339(),
    }
}

/// 根据模型名称确定提供商
pub fn get_provider_from_model(model: &str) -> &'static str {
    match model {
        m if m.starts_with("claude-") => "claude",
        m if m.starts_with("deepseek-") => "deepseek",
        m if m.starts_with("gpt-") => "openai",
        m if m.starts_with("kimi-") => "kimi",
        m if m.starts_with("qwen-") => "qwen",
        _ => "deepseek", // 默认
    }
}
