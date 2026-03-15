//! AgentRuntime Manager - 统一入口
//!
//! 提供 AgentRuntime 的统一管理和访问接口

use std::sync::Arc;
use serde_json::Value;
use tokio::sync::RwLock;
use crate::agent_runtime::{
    AgentRuntime,
    AgentRuntimeState,
    agent_registry::{AgentInfo, AgentConfig},
    message_bus::{Event, GroupMessage},
    tool_bus::ToolBus,
};
use crate::tools::{ToolRegistry, ToolFacade};

/// AgentRuntime Manager
pub struct AgentRuntimeManager {
    runtime: Arc<AgentRuntime>,
    tool_facade: Arc<ToolFacade>,
}

impl AgentRuntimeManager {
    /// 创建新的 Manager
    pub async fn new(
        tool_registry: Arc<ToolRegistry>,
        tool_bus: Arc<ToolBus>,
    ) -> Result<Self, String> {
        // 创建统一工具入口
        let tool_facade = Arc::new(ToolFacade::new(tool_registry, tool_bus));
        
        // 创建 AgentRuntime
        let runtime = Arc::new(AgentRuntime::new().await?);
        
        Ok(Self {
            runtime,
            tool_facade,
        })
    }

    /// 获取 AgentRuntime 状态
    pub fn state(&self) -> &AgentRuntimeState {
        &self.runtime.state
    }

    /// 获取工具门面（用于 RalphLoop 执行器）
    pub fn tool_facade(&self) -> Arc<ToolFacade> {
        self.tool_facade.clone()
    }

    /// 创建 Agent
    pub async fn create_agent(
        &self,
        agent_info: AgentInfo,
        agent_config: AgentConfig,
    ) -> Result<String, String> {
        log::info!("创建 Agent: {} ({})", agent_info.name, agent_info.id);

        // 注册 Agent 到 Supervisor
        self.runtime.state.agent_supervisor.spawn(agent_info.clone(), agent_config.clone()).await;

        log::info!("Agent 创建成功：{}", agent_info.id);
        Ok(agent_info.id.clone())
    }

    /// 发送消息到 Agent
    pub async fn send_to_agent(
        &self,
        agent_id: &str,
        message: GroupMessage,
        history: Vec<GroupMessage>,
    ) -> Result<(), String> {
        log::info!("发送消息到 Agent: {} - {}", agent_id, message.content.chars().take(50).collect::<String>());

        // 通过 Supervisor 发送
        self.runtime.state.agent_supervisor.send_to_agent(agent_id, message, history).await
    }

    /// 广播消息到 MessageBus
    pub async fn broadcast(&self, message: GroupMessage) -> Result<(), String> {
        log::info!("广播消息到 MessageBus");

        // 发布到 MessageBus
        self.runtime.state.message_bus.publish(Event::GroupMessage(message)).await;
        Ok(())
    }

    /// 列出所有活跃 Agent
    pub async fn list_agents(&self) -> Vec<String> {
        self.runtime.state.agent_supervisor.list_active_agents().await
    }

    /// 停止 Agent
    pub async fn stop_agent(&self, agent_id: &str) -> Result<(), String> {
        log::info!("停止 Agent: {}", agent_id);
        self.runtime.state.agent_supervisor.stop(agent_id).await
    }

    /// 获取工具列表（统一入口）
    pub async fn list_tools(&self) -> Vec<crate::tools::ToolInfo> {
        self.tool_facade.list_tools().await
    }

    /// 执行工具（统一入口）
    pub async fn execute_tool(
        &self,
        name: &str,
        args: Value,
        context: crate::tools::ExecutionContext,
    ) -> Result<Value, String> {
        self.tool_facade.execute(name, args, context).await
    }

    /// 获取 Manager 状态
    pub async fn get_status(&self) -> AgentRuntimeStatus {
        let active_agents = self.list_agents().await.len();
        let tool_count = self.list_tools().await.len();

        AgentRuntimeStatus {
            active_agents,
            tool_count,
            message_bus_subscribers: self.runtime.state.message_bus.subscriber_count().await,
        }
    }
}

/// AgentRuntime 状态
#[derive(Debug, Clone, serde::Serialize)]
pub struct AgentRuntimeStatus {
    pub active_agents: usize,
    pub tool_count: usize,
    pub message_bus_subscribers: usize,
}
