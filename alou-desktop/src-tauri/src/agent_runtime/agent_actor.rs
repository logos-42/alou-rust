//! Agent Actor - Agent Actor（集成 RalphLoop 执行逻辑）
//!
//! 每个 Agent 对应一个独立的 Actor，支持：
//! - 接收群聊消息
//! - 使用 RalphLoop 执行任务
//! - 多 Agent 协作

use std::sync::Arc;
use crate::agent_runtime::agent_registry::{AgentInfo, AgentConfig};
use crate::agent_runtime::message_bus::GroupMessage;
use crate::agent::executor::RalphLoopExecutor;
use crate::agent::task::TaskManager;
use crate::agent::ai_client::{AiClient, AiMessage};
use crate::agent::ai_client_pool::AiClientPool;
use crate::agent::perception::PerceptionEngine;
use crate::tools::ToolFacade;
use crate::bridges::BridgeManager;
use tokio::sync::mpsc;

/// Agent Actor 消息
pub enum ActorMessage {
    /// 处理群聊消息
    HandleMessage {
        message: GroupMessage,
        history: Vec<GroupMessage>,
    },
    /// 执行任务（直接调用 RalphLoop）
    ExecuteTask {
        content: String,
        reply_to: Option<String>,
    },
    /// 停止 actor
    Stop,
}

/// Agent Actor 句柄
pub struct AgentHandle {
    tx: mpsc::Sender<ActorMessage>,
}

impl AgentHandle {
    pub fn new(tx: mpsc::Sender<ActorMessage>) -> Self {
        Self { tx }
    }

    pub async fn send_message(&self, message: GroupMessage, history: Vec<GroupMessage>) -> Result<(), String> {
        self.tx.send(ActorMessage::HandleMessage { message, history }).await
            .map_err(|e| format!("发送消息失败：{}", e))
    }

    pub async fn execute_task(&self, content: String, reply_to: Option<String>) -> Result<(), String> {
        self.tx.send(ActorMessage::ExecuteTask { content, reply_to }).await
            .map_err(|e| format!("发送任务失败：{}", e))
    }

    pub async fn stop(&self) -> Result<(), String> {
        self.tx.send(ActorMessage::Stop).await
            .map_err(|e| format!("发送停止消息失败：{}", e))
    }
}

/// Agent Actor
pub struct AgentActor {
    pub agent: AgentInfo,
    pub config: AgentConfig,
    pub inbox: mpsc::Receiver<ActorMessage>,
    pub executor: Arc<RalphLoopExecutor>,
    pub task_manager: Arc<TaskManager>,
    pub ai_client_pool: Arc<AiClientPool>,
}

impl AgentActor {
    /// 创建新的 Agent Actor（带 RalphLoop 执行器）
    pub async fn new(
        agent: AgentInfo,
        config: AgentConfig,
        inbox: mpsc::Receiver<ActorMessage>,
        tool_facade: Arc<ToolFacade>,
        bridge_manager: Arc<BridgeManager>,
        ai_client_pool: Arc<AiClientPool>,
    ) -> Self {
        // 从 Pool 获取 AI Client（复用）
        // Create a default config for the AI client
        let default_config = crate::agent::config::UserApiConfig {
            id: "default".to_string(),
            provider: "anthropic".to_string(),
            api_key: "".to_string(),
            base_url: None,
            model: Some("claude-3-7-sonnet-20250219".to_string()),
            is_active: true,
        };
        let ai_client = ai_client_pool.get(&default_config).await.unwrap_or_else(|_| {
            Arc::new(crate::agent::ai_client::AiClient::new(&default_config).unwrap())
        });

        // 创建 TaskManager
        let task_manager = Arc::new(TaskManager::new());

        // 创建 ExecutorCore
        let core = crate::agent::executor::core::ExecutorCore::new(
            ai_client,
            task_manager.clone(),
            bridge_manager.tool_bridge(),
            Arc::new(crate::tools::ToolRegistry::new()),
        );

        // 创建 RalphLoopExecutor
        let executor = Arc::new(RalphLoopExecutor::new(core));

        Self {
            agent,
            config,
            inbox,
            executor,
            task_manager,
            ai_client_pool,
        }
    }

    /// 运行 actor（顺序处理所有消息）
    pub async fn run(mut self) -> Result<(), String> {
        log::info!("Agent Actor 启动：{} ({})", self.agent.name, self.agent.id);

        while let Some(msg) = self.inbox.recv().await {
            match msg {
                ActorMessage::HandleMessage { message, history } => {
                    if let Err(e) = self.handle_message(message, history).await {
                        log::error!("Agent 处理消息失败：{} - {}", self.agent.name, e);
                    }
                }
                ActorMessage::ExecuteTask { content, reply_to } => {
                    if let Err(e) = self.execute_task(content, reply_to).await {
                        log::error!("Agent 执行任务失败：{} - {}", self.agent.name, e);
                    }
                }
                ActorMessage::Stop => {
                    log::info!("Agent Actor 停止：{}", self.agent.name);
                    break;
                }
            }
        }

        Ok(())
    }

    /// 处理群聊消息
    async fn handle_message(&self, message: GroupMessage, history: Vec<GroupMessage>) -> Result<(), String> {
        // 检查是否被提及（如果配置了 mention_only）
        if self.config.mention_only && !self.is_mentioned(&message) {
            log::debug!("Agent {} 未被提及，跳过", self.agent.name);
            return Ok(());
        }

        log::info!(
            "Agent {} 处理消息：{} (history: {})",
            self.agent.name,
            message.content.chars().take(50).collect::<String>(),
            history.len()
        );

        // 构建历史消息
        let mut messages: Vec<AiMessage> = Vec::new();

        // 添加 system prompt (using agent name as default since system_prompt field doesn't exist)
        messages.push(AiMessage {
            role: "system".to_string(),
            content: format!("You are {}, a helpful AI assistant.", self.agent.name),
            tool_call_id: None,
            tool_calls: None,
        });

        // 添加历史消息（最近 20 条）
        for msg in history.into_iter().rev().take(20).rev() {
            messages.push(AiMessage {
                role: "user".to_string(),
                content: msg.content,
                tool_call_id: None,
                tool_calls: None,
            });
        }

        // 添加当前消息
        messages.push(AiMessage {
            role: "user".to_string(),
            content: message.content.clone(),
            tool_call_id: None,
            tool_calls: None,
        });

        // 创建任务并执行
        let task_id = self.task_manager.create_task_with_messages(
            self.agent.id.clone(),
            messages,
        ).await;

        log::info!("Agent {} 创建任务：{}", self.agent.name, task_id);

        // 执行 RalphLoop
        match self.executor.execute(&task_id).await {
            Ok(result) => {
                log::info!("Agent {} 任务完成：{} - 结果：{}",
                    self.agent.name, task_id, result.chars().take(100).collect::<String>());

                // TODO: 将结果发布回 MessageBus
                Ok(())
            }
            Err(e) => {
                log::error!("Agent {} 任务失败：{} - {}", self.agent.name, task_id, e);
                Err(format!("任务执行失败：{}", e))
            }
        }
    }

    /// 执行任务（直接调用）
    async fn execute_task(&self, content: String, _reply_to: Option<String>) -> Result<(), String> {
        log::info!("Agent {} 执行任务：{}", self.agent.name, content.chars().take(50).collect::<String>());

        // 创建任务
        let task_id = self.task_manager.create_task(self.agent.id.clone(), content).await;

        // 执行
        match self.executor.execute(&task_id).await {
            Ok(result) => {
                log::info!("Agent {} 任务完成：{}", self.agent.name, task_id);
                Ok(())
            }
            Err(e) => {
                log::error!("Agent {} 任务失败：{} - {}", self.agent.name, task_id, e);
                Err(format!("任务执行失败：{}", e))
            }
        }
    }

    /// 检查是否被提及
    fn is_mentioned(&self, message: &GroupMessage) -> bool {
        message.content.contains(&format!("@{}", self.agent.name))
            || message.content.contains(&format!("@{}", self.agent.display_name))
    }
}

impl Clone for AgentActor {
    fn clone(&self) -> Self {
        Self {
            agent: self.agent.clone(),
            config: self.config.clone(),
            inbox: mpsc::channel(100).1,
            executor: self.executor.clone(),
            task_manager: self.task_manager.clone(),
            ai_client_pool: self.ai_client_pool.clone(),
        }
    }
}
