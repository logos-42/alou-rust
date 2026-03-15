//! Agent Runtime - 本地 Agent 运行时
//! 
//! 包含：
//! - MessageBus (不丢消息)
//! - EventRouter (事件路由)
//! - AgentRegistry (原子更新)
//! - Agent Actor (顺序处理)
//! - Agent Supervisor (崩溃恢复)
//! - Agent Router (三种路由)
//! - JS Runtime Pool (沙箱)
//! - Tool Bus (工具总线)
//! - Task Queue (全局任务队列)
//! - Storage (SQLite)

pub mod message_bus;
pub mod event_router;
pub mod agent_registry;
pub mod agent_actor;
pub mod agent_supervisor;
pub mod agent_router;
pub mod js_runtime;
pub mod tool_bus;
pub mod task_queue;
pub mod storage;
pub mod media_tools;

pub use message_bus::*;
pub use event_router::*;
pub use agent_registry::*;
pub use agent_actor::*;
pub use agent_supervisor::*;
pub use agent_router::*;
pub use js_runtime::*;
pub use tool_bus::*;
pub use task_queue::*;
pub use storage::*;

use std::sync::Arc;
use tokio::sync::RwLock;
use crate::agent::providers::ProviderRegistry;
use crate::agent::media_config::MediaApiConfig;

/// Agent Runtime 状态
pub struct AgentRuntimeState {
    pub message_bus: MessageBus,
    pub event_router: EventRouter,
    pub agent_registry: AgentRegistry,
    pub agent_router: AgentRouter,
    pub agent_supervisor: AgentSupervisor,
    pub js_runtime_pool: JsRuntimePool,
    pub tool_bus: ToolBus,
    pub task_queue: TaskQueue,
    pub storage: Storage,
    pub provider_registry: Arc<ProviderRegistry>,
}

impl AgentRuntimeState {
    pub async fn new() -> Result<Self, String> {
        let message_bus = MessageBus::with_default_capacity();
        let event_router = EventRouter::new(message_bus.clone()).await;
        let agent_registry = AgentRegistry::new();
        let agent_router = AgentRouter::new(agent_registry.clone());
        let agent_supervisor = AgentSupervisor::new(RestartPolicy::OnFailure(3));
        let js_runtime_pool = JsRuntimePool::new(JsRuntimeConfig::default()).await?;
        let tool_bus = ToolBus::new();
        let task_queue = TaskQueue::new();
        let storage = Storage::new("./alou_runtime.db").await?;

        // 加载媒体配置并创建 Provider Registry
        let media_config = MediaApiConfig::load().unwrap_or_else(|_| MediaApiConfig::default());
        let provider_registry = Arc::new(
            ProviderRegistry::new(&media_config)
                .unwrap_or_else(|e| {
                    log::warn!("ProviderRegistry 创建失败：{}, 使用空配置", e);
                    ProviderRegistry::new(&MediaApiConfig::default()).unwrap()
                })
        );

        // 注册媒体工具
        tool_bus.register_media_tools(provider_registry.clone());

        Ok(Self {
            message_bus,
            event_router,
            agent_registry,
            agent_router,
            agent_supervisor,
            js_runtime_pool,
            tool_bus,
            task_queue,
            storage,
            provider_registry,
        })
    }
}

/// Agent Runtime
pub struct AgentRuntime {
    pub state: Arc<AgentRuntimeState>,
}

impl AgentRuntime {
    pub async fn new() -> Result<Self, String> {
        let state = Arc::new(AgentRuntimeState::new().await?);
        Ok(Self { state })
    }
    
    /// 启动运行时
    pub async fn start(&self) -> Result<(), String> {
        log::info!("Agent Runtime 启动");
        Ok(())
    }
    
    /// 停止运行时
    pub async fn stop(&self) -> Result<(), String> {
        log::info!("Agent Runtime 停止");
        Ok(())
    }
}
