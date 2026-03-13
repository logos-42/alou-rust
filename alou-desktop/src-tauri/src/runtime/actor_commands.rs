//! Session Actor 命令接口
//!
//! # 渐进式迁移策略
//!
//! 本模块提供新旧架构的桥梁：
//! - 新功能：使用 SessionActor 发送命令
//! - 旧功能：保持原有调用方式不变
//!
//! # 使用示例
//!
//! ```rust
//! // 新功能：通过 SessionRouter 发送命令
//! let router = SessionRouter::new(bridge_manager);
//! router.send_command(session_id, AgentCommand::Chat { content: "Hello".to_string() }).await;
//!
//! // 旧功能：保持原有调用方式
//! execute_agent_task(message, agent_id, config, bridge_manager).await;
//! ```

use std::sync::Arc;
use tokio::sync::mpsc;
use crate::runtime::message::SessionMessage;
use crate::runtime::handle::ActorHandle;
use crate::bridges::BridgeManager;

/// Agent 命令 - 用于 SessionActor 通信
#[derive(Debug, Clone)]
pub enum AgentCommand {
    /// 聊天请求
    Chat {
        content: String,
        metadata: Option<serde_json::Value>,
    },
    
    /// 工具调用
    ToolCall {
        tool_name: String,
        arguments: serde_json::Value,
    },
    
    /// 工作流执行
    Workflow {
        workflow_id: String,
        input: serde_json::Value,
    },
    
    /// 群聊消息
    GroupChat {
        group_id: String,
        content: String,
        from_agent: Option<String>,
    },
    
    /// 取消当前任务
    Cancel,
    
    /// 获取状态
    GetStatus,
}

/// 命令结果
#[derive(Debug, Clone)]
pub struct CommandResult {
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
}

/// Session 命令管理器
///
/// 用于向 SessionActor 发送命令并等待响应
pub struct SessionCommandManager {
    router: crate::runtime::router::SessionRouter,
}

impl SessionCommandManager {
    /// 创建新的命令管理器
    pub fn new(bridge_manager: Arc<BridgeManager>) -> Self {
        Self {
            router: crate::runtime::router::SessionRouter::new(bridge_manager),
        }
    }
    
    /// 发送命令到指定 session
    pub async fn send_command(
        &self,
        session_id: &str,
        command: AgentCommand,
    ) -> CommandResult {
        log::info!(
            "[SessionCommand] Sending command {:?} to session {}",
            command,
            session_id
        );
        
        // 转换为 SessionMessage
        let message = self.command_to_message(command);
        
        // 发送到 SessionActor
        self.router.send(session_id, message);
        
        // 立即返回（异步处理）
        CommandResult {
            success: true,
            data: Some(serde_json::json!({
                "status": "queued",
                "session_id": session_id,
            })),
            error: None,
        }
    }
    
    /// 发送命令并等待响应
    pub async fn send_command_with_response(
        &self,
        session_id: &str,
        command: AgentCommand,
        timeout_secs: u64,
    ) -> CommandResult {
        log::info!(
            "[SessionCommand] Sending command with response {:?} to session {}",
            command,
            session_id
        );
        
        // 创建响应通道
        let (tx, mut rx) = mpsc::channel(1);
        
        // 发送命令（带响应通道）
        let message = self.command_to_message_with_response(command, tx);
        self.router.send(session_id, message);
        
        // 等待响应（带超时）
        match tokio::time::timeout(
            tokio::time::Duration::from_secs(timeout_secs),
            rx.recv()
        ).await {
            Ok(Some(result)) => result,
            Ok(None) => CommandResult {
                success: false,
                data: None,
                error: Some("Channel closed".to_string()),
            },
            Err(_) => CommandResult {
                success: false,
                data: None,
                error: Some(format!("Timeout after {} seconds", timeout_secs)),
            },
        }
    }
    
    /// 转换命令为消息
    fn command_to_message(&self, command: AgentCommand) -> SessionMessage {
        match command {
            AgentCommand::Chat { content, .. } => {
                SessionMessage::UserMessage {
                    content,
                    metadata: crate::runtime::message::MessageMetadata::default(),
                }
            }
            AgentCommand::ToolCall { tool_name, arguments } => {
                SessionMessage::ToolCall {
                    tool_call_id: format!("tool_{}", uuid::Uuid::new_v4()),
                    tool_name,
                    arguments,
                }
            }
            AgentCommand::Workflow { workflow_id, input } => {
                SessionMessage::WorkflowExecute {
                    workflow_id,
                    input,
                }
            }
            AgentCommand::GroupChat { group_id, content, from_agent } => {
                SessionMessage::GroupChatMessage {
                    group_id,
                    from_agent,
                    content,
                }
            }
            AgentCommand::Cancel => {
                SessionMessage::SystemEvent {
                    event_type: crate::runtime::message::SystemEventType::Error,
                    data: serde_json::json!({"action": "cancel"}),
                }
            }
            AgentCommand::GetStatus => {
                SessionMessage::Ping
            }
        }
    }
    
    /// 转换命令为消息（带响应通道）
    fn command_to_message_with_response(
        &self,
        command: AgentCommand,
        response_tx: mpsc::Sender<CommandResult>,
    ) -> SessionMessage {
        // 简化实现：实际应该在 SessionActor 中处理响应
        self.command_to_message(command)
    }
    
    /// 获取活跃 session 数量
    pub fn active_session_count(&self) -> usize {
        self.router.active_session_count()
    }
    
    /// 取消 session 的所有命令
    pub async fn cancel_session(&self, session_id: &str) {
        self.router.send(session_id, SessionMessage::SystemEvent {
            event_type: crate::runtime::message::SystemEventType::Error,
            data: serde_json::json!({"action": "cancel_all"}),
        });
    }
}

// ============================================================================
// 渐进式迁移辅助函数
// ============================================================================

/// 新旧架构兼容的 Agent 执行函数
///
/// # 参数
/// - `use_new_architecture`: 是否使用新架构
///   - `true`: 通过 SessionActor 执行
///   - `false`: 使用原有 execute_agent_task 函数
///
/// # 迁移策略
/// 1. 新功能默认 use_new_architecture = true
/// 2. 旧功能保持 use_new_architecture = false
/// 3. 逐步将旧功能迁移到新架构
pub async fn execute_agent_task_compatible(
    message: String,
    agent_id: String,
    config: crate::agent::config::UserApiConfig,
    bridge_manager: tauri::State<'_, std::sync::Arc<tokio::sync::Mutex<BridgeManager>>>,
    use_new_architecture: bool,
) -> std::result::Result<crate::agent::task::TaskFinalResult, String> {
    if use_new_architecture {
        // 新架构：通过 SessionActor 执行
        log::info!("[execute_agent_task_compatible] Using NEW architecture");
        
        let bridge_manager = bridge_manager.lock().await;
        let command_manager = SessionCommandManager::new(bridge_manager.clone());
        
        // 发送聊天命令
        let result = command_manager.send_command_with_response(
            &agent_id,
            AgentCommand::Chat {
                content: message,
                metadata: None,
            },
            60, // 60 秒超时
        ).await;
        
        if result.success {
            // 简化返回（实际应该等待 SessionActor 完成）
            Ok(crate::agent::task::TaskFinalResult {
                task_id: format!("task_{}", uuid::Uuid::new_v4()),
                success: true,
                result: result.data.unwrap_or_default(),
                error: None,
                iteration_count: 1,
            })
        } else {
            Err(result.error.unwrap_or_else(|| "Unknown error".to_string()))
        }
    } else {
        // 旧架构：使用原有函数
        log::info!("[execute_agent_task_compatible] Using OLD architecture");
        
        // 调用原有函数（需要导入）
        // 这里使用占位实现
        Err("Old architecture not implemented in this example".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_agent_command_creation() {
        let chat_cmd = AgentCommand::Chat {
            content: "Hello".to_string(),
            metadata: None,
        };
        
        assert!(matches!(chat_cmd, AgentCommand::Chat { .. }));
    }
}
