//! 执行器核心模块
//!
//! 负责：
//! - 组合所有层（Perception, Reasoning, Action, Integration）
//! - 主循环协调
//! - 错误处理和恢复

use std::sync::Arc;

use tauri::{Emitter, Manager};
use crate::agent::ai_client::AiClient;
use crate::agent::task::{TaskManager, TaskStatus};
use crate::agent::perception::PerceptionEngine;
use crate::agent::goal::GoalTracker;
use crate::bridges::ToolBridge;
use crate::tools::ToolRegistry;
use crate::agent::executor::types::{
    ExecutorError, ExecutionResult, Action,
};
use crate::agent::executor::perception::PerceptionLayer;
use crate::agent::executor::reasoning::ReasoningLayer;
use crate::agent::executor::action::ActionLayer;
use crate::agent::executor::integration::IntegrationLayer;
use crate::agent::executor::types::{ContextDocuments, Task};

/// 执行器核心结构
pub struct ExecutorCore {
    ai_client: Arc<AiClient>,
    task_manager: Arc<TaskManager>,
    tool_bridge: Arc<ToolBridge>,
    tool_registry: Arc<ToolRegistry>,
    perception_engine: Option<Arc<PerceptionEngine>>,
    goal_tracker: Option<Arc<GoalTracker>>,
    app_handle: Option<tauri::AppHandle>,
    agent_id: Option<String>,
}

impl ExecutorCore {
    /// 创建 ExecutorCore
    pub fn new(
        ai_client: Arc<AiClient>,
        task_manager: Arc<TaskManager>,
        tool_bridge: Arc<ToolBridge>,
        tool_registry: Arc<ToolRegistry>,
    ) -> Self {
        Self {
            ai_client,
            task_manager,
            tool_bridge,
            tool_registry,
            perception_engine: None,
            goal_tracker: None,
            app_handle: None,
            agent_id: None,
        }
    }

    /// 设置感知引擎
    pub fn with_perception_engine(mut self, engine: Arc<PerceptionEngine>) -> Self {
        self.perception_engine = Some(engine);
        self
    }

    /// 设置目标追踪器
    pub fn with_goal_tracker(mut self, tracker: Arc<GoalTracker>) -> Self {
        self.goal_tracker = Some(tracker);
        self
    }

    /// 设置 AppHandle（用于向前端发送事件）
    pub fn with_app_handle(mut self, app_handle: tauri::AppHandle) -> Self {
        self.app_handle = Some(app_handle);
        self
    }

    /// 设置 Agent ID（用于 agent_document 等操作）
    pub fn with_agent_id(mut self, agent_id: String) -> Self {
        self.agent_id = Some(agent_id);
        self
    }

    // Getters
    pub fn ai_client(&self) -> Arc<AiClient> {
        self.ai_client.clone()
    }

    pub fn task_manager(&self) -> Arc<TaskManager> {
        self.task_manager.clone()
    }

    pub fn tool_bridge(&self) -> Arc<ToolBridge> {
        self.tool_bridge.clone()
    }

    pub fn tool_registry(&self) -> Arc<ToolRegistry> {
        self.tool_registry.clone()
    }

    pub fn perception_engine(&self) -> Option<Arc<PerceptionEngine>> {
        self.perception_engine.clone()
    }

    pub fn goal_tracker(&self) -> Option<Arc<GoalTracker>> {
        self.goal_tracker.clone()
    }

    pub fn app_handle(&self) -> Option<tauri::AppHandle> {
        self.app_handle.clone()
    }

    pub fn agent_id(&self) -> Option<String> {
        self.agent_id.clone()
    }
}

/// Ralph Loop 执行器
///
/// 实现了 Perceive → Reason → Act → Integrate 循环
pub struct RalphLoopExecutor {
    core: ExecutorCore,
    perception_layer: PerceptionLayer,
    reasoning_layer: ReasoningLayer,
    action_layer: ActionLayer,
    integration_layer: IntegrationLayer,
}

impl RalphLoopExecutor {
    /// 最大迭代次数
    pub const MAX_ITERATIONS: u32 = 500;  // 🔥 增加到 500 次，支持长周期任务
    /// 最大工具调用次数
    pub const MAX_TOOL_CALLS: u32 = 1000; // 🔥 增加到 1000 次，支持复杂任务

    /// 创建执行器
    pub fn new(core: ExecutorCore) -> Self {
        let ai_client = core.ai_client.clone();
        let tool_bridge = core.tool_bridge.clone();
        let tool_registry = core.tool_registry.clone();

        // 🔥 使用带元行动工具的 ActionLayer
        let action_layer = if let (Some(ref perception_engine), Some(ref goal_tracker)) = 
            (core.perception_engine(), core.goal_tracker()) {
            // 从 PerceptionEngine 获取 MemoryManager 和 TaskManager
            ActionLayer::with_meta_tools(
                tool_bridge,
                tool_registry,
                perception_engine.memory_manager(),
                perception_engine.task_manager(),
            )
        } else {
            ActionLayer::new(tool_bridge, tool_registry)
        };

        Self {
            core,
            perception_layer: PerceptionLayer::new(),
            reasoning_layer: ReasoningLayer::new(ai_client),
            action_layer,
            integration_layer: IntegrationLayer::new(),
        }
    }

    /// 创建执行器（带 GoalTracker 初始化）
    pub fn new_with_goal_tracker(
        core: ExecutorCore,
        hybrid_storage: Arc<crate::agent::goal_storage_hybrid::HybridGoalStorage>,
    ) -> Self {
        let ai_client = core.ai_client.clone();
        let tool_bridge = core.tool_bridge.clone();
        let tool_registry = core.tool_registry.clone();

        // 🔥 初始化 GoalTracker
        let goal_tracker = Arc::new(GoalTracker::new(hybrid_storage.clone()));

        // 更新 core 的 goal_tracker
        let core = core.with_goal_tracker(goal_tracker.clone());

        // 🔥 使用带元行动工具的 ActionLayer
        let action_layer = if let Some(ref perception_engine) = core.perception_engine() {
            ActionLayer::with_meta_tools(
                tool_bridge,
                tool_registry,
                perception_engine.memory_manager(),
                perception_engine.task_manager(),
            )
        } else {
            ActionLayer::new(tool_bridge, tool_registry)
        };

        Self {
            core,
            perception_layer: PerceptionLayer::new(),
            reasoning_layer: ReasoningLayer::new(ai_client),
            action_layer,
            integration_layer: IntegrationLayer::new(),
        }
    }

    /// 执行循环（主入口）
    pub async fn execute(&self, task_id: &str) -> Result<String, ExecutorError> {
        log::info!("[RalphLoop] 开始执行任务：{}", task_id);

        // 🔥 1. 加载文档到缓存上下文（执行前准备）
        let context_docs = self.load_context_documents(task_id).await;

        // 更新任务状态为运行中
        let _ = self.core.task_manager().update_task(task_id, |task| {
            task.status = TaskStatus::Running;
        }).await;

        // 发送开始事件
        self.emit_event(task_id, "started", Some(&serde_json::json!({
            "task_id": task_id,
        })));

        loop {
            let loop_start = std::time::Instant::now();

            // ========== 1. 感知环境 ==========
            log::info!("[RalphLoop:{}] 迭代开始 - 感知环境 (+{:?})", task_id, loop_start.elapsed());
            let state = self.perception_layer
                .perceive(&self.core, task_id, &self.core.perception_engine)
                .await?;

            log::info!(
                "[RalphLoop:{}] 迭代 {} - 工具调用 {}",
                task_id,
                state.iteration_count,
                state.tool_call_count
            );

            // 检查资源限制
            if state.iteration_count >= Self::MAX_ITERATIONS {
                log::warn!("[RalphLoop:{}] 达到最大迭代次数", task_id);
                return Err(ExecutorError::InternalError(
                    "达到最大迭代次数".to_string()
                ));
            }

            if state.tool_call_count >= Self::MAX_TOOL_CALLS {
                log::warn!("[RalphLoop:{}] 达到最大工具调用次数", task_id);
                return Err(ExecutorError::InternalError(
                    "达到最大工具调用次数".to_string()
                ));
            }

            // 分析意图
            let intent = self.perception_layer
                .analyze_intent(&self.core, &state, &self.core.perception_engine)
                .await;

            // ========== 2. 推理 ==========
            log::info!("[RalphLoop:{}] 推理决策开始 (+{:?})", task_id, loop_start.elapsed());
            let ai_start = std::time::Instant::now();
            let thought = self.reasoning_layer
                .reason(&state, intent, self.core.tool_registry())
                .await?;
            log::info!("[RalphLoop:{}] 推理完成 (+{:?} 耗时 {:?})", task_id, loop_start.elapsed(), ai_start.elapsed());

            log::debug!("[RalphLoop:{}] 分析：{}", task_id, thought.analysis);

            // 发送 AI 响应事件
            self.emit_event(task_id, "thinking", Some(&serde_json::json!({
                "task_id": task_id,
                "content": thought.analysis,
            })));

            if let Some(ref reflection) = thought.reflection {
                log::debug!("[RalphLoop:{}] 置信度：{:.0}%, 进度：{:.0}%",
                    task_id,
                    reflection.confidence * 100.0,
                    reflection.task_progress.overall_progress * 100.0);

                // 低置信度干预
                if reflection.confidence < 0.6 {
                    log::warn!("[RalphLoop:{}] 低置信度检测 ({:.0}%)，建议：{}",
                        task_id,
                        reflection.confidence * 100.0,
                        reflection.information_assessment.suggestion);
                }
            }

            // ========== 3. 行动 ==========
            log::info!("[RalphLoop:{}] 执行行动 (+{:?})", task_id, loop_start.elapsed());
            let tool_start = std::time::Instant::now();

            // 更新任务状态：工具执行中
            if let Action::ToolCall { tool, .. } = &thought.action {
                let _ = self.core.task_manager().update_task(task_id, |task| {
                    task.status = TaskStatus::ProcessingTools;
                }).await;

                // 发送工具执行开始事件
                self.emit_event(task_id, "tool_calling", Some(&serde_json::json!({
                    "task_id": task_id,
                    "tool_name": tool,
                })));
            }

            let results = self.action_layer
                .execute(&thought, &self.core)
                .await?;

            log::info!("[RalphLoop:{}] 行动执行完成 (+{:?} 耗时 {:?})", task_id, loop_start.elapsed(), tool_start.elapsed());

            // ========== 4. 整合 ==========
            log::info!("[RalphLoop:{}] 整合结果", task_id);
            let execution_result = self.integration_layer
                .integrate(&self.core, task_id, &thought, &results)
                .await?;

            match execution_result {
                ExecutionResult::Completed(answer) => {
                    log::info!("[RalphLoop:{}] 任务完成：{}", task_id, answer.chars().take(50).collect::<String>());
                    
                    // 🔥 任务完成后保存持久化数据（SOUL.md, MEMORY.md 等）
                    self.save_persistence_data(task_id).await;
                    
                    return Ok(answer);
                }
                ExecutionResult::Failed(reason) => {
                    log::error!("[RalphLoop:{}] 任务失败：{}", task_id, reason);
                    let _ = self.integration_layer.handle_error(&self.core, task_id, &ExecutorError::InternalError(reason.clone())).await;
                    return Err(ExecutorError::InternalError(reason));
                }
                ExecutionResult::NeedsMoreIterations => {
                    // 继续循环
                    continue;
                }
            }
        }
    }

    /// 🔥 保存持久化数据（SOUL.md, MEMORY.md, TASKS.md 等）
    async fn save_persistence_data(&self, task_id: &str) {
        use crate::soul::SoulManager;
        use crate::tasks::manager::TasksManager;
        use chrono::Local;
        use std::fs;
        use std::path::PathBuf;

        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        
        // 🔥 使用 agent-specific 路径：AppData/agent-documents/{agent_id}/
        let agent_id = self.core.agent_id().unwrap_or_default();
        let base_dir = if let Some(app_handle) = self.core.app_handle() {
            app_handle.path().app_data_dir()
                .map(|p| p.join("agent-documents").join(&agent_id))
                .unwrap_or_else(|_| PathBuf::from("./.alou"))
        } else {
            // 如果没有 app_handle，使用 ~/.alou/{agent_id}/
            dirs::home_dir()
                .map(|h| h.join(".alou").join(&agent_id))
                .unwrap_or_else(|| PathBuf::from("./.alou"))
        };

        let _ = fs::create_dir_all(&base_dir);

        // Save SOUL.md
        match SoulManager::new(&base_dir).load() {
            Ok(_) => {
                log::info!("[RalphLoop:{}] [{}] Soul module: saved (agent: {})", task_id, now, agent_id);
            }
            Err(e) => {
                log::warn!("[RalphLoop:{}] [{}] Soul module: created default ({})", task_id, now, e);
            }
        }

        // Save TASKS.md
        let tasks_manager = TasksManager::new(&base_dir);
        // 首次运行时创建默认文件
        let _ = tasks_manager.load();
        log::info!("[RalphLoop:{}] [{}] Tasks module: saved (agent: {})", task_id, now, agent_id);
    }

    /// 🔥 加载文档到缓存上下文（执行前准备）
    async fn load_context_documents(&self, task_id: &str) -> ContextDocuments {
        use crate::soul::SoulManager;
        use crate::tasks::manager::TasksManager;
        use chrono::Local;
        use std::fs;
        use std::path::PathBuf;

        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        
        // 🔥 使用 agent-specific 路径：AppData/agent-documents/{agent_id}/
        let agent_id = self.core.agent_id().unwrap_or_default();
        let base_dir = if let Some(app_handle) = self.core.app_handle() {
            app_handle.path().app_data_dir()
                .map(|p| p.join("agent-documents").join(&agent_id))
                .unwrap_or_else(|_| PathBuf::from("./.alou"))
        } else {
            // 如果没有 app_handle，使用 ~/.alou/{agent_id}/
            dirs::home_dir()
                .map(|h| h.join(".alou").join(&agent_id))
                .unwrap_or_else(|| PathBuf::from("./.alou"))
        };

        let _ = fs::create_dir_all(&base_dir);

        log::info!("[RalphLoop:{}] [{}] 加载上下文文档 (agent: {})...", task_id, now, agent_id);

        // 加载 SOUL.md
        let soul_content = match SoulManager::new(&base_dir).load() {
            Ok(content) => {
                log::info!("[RalphLoop:{}] [{}] SOUL.md: loaded", task_id, now);
                Some(content)
            }
            Err(e) => {
                log::warn!("[RalphLoop:{}] [{}] SOUL.md: created default ({})", task_id, now, e);
                None
            }
        };

        // 加载 TASKS.md
        let tasks_content = {
            let tasks_manager = TasksManager::new(&base_dir);
            match tasks_manager.load() {
                Ok(tasks) => {
                    log::info!("[RalphLoop:{}] [{}] TASKS.md: loaded {} tasks", task_id, now, tasks.len());
                    Some(tasks)
                }
                Err(_) => None
            }
        };

        // 加载 MEMORY.md
        let memory_content = match fs::read_to_string(base_dir.join("MEMORY.md")) {
            Ok(content) => {
                log::info!("[RalphLoop:{}] [{}] MEMORY.md: loaded", task_id, now);
                Some(content)
            }
            Err(_) => {
                log::info!("[RalphLoop:{}] [{}] MEMORY.md: not found", task_id, now);
                None
            }
        };

        ContextDocuments {
            soul: soul_content,
            tasks: tasks_content,
            memory: memory_content,
        }
    }

    /// 执行单步（用于调试或手动控制）
    pub async fn execute_step(&self, task_id: &str) -> Result<ExecutionResult, ExecutorError> {
        // 感知
        let state = self.perception_layer
            .perceive(&self.core, task_id, &self.core.perception_engine)
            .await?;

        let intent = self.perception_layer
            .analyze_intent(&self.core, &state, &self.core.perception_engine)
            .await;

        // 推理
        let thought = self.reasoning_layer
            .reason(&state, intent, self.core.tool_registry())
            .await?;

        // 行动
        let results = self.action_layer
            .execute(&thought, &self.core)
            .await?;

        // 整合
        self.integration_layer
            .integrate(&self.core, task_id, &thought, &results)
            .await
    }

    /// 快速执行（简化流程，用于简单任务）
    pub async fn execute_quick(&self, task_id: &str) -> Result<String, ExecutorError> {
        let state = self.perception_layer
            .perceive_fast(&self.core, task_id)
            .await?;

        let thought = self.reasoning_layer
            .reason_simple(&state)
            .await?;

        if let crate::agent::executor::types::Action::Complete(content) = thought.action {
            Ok(content)
        } else {
            // 如果不是 Complete，走完整流程
            self.execute(task_id).await
        }
    }

    /// 发送事件到前端
    fn emit_event(&self, task_id: &str, event_type: &str, payload: Option<&serde_json::Value>) {
        if let Some(app_handle) = self.core.app_handle() {
            let event_payload = payload.cloned().unwrap_or_else(|| serde_json::json!({
                "task_id": task_id,
            }));

            if let Err(e) = app_handle.emit("agent:progress", &event_payload) {
                log::warn!("[RalphLoop] 发送 agent:progress 事件失败：{}", e);
            }
        }
    }
}

/// 任务执行结果
#[derive(Debug)]
pub struct TaskFinalResult {
    pub success: bool,
    pub answer: Option<String>,
    pub error: Option<String>,
    pub iterations: u32,
    pub tool_calls: u32,
}

/// 构建器模式创建执行器
pub struct RalphLoopExecutorBuilder {
    ai_client: Option<Arc<AiClient>>,
    task_manager: Option<Arc<TaskManager>>,
    tool_bridge: Option<Arc<ToolBridge>>,
    tool_registry: Option<Arc<ToolRegistry>>,
    perception_engine: Option<Arc<PerceptionEngine>>,
    goal_tracker: Option<Arc<GoalTracker>>,
    app_handle: Option<tauri::AppHandle>,
    agent_id: Option<String>,
}

impl RalphLoopExecutorBuilder {
    pub fn new() -> Self {
        Self {
            ai_client: None,
            task_manager: None,
            tool_bridge: None,
            tool_registry: None,
            perception_engine: None,
            goal_tracker: None,
            app_handle: None,
            agent_id: None,
        }
    }

    pub fn ai_client(mut self, client: Arc<AiClient>) -> Self {
        self.ai_client = Some(client);
        self
    }

    pub fn task_manager(mut self, manager: Arc<TaskManager>) -> Self {
        self.task_manager = Some(manager);
        self
    }

    pub fn tool_bridge(mut self, bridge: Arc<ToolBridge>) -> Self {
        self.tool_bridge = Some(bridge);
        self
    }

    pub fn tool_registry(mut self, registry: Arc<ToolRegistry>) -> Self {
        self.tool_registry = Some(registry);
        self
    }

    pub fn perception_engine(mut self, engine: Arc<PerceptionEngine>) -> Self {
        self.perception_engine = Some(engine);
        self
    }

    pub fn goal_tracker(mut self, tracker: Arc<GoalTracker>) -> Self {
        self.goal_tracker = Some(tracker);
        self
    }

    pub fn app_handle(mut self, app_handle: tauri::AppHandle) -> Self {
        self.app_handle = Some(app_handle);
        self
    }

    pub fn agent_id(mut self, agent_id: String) -> Self {
        self.agent_id = Some(agent_id);
        self
    }

    pub fn build(self) -> Result<RalphLoopExecutor, String> {
        let ai_client = self.ai_client.ok_or("AiClient 未设置")?;
        let task_manager = self.task_manager.ok_or("TaskManager 未设置")?;
        let tool_bridge = self.tool_bridge.ok_or("ToolBridge 未设置")?;
        let tool_registry = self.tool_registry.ok_or("ToolRegistry 未设置")?;

        let mut core = ExecutorCore::new(ai_client, task_manager, tool_bridge, tool_registry);

        if let Some(engine) = self.perception_engine {
            core = core.with_perception_engine(engine);
        }

        if let Some(tracker) = self.goal_tracker {
            core = core.with_goal_tracker(tracker);
        }

        if let Some(app_handle) = self.app_handle {
            core = core.with_app_handle(app_handle);
        }

        if let Some(agent_id) = self.agent_id {
            core = core.with_agent_id(agent_id);
        }

        Ok(RalphLoopExecutor::new(core))
    }
}

impl Default for RalphLoopExecutorBuilder {
    fn default() -> Self {
        Self::new()
    }
}
