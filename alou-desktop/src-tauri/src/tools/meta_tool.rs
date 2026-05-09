//! 元行动工具 - Meta-Action Tools
//!
//! 提供高级认知能力工具：
//! - memory_search: 搜索长期记忆
//! - read_document: 读取文档
//! - manage_goal: 管理目标
//! - remember: 记录到记忆

use serde_json::{Value, json};
use std::sync::Arc;
use crate::agent::memory::{MemoryManager, MemoryType, Importance};
use crate::agent::task::TaskManager;
use crate::agent::perception::PerceptionEngine;
use crate::agent::embedding::{EmbeddingService, SemanticMemorySearch};

/// 元行动工具集合
pub struct MetaActionTools {
    memory_manager: Arc<MemoryManager>,
    perception_engine: Option<Arc<PerceptionEngine>>,
    embedding_service: Option<Arc<EmbeddingService>>,
}

impl MetaActionTools {
    /// 创建新的元行动工具集合
    pub fn new(memory_manager: Arc<MemoryManager>, task_manager: Arc<TaskManager>) -> Self {
        let perception_engine = Arc::new(PerceptionEngine::new(
            memory_manager.clone(),
            task_manager,
        ));

        Self {
            memory_manager,
            perception_engine: Some(perception_engine),
            embedding_service: None,
        }
    }

    /// 创建带 Embedding 的元行动工具
    pub fn with_embedding(
        memory_manager: Arc<MemoryManager>,
        task_manager: Arc<TaskManager>,
        embedding_config: crate::agent::embedding::EmbeddingConfig,
    ) -> Self {
        let perception_engine = Arc::new(PerceptionEngine::new(
            memory_manager.clone(),
            task_manager,
        ));

        let embedding_service = Arc::new(EmbeddingService::new(embedding_config));

        Self {
            memory_manager,
            perception_engine: Some(perception_engine),
            embedding_service: Some(embedding_service),
        }
    }

    /// 创建简化的元行动工具（无感知引擎）
    pub fn new_simple(memory_manager: Arc<MemoryManager>) -> Self {
        Self {
            memory_manager,
            perception_engine: None,
            embedding_service: None,
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
                    }
                },
                "required": ["doc_name"]
            }
        })
    }

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

        // 🔥 使用 Embedding 语义搜索（如果可用）
        let memories = if let Some(ref embedding_service) = self.embedding_service {
            match self.semantic_memory_search(query, limit, embedding_service).await {
                Ok(memories) => memories,
                Err(e) => {
                    log::warn!("[MetaTool] 语义搜索失败，回退到关键词搜索：{}", e);
                    self.memory_manager.retrieve(query, memory_type, limit).await
                }
            }
        } else {
            // 无 embedding 服务，使用传统关键词搜索
            self.memory_manager.retrieve(query, memory_type, limit).await
        };

        Ok(json!({
            "success": true,
            "count": memories.len(),
            "query": query,
            "memories": memories.iter().map(|m| json!({
                "id": m.id,
                "type": format!("{:?}", m.memory_type),
                "content": m.content.chars().take(200).collect::<String>(),
                "importance": format!("{:?}", m.importance),
            })).collect::<Vec<_>>()
        }))
    }

    /// 🔥 语义记忆搜索（基于 Embedding）
    async fn semantic_memory_search(
        &self,
        query: &str,
        limit: usize,
        embedding_service: &EmbeddingService,
    ) -> Result<Vec<crate::agent::memory::Memory>, MetaToolError> {
        // 1. 生成 query 的 embedding
        let query_embedding = embedding_service
            .embed(query)
            .await
            .map_err(|e| MetaToolError::ExecutionFailed(format!("Embedding 生成失败：{}", e)))?;

        // 2. 获取所有记忆
        let all_memories = self.memory_manager.get_all_memories().await;

        // 3. 计算相似度并排序
        let mut scored_memories: Vec<(crate::agent::memory::Memory, f32)> = all_memories
            .iter()
            .map(|memory| {
                // 如果有缓存的 embedding，直接计算相似度
                if let Some(ref cached_embedding) = memory.embedding {
                    let similarity = EmbeddingService::cosine_similarity(&query_embedding, cached_embedding);
                    (memory.clone(), similarity)
                } else {
                    // 没有缓存，使用关键词搜索作为 fallback
                    let keyword_score = if memory.content.to_lowercase().contains(&query.to_lowercase())
                        || memory.tags.iter().any(|t| t.to_lowercase().contains(&query.to_lowercase()))
                    {
                        0.5
                    } else {
                        0.0
                    };
                    (memory.clone(), keyword_score)
                }
            })
            .collect();

        // 按相似度降序排序
        scored_memories.sort_by(|a, b| {
            b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
        });

        // 4. 返回 top_k
        let results: Vec<crate::agent::memory::Memory> = scored_memories
            .into_iter()
            .take(limit)
            .map(|(memory, _)| memory)
            .collect();

        log::info!("[MetaTool:SemanticSearch] 找到 {} 条相关记忆", results.len());

        Ok(results)
    }

    async fn read_document(&self, args: Value) -> Result<Value, MetaToolError> {
        let doc_name = args["doc_name"].as_str()
            .ok_or_else(|| MetaToolError::InvalidArguments("缺少 doc_name 参数".to_string()))?;

        log::info!("[MetaTool] read_document: {}", doc_name);

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
                    "content": content.chars().take(2000).collect::<String>(),
                    "truncated": content.len() > 2000,
                }));
            }
        }

        Err(MetaToolError::ExecutionFailed(format!("文档 '{}' 未找到", doc_name)))
    }

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
        }))
    }

    async fn manage_goal(&self, args: Value) -> Result<Value, MetaToolError> {
        let action = args["action"].as_str()
            .ok_or_else(|| MetaToolError::InvalidArguments("缺少 action 参数".to_string()))?;
        
        let description = args["description"].as_str()
            .ok_or_else(|| MetaToolError::InvalidArguments("缺少 description 参数".to_string()))?;

        match action {
            "create" => {
                let priority = args["priority"].as_str().unwrap_or("medium");
                
                let id = self.memory_manager.create_memory(
                    MemoryType::Experience,
                    format!("目标: {}", description),
                    vec!["goal".to_string(), "active".to_string()],
                    Importance::High,
                ).await;

                self.memory_manager.record_preference(
                    "system".to_string(),
                    format!("goal_{}_progress", id),
                    "0.0".to_string(),
                    1.0,
                ).await;

                Ok(json!({
                    "success": true,
                    "goal_id": id,
                    "description": description,
                    "priority": priority,
                }))
            }

            "update" => {
                let progress = args["progress"].as_f64().unwrap_or(0.0);
                
                self.memory_manager.record_preference(
                    "system".to_string(),
                    format!("goal_{}_progress", description),
                    progress.to_string(),
                    1.0,
                ).await;

                Ok(json!({
                    "success": true,
                    "goal_id": description,
                    "progress": progress,
                }))
            }

            "complete" => {
                self.memory_manager.record_preference(
                    "system".to_string(),
                    format!("goal_{}_progress", description),
                    "1.0".to_string(),
                    1.0,
                ).await;

                let _ = self.memory_manager.create_memory(
                    MemoryType::Experience,
                    format!("已完成目标 (ID: {})", description),
                    vec!["goal".to_string(), "completed".to_string()],
                    Importance::Medium,
                ).await;

                Ok(json!({
                    "success": true,
                    "goal_id": description,
                }))
            }

            "list_active" => {
                let goals = self.memory_manager
                    .retrieve("goal active", None, 10)
                    .await;

                Ok(json!({
                    "success": true,
                    "count": goals.len(),
                    "goals": goals.iter().map(|m| json!({
                        "id": m.id,
                        "description": m.content,
                        "tags": m.tags,
                    })).collect::<Vec<_>>(),
                }))
            }

            _ => Err(MetaToolError::InvalidArguments(format!("未知 action: {}", action))),
        }
    }
}

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
