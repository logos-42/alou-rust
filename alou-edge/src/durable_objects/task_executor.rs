//! 任务执行器 - 负责执行AI任务

use crate::compatibility::models::{CompatibleRequest, CompatibleResponse, HistoryMessage, Tool};
use crate::agent::ai_client::{AiClient, AiMessage, AiTool, AiResponse};
use serde_json::Value;
use wasm_bindgen_futures::spawn_local;
use worker::{Env, Request, Result};

/// 任务执行器
pub struct TaskExecutor;

impl TaskExecutor {
    /// 创建异步任务
    pub async fn create_async_task(
        env: &Env,
        request: CompatibleRequest,
    ) -> Result<String> {
        // 生成任务ID
        let task_id = Self::generate_task_id();
        
        // 获取Durable Object stub
        let namespace = env.durable_object("AI_TASKS")?;
        let id = namespace.id_from_name(&task_id)?;
        let stub = id.get_stub()?;
        
        // 初始化任务
        let init_response = stub
            .fetch_with_str("/init")
            .await?;
        
        if init_response.status_code() != 200 {
            return Err(worker::Error::RustError(
                format!("Failed to initialize task: {}", init_response.status_code())
            ));
        }
        
        // 异步开始执行任务
        let env_clone = env.clone();
        let task_id_clone = task_id.clone();
        spawn_local(async move {
            // 重新获取stub来开始执行
            if let Ok(namespace) = env_clone.durable_object("AI_TASKS") {
                if let Ok(id) = namespace.id_from_name(&task_id_clone) {
                    if let Ok(stub) = id.get_stub() {
                        let _ = stub
                            .fetch_with_str("/start")
                            .await;
                    }
                }
            }
        });
        
        Ok(task_id)
    }
    
    /// 执行同步任务
    pub async fn execute_sync_task(
        env: &Env,
        request: CompatibleRequest,
    ) -> Result<CompatibleResponse> {
        // 转换请求格式
        let messages = Self::convert_to_ai_messages(&request);
        let tools = Self::convert_to_ai_tools(&request.tools);
        
        // 创建AI客户端
        let ai_api_key = match env.var("AI_API_KEY") {
            Ok(key) => key.to_string(),
            Err(_) => {
                return Ok(CompatibleResponse::error_response(
                    "AI_API_KEY not configured".to_string(),
                ));
            }
        };
        
        let ai_client = match AiClient::new("deepseek", ai_api_key, Some(request.model.clone())) {
            Ok(client) => client,
            Err(e) => {
                return Ok(CompatibleResponse::error_response(
                    format!("Failed to create AI client: {}", e),
                ));
            }
        };
        
        // 调用AI服务
        match ai_client.send_message(messages, Some(tools)).await {
            Ok(ai_response) => {
                // 转换响应格式
                let response = Self::convert_from_ai_response(ai_response);
                Ok(response)
            }
            Err(e) => {
                Ok(CompatibleResponse::error_response(
                    format!("AI service error: {}", e),
                ))
            }
        }
    }
    
    /// 取消任务
    pub async fn cancel_task(env: &Env, task_id: &str) -> Result<bool> {
        let namespace = env.durable_object("AI_TASKS")?;
        let id = namespace.id_from_name(task_id)?;
        let stub = id.get_stub()?;
        
        let response = stub
            .fetch_with_str("/cancel")
            .await?;
        
        Ok(response.status_code() == 200)
    }
    
    /// 提交工具调用结果
    pub async fn submit_tool_result(
        env: &Env,
        task_id: &str,
        tool_result: serde_json::Value,
    ) -> Result<bool> {
        let namespace = env.durable_object("AI_TASKS")?;
        let id = namespace.id_from_name(task_id)?;
        let stub = id.get_stub()?;
        
        let response = stub
            .fetch_with_str("/tool-result")
            .await?;
        
        Ok(response.status_code() == 200)
    }
    
    /// 生成任务ID
    fn generate_task_id() -> String {
        // 使用时间戳和随机数生成任务ID
        let timestamp = worker::Date::now().as_millis();
        let random_part = (timestamp % 1000000) as u32;
        format!("task_{:x}_{:x}", timestamp, random_part)
    }
    
    /// 估计任务执行时间
    pub fn estimate_execution_time(request: &CompatibleRequest) -> u64 {
        crate::compatibility::models::estimate_execution_time(request)
    }
    
    /// 判断是否应该使用异步处理
    pub fn should_use_async(request: &CompatibleRequest) -> bool {
        crate::compatibility::models::should_use_async(request)
    }
    
    /// 转换到AI消息格式
    fn convert_to_ai_messages(request: &CompatibleRequest) -> Vec<AiMessage> {
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
    fn convert_to_ai_tools(tools: &[Tool]) -> Vec<AiTool> {
        tools.iter().map(|tool| {
            AiTool {
                name: tool.name.clone(),
                description: tool.description.clone().unwrap_or_default(),
                parameters: tool.parameters.clone().unwrap_or_default(),
            }
        }).collect()
    }
    
    /// 从AI响应转换
    fn convert_from_ai_response(ai_response: AiResponse) -> CompatibleResponse {
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
            task_id: None,
            status: Some("completed".to_string()),
            progress: Some(1.0),
            estimated_time: None,
            error: None,
        }
    }
}
