//! Agent Runtime - 本地 Agent 运行时
//!
//! 包含：
//! - MessageBus (不丢消息)
//! - EventRouter (事件路由)
//! - AgentRegistry (原子更新)
//! - Agent Actor (顺序处理，集成 RalphLoop)
//! - Agent Supervisor (崩溃恢复)
//! - Agent Router (三种路由)
//! - JS Runtime Pool (沙箱)
//! - Tool Bus (工具总线)
//! - Task Queue (全局任务队列)
//! - Storage (SQLite)
//! - Manager (统一管理入口)

pub mod message_bus;
pub mod event_router;
pub mod agent_registry;
pub mod agent_actor;
pub mod agent_supervisor;
pub mod agent_router;
pub mod agent_scheduler;  // Agent 调度器
pub mod js_runtime;
pub mod tool_bus;
pub mod task_queue;
pub mod storage;
pub mod group_chat_bridge;
pub mod manager;  // 统一管理入口
pub mod commands;  // Tauri 命令

pub use message_bus::*;
pub use event_router::*;
pub use agent_registry::*;
pub use agent_actor::*;
pub use agent_supervisor::*;
pub use agent_router::*;
pub use agent_scheduler::AgentScheduler;  // ← 导出 AgentScheduler
pub use js_runtime::*;
pub use tool_bus::*;
pub use task_queue::*;
pub use storage::*;
pub use manager::*;

use std::sync::Arc;
use tokio::sync::RwLock;
use crate::agent::providers::ProviderRegistry;
use crate::agent::media_config::MediaApiConfig;
use crate::agent::ai_client_pool::AiClientPool;
use crate::agent::perception::{PerceptionEngine};
use crate::agent::memory::MemoryManager;
use crate::media_archive::MediaArchiveManager;
use crate::agent::task::TaskManager;
use crate::tools::{ToolRegistry, ToolFacade};
use crate::bridges::BridgeManager;

/// Agent Runtime 状态
pub struct AgentRuntimeState {
    pub message_bus: MessageBus,
    pub event_router: EventRouter,
    pub agent_registry: Arc<AgentRegistry>,
    pub agent_router: AgentRouter,
    pub agent_supervisor: AgentSupervisor,
    pub js_runtime_pool: JsRuntimePool,
    pub tool_bus: Arc<ToolBus>,
    pub task_queue: TaskQueue,
    pub storage: Storage,
    pub provider_registry: Arc<ProviderRegistry>,
    pub tool_registry: Arc<ToolRegistry>,
    pub tool_facade: Arc<ToolFacade>,
    pub bridge_manager: Arc<BridgeManager>,
    pub ai_client_pool: Arc<AiClientPool>,
    pub agent_scheduler: Arc<AgentScheduler>,
    pub perception_engine: Arc<PerceptionEngine>,  // ← 新增：智能感知引擎
}

impl AgentRuntimeState {
    pub async fn new(
        tool_registry: Arc<ToolRegistry>,
        bridge_manager: Arc<BridgeManager>,
    ) -> Result<Self, String> {
        // 加载媒体配置并创建 Provider Registry（需要先创建）
        let media_config = MediaApiConfig::load().unwrap_or_else(|_| MediaApiConfig::default());
        let provider_registry = Arc::new(
            ProviderRegistry::new(&media_config)
                .unwrap_or_else(|e| {
                    log::warn!("ProviderRegistry 创建失败：{}, 使用空配置", e);
                    ProviderRegistry::new(&MediaApiConfig::default()).unwrap()
                })
        );
        
        let message_bus = MessageBus::with_default_capacity();
        let event_router = EventRouter::new(message_bus.clone()).await;
        let agent_registry = Arc::new(AgentRegistry::new());
        let agent_router = AgentRouter::new(agent_registry.clone());
        
        // 创建媒体存档管理器
        let archive_manager = Arc::new(
            MediaArchiveManager::new()
                .map_err(|e| format!("ArchiveManager 创建失败: {}", e))?
        );

        // 创建 ToolBus 并注册媒体工具
        let mut tool_bus = ToolBus::new();
        tool_bus.register_media_tools(provider_registry.clone(), archive_manager.clone());
        let tool_bus = Arc::new(tool_bus);
        
        // 创建统一工具入口
        let tool_facade = Arc::new(ToolFacade::new(tool_registry.clone(), tool_bus.clone()));

        // 创建 AI Client Pool（复用 AI Client 实例）
        let ai_client_pool = Arc::new(AiClientPool::new());

        // 创建 Agent Supervisor（传入共享的 tool_registry）
        let agent_supervisor = AgentSupervisor::new(
            RestartPolicy::OnFailure(3),
            tool_facade.clone(),
            bridge_manager.clone(),
            ai_client_pool.clone(),
            tool_registry.clone(),  // ← 传递共享的 ToolRegistry
        );
        
        let js_runtime_pool = JsRuntimePool::new(JsRuntimeConfig::default()).await?;
        let task_queue = TaskQueue::new();
        let storage = Storage::new("./alou_runtime.db").await?;

        // 创建 Agent 调度器（每 3 秒 tick 一次）
        let agent_scheduler = Arc::new(AgentScheduler::new(3000));

        // 🔥 创建智能感知引擎
        let memory_manager = Arc::new(
            MemoryManager::new().map_err(|e| format!("MemoryManager 创建失败: {}", e))?
        );
        
        // 创建一个共享的 TaskManager 给 PerceptionEngine
        let perception_task_manager = Arc::new(TaskManager::new());
        
        let perception_engine = Arc::new(PerceptionEngine::new(
            memory_manager,
            perception_task_manager,
        ));

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
            tool_registry,
            tool_facade,
            bridge_manager,
            ai_client_pool,
            agent_scheduler,
            perception_engine,  // ← 新增：智能感知引擎
        })
    }
}

/// Agent Runtime
pub struct AgentRuntime {
    pub state: Arc<AgentRuntimeState>,
}

impl AgentRuntime {
    pub async fn new(
        tool_registry: Arc<ToolRegistry>,
        bridge_manager: Arc<BridgeManager>,
    ) -> Result<Self, String> {
        let state = Arc::new(AgentRuntimeState::new(tool_registry, bridge_manager).await?);
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
