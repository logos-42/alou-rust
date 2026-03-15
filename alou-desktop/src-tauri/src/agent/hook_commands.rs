//! Agent Hook Tauri 命令
//!
//! 提供前端与 Agent Hook 系统交互的 Tauri 命令接口

// use crate::agent::agent_hook::{  // 模块不存在，暂时注释
//     AgentHookManager, AgentHookFactory, AgentHookConfig,
//     UserInstruction, InstructionType, InstructionPriority,
//     AgentExecutionStatus,
// };
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 全局 Hook 管理器注册表
pub struct HookRegistry {
    hooks: RwLock<HashMap<String, Arc<AgentHookManager>>>,
}

impl HookRegistry {
    pub fn new() -> Self {
        Self {
            hooks: RwLock::new(HashMap::new()),
        }
    }

    pub async fn get_or_create(
        &self,
        agent_id: &str,
        session_id: &str,
    ) -> Arc<AgentHookManager> {
        let hooks = self.hooks.read().await;
        let key = format!("{}:{}", agent_id, session_id);
        
        if let Some(hook) = hooks.get(&key) {
            return hook.clone();
        }
        drop(hooks);

        // 创建新的 Hook 管理器
        let mut hooks = self.hooks.write().await;
        let hook = AgentHookFactory::create(
            agent_id.to_string(),
            session_id.to_string(),
        );
        hooks.insert(key.clone(), hook.clone());
        hook
    }

    pub async fn get(&self, agent_id: &str, session_id: &str) -> Option<Arc<AgentHookManager>> {
        let hooks = self.hooks.read().await;
        let key = format!("{}:{}", agent_id, session_id);
        hooks.get(&key).cloned()
    }
}

// 全局 Hook 注册表（使用 lazy_static 或 once_cell）
use once_cell::sync::Lazy;

static HOOK_REGISTRY: Lazy<HookRegistry> = Lazy::new(HookRegistry::new);

/// 指令优先级（用于 Tauri 命令）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CmdInstructionPriority {
    Low,
    Medium,
    High,
    Critical,
}

impl Default for CmdInstructionPriority {
    fn default() -> Self {
        CmdInstructionPriority::Medium
    }
}

impl From<CmdInstructionPriority> for InstructionPriority {
    fn from(cmd: CmdInstructionPriority) -> Self {
        match cmd {
            CmdInstructionPriority::Low => InstructionPriority::Low,
            CmdInstructionPriority::Medium => InstructionPriority::Medium,
            CmdInstructionPriority::High => InstructionPriority::High,
            CmdInstructionPriority::Critical => InstructionPriority::Critical,
        }
    }
}

/// 注入指令请求
#[derive(Debug, Deserialize)]
pub struct InjectInstructionRequest {
    pub agent_id: String,
    pub session_id: String,
    pub content: String,
    #[serde(default)]
    pub priority: CmdInstructionPriority,
    #[serde(default)]
    pub instruction_type: Option<String>,
}

/// 注入指令响应
#[derive(Debug, Serialize)]
pub struct InjectInstructionResponse {
    pub success: bool,
    pub instruction_id: Option<String>,
    pub message: String,
}

/// 暂停 Agent 请求
#[derive(Debug, Deserialize)]
pub struct PauseAgentRequest {
    pub agent_id: String,
    pub session_id: String,
    pub reason: String,
}

/// 恢复 Agent 请求
#[derive(Debug, Deserialize)]
pub struct ResumeAgentRequest {
    pub agent_id: String,
    pub session_id: String,
}

/// 取消 Agent 请求
#[derive(Debug, Deserialize)]
pub struct CancelAgentRequest {
    pub agent_id: String,
    pub session_id: String,
    pub reason: String,
}

/// 获取 Agent 状态请求
#[derive(Debug, Deserialize)]
pub struct GetAgentStatusRequest {
    pub agent_id: String,
    pub session_id: String,
}

/// Agent 状态响应
#[derive(Debug, Serialize)]
pub struct AgentStatusResponse {
    pub success: bool,
    pub status: String,
    pub current_task: Option<String>,
    pub progress: f64,
    pub pending_instructions: usize,
    pub processed_instructions: usize,
    pub snapshot_time: i64,
}

/// 获取指令历史请求
#[derive(Debug, Deserialize)]
pub struct GetInstructionHistoryRequest {
    pub agent_id: String,
    pub session_id: String,
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize {
    20
}

/// 指令历史响应
#[derive(Debug, Serialize)]
pub struct InstructionHistoryResponse {
    pub success: bool,
    pub instructions: Vec<InstructionInfo>,
    pub count: usize,
}

#[derive(Debug, Serialize)]
pub struct InstructionInfo {
    pub id: String,
    pub content: String,
    pub instruction_type: String,
    pub priority: String,
    pub created_at: i64,
    pub processed: bool,
    pub result: Option<String>,
}

/// 清除指令队列请求
#[derive(Debug, Deserialize)]
pub struct ClearQueueRequest {
    pub agent_id: String,
    pub session_id: String,
}

// ============ Tauri 命令实现 ============

/// 注入指令到 Agent
#[tauri::command]
pub async fn agent_hook_inject(
    req: InjectInstructionRequest,
) -> Result<InjectInstructionResponse, String> {
    let hook = HOOK_REGISTRY
        .get_or_create(&req.agent_id, &req.session_id)
        .await;

    let instruction_type = match req.instruction_type.as_deref() {
        Some("modify") => InstructionType::Modify,
        Some("pause") => InstructionType::Pause,
        Some("resume") => InstructionType::Resume,
        Some("cancel") => InstructionType::Cancel,
        Some("reset") => InstructionType::Reset,
        Some("query_status") => InstructionType::QueryStatus,
        _ => InstructionType::Normal,
    };

    let instruction = UserInstruction::new(
        req.session_id.clone(),
        req.content.clone(),
        instruction_type,
    ).with_priority(req.priority.into());

    match hook.submit_instruction(instruction).await {
        Ok(instruction_id) => Ok(InjectInstructionResponse {
            success: true,
            instruction_id: Some(instruction_id),
            message: "Instruction injected successfully".to_string(),
        }),
        Err(e) => Ok(InjectInstructionResponse {
            success: false,
            instruction_id: None,
            message: e,
        }),
    }
}

/// 注入高优先级指令
#[tauri::command]
pub async fn agent_hook_inject_high_priority(
    agent_id: String,
    session_id: String,
    content: String,
) -> Result<InjectInstructionResponse, String> {
    let hook = HOOK_REGISTRY
        .get_or_create(&agent_id, &session_id)
        .await;

    match hook.inject_high_priority(content).await {
        Ok(instruction_id) => Ok(InjectInstructionResponse {
            success: true,
            instruction_id: Some(instruction_id),
            message: "High priority instruction injected successfully".to_string(),
        }),
        Err(e) => Ok(InjectInstructionResponse {
            success: false,
            instruction_id: None,
            message: e,
        }),
    }
}

/// 暂停 Agent 执行
#[tauri::command]
pub async fn agent_hook_pause(
    req: PauseAgentRequest,
) -> Result<InjectInstructionResponse, String> {
    let hook = HOOK_REGISTRY
        .get_or_create(&req.agent_id, &req.session_id)
        .await;

    match hook.pause(req.reason).await {
        Ok(instruction_id) => Ok(InjectInstructionResponse {
            success: true,
            instruction_id: Some(instruction_id),
            message: "Agent paused successfully".to_string(),
        }),
        Err(e) => Ok(InjectInstructionResponse {
            success: false,
            instruction_id: None,
            message: e,
        }),
    }
}

/// 恢复 Agent 执行
#[tauri::command]
pub async fn agent_hook_resume(
    req: ResumeAgentRequest,
) -> Result<InjectInstructionResponse, String> {
    let hook = HOOK_REGISTRY
        .get_or_create(&req.agent_id, &req.session_id)
        .await;

    match hook.resume().await {
        Ok(instruction_id) => Ok(InjectInstructionResponse {
            success: true,
            instruction_id: Some(instruction_id),
            message: "Agent resumed successfully".to_string(),
        }),
        Err(e) => Ok(InjectInstructionResponse {
            success: false,
            instruction_id: None,
            message: e,
        }),
    }
}

/// 取消 Agent 执行
#[tauri::command]
pub async fn agent_hook_cancel(
    req: CancelAgentRequest,
) -> Result<InjectInstructionResponse, String> {
    let hook = HOOK_REGISTRY
        .get_or_create(&req.agent_id, &req.session_id)
        .await;

    match hook.cancel(req.reason).await {
        Ok(instruction_id) => Ok(InjectInstructionResponse {
            success: true,
            instruction_id: Some(instruction_id),
            message: "Agent cancelled successfully".to_string(),
        }),
        Err(e) => Ok(InjectInstructionResponse {
            success: false,
            instruction_id: None,
            message: e,
        }),
    }
}

/// 获取 Agent 状态
#[tauri::command]
pub async fn agent_hook_get_status(
    req: GetAgentStatusRequest,
) -> Result<AgentStatusResponse, String> {
    let hook = HOOK_REGISTRY
        .get_or_create(&req.agent_id, &req.session_id)
        .await;

    let snapshot = hook.get_snapshot().await;

    let status_str = match &snapshot.execution_status {
        AgentExecutionStatus::Idle => "idle".to_string(),
        AgentExecutionStatus::Executing { .. } => "executing".to_string(),
        AgentExecutionStatus::Paused { .. } => "paused".to_string(),
        AgentExecutionStatus::Completed { .. } => "completed".to_string(),
        AgentExecutionStatus::Cancelled { .. } => "cancelled".to_string(),
        AgentExecutionStatus::Failed { .. } => "failed".to_string(),
    };

    Ok(AgentStatusResponse {
        success: true,
        status: status_str,
        current_task: snapshot.current_task,
        progress: snapshot.progress,
        pending_instructions: snapshot.pending_instructions,
        processed_instructions: snapshot.processed_instructions,
        snapshot_time: snapshot.snapshot_time,
    })
}

/// 获取指令历史
#[tauri::command]
pub async fn agent_hook_get_history(
    req: GetInstructionHistoryRequest,
) -> Result<InstructionHistoryResponse, String> {
    let hook = HOOK_REGISTRY
        .get_or_create(&req.agent_id, &req.session_id)
        .await;

    let history = hook.get_instruction_history(req.limit).await;

    let instructions: Vec<InstructionInfo> = history
        .into_iter()
        .map(|instr| InstructionInfo {
            id: instr.id,
            content: instr.content,
            instruction_type: format!("{:?}", instr.instruction_type).to_lowercase(),
            priority: format!("{:?}", instr.priority).to_lowercase(),
            created_at: instr.created_at,
            processed: instr.processed,
            result: instr.result,
        })
        .collect();

    let count = instructions.len();

    Ok(InstructionHistoryResponse {
        success: true,
        instructions,
        count,
    })
}

/// 获取待处理指令
#[tauri::command]
pub async fn agent_hook_get_pending(
    agent_id: String,
    session_id: String,
) -> Result<InstructionHistoryResponse, String> {
    let hook = HOOK_REGISTRY
        .get_or_create(&agent_id, &session_id)
        .await;

    let pending = hook.get_pending_instructions().await;

    let instructions: Vec<InstructionInfo> = pending
        .into_iter()
        .map(|instr| InstructionInfo {
            id: instr.id,
            content: instr.content,
            instruction_type: format!("{:?}", instr.instruction_type).to_lowercase(),
            priority: format!("{:?}", instr.priority).to_lowercase(),
            created_at: instr.created_at,
            processed: instr.processed,
            result: instr.result,
        })
        .collect();

    let count = instructions.len();

    Ok(InstructionHistoryResponse {
        success: true,
        instructions,
        count,
    })
}

/// 清除指令队列
#[tauri::command]
pub async fn agent_hook_clear_queue(
    req: ClearQueueRequest,
) -> Result<serde_json::Value, String> {
    let hook = HOOK_REGISTRY
        .get_or_create(&req.agent_id, &req.session_id)
        .await;

    hook.clear_queue().await;

    Ok(serde_json::json!({
        "success": true,
        "message": "Queue cleared successfully"
    }))
}

/// 创建自定义 Hook 管理器
#[tauri::command]
pub async fn agent_hook_create(
    agent_id: String,
    session_id: String,
    config: Option<serde_json::Value>,
) -> Result<serde_json::Value, String> {
    let hook_config = if let Some(config_json) = config {
        serde_json::from_value::<AgentHookConfig>(config_json).unwrap_or_default()
    } else {
        AgentHookConfig::default()
    };

    let hook = AgentHookFactory::create_with_config(
        agent_id.clone(),
        session_id.clone(),
        hook_config,
    );

    // 注册到全局注册表
    let key = format!("{}:{}", agent_id, session_id);
    {
        let mut hooks = HOOK_REGISTRY.hooks.write().await;
        hooks.insert(key.clone(), hook);
    }

    Ok(serde_json::json!({
        "success": true,
        "key": key,
        "message": "Hook manager created successfully"
    }))
}

/// 注册所有 Agent Hook 命令到 Tauri 应用
pub fn register_commands<R: tauri::Runtime>(builder: tauri::Builder<R>) -> tauri::Builder<R> {
    builder
        .invoke_handler(tauri::generate_handler![
            agent_hook_inject,
            agent_hook_inject_high_priority,
            agent_hook_pause,
            agent_hook_resume,
            agent_hook_cancel,
            agent_hook_get_status,
            agent_hook_get_history,
            agent_hook_get_pending,
            agent_hook_clear_queue,
            agent_hook_create,
        ])
}
