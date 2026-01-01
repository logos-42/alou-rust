use serde::{Deserialize, Serialize};
use serde_json::json;
use worker::*;

use crate::agent::ai_client::{AiClient, AiMessage, AiResponse, AiTool};
use crate::compatibility::claude_sdk::{ClaudeSdkRequest, create_default_metadata, get_provider_from_model};
use crate::router::json_response;
use crate::router::json_response_with_status;
use crate::utils::error::AloudError;

/// Claude API 请求结构
#[derive(Debug, Deserialize)]
pub struct ClaudeMessageRequest {
    pub model: String,
    pub max_tokens: u32,
    pub messages: Vec<ClaudeMessage>,
    #[serde(default)]
    pub system: Option<String>,
    #[serde(default)]
    pub tools: Option<Vec<ClaudeTool>>,
    #[serde(default)]
    pub temperature: Option<f32>,
    #[serde(default)]
    pub top_p: Option<f32>,
    #[serde(default)]
    pub top_k: Option<u32>,
    #[serde(default)]
    pub stop_sequences: Option<Vec<String>>,
}

/// Claude 消息结构
#[derive(Debug, Deserialize)]
pub struct ClaudeMessage {
    pub role: String,
    pub content: String,
}

/// Claude 工具结构
#[derive(Debug, Deserialize)]
pub struct ClaudeTool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

/// Claude API 响应结构
#[derive(Debug, Serialize)]
pub struct ClaudeMessageResponse {
    pub id: String,
    #[serde(rename = "type")]
    pub response_type: String,
    pub role: String,
    pub content: Vec<ClaudeContent>,
    pub model: String,
    pub stop_reason: String,
    pub stop_sequence: Option<String>,
    pub usage: ClaudeUsage,
}

/// Claude 内容结构
#[derive(Debug, Serialize)]
pub struct ClaudeContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
}

/// Claude 使用情况
#[derive(Debug, Serialize)]
pub struct ClaudeUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

/// Claude 错误响应
#[derive(Debug, Serialize)]
pub struct ClaudeErrorResponse {
    #[serde(rename = "type")]
    pub error_type: String,
    pub error: ClaudeErrorDetail,
}

/// Claude 错误详情
#[derive(Debug, Serialize)]
pub struct ClaudeErrorDetail {
    #[serde(rename = "type")]
    pub error_type: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub param: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

/// 处理 Claude API 消息请求
pub async fn handle_claude_messages(req: &mut Request, env: &Env) -> Result<Response> {
    // 验证请求头
    let api_key = match req.headers().get("x-api-key") {
        Ok(Some(key)) => key,
        _ => {
            let error = ClaudeErrorResponse {
                error_type: "error".to_string(),
                error: ClaudeErrorDetail {
                    error_type: "authentication_error".to_string(),
                    message: "Missing x-api-key header".to_string(),
                    param: Some("x-api-key".to_string()),
                    code: Some("missing_api_key".to_string()),
                },
            };
            return json_response_with_status(&error, 401);
        }
    };

    // 验证 anthropic-version 头
    let _anthropic_version = match req.headers().get("anthropic-version") {
        Ok(Some(version)) => version,
        _ => {
            let error = ClaudeErrorResponse {
                error_type: "error".to_string(),
                error: ClaudeErrorDetail {
                    error_type: "invalid_request_error".to_string(),
                    message: "Missing anthropic-version header".to_string(),
                    param: Some("anthropic-version".to_string()),
                    code: Some("missing_version".to_string()),
                },
            };
            return json_response_with_status(&error, 400);
        }
    };

    // 解析请求体
    let claude_request: ClaudeMessageRequest = match req.json().await {
        Ok(request) => request,
        Err(e) => {
            console_error!("Failed to parse Claude request: {}", e);
            let error = ClaudeErrorResponse {
                error_type: "error".to_string(),
                error: ClaudeErrorDetail {
                    error_type: "invalid_request_error".to_string(),
                    message: format!("Invalid request body: {}", e),
                    param: None,
                    code: Some("invalid_request".to_string()),
                },
            };
            return json_response_with_status(&error, 400);
        }
    };

    console_log!(
        "Claude API request - model: {}, messages: {}, max_tokens: {}",
        claude_request.model,
        claude_request.messages.len(),
        claude_request.max_tokens
    );

    // 验证 API 密钥
    if !is_valid_api_key(&api_key, &env) {
        let error = ClaudeErrorResponse {
            error_type: "error".to_string(),
            error: ClaudeErrorDetail {
                error_type: "authentication_error".to_string(),
                message: "Invalid API key".to_string(),
                param: Some("x-api-key".to_string()),
                code: Some("invalid_api_key".to_string()),
            },
        };
        return json_response_with_status(&error, 401);
    }

    // 路由到相应的 AI 服务
    match route_to_ai_service(&claude_request, &env).await {
        Ok(response) => {
            console_log!("Claude API request completed successfully");
            json_response(&response)
        }
        Err(e) => {
            console_error!("Claude API request failed: {}", e);
            let error = ClaudeErrorResponse {
                error_type: "error".to_string(),
                error: ClaudeErrorDetail {
                    error_type: "api_error".to_string(),
                    message: format!("AI service error: {}", e),
                    param: None,
                    code: Some("service_error".to_string()),
                },
            };
            json_response_with_status(&error, 500)
        }
    }
}

/// 验证 API 密钥
fn is_valid_api_key(api_key: &str, env: &Env) -> bool {
    // 从环境变量获取有效的 API 密钥
    let valid_keys = match env.secret("CLAUDE_PROXY_API_KEYS") {
        Ok(secret) => secret.to_string(),
        Err(_) => {
            console_warn!("CLAUDE_PROXY_API_KEYS not set, using default validation");
            "".to_string()
        }
    };

    // 检查 API 密钥是否在有效列表中
    valid_keys.split(',').any(|key| key.trim() == api_key)
}

/// 路由到 AI 服务
async fn route_to_ai_service(
    request: &ClaudeMessageRequest,
    env: &Env,
) -> std::result::Result<ClaudeMessageResponse, AloudError> {
    // 根据模型名称选择目标服务
    let (provider_type, model_name) = match request.model.as_str() {
        model if model.starts_with("claude-") => ("claude", request.model.clone()),
        model if model.starts_with("deepseek-") => ("deepseek", request.model.clone()),
        model if model.starts_with("gpt-") => ("openai", request.model.clone()),
        model if model.starts_with("kimi-") => ("kimi", request.model.clone()),
        model if model.starts_with("qwen-") => ("qwen", request.model.clone()),
        _ => ("deepseek", "deepseek-chat".to_string()), // 默认使用 DeepSeek
    };

    console_log!("Routing to AI service: {} (model: {})", provider_type, model_name);

    // 转换 Claude 格式到 AI 客户端格式
    let messages = convert_claude_messages_to_ai(&request.messages, &request.system);
    let tools = convert_claude_tools_to_ai(&request.tools);

    // 获取 API 密钥
    let api_key = match get_api_key_for_provider(provider_type, env) {
        Ok(key) => key,
        Err(e) => {
            console_error!("Failed to get API key for provider {}: {}", provider_type, e);
            return Err(AloudError::AgentError(format!("API key not configured for {}", provider_type)));
        }
    };

    // 创建 AI 客户端
    let ai_client = match AiClient::new(provider_type, api_key, Some(model_name)) {
        Ok(client) => client,
        Err(e) => {
            console_error!("Failed to create AI client: {}", e);
            return Err(AloudError::AgentError(format!("Failed to create AI client: {}", e)));
        }
    };

    // 调用 AI 服务
    let ai_response = match ai_client.send_message(messages, tools).await {
        Ok(response) => response,
        Err(e) => {
            console_error!("AI service error: {}", e);
            return Err(AloudError::AgentError(format!("AI service error: {}", e)));
        }
    };

    // 转换 AI 响应到 Claude 格式
    convert_ai_response_to_claude(ai_response, &request.model)
}

/// 创建模拟响应（临时实现）
fn create_mock_response(request: &ClaudeMessageRequest) -> ClaudeMessageResponse {
    // 从消息中提取文本
    let user_message = request
        .messages
        .iter()
        .find(|msg| msg.role == "user")
        .map(|msg| msg.content.as_str())
        .unwrap_or("");

    ClaudeMessageResponse {
        id: format!("msg_{}", uuid::Uuid::new_v4()),
        response_type: "message".to_string(),
        role: "assistant".to_string(),
        content: vec![ClaudeContent {
            content_type: "text".to_string(),
            text: format!("这是来自 {} 模型的模拟响应。用户说：{}", request.model, user_message),
        }],
        model: request.model.clone(),
        stop_reason: "end_turn".to_string(),
        stop_sequence: None,
        usage: ClaudeUsage {
            input_tokens: 10,
            output_tokens: 20,
        },
    }
}

/// 处理 Claude API 模型列表请求
pub async fn handle_claude_models(_req: &mut Request, _env: &Env) -> Result<Response> {
    let models_response = json!({
        "data": [
            {
                "id": "claude-3-5-sonnet-20241022",
                "object": "model",
                "created": 1697130000,
                "owned_by": "anthropic"
            },
            {
                "id": "deepseek-chat",
                "object": "model",
                "created": 1700000000,
                "owned_by": "deepseek"
            },
            {
                "id": "deepseek-reasoner",
                "object": "model",
                "created": 1701000000,
                "owned_by": "deepseek"
            },
            {
                "id": "gpt-4o",
                "object": "model",
                "created": 1710000000,
                "owned_by": "openai"
            },
            {
                "id": "kimi-k2",
                "object": "model",
                "created": 1715000000,
                "owned_by": "moonshot"
            },
            {
                "id": "qwen-max",
                "object": "model",
                "created": 1720000000,
                "owned_by": "qwen"
            }
        ],
        "object": "list"
    });

    json_response(&models_response)
}

/// 转换 Claude 消息到 AI 客户端格式
fn convert_claude_messages_to_ai(
    claude_messages: &[ClaudeMessage],
    system_prompt: &Option<String>,
) -> Vec<AiMessage> {
    let mut messages = Vec::new();

    // 添加系统提示词
    if let Some(system) = system_prompt {
        messages.push(AiMessage::text("system", system.clone()));
    }

    // 转换 Claude 消息
    for claude_msg in claude_messages {
        let role = match claude_msg.role.as_str() {
            "user" => "user",
            "assistant" => "assistant",
            "tool" => "tool",
            _ => "user", // 默认
        };

        // 注意：Claude 格式没有 tool_call_id 和 tool_calls 字段
        // 如果需要支持工具调用，需要从其他地方获取这些信息
        messages.push(AiMessage {
            role: role.to_string(),
            content: claude_msg.content.clone(),
            tool_call_id: None,
            tool_calls: None,
        });
    }

    messages
}

/// 转换 Claude 工具到 AI 客户端格式
fn convert_claude_tools_to_ai(
    claude_tools: &Option<Vec<ClaudeTool>>,
) -> Option<Vec<AiTool>> {
    claude_tools.as_ref().map(|tools| {
        tools.iter().map(|tool| {
            AiTool {
                name: tool.name.clone(),
                description: tool.description.clone(),
                parameters: tool.input_schema.clone(),
            }
        }).collect()
    })
}

/// 获取提供商对应的 API 密钥
fn get_api_key_for_provider(provider_type: &str, env: &Env) -> std::result::Result<String, AloudError> {
    let env_var_name = match provider_type {
        "deepseek" => "DEEPSEEK_API_KEY",
        "openai" => "OPENAI_API_KEY",
        "claude" => "ANTHROPIC_API_KEY",
        "qwen" => "QWEN_API_KEY",
        "kimi" => "KIMI_API_KEY",
        _ => "AI_API_KEY", // 默认
    };

    match env.var(env_var_name) {
        Ok(secret) => Ok(secret.to_string()),
        Err(_) => {
            // 尝试使用通用的 AI_API_KEY
            match env.var("AI_API_KEY") {
                Ok(secret) => Ok(secret.to_string()),
                Err(_) => Err(AloudError::AgentError(format!(
                    "API key not found for provider {}. Please set {} or AI_API_KEY environment variable.",
                    provider_type, env_var_name
                ))),
            }
        }
    }
}

/// 转换 AI 响应到 Claude 格式
fn convert_ai_response_to_claude(
    ai_response: AiResponse,
    model: &str,
) -> std::result::Result<ClaudeMessageResponse, AloudError> {
    // 转换工具调用
    let tool_calls_in_content = if !ai_response.tool_calls.is_empty() {
        // 如果有工具调用，需要特殊处理 Claude 格式
        // 这里先简单地将工具调用信息添加到内容中
        let tool_calls_text: Vec<String> = ai_response.tool_calls.iter()
            .map(|tc| format!("[工具调用: {}]", tc.name))
            .collect();
        format!("{}\n\n工具调用: {}", ai_response.content, tool_calls_text.join(", "))
    } else {
        ai_response.content
    };

    Ok(ClaudeMessageResponse {
        id: format!("msg_{}", uuid::Uuid::new_v4()),
        response_type: "message".to_string(),
        role: "assistant".to_string(),
        content: vec![ClaudeContent {
            content_type: "text".to_string(),
            text: tool_calls_in_content,
        }],
        model: model.to_string(),
        stop_reason: ai_response.finish_reason,
        stop_sequence: None,
        usage: ClaudeUsage {
            input_tokens: 0, // 需要从 AI 响应中获取，但当前 AiResponse 没有这个字段
            output_tokens: 0,
        },
    })
}

/// 处理 Claude Agent SDK 格式的请求
pub async fn handle_claude_sdk_query(
    req: &mut Request,
    env: &Env,
) -> Result<Response> {
    // 解析 Claude SDK 格式的请求
    let claude_sdk_req: ClaudeSdkRequest = match req.json().await {
        Ok(req) => req,
        Err(e) => {
            console_error!("Failed to parse Claude SDK request: {}", e);
            let metadata = create_default_metadata("unknown", "unknown");
            let error_response = crate::compatibility::claude_sdk::create_error_response(
                &format!("Failed to parse request: {}", e),
                metadata,
            );
            return json_response(&error_response);
        }
    };

    console_log!(
        "Claude SDK request - model: {}, prompt length: {}, tools: {}",
        claude_sdk_req.model,
        claude_sdk_req.prompt.len(),
        claude_sdk_req.tools.len()
    );

    // 确定提供商
    let provider = get_provider_from_model(&claude_sdk_req.model);
    let metadata = create_default_metadata(&claude_sdk_req.model, provider);

    // 转换到兼容性格式
    let compat_req = crate::compatibility::claude_sdk::convert_from_claude_sdk(&claude_sdk_req);
    
    // 转换到 AI 客户端格式
    let messages = match crate::compatibility::claude_sdk::convert_to_ai_messages(&compat_req) {
        Ok(msgs) => msgs,
        Err(e) => {
            console_error!("Failed to convert messages: {}", e);
            let error_response = crate::compatibility::claude_sdk::create_error_response(
                &format!("Failed to convert messages: {}", e),
                metadata,
            );
            return json_response(&error_response);
        }
    };

    let tools = crate::compatibility::claude_sdk::convert_to_ai_tools(&compat_req.tools);
    let tools_option = if tools.is_empty() { None } else { Some(tools) };

    // 获取 API 密钥
    let api_key = match get_api_key_for_provider(provider, env) {
        Ok(key) => key,
        Err(e) => {
            console_error!("Failed to get API key: {}", e);
            let error_response = crate::compatibility::claude_sdk::create_error_response(
                &e.to_string().as_str(),
                metadata,
            );
            return json_response(&error_response);
        }
    };

    // 创建 AI 客户端
    let ai_client = match AiClient::new(provider, api_key, Some(claude_sdk_req.model.clone())) {
        Ok(client) => client,
        Err(e) => {
            console_error!("Failed to create AI client: {}", e);
            let error_response = crate::compatibility::claude_sdk::create_error_response(
                &format!("Failed to create AI client: {}", e),
                metadata,
            );
            return json_response(&error_response);
        }
    };

    // 调用 AI 服务
    let ai_response = match ai_client.send_message(messages, tools_option).await {
        Ok(response) => response,
        Err(e) => {
            console_error!("AI service error: {}", e);
            let error_response = crate::compatibility::claude_sdk::create_error_response(
                &format!("AI service error: {}", e),
                metadata,
            );
            return json_response(&error_response);
        }
    };

    // 转换回 Claude SDK 格式
    match crate::compatibility::claude_sdk::convert_to_claude_sdk(ai_response, &compat_req, metadata.clone()) {
        Ok(sdk_response) => {
            console_log!("Claude SDK request completed successfully");
            json_response(&sdk_response)
        }
        Err(e) => {
            console_error!("Failed to convert response: {}", e);
            let error_response = crate::compatibility::claude_sdk::create_error_response(
                &format!("Failed to convert response: {}", e),
                metadata,
            );
            json_response(&error_response)
        }
    }
}
