//! 自主执行器 Tauri命令工具
//!
//! 提供AI自主执行能力的接口

use crate::tools::autonomous_executor::{
    AutonomousExecutor, AutonomousExecutorConfig, ExecutionResult,
    AutoResponder, AutoRespondConfig, ChatMessage,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

/// 执行计划参数
#[derive(Serialize, Deserialize)]
pub struct ExecutePlanParams {
    plan: serde_json::Value,
    auto_confirm: Option<bool>,
}

/// 执行结果列表
#[derive(Serialize, Deserialize)]
pub struct ExecuteResults {
    pub success: bool,
    pub total_steps: usize,
    pub executed: usize,
    pub results: Vec<ExecutionResult>,
    pub message: String,
}

/// 配置参数
#[derive(Serialize, Deserialize)]
pub struct ExecutorConfigParams {
    enabled: Option<bool>,
    auto_execute: Option<bool>,
    max_concurrent: Option<usize>,
    timeout_seconds: Option<u64>,
    retry_on_failure: Option<bool>,
    max_retries: Option<u32>,
}

/// 响应配置参数
#[derive(Serialize, Deserialize)]
pub struct RespondConfigParams {
    enabled: Option<bool>,
    respond_to_mentions: Option<bool>,
    respond_to_keywords: Option<bool>,
    respond_to_all: Option<bool>,
    keywords: Option<Vec<String>>,
    ignore_users: Option<Vec<String>>,
    cooldown_seconds: Option<u64>,
}

/// 消息参数
#[derive(Serialize, Deserialize)]
pub struct MessageParams {
    id: String,
    topic: String,
    content: String,
    sender: String,
    timestamp: Option<i64>,
    message_type: Option<String>,
}

/// Tauri命令：执行计划
#[tauri::command]
pub async fn autonomous_execute_plan(
    plan: serde_json::Value,
    auto_confirm: Option<bool>,
) -> Result<ExecuteResults, String> {
    let executor = AutonomousExecutor::new();
    let results = executor.execute_plan(&plan, auto_confirm.unwrap_or(false)).await;
    let executed_count = results.iter().filter(|r| r.success).count();
    let total_steps = results.len();
    
    Ok(ExecuteResults {
        success: executed_count == total_steps,
        total_steps,
        executed: executed_count,
        results: results.clone(),
        message: format!("执行了 {} / {} 个步骤", executed_count, total_steps),
    })
}

/// Tauri命令：执行单个工具
#[tauri::command]
pub async fn autonomous_execute_tool(
    tool_id: String,
    params: serde_json::Value,
) -> Result<ExecutionResult, String> {
    let executor = AutonomousExecutor::new();
    let step = json!({
        "tool": tool_id,
        "params": params,
        "action": format!("执行 {} 工具", tool_id),
        "confidence": 1.0
    });
    let result = executor.execute_plan(&json!({"plan": [step]}), true).await;
    
    if let Some(exec_result) = result.first() {
        Ok(exec_result.clone())
    } else {
        Err("执行失败".to_string())
    }
}

/// Tauri命令：获取执行历史
#[tauri::command]
pub async fn autonomous_get_history() -> Result<Vec<ExecutionResult>, String> {
    let executor = AutonomousExecutor::new();
    let history = executor.get_history().await;
    Ok(history)
}

/// Tauri命令：清除执行历史
#[tauri::command]
pub async fn clear_execution_history() -> Result<bool, String> {
    let executor = AutonomousExecutor::new();
    executor.clear_history().await;
    Ok(true)
}

/// Tauri命令：更新执行器配置
#[tauri::command]
pub async fn update_executor_config(
    enabled: Option<bool>,
    auto_execute: Option<bool>,
    max_concurrent: Option<usize>,
    timeout_seconds: Option<u64>,
    retry_on_failure: Option<bool>,
    max_retries: Option<u32>,
) -> Result<serde_json::Value, String> {
    let config = AutonomousExecutorConfig {
        enabled: enabled.unwrap_or(true),
        auto_execute: auto_execute.unwrap_or(false),
        max_concurrent: max_concurrent.unwrap_or(3),
        timeout_seconds: timeout_seconds.unwrap_or(60),
        retry_on_failure: retry_on_failure.unwrap_or(true),
        max_retries: max_retries.unwrap_or(3),
    };
    
    Ok(json!({
        "success": true,
        "config": {
            "enabled": config.enabled,
            "auto_execute": config.auto_execute,
            "max_concurrent": config.max_concurrent,
            "timeout_seconds": config.timeout_seconds,
            "retry_on_failure": config.retry_on_failure,
            "max_retries": config.max_retries,
        },
        "message": "配置更新成功"
    }))
}

/// Tauri命令：获取执行器配置
#[tauri::command]
pub async fn get_executor_config() -> Result<serde_json::Value, String> {
    let config = AutonomousExecutorConfig::default();
    
    Ok(json!({
        "success": true,
        "config": {
            "enabled": config.enabled,
            "auto_execute": config.auto_execute,
            "max_concurrent": config.max_concurrent,
            "timeout_seconds": config.timeout_seconds,
            "retry_on_failure": config.retry_on_failure,
            "max_retries": config.max_retries,
        }
    }))
}

/// Tauri命令：处理群聊消息
#[tauri::command]
pub async fn handle_chat_message(
    id: String,
    topic: String,
    content: String,
    sender: String,
    timestamp: Option<i64>,
    message_type: Option<String>,
) -> Result<serde_json::Value, String> {
    let responder = AutoResponder::new();
    
    let message = ChatMessage {
        id,
        topic,
        content,
        sender,
        timestamp: timestamp.unwrap_or(chrono::Utc::now().timestamp()),
        message_type: message_type.unwrap_or("chat".to_string()),
    };
    
    match responder.handle_message(&message).await {
        Some(response) => Ok(json!({
            "success": true,
            "responded": true,
            "response": response,
        })),
        None => Ok(json!({
            "success": true,
            "responded": false,
            "reason": "不需要响应",
        })),
    }
}

/// Tauri命令：更新响应配置
#[tauri::command]
pub async fn update_respond_config(
    enabled: Option<bool>,
    respond_to_mentions: Option<bool>,
    respond_to_keywords: Option<bool>,
    respond_to_all: Option<bool>,
    keywords: Option<Vec<String>>,
    ignore_users: Option<Vec<String>>,
    cooldown_seconds: Option<u64>,
) -> Result<serde_json::Value, String> {
    let mut responder = AutoResponder::new();
    
    let config = AutoRespondConfig {
        enabled: enabled.unwrap_or(false),
        respond_to_mentions: respond_to_mentions.unwrap_or(true),
        respond_to_keywords: respond_to_keywords.unwrap_or(true),
        respond_to_all: respond_to_all.unwrap_or(false),
        keywords: keywords.unwrap_or(vec![
            "alou".to_string(),
            "智能体".to_string(),
            "agent".to_string(),
        ]),
        ignore_users: ignore_users.unwrap_or(Vec::new()),
        cooldown_seconds: cooldown_seconds.unwrap_or(5),
    };
    
    responder.configure(config);
    
    Ok(json!({
        "success": true,
        "message": "响应配置更新成功"
    }))
}

/// Tauri命令：获取响应配置
#[tauri::command]
pub async fn get_respond_config() -> Result<serde_json::Value, String> {
    let responder = AutoResponder::new();
    let config = responder.get_config();
    
    Ok(json!({
        "success": true,
        "config": {
            "enabled": config.enabled,
            "respond_to_mentions": config.respond_to_mentions,
            "respond_to_keywords": config.respond_to_keywords,
            "respond_to_all": config.respond_to_all,
            "keywords": config.keywords,
            "ignore_users": config.ignore_users,
            "cooldown_seconds": config.cooldown_seconds,
        }
    }))
}

/// 初始化自主执行器工具
pub async fn initialize_autonomous_executor_tool() -> Result<Arc<Mutex<AutonomousExecutor>>, Box<dyn std::error::Error>> {
    Ok(Arc::new(Mutex::new(AutonomousExecutor::new())))
}
