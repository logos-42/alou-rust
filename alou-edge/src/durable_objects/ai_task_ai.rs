//! AI调用模块
//! 
//! 负责AI客户端调用、消息转换和工具处理

use crate::compatibility::models::{CompatibleRequest, CompatibleResponse, Tool};
use crate::agent::ai_client::{AiClient, AiMessage, AiTool, AiResponse};
use worker::{Result, console_log, console_error};

#[cfg(target_arch = "wasm32")]
use gloo_timers::future::TimeoutFuture;
#[cfg(target_arch = "wasm32")]
use futures::future::{select, Either};
#[cfg(target_arch = "wasm32")]
use std::pin::Pin;

/// AI调用trait
pub trait AiCaller {
    /// 调用AI服务（带超时保护）
    async fn call_ai_with_timeout(
        &self,
        ai_client: AiClient,
        messages: Vec<AiMessage>,
        tools: Vec<AiTool>,
    ) -> Result<AiResponse>;
    
    /// 简化版AI调用
    async fn call_ai_simplified(
        &self,
        ai_client: AiClient,
        messages: Vec<AiMessage>,
    ) -> Result<AiResponse>;
    
    /// 转换到AI消息格式
    fn convert_to_ai_messages(&self, request: &CompatibleRequest) -> Vec<AiMessage>;
    
    /// 转换到AI工具格式
    fn convert_to_ai_tools(&self, tools: &[Tool]) -> Vec<AiTool>;
    
    /// 从AI响应转换
    fn convert_from_ai_response(&self, ai_response: AiResponse) -> CompatibleResponse;
}

/// 默认AI调用实现
pub struct DefaultAiCaller;

impl DefaultAiCaller {
    /// 调用AI服务（带超时保护）
    pub async fn call_ai_with_timeout(
        &self,
        ai_client: AiClient,
        messages: Vec<AiMessage>,
        tools: Vec<AiTool>,
        task_name: &str,
    ) -> Result<AiResponse> {
        console_log!("=== CALL_AI_WITH_TIMEOUT START ===");
        console_log!("Calling AI service for task: {}", task_name);
        
        #[cfg(target_arch = "wasm32")]
        {
            let ai_future = ai_client.send_message(messages, Some(tools));
            let timeout_future = TimeoutFuture::new(55_000);
            
            match select(Pin::from(Box::pin(ai_future)), Pin::from(Box::pin(timeout_future))).await {
                Either::Left((Ok(response), _)) => {
                    console_log!("✅ AI call completed successfully");
                    Ok(response)
                }
                Either::Left((Err(e), _)) => {
                    console_error!("❌ AI call failed: {}", e);
                    Err(worker::Error::RustError(format!("AI service error: {}", e)))
                }
                Either::Right((_, _)) => {
                    console_error!("⏰ AI call timed out");
                    Err(worker::Error::RustError("AI service call timeout".to_string()))
                }
            }
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            match ai_client.send_message(messages, Some(tools)).await {
                Ok(response) => {
                    console_log!("✅ AI call completed successfully");
                    Ok(response)
                }
                Err(e) => {
                    console_error!("❌ AI call failed: {}", e);
                    Err(worker::Error::RustError(format!("AI service error: {}", e)))
                }
            }
        }
    }
    
    /// 简化版AI调用
    pub async fn call_ai_simplified(
        &self,
        ai_client: AiClient,
        messages: Vec<AiMessage>,
        _task_name: &str,
    ) -> Result<AiResponse> {
        console_log!("[SIMPLIFIED-AI] Starting AI call");
        
        #[cfg(target_arch = "wasm32")]
        {
            let ai_future = ai_client.send_message(messages, None);
            let timeout_future = TimeoutFuture::new(55_000);
            
            match select(Pin::from(Box::pin(ai_future)), Pin::from(Box::pin(timeout_future))).await {
                Either::Left((Ok(response), _)) => {
                    console_log!("✅ [SIMPLIFIED-AI] AI call completed");
                    Ok(response)
                }
                Either::Left((Err(e), _)) => {
                    console_error!("❌ [SIMPLIFIED-AI] AI call failed: {}", e);
                    Err(worker::Error::RustError(format!("AI call failed: {}", e)))
                }
                Either::Right((_, _)) => {
                    console_error!("⏰ [SIMPLIFIED-AI] AI call timed out");
                    Err(worker::Error::RustError("AI call timeout".to_string()))
                }
            }
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            match ai_client.send_message(messages, None).await {
                Ok(response) => {
                    console_log!("✅ [SIMPLIFIED-AI] AI call completed");
                    Ok(response)
                }
                Err(e) => {
                    console_error!("❌ [SIMPLIFIED-AI] AI call failed: {}", e);
                    Err(worker::Error::RustError(format!("AI call failed: {}", e)))
                }
            }
        }
    }
    
    /// 转换到AI消息格式
    pub fn convert_to_ai_messages(&self, request: &CompatibleRequest) -> Vec<AiMessage> {
        let mut messages = Vec::new();
        
        // 添加系统提示
        if let Some(system_prompt) = &request.system_prompt {
            messages.push(AiMessage {
                role: "system".to_string(),
                content: system_prompt.clone(),
                tool_call_id: None,
                tool_calls: None,
            });
        }
        
        // 添加历史消息
        for history_msg in &request.history {
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
            content: request.prompt.clone(),
            tool_call_id: None,
            tool_calls: None,
        });
        
        messages
    }
    
    /// 转换到AI工具格式
    pub fn convert_to_ai_tools(&self, tools: &[Tool]) -> Vec<AiTool> {
        tools.iter().map(|tool| {
            AiTool {
                name: tool.name.clone(),
                description: tool.description.clone().unwrap_or_default(),
                parameters: tool.parameters.clone().unwrap_or_default(),
            }
        }).collect()
    }
    
    /// 从AI响应转换
    pub fn convert_from_ai_response(&self, ai_response: AiResponse, task_name: &str) -> CompatibleResponse {
        CompatibleResponse {
            success: true,
            response: Some(ai_response.content),
            tool_calls: Some(
                ai_response.tool_calls.into_iter().map(|tc| {
                    crate::compatibility::models::ToolCall {
                        tool: tc.name,
                        arguments: tc.arguments,
                    }
                }).collect()
            ),
            metadata: None,
            task_id: Some(task_name.to_string()),
            status: Some("completed".to_string()),
            progress: Some(1.0),
            estimated_time: None,
            error: None,
        }
    }
}
