//! 感知层模块
//!
//! 负责：
//! - 感知环境（获取任务状态、消息历史）
//! - 调用 PerceptionEngine 智能检索上下文
//! - 更新 EnvironmentState

use std::sync::Arc;

use crate::agent::executor::types::EnvironmentState;
use crate::agent::executor::core::ExecutorCore;
use crate::agent::perception::{PerceptionEngine, Intent, RetrievedContext};
use crate::agent::task::TaskManager;

/// 感知层
pub struct PerceptionLayer;

impl PerceptionLayer {
    /// 创建感知层实例
    pub fn new() -> Self {
        Self
    }

    /// 感知环境（完整流程）
    pub async fn perceive(
        &self,
        core: &ExecutorCore,
        task_id: &str,
        perception_engine: &Option<Arc<PerceptionEngine>>,
    ) -> Result<EnvironmentState, crate::agent::executor::types::ExecutorError> {
        let task_manager = core.task_manager();
        let tool_registry = core.tool_registry();

        // 1. 获取任务
        let task = task_manager
            .get_task(task_id)
            .await
            .ok_or_else(|| crate::agent::executor::types::ExecutorError::TaskNotFound(
                task_id.to_string()
            ))?;

        // 2. 获取消息历史
        let messages = task.messages.clone();

        // 3. 获取迭代计数
        let iteration_count = task.metadata.iteration_count;
        
        // 4. 计算工具调用次数
        let tool_call_count = task.tool_results.len() as u32;

        // 5. 使用 PerceptionEngine 智能感知上下文
        let retrieved_context = if let Some(ref engine) = perception_engine {
            engine.gather_context(&task).await
        } else {
            RetrievedContext::default()
        };

        // 6. 获取可用工具
        let available_tools = tool_registry
            .list_all()
            .await
            .into_iter()
            .map(|t| t.name)
            .collect();

        Ok(EnvironmentState {
            task_id: task_id.to_string(),
            messages,
            iteration_count,
            tool_call_count,
            available_tools,
            retrieved_context,
            context_documents: crate::agent::executor::types::ContextDocuments::default(),
        })
    }

    /// 快速感知（不触发 LLM 调用）
    pub async fn perceive_fast(
        &self,
        core: &ExecutorCore,
        task_id: &str,
    ) -> Result<EnvironmentState, crate::agent::executor::types::ExecutorError> {
        let task_manager = core.task_manager();
        let tool_registry = core.tool_registry();

        let task = task_manager
            .get_task(task_id)
            .await
            .ok_or_else(|| crate::agent::executor::types::ExecutorError::TaskNotFound(
                task_id.to_string()
            ))?;

        let messages = task.messages.clone();
        let iteration_count = task.metadata.iteration_count;
        let tool_call_count = task.tool_results.len() as u32;

        let available_tools = tool_registry
            .list_all()
            .await
            .into_iter()
            .map(|t| t.name)
            .collect();

        Ok(EnvironmentState {
            task_id: task_id.to_string(),
            messages,
            iteration_count,
            tool_call_count,
            available_tools,
            retrieved_context: RetrievedContext::default(),
            context_documents: crate::agent::executor::types::ContextDocuments::default(),
        })
    }

    /// 分析用户意图（基于 PerceptionEngine）
    pub async fn analyze_intent(
        &self,
        _core: &ExecutorCore,
        state: &EnvironmentState,
        perception_engine: &Option<Arc<PerceptionEngine>>,
    ) -> Intent {
        // 获取最后一条用户消息作为查询
        let query = state.messages
            .iter()
            .rev()
            .find(|m| m.role == "user")
            .map(|m| m.content.clone())
            .unwrap_or_default();

        if let Some(ref engine) = perception_engine {
            // analyze_intent 是 pub(crate) 方法，在 agent crate 内部可以访问
            engine.analyze_intent(&query).await
        } else {
            // 简单启发式分类
            Self::heuristic_intent_classification(&state.messages)
        }
    }

    /// 启发式意图分类（无 PerceptionEngine 时使用）
    fn heuristic_intent_classification(messages: &[crate::agent::ai_client::AiMessage]) -> Intent {
        if messages.is_empty() {
            return Intent::Unclear;
        }

        let last_message = messages.last().unwrap();
        let content = last_message.content.to_lowercase();

        // 简单关键字匹配
        if content.contains("你好") || content.contains("hi") || content.contains("hello") {
            Intent::Greeting
        } else if content.contains("搜索") || content.contains("查询") || content.contains("find") {
            Intent::SearchQuery
        } else if content.contains("错误") || content.contains("bug") || content.contains("debug") {
            Intent::Debugging
        } else if content.contains("创建") || content.contains("生成") || content.contains("make") {
            Intent::CodeTask
        } else {
            Intent::Unclear
        }
    }
}

impl Default for PerceptionLayer {
    fn default() -> Self {
        Self::new()
    }
}
