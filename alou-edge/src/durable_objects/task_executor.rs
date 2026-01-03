//! 任务执行器 - 负责执行AI任务

use crate::compatibility::models::{CompatibleRequest, CompatibleResponse, Tool};
use crate::agent::ai_client::{AiClient, AiMessage, AiTool, AiResponse};
use worker::{Env, Method, Request, RequestInit, Result, console_error};

/// 任务执行器
pub struct TaskExecutor;

impl TaskExecutor {
    /// 创建异步任务
    pub async fn create_async_task(
        env: &Env,
        request: CompatibleRequest,
    ) -> Result<String> {
        console_error!("[TaskExecutor] Starting create_async_task");
        
        // 生成任务ID
        let task_id = Self::generate_task_id();
        console_error!("[TaskExecutor] Generated task_id: {}", task_id);
        
        // 获取Durable Object stub
        let namespace = match env.durable_object("AI_TASKS") {
            Ok(ns) => {
                console_error!("[TaskExecutor] Got AI_TASKS namespace");
                ns
            }
            Err(e) => {
                console_error!("[TaskExecutor] Failed to get AI_TASKS namespace: {}", e);
                return Err(e);
            }
        };
        
        let id = match namespace.id_from_name(&task_id) {
            Ok(id) => {
                console_error!("[TaskExecutor] Got DO id from name");
                id
            }
            Err(e) => {
                console_error!("[TaskExecutor] Failed to get DO id from name: {}", e);
                return Err(e);
            }
        };
        
        let stub = match id.get_stub() {
            Ok(stub) => {
                console_error!("[TaskExecutor] Got DO stub");
                stub
            }
            Err(e) => {
                console_error!("[TaskExecutor] Failed to get DO stub: {}", e);
                return Err(e);
            }
        };
        
        // 初始化任务 - 使用POST方法发送请求体
        let request_json = match serde_json::to_string(&request) {
            Ok(json) => {
                console_error!("[TaskExecutor] Serialized request to JSON");
                json
            }
            Err(e) => {
                console_error!("[TaskExecutor] Failed to serialize request: {}", e);
                return Err(worker::Error::RustError(
                    format!("Failed to serialize request: {}", e)
                ));
            }
        };
        
        // 使用 fetch_with_request 调用 Durable Object
        console_error!("[TaskExecutor] Creating init-and-start request");
        
        // 1. 设置正确的Content-Type头
        let headers = worker::Headers::new();
        
        if let Err(e) = headers.set("Content-Type", "application/json") {
            console_error!("[TaskExecutor] Failed to set Content-Type header: {}", e);
            return Err(e);
        }
        
        // 2. 合并init和start操作，只调用一次DO
        let mut init = RequestInit::new();
        init.with_method(Method::Post)
            .with_headers(headers)
            .with_body(Some(request_json.into()));
        
        // 使用合并的端点
        // Durable Object 期望的 URL 格式是 http://dummy/path
        let combined_request = match Request::new_with_init("http://dummy/init-and-start", &init) {
            Ok(req) => {
                console_error!("[TaskExecutor] Created combined request");
                req
            }
            Err(e) => {
                console_error!("[TaskExecutor] Failed to create combined request: {}", e);
                return Err(e);
            }
        };
        
        console_error!("[TaskExecutor] Calling DO /init-and-start endpoint");
        
        // 等待DO调用完成，确保任务正确初始化
        match stub.fetch_with_request(combined_request).await {
            Ok(response) => {
                console_error!("[TaskExecutor] DO call succeeded, status: {}", response.status_code());
                if response.status_code() != 200 {
                    console_error!("[TaskExecutor] DO returned non-200 status: {}", response.status_code());
                }
            }
            Err(e) => {
                console_error!("[TaskExecutor] DO call failed: {}", e);
                // 即使DO调用失败，仍然返回任务ID，让用户可以检查状态
            }
        }
        
        console_error!("[TaskExecutor] Returning task_id: {}", task_id);
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
        let ai_api_key = match env.secret("AI_API_KEY") {
            Ok(key) => key.to_string(),
            Err(_) => {
                // 尝试DEEPSEEK_API_KEY作为备选
                match env.secret("DEEPSEEK_API_KEY") {
                    Ok(key) => key.to_string(),
                    Err(_) => {
                        return Ok(CompatibleResponse::error_response(
                            "AI_API_KEY or DEEPSEEK_API_KEY not configured".to_string(),
                        ));
                    }
                }
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
        
        let mut init = RequestInit::new();
        init.with_method(Method::Post);
        
        let cancel_request = Request::new_with_init("http://dummy/cancel", &init)?;
        
        let response = stub
            .fetch_with_request(cancel_request)
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
        
        let tool_result_json = match serde_json::to_string(&tool_result) {
            Ok(json) => json,
            Err(e) => {
                return Err(worker::Error::RustError(
                    format!("Failed to serialize tool result: {}", e)
                ));
            }
        };
        
        let mut init = RequestInit::new();
        init.with_method(Method::Post)
            .with_body(Some(tool_result_json.into()));
        
        let tool_request = Request::new_with_init("http://dummy/tool-result", &init)?;
        
        let response = stub
            .fetch_with_request(tool_request)
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
