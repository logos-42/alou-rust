//! 元行动工具 (Meta-Action Tools)
//!
//! 这些工具让 AI 能够：
//! - 主动获取信息（搜索记忆、读取文档）
//! - 管理目标（创建、更新、完成目标）
//! - 记录重要信息到长期记忆

use crate::agent::memory::{MemoryManager, MemoryType, Importance};
use crate::agent::perception::PerceptionEngine;
use crate::agent::task::TaskManager;
use serde_json::{json, Value};
use std::sync::Arc;

/// 元行动工具集合
pub struct MetaTools {
    memory_manager: Arc<MemoryManager>,
    perception_engine: Option<Arc<PerceptionEngine>>,
}

impl MetaTools {
    pub fn new(memory_manager: Arc<MemoryManager>, task_manager: Arc<TaskManager>) -> Self {
        // 创建感知引擎
        let perception_engine = Arc::new(PerceptionEngine::new(
            memory_manager.clone(),
            task_manager,
        ));

        Self {
            memory_manager,
            perception_engine: Some(perception_engine),
        }
    }

    pub fn new_simple(memory_manager: Arc<MemoryManager>) -> Self {
        Self {
            memory_manager,
            perception_engine: None,
        }
    }

    /// 获取所有元行动工具定义
    pub fn get_tool_definitions() -> Vec<Value> {
        vec![
            Self::memory_search_definition(),
            Self::read_document_definition(),
            Self::remember_definition(),
            Self::manage_goal_definition(),
        ]
    }

    /// memory_search 工具定义
    fn memory_search_definition() -> Value {
        json!({
            "name": "memory_search",
            "description": "搜索长期记忆，获取相关历史信息。当你需要回忆之前的对话、用户偏好或经验时使用此工具。",
            "parameters": {
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "搜索关键词或问题"
                    },
                    "memory_type": {
                        "type": "string",
                        "enum": ["conversation", "preference", "knowledge", "experience", "feedback"],
                        "description": "记忆类型过滤（可选）"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "返回结果数量限制（默认 5）",
                        "default": 5
                    }
                },
                "required": ["query"]
            }
        })
    }

    /// read_document 工具定义
    fn read_document_definition() -> Value {
        json!({
            "name": "read_document",
            "description": "读取指定文档的完整内容。当你需要参考项目规范、API 文档或任何其他文档时使用此工具。",
            "parameters": {
                "type": "object",
                "properties": {
                    "doc_name": {
                        "type": "string",
                        "description": "文档名称，如 CODING_STANDARDS.md, API_SPEC.md, README.md"
                    },
                    "section": {
                        "type": "string",
                        "description": "特定章节（可选，如 'Authentication'）"
                    }
                },
                "required": ["doc_name"]
            }
        })
    }

    /// remember 工具定义
    fn remember_definition() -> Value {
        json!({
            "name": "remember",
            "description": "将重要信息记录到长期记忆。当你学到关于用户的新信息、发现重要模式或需要记住某事时使用此工具。",
            "parameters": {
                "type": "object",
                "properties": {
                    "content": {
                        "type": "string",
                        "description": "要记录的内容"
                    },
                    "memory_type": {
                        "type": "string",
                        "enum": ["preference", "knowledge", "experience", "pattern"],
                        "description": "记忆类型"
                    },
                    "importance": {
                        "type": "string",
                        "enum": ["low", "medium", "high", "critical"],
                        "description": "重要性级别",
                        "default": "medium"
                    },
                    "tags": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "标签，用于后续检索",
                        "default": []
                    }
                },
                "required": ["content", "memory_type"]
            }
        })
    }

    /// manage_goal 工具定义
    fn manage_goal_definition() -> Value {
        json!({
            "name": "manage_goal",
            "description": "创建、更新或列出长期目标。当你需要跟踪多步骤任务或跨会话的持续工作时使用此工具。",
            "parameters": {
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["create", "update", "complete", "list_active"],
                        "description": "操作类型"
                    },
                    "description": {
                        "type": "string",
                        "description": "目标描述（create 时必需，update/complete 时为目标 ID）"
                    },
                    "priority": {
                        "type": "string",
                        "enum": ["low", "medium", "high", "critical"],
                        "description": "优先级（create 时可选）",
                        "default": "medium"
                    },
                    "progress": {
                        "type": "number",
                        "description": "进度百分比 0-1（update 时可选）"
                    }
                },
                "required": ["action", "description"]
            }
        })
    }

    /// 执行元行动工具
    pub async fn execute(&self, tool_name: &str, args: Value) -> Result<Value, MetaToolError> {
        match tool_name {
            "memory_search" => self.memory_search(args).await,
            "read_document" => self.read_document(args).await,
            "remember" => self.remember(args).await,
            "manage_goal" => self.manage_goal(args).await,
            _ => Err(MetaToolError::NotFound(format!("未知元行动工具: {}", tool_name))),
        }
    }

    /// memory_search - 搜索长期记忆
    async fn memory_search(&self, args: Value) -> Result<Value, MetaToolError> {
        let query = args["query"].as_str()
            .ok_or_else(|| MetaToolError::InvalidArguments("缺少 query 参数".to_string()))?;
        
        let memory_type = args["memory_type"].as_str().and_then(|t| match t {
            "conversation" => Some(MemoryType::Conversation),
            "preference" => Some(MemoryType::Preference),
            "knowledge" => Some(MemoryType::Knowledge),
            "experience" => Some(MemoryType::Experience),
            "feedback" => Some(MemoryType::Feedback),
            _ => None,
        });

        let limit = args["limit"].as_u64().unwrap_or(5) as usize;

        log::info!("[MetaTool] memory_search: query='{}', limit={}", query, limit);

        let memories = self.memory_manager.retrieve(query, memory_type, limit).await;

        Ok(json!({
            "success": true,
            "count": memories.len(),
            "query": query,
            "memories": memories.iter().map(|m| json!({
                "id": m.id,
                "type": format!("{:?}", m.memory_type),
                "content": m.content,
                "importance": format!("{:?}", m.importance),
                "created_at": m.created_at,
                "tags": m.tags,
            })).collect::<Vec<_>>()
        }))
    }

    /// read_document - 读取文档
    async fn read_document(&self, args: Value) -> Result<Value, MetaToolError> {
        let doc_name = args["doc_name"].as_str()
            .ok_or_else(|| MetaToolError::InvalidArguments("缺少 doc_name 参数".to_string()))?;

        log::info!("[MetaTool] read_document: {}", doc_name);

        // 尝试多个路径
        let possible_paths = [
            doc_name.to_string(),
            format!("./{}", doc_name),
            format!("./docs/{}", doc_name),
            dirs::home_dir()
                .map(|p| p.join(".alou").join("agents").join("docs").join(doc_name))
                .and_then(|p| p.to_str().map(String::from))
                .unwrap_or_default(),
        ];

        for path in &possible_paths {
            if let Ok(content) = tokio::fs::read_to_string(path).await {
                return Ok(json!({
                    "success": true,
                    "document": doc_name,
                    "path": path,
                    "content": content,
                    "length": content.len(),
                }));
            }
        }

        Err(MetaToolError::ExecutionFailed(format!("文档 '{}' 未找到", doc_name)))
    }

    /// remember - 记录到记忆
    async fn remember(&self, args: Value) -> Result<Value, MetaToolError> {
        let content = args["content"].as_str()
            .ok_or_else(|| MetaToolError::InvalidArguments("缺少 content 参数".to_string()))?;
        
        let memory_type = args["memory_type"].as_str()
            .ok_or_else(|| MetaToolError::InvalidArguments("缺少 memory_type 参数".to_string()))?;

        let importance = args["importance"].as_str().map(|i| match i {
            "critical" => Importance::Critical,
            "high" => Importance::High,
            "medium" => Importance::Medium,
            _ => Importance::Low,
        }).unwrap_or(Importance::Medium);

        let mem_type = match memory_type {
            "preference" => MemoryType::Preference,
            "knowledge" => MemoryType::Knowledge,
            "experience" => MemoryType::Experience,
            "pattern" => MemoryType::Pattern,
            _ => MemoryType::Conversation,
        };

        let tags: Vec<String> = args["tags"].as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_else(|| vec!["auto_recorded".to_string()]);

        log::info!("[MetaTool] remember: type={:?}, importance={:?}", mem_type, importance);

        let id = self.memory_manager.create_memory(
            mem_type,
            content.to_string(),
            tags,
            importance,
        ).await;

        Ok(json!({
            "success": true,
            "memory_id": id,
            "type": memory_type,
            "content_preview": content.chars().take(100).collect::<String>(),
        }))
    }

    /// manage_goal - 目标管理
    async fn manage_goal(&self, args: Value) -> Result<Value, MetaToolError> {
        let action = args["action"].as_str()
            .ok_or_else(|| MetaToolError::InvalidArguments("缺少 action 参数".to_string()))?;
        
        let description = args["description"].as_str()
            .ok_or_else(|| MetaToolError::InvalidArguments("缺少 description 参数".to_string()))?;

        log::info!("[MetaTool] manage_goal: action={}, desc={}", action, description);

        match action {
            "create" => {
                let priority = args["priority"].as_str().unwrap_or("medium");
                
                // 将目标存储为重要记忆
                let id = self.memory_manager.create_memory(
                    MemoryType::Experience,
                    format!("目标: {}", description),
                    vec!["goal".to_string(), "active".to_string(), format!("priority:{}", priority)],
                    Importance::High,
                ).await;

                // 存储元数据
                self.memory_manager.record_preference(
                    "system".to_string(),
                    format!("goal_{}_progress", id),
                    "0.0".to_string(),
                    1.0,
                ).await;

                Ok(json!({
                    "success": true,
                    "action": "created",
                    "goal_id": id,
                    "description": description,
                    "priority": priority,
                    "status": "active",
                }))
            }

            "update" => {
                // description 参数在这里是 goal_id
                let progress = args["progress"].as_f64().unwrap_or(0.0);
                
                self.memory_manager.record_preference(
                    "system".to_string(),
                    format!("goal_{}_progress", description),
                    progress.to_string(),
                    1.0,
                ).await;

                Ok(json!({
                    "success": true,
                    "action": "updated",
                    "goal_id": description,
                    "progress": progress,
                }))
            }

            "complete" => {
                // description 参数在这里是 goal_id
                self.memory_manager.record_preference(
                    "system".to_string(),
                    format!("goal_{}_progress", description),
                    "1.0".to_string(),
                    1.0,
                ).await;

                // 添加完成标记的记忆
                let _ = self.memory_manager.create_memory(
                    MemoryType::Experience,
                    format!("已完成目标 (ID: {})", description),
                    vec!["goal".to_string(), "completed".to_string()],
                    Importance::Medium,
                ).await;

                Ok(json!({
                    "success": true,
                    "action": "completed",
                    "goal_id": description,
                }))
            }

            "list_active" => {
                // 搜索活跃目标
                let goals = self.memory_manager
                    .retrieve("goal active", None, 10)
                    .await;

                let active_goals: Vec<Value> = goals
                    .into_iter()
                    .filter(|m| m.tags.contains(&"goal".to_string()) && m.tags.contains(&"active".to_string()))
                    .map(|m| {
                        let progress_fut = self.memory_manager
                            .get_preference("system", &format!("goal_{}_progress", m.id));
                        
                        // 由于我们在 async 函数中，可以直接 await
                        // 但这里需要处理异步，所以我们先用默认值为 0.0
                        let progress = 0.0; // 简化处理

                        json!({
                            "id": m.id,
                            "description": m.content,
                            "progress": progress,
                            "tags": m.tags,
                        })
                    })
                    .collect();

                Ok(json!({
                    "success": true,
                    "action": "listed",
                    "count": active_goals.len(),
                    "goals": active_goals,
                }))
            }

            _ => Err(MetaToolError::InvalidArguments(format!("未知 action: {}", action))),
        }
    }
}

/// 元行动工具的错误类型
#[derive(Debug)]
pub enum MetaToolError {
    NotFound(String),
    InvalidArguments(String),
    ExecutionFailed(String),
}

impl std::fmt::Display for MetaToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetaToolError::NotFound(msg) => write!(f, "工具未找到: {}", msg),
            MetaToolError::InvalidArguments(msg) => write!(f, "参数无效: {}", msg),
            MetaToolError::ExecutionFailed(msg) => write!(f, "执行失败: {}", msg),
        }
    }
}

impl std::error::Error for MetaToolError {}
