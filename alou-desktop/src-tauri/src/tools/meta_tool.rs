//! 元行动工具 - Meta-Action Tools
//!
//! 提供高级认知能力工具：
//! - memory_search: 搜索长期记忆
//! - read_document: 读取文档
//! - manage_goal: 管理目标
//! - remember: 记录到记忆

use serde_json::{Value, json};
use std::sync::Arc;

/// 元行动工具集合
pub struct MetaActionTools {
    /// 记忆管理器
    pub memory_manager: Arc<crate::context::memory::MemoryManager>,
    /// 目标追踪器
    pub goal_tracker: Arc<crate::agent::perception::GoalTracker>,
    /// 文档加载器
    pub document_loader: Arc<crate::agent::perception::DocumentLoader>,
}

impl MetaActionTools {
    /// 创建新的元行动工具集合
    pub fn new(
        memory_manager: Arc<crate::context::memory::MemoryManager>,
        goal_tracker: Arc<crate::agent::perception::GoalTracker>,
        document_loader: Arc<crate::agent::perception::DocumentLoader>,
    ) -> Self {
        Self {
            memory_manager,
            goal_tracker,
            document_loader,
        }
    }

    /// 工具 1: 搜索记忆
    pub async fn memory_search(&self, query: &str, memory_type: Option<&str>) -> Result<Value, String> {
        use crate::context::memory::MemoryType;
        
        let mem_type = memory_type.map(|t| match t {
            "conversation" => MemoryType::Conversation,
            "preference" => MemoryType::Preference,
            "knowledge" => MemoryType::Knowledge,
            "experience" => MemoryType::Experience,
            _ => MemoryType::Conversation,
        });

        // 使用 memory_manager 搜索
        let results = if let Some(mt) = mem_type {
            self.memory_manager.retrieve_by_type(mt, 10).await
        } else {
            self.memory_manager.retrieve(query, None, 10).await
        };

        let memories = results.iter().map(|m| json!({
            "id": m.id,
            "type": format!("{:?}", m.memory_type),
            "content": m.content,
            "importance": format!("{:?}", m.importance),
            "created_at": m.created_at,
            "tags": m.tags,
        })).collect::<Vec<_>>();

        Ok(json!({
            "success": true,
            "query": query,
            "memory_type": memory_type.unwrap_or("all"),
            "count": memories.len(),
            "memories": memories
        }))
    }

    /// 工具 2: 读取文档
    pub async fn read_document(&self, doc_name: &str) -> Result<Value, String> {
        let doc = self.document_loader.load(doc_name).await?;

        Ok(json!({
            "success": true,
            "document": doc.name,
            "content": doc.content,
            "relevance_score": doc.relevance_score,
            "loaded_at": chrono::Utc::now().to_rfc3339(),
        }))
    }

    /// 工具 3: 创建/更新目标
    pub async fn manage_goal(&self, action: &str, description: &str, priority: Option<&str>) -> Result<Value, String> {
        use crate::agent::perception::Priority;
        
        match action {
            "create" => {
                let priority_val = match priority.unwrap_or("medium") {
                    "low" => Priority::Low,
                    "high" => Priority::High,
                    "critical" => Priority::Critical,
                    _ => Priority::Medium,
                };

                let goal = self.goal_tracker.create_goal(description, priority_val).await;
                Ok(json!({
                    "success": true,
                    "action": "created",
                    "goal_id": goal.id,
                    "description": goal.description,
                    "priority": format!("{:?}", goal.priority),
                    "status": "active",
                }))
            },
            "complete" => {
                self.goal_tracker.complete_goal(description).await?;
                Ok(json!({
                    "success": true,
                    "action": "completed",
                    "goal_id": description,
                }))
            },
            "list_active" => {
                let goals = self.goal_tracker.get_active_goals().await;
                let goals_json = goals.iter().map(|g| json!({
                    "id": g.id,
                    "description": g.description,
                    "priority": format!("{:?}", g.priority),
                    "progress": g.progress,
                    "created_at": g.created_at,
                })).collect::<Vec<_>>();

                Ok(json!({
                    "success": true,
                    "count": goals_json.len(),
                    "active_goals": goals_json,
                }))
            },
            _ => Err(format!("Unknown goal action: {}. Use 'create', 'complete', or 'list_active'", action)),
        }
    }

    /// 工具 4: 记录到记忆
    pub async fn remember(&self, content: &str, memory_type: &str, importance: &str) -> Result<Value, String> {
        use crate::context::memory::{MemoryType, Importance};
        
        let mem_type = match memory_type {
            "preference" => MemoryType::Preference,
            "knowledge" => MemoryType::Knowledge,
            "experience" => MemoryType::Experience,
            _ => MemoryType::Conversation,
        };

        let importance_val = match importance {
            "critical" => Importance::Critical,
            "high" => Importance::High,
            "medium" => Importance::Medium,
            _ => Importance::Low,
        };

        let id = self.memory_manager.create_memory(
            mem_type,
            content.to_string(),
            vec!["auto_recorded".to_string()],
            importance_val,
        ).await;

        Ok(json!({
            "success": true,
            "recorded": true,
            "memory_id": id,
            "type": memory_type,
            "importance": importance,
        }))
    }
}
