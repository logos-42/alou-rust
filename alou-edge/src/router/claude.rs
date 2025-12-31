use serde::{Deserialize, Serialize};
use serde_json::json;
use worker::*;

use crate::router::json_response;
use crate::router::json_response_with_status;
use crate::router::ErrorResponse;
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
    let target_service = match request.model.as_str() {
        model if model.starts_with("claude-") => "anthropic",
        model if model.starts_with("deepseek-") => "deepseek",
        model if model.starts_with("gpt-") => "openai",
        model if model.starts_with("kimi-") => "kimi",
        model if model.starts_with("qwen-") => "qwen",
        _ => "default",
    };

    console_log!("Routing to AI service: {}", target_service);

    // 这里应该调用相应的 AI 服务
    // 目前先返回一个模拟响应
    Ok(create_mock_response(request))
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
