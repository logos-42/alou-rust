use crate::agent::ai_client::{AiMessage, AiProvider, AiResponse, AiTool, AiToolCall};
use crate::utils::error::{AloudError, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use worker::{console_log, Fetch, Headers, Method, RequestInit};

const DEEPSEEK_API_URL: &str = "https://api.deepseek.com/v1/chat/completions";

pub struct DeepSeekProvider {
    api_key: String,
    model: String,
}

impl DeepSeekProvider {
    pub fn new(api_key: String, model: String) -> Self {
        Self { api_key, model }
    }
}

#[derive(Serialize)]
struct DeepSeekRequest {
    model: String,
    messages: Vec<DeepSeekMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<DeepSeekTool>>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Serialize)]
struct DeepSeekMessage {
    role: String,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<DeepSeekToolCallInMessage>>,
}

#[derive(Serialize)]
struct DeepSeekToolCallInMessage {
    id: String,
    #[serde(rename = "type")]
    call_type: String,
    function: DeepSeekFunctionCallInMessage,
}

#[derive(Serialize)]
struct DeepSeekFunctionCallInMessage {
    name: String,
    arguments: String,
}

#[derive(Serialize)]
struct DeepSeekTool {
    #[serde(rename = "type")]
    tool_type: String,
    function: DeepSeekFunction,
}

#[derive(Serialize)]
struct DeepSeekFunction {
    name: String,
    description: String,
    parameters: Value,
}

#[derive(Deserialize)]
struct DeepSeekResponse {
    choices: Vec<DeepSeekChoice>,
}

#[derive(Deserialize)]
struct DeepSeekChoice {
    message: DeepSeekResponseMessage,
    finish_reason: String,
}

#[derive(Deserialize)]
struct DeepSeekResponseMessage {
    content: Option<String>,
    #[serde(default)]
    tool_calls: Vec<DeepSeekToolCall>,
}

#[derive(Deserialize)]
struct DeepSeekToolCall {
    id: String,
    function: DeepSeekFunctionCall,
}

#[derive(Deserialize)]
struct DeepSeekFunctionCall {
    name: String,
    arguments: String,
}

#[async_trait(?Send)]
impl AiProvider for DeepSeekProvider {
    async fn send_message(
        &self,
        messages: Vec<AiMessage>,
        tools: Option<Vec<AiTool>>,
    ) -> Result<AiResponse> {
        console_log!("DeepSeek: Sending request to {}", DEEPSEEK_API_URL);
        console_log!(
            "DeepSeek: Model: {}, Messages: {}",
            self.model,
            messages.len()
        );

        let deepseek_messages: Vec<DeepSeekMessage> = messages
            .into_iter()
            .map(|m| {
                // 只在有工具调用时才添加 tool_calls，避免空数组
                let tool_calls_in_msg = m.tool_calls.and_then(|tcs| {
                    if tcs.is_empty() {
                        None
                    } else {
                        Some(tcs.into_iter()
                            .map(|tc| DeepSeekToolCallInMessage {
                                id: tc.id,
                                call_type: "function".to_string(),
                                function: DeepSeekFunctionCallInMessage {
                                    name: tc.name,
                                    arguments: serde_json::to_string(&tc.arguments).unwrap_or_default(),
                                },
                            })
                            .collect())
                    }
                });

                DeepSeekMessage {
                    role: m.role,
                    content: m.content,
                    tool_call_id: m.tool_call_id,
                    tool_calls: tool_calls_in_msg,
                }
            })
            .collect();

        let deepseek_tools = tools.map(|tools| {
            tools
                .into_iter()
                .map(|t| DeepSeekTool {
                    tool_type: "function".to_string(),
                    function: DeepSeekFunction {
                        name: t.name,
                        description: t.description,
                        parameters: t.parameters,
                    },
                })
                .collect()
        });

        // 记录请求详情用于调试（在移动之前）
        let messages_count = deepseek_messages.len();
        console_log!("DeepSeek: Model: {}, Messages count: {}", self.model, messages_count);
        console_log!("DeepSeek: Request URL: {}", DEEPSEEK_API_URL);
        
        let request = DeepSeekRequest {
            model: self.model.clone(),
            messages: deepseek_messages,
            tools: deepseek_tools,
            temperature: 0.7,
            max_tokens: 4096,
        };

        let body = serde_json::to_string(&request)
            .map_err(|e| AloudError::AgentError(format!("Serialize error: {}", e)))?;

        // 记录完整请求用于调试
        console_log!("DeepSeek: Full request body: {}", body);
        
        // 特别记录工具信息
        if let Some(ref tools) = request.tools {
            console_log!("DeepSeek: Sending {} tools to API", tools.len());
            for (i, tool) in tools.iter().enumerate() {
                console_log!("DeepSeek: Tool {}: type={}, function.name={}", i, tool.tool_type, tool.function.name);
            }
        } else {
            console_log!("DeepSeek: No tools in request");
        }

        // 记录请求预览
        let request_preview = body.chars().take(500).collect::<String>();
        console_log!("DeepSeek: Request preview (500 chars): {}", request_preview);

        // 验证 API key 不为空
        if self.api_key.is_empty() {
            return Err(AloudError::AgentError(
                "DeepSeek API key is empty. Please configure AI_API_KEY secret.".to_string()
            ));
        }
        
        // 记录 API key 的前几个字符用于调试（不记录完整 key）
        let api_key_preview = if self.api_key.len() > 8 {
            format!("{}...", &self.api_key[..8])
        } else {
            "***".to_string()
        };
        console_log!("DeepSeek: Using API key: {}", api_key_preview);
        
        let headers = {
            let h = Headers::new();
            h.set("Content-Type", "application/json")
                .map_err(|e| AloudError::AgentError(e.to_string()))?;
            h.set("Authorization", &format!("Bearer {}", self.api_key))
                .map_err(|e| AloudError::AgentError(e.to_string()))?;
            h
        };

        let init = {
            let mut i = RequestInit::new();
            i.with_method(Method::Post)
                .with_headers(headers)
                .with_body(Some(body.into()));
            i
        };
        
        let mut response = Fetch::Request(
            worker::Request::new_with_init(DEEPSEEK_API_URL, &init)
                .map_err(|e| AloudError::AgentError(e.to_string()))?,
        )
        .send()
        .await
        .map_err(|e| AloudError::AgentError(e.to_string()))?;

        let status_code = response.status_code();
        console_log!("DeepSeek: Response status: {}", status_code);
        
        if !status_code.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            console_log!("DeepSeek API error {}: {}", status_code, error_text);
            console_log!("DeepSeek: API key preview: {}", api_key_preview);
            
            // 尝试解析错误响应 JSON 以获取更详细的错误信息
            let error_message = if let Ok(error_json) = serde_json::from_str::<serde_json::Value>(&error_text) {
                if let Some(error_obj) = error_json.get("error") {
                    let message = error_obj.get("message")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown error");
                    let error_type = error_obj.get("type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown_error");
                    let error_code = error_obj.get("code")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown");
                    
                    // 特殊处理常见错误
                    if message.contains("Insufficient Balance") || message.contains("余额不足") {
                        format!("DeepSeek API 账户余额不足。请前往 DeepSeek 平台充值后再试。\n\n错误详情: {} (type: {}, code: {})",
                            message, error_type, error_code
                        )
                    } else if message.contains("governor") || message.contains("Governor") || 
                             message.contains("rate limit") || message.contains("限流") {
                        format!("DeepSeek API 限流或临时限制：{}\n\n可能原因：\n1. 请求频率过高，触发了限流保护\n2. API key 可能被临时限制\n3. 账户可能存在异常行为\n\n建议：\n1. 等待几分钟后重试\n2. 检查 DeepSeek 平台的账户状态\n3. 如果持续出现，可能需要联系 DeepSeek 支持\n\n错误详情: {} (type: {}, code: {})",
                            message, message, error_type, error_code
                        )
                    } else if error_type == "authentication_error" || error_type == "invalid_api_key" || 
                             message.contains("Invalid API key") || message.contains("API key") ||
                             message.contains("authentication") || message.contains("认证") ||
                             message.contains("Authentication Fails") {
                        format!("DeepSeek API 认证失败：{}\n\n请检查：\n1. AI_API_KEY 是否正确配置（运行: wrangler secret put AI_API_KEY）\n2. API key 是否有效且未过期\n3. API key 格式是否正确\n4. 如果错误包含 'governor'，可能是限流问题，请等待后重试\n\n错误详情: {} (type: {}, code: {})",
                            message, message, error_type, error_code
                        )
                    } else {
                        format!("DeepSeek API error {}: {} (type: {}, code: {})",
                            response.status_code(),
                            message,
                            error_type,
                            error_code
                        )
                    }
                } else {
                    format!("DeepSeek API error {}: {}", response.status_code(), error_text)
                }
            } else {
                format!("DeepSeek API error {}: {}", response.status_code(), error_text)
            };
            
            return Err(AloudError::AgentError(error_message));
        }

        let response_text = response
            .text()
            .await
            .map_err(|e| AloudError::AgentError(e.to_string()))?;

        let preview = response_text.chars().take(500).collect::<String>();
        console_log!("DeepSeek: Raw response: {}", preview);

        let deepseek_response: DeepSeekResponse =
            serde_json::from_str(&response_text).map_err(|e| {
                AloudError::AgentError(format!("Parse error: {} | Response: {}", e, response_text))
            })?;

        console_log!(
            "DeepSeek: Parsed response, {} choices",
            deepseek_response.choices.len()
        );

        let choice = deepseek_response
            .choices
            .first()
            .ok_or_else(|| AloudError::AgentError("No choices in response".to_string()))?;

        console_log!("DeepSeek: Choice finish_reason: {}", choice.finish_reason);
        console_log!(
            "DeepSeek: Message content is_some: {}",
            choice.message.content.is_some()
        );

        let content = choice.message.content.clone().unwrap_or_default();
        let content_preview = content.chars().take(100).collect::<String>();
        console_log!(
            "DeepSeek: Content length: {}, content: '{}'",
            content.len(),
            content_preview
        );

        let tool_calls: Vec<AiToolCall> = choice
            .message
            .tool_calls
            .iter()
            .filter_map(|tc| {
                let arguments = serde_json::from_str(&tc.function.arguments).ok()?;
                Some(AiToolCall {
                    id: tc.id.clone(),
                    name: tc.function.name.clone(),
                    arguments,
                })
            })
            .collect();

        console_log!(
            "DeepSeek: Response received, {} tool calls",
            tool_calls.len()
        );

        Ok(AiResponse {
            content,
            tool_calls,
            finish_reason: choice.finish_reason.clone(),
        })
    }
}

trait StatusCodeExt {
    fn is_success(&self) -> bool;
}

impl StatusCodeExt for u16 {
    fn is_success(&self) -> bool {
        (200..300).contains(self)
    }
}
