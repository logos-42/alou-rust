//! Agent creation functionality
//! 智能体创建功能

use crate::router::agent::{CreateClaudeAgentRequest, CreateClaudeAgentResult};
use crate::session::SessionManager;
use crate::utils::error::Result;
use serde_json::Value;
use worker::Env;

/// Create Claude agent from parsed information
/// 从解析的信息创建Claude智能体
pub async fn create_claude_agent_from_parsed(
    session_manager: &SessionManager,
    _env: &Env,
    create_request: Value,
) -> Result<CreateClaudeAgentResult> {
    // 将JSON请求转换为结构体
    let request: CreateClaudeAgentRequest = serde_json::from_value(create_request)
        .map_err(|e| format!("Invalid create request: {}", e))?;

    // 调用现有的创建逻辑
    super::router::agent::handle_create_claude_agent_internal(session_manager, request).await
}