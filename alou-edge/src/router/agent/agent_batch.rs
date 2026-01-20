use crate::agent::batch_create::{
    AgentCreationResult, AgentStatus, BatchCreateAgentRequest, BatchCreateTask, TaskStatus,
};
use crate::agent::batch_processor::BatchProcessor;
use crate::agent::content_generator::ContentGenerator;
use crate::agent::session::SessionManager;
use crate::agent::ai_client::AiClient;
use crate::storage::kv::KvStore;
use serde_json::json;
use worker::*;

use crate::router::{json_response, json_response_with_status, ErrorResponse};

/// 批量创建智能体
pub(crate) async fn handle_batch_create_agent(
    _session_manager: &SessionManager,
    req: &mut Request,
    env: &Env,
) -> Result<Response> {
    let body: BatchCreateAgentRequest = match req.json().await {
        Ok(body) => body,
        Err(e) => {
            return json_response_with_status(
                &ErrorResponse {
                    error: format!("Invalid request body: {}", e),
                },
                400,
            );
        }
    };

    // 验证批量大小限制
    if body.agents.len() > 10 {
        return json_response_with_status(
            &ErrorResponse {
                error: format!("Batch size exceeds limit: {} (max 10)", body.agents.len()),
            },
            400,
        );
    }

    if body.agents.is_empty() {
        return json_response_with_status(
            &ErrorResponse {
                error: "Agents list cannot be empty".to_string(),
            },
            400,
        );
    }

    let task_id = uuid::Uuid::new_v4().to_string();
    let now = crate::utils::time::now_timestamp();

    // 创建任务记录
    let task = BatchCreateTask {
        task_id: task_id.clone(),
        status: TaskStatus::Pending,
        agents: body
            .agents
            .iter()
            .map(|spec| AgentCreationResult {
                agent_id: None,
                session_id: None,
                name: spec.name.clone(),
                role_description: spec.role_description.clone(),
                avatar_cid: spec.avatar_cid.clone(),
                mcp_config_cid: spec.mcp_config_cid.clone(),
                mcp_ports: spec.mcp_ports.clone(),
                status: AgentStatus::Pending,
                error: None,
            })
            .collect(),
        generation_config: body.generation_config.clone(),
        agent_specs: body.agents.clone(),
        created_at: now,
        updated_at: now,
    };

    // 保存任务到KV
    let kv_store = KvStore::new(env.kv("SESSIONS")?);
    let task_key = format!("batch_task:{}", task_id);
    kv_store
        .put(&task_key, &task, Some(86400 * 7))
        .await
        .map_err(|e| {
            worker::Error::RustError(format!("Failed to save task: {}", e))
        })?;

    // TODO: 队列功能暂时禁用，等待后续实现
    // let queue = env.queue("AGENT_CREATION_QUEUE")?;
    // queue
    //     .send(&task, QueueOptions::default())
    //     .await
    //     .map_err(|e| {
    //         worker::Error::RustError(format!("Failed to send task to queue: {}", e))
    //     })?;

    Ok(json_response(&json!({
        "success": true,
        "task_id": task_id,
        "status": "queued",
        "agents_count": body.agents.len(),
    }))?)
}

/// 查询批量创建任务状态
pub(crate) async fn handle_get_batch_create_task(
    _session_manager: &SessionManager,
    req: &mut Request,
    env: &Env,
) -> Result<Response> {
    // 从路径中提取task_id
    let path = req.path();
    let task_id = path
        .split('/')
        .last()
        .ok_or_else(|| {
            worker::Error::RustError("Invalid path: task_id not found".to_string())
        })?;

    // 初始化服务以获取任务
    let sessions_kv = env.kv("SESSIONS")?;
    let kv_store = KvStore::new(sessions_kv.clone());
    
    // 创建临时SessionManager和ContentGenerator（仅用于获取任务）
    let session_manager = SessionManager::new(kv_store.clone());
    
    // 创建AI客户端（用于ContentGenerator，但这里不需要）
    let ai_provider = env
        .var("AI_PROVIDER")
        .map(|v| v.to_string())
        .unwrap_or_else(|_| "deepseek".to_string());
    let api_key = env
        .secret("AI_API_KEY")
        .map(|s| s.to_string())
        .unwrap_or_else(|_| String::new());
    
    let ai_client = match AiClient::new(&ai_provider, api_key, None) {
        Ok(client) => client,
        Err(_) => {
            // 如果AI客户端创建失败，仍然可以查询任务状态
            // 创建一个空的processor只用于查询
            return match kv_store.get::<BatchCreateTask>(&format!("batch_task:{}", task_id)).await {
                Ok(Some(task)) => json_response(&task),
                Ok(None) => json_response_with_status(
                    &ErrorResponse {
                        error: "Task not found".to_string(),
                    },
                    404,
                ),
                Err(e) => json_response_with_status(
                    &ErrorResponse {
                        error: format!("Failed to get task: {}", e),
                    },
                    500,
                ),
            };
        }
    };
    
    let content_generator = ContentGenerator::new(ai_client);
    let processor = BatchProcessor::new(session_manager, content_generator, kv_store);
    
    match processor.get_task(task_id, env).await {
        Ok(Some(task)) => json_response(&task),
        Ok(None) => json_response_with_status(
            &ErrorResponse {
                error: "Task not found".to_string(),
            },
            404,
        ),
        Err(e) => json_response_with_status(
            &ErrorResponse {
                error: format!("Failed to get task: {}", e),
            },
            500,
        ),
    }
}

/// 获取批量创建的智能体会话ID列表
/// 
/// 这个端点返回批量创建任务中所有已创建的智能体的会话ID
/// 前端可以使用这些会话ID自动将智能体添加到频道
pub(crate) async fn handle_get_batch_agent_sessions(
    _session_manager: &SessionManager,
    req: &mut Request,
    env: &Env,
) -> Result<Response> {
    // 从路径中提取task_id
    let path = req.path();
    let task_id = path
        .split('/')
        .last()
        .ok_or_else(|| {
            worker::Error::RustError("Invalid path: task_id not found".to_string())
        })?;

    // 初始化服务
    let sessions_kv = env.kv("SESSIONS")?;
    let kv_store = KvStore::new(sessions_kv.clone());
    let session_manager = SessionManager::new(kv_store.clone());
    
    // 创建AI客户端（用于ContentGenerator）
    let ai_provider = env
        .var("AI_PROVIDER")
        .map(|v| v.to_string())
        .unwrap_or_else(|_| "deepseek".to_string());
    let api_key = env
        .secret("AI_API_KEY")
        .map(|s| s.to_string())
        .unwrap_or_else(|_| String::new());
    
    let ai_client = match AiClient::new(&ai_provider, api_key, None) {
        Ok(client) => client,
        Err(_) => {
            return json_response_with_status(
                &ErrorResponse {
                    error: "Failed to create AI client".to_string(),
                },
                500,
            );
        }
    };
    
    let content_generator = ContentGenerator::new(ai_client);
    let processor = BatchProcessor::new(session_manager, content_generator, kv_store);
    
    // 获取批量任务中所有已创建的智能体会话ID
    match processor.get_batch_agent_sessions(task_id, env).await {
        Ok(session_ids) => {
            Ok(json_response(&json!({
                "success": true,
                "task_id": task_id,
                "session_ids": session_ids,
                "count": session_ids.len(),
            }))?)
        }
        Err(e) => json_response_with_status(
            &ErrorResponse {
                error: format!("Failed to get agent sessions: {}", e),
            },
            500,
        ),
    }
}