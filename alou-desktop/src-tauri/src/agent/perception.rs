//! 智能感知层 (Perception Layer)
//!
//! 负责在每次 Agent 迭代时智能地收集和加载相关上下文信息。
//! 根据用户意图动态决定加载哪些信息源，而非简单地加载所有内容。

use super::memory::{Memory, MemoryManager, MemoryType, Importance};
use super::task::{Task, TaskManager};
use super::goal::{GoalTracker, GoalSummary as GoalInfo};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 用户意图类型
#[derive(Debug, Clone, PartialEq)]
pub enum Intent {
    /// 简单问候
    Greeting,
    /// 代码相关任务
    CodeTask,
    /// 文件操作
    FileOperation,
    /// 继续之前的工作
    ContinueProject,
    /// 需要搜索/查找信息
    SearchQuery,
    /// 计划/组织任务
    Planning,
    /// 学习/理解某事物
    Learning,
    /// 调试/修复问题
    Debugging,
    /// 不确定意图
    Unclear,
}

/// 检索到的上下文信息
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RetrievedContext {
    /// 用户画像摘要
    pub user_profile: Option<String>,
    /// 相关记忆
    pub relevant_memories: Vec<Memory>,
    /// 相关文档内容
    pub relevant_docs: Vec<DocumentContent>,
    /// 工作目录中的相关文件
    pub related_files: Vec<String>,
    /// 活跃目标
    pub active_goals: Vec<GoalInfo>,
    /// 最后一条消息的分析
    pub intent_analysis: String,
}

/// 文档内容
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentContent {
    pub name: String,
    pub content: String,
    pub relevance_score: f32,
}



/// 感知引擎
pub struct PerceptionEngine {
    /// 记忆管理器
    memory_manager: Arc<MemoryManager>,
    /// 任务管理器
    task_manager: Arc<TaskManager>,
    /// 目标追踪器
    goal_tracker: Option<Arc<GoalTracker>>,
    /// 文档加载路径
    docs_path: RwLock<std::path::PathBuf>,
}

impl PerceptionEngine {
    /// 创建新的感知引擎
    pub fn new(memory_manager: Arc<MemoryManager>, task_manager: Arc<TaskManager>) -> Self {
        let docs_path = dirs::home_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join(".alou")
            .join("agents")
            .join("docs");

        Self {
            memory_manager,
            task_manager,
            goal_tracker: None,
            docs_path: RwLock::new(docs_path),
        }
    }

    /// 设置目标追踪器
    pub fn with_goal_tracker(mut self, goal_tracker: Arc<GoalTracker>) -> Self {
        self.goal_tracker = Some(goal_tracker);
        self
    }

    /// 设置文档路径
    pub async fn set_docs_path(&self, path: std::path::PathBuf) {
        let mut docs_path = self.docs_path.write().await;
        *docs_path = path;
    }

    /// 核心方法：收集上下文
    pub async fn gather_context(&self, task: &Task) -> RetrievedContext {
        let last_message = task.messages.last();
        let query = last_message.map(|m| m.content.as_str()).unwrap_or("");

        // 1. 分析意图
        let intent = self.analyze_intent(query).await;
        log::info!("[Perception] 检测到意图: {:?}", intent);

        // 2. 根据意图收集信息
        let mut context = RetrievedContext {
            intent_analysis: format!("检测到意图: {:?}", intent),
            ..Default::default()
        };

        // 始终加载用户画像
        context.user_profile = self.load_user_profile().await;

        match intent {
            Intent::Greeting => {
                // 简单问候：只加载用户画像和少量近期记忆
                context.relevant_memories = self
                    .memory_manager
                    .retrieve("user greeting preference", Some(MemoryType::Preference), 3)
                    .await;
            }

            Intent::CodeTask => {
                // 代码任务：加载编码规范、项目上下文、代码相关记忆
                context.relevant_docs = self
                    .load_documents(&["CODING_STANDARDS.md", "PROJECT_CONTEXT.md"])
                    .await;
                context.relevant_memories = self
                    .memory_manager
                    .retrieve(&format!("{} code programming", query), None, 5)
                    .await;
                context.related_files = self.find_related_files(query).await;
            }

            Intent::FileOperation => {
                // 文件操作：加载文件相关记忆和工作目录结构
                context.relevant_memories = self
                    .memory_manager
                    .retrieve(&format!("{} filesystem", query), None, 5)
                    .await;
                context.related_files = self.list_working_directory().await;
            }

            Intent::ContinueProject => {
                // 继续项目：加载活跃目标、项目文档、近期任务记忆
                context.active_goals = self.load_active_goals().await;
                context.relevant_docs = self.load_all_project_docs().await;
                context.relevant_memories = self
                    .memory_manager
                    .retrieve("project task work", Some(MemoryType::Experience), 5)
                    .await;
            }

            Intent::SearchQuery => {
                // 搜索查询：加载搜索相关记忆，可能需要网络搜索
                context.relevant_memories = self
                    .memory_manager
                    .retrieve(query, None, 8)
                    .await;
            }

            Intent::Planning => {
                // 计划任务：加载计划相关经验和活跃目标
                context.active_goals = self.load_active_goals().await;
                context.relevant_memories = self
                    .memory_manager
                    .retrieve("planning organization task", Some(MemoryType::Experience), 5)
                    .await;
            }

            Intent::Learning => {
                // 学习理解：加载知识类记忆和相关文档
                context.relevant_memories = self
                    .memory_manager
                    .retrieve(query, Some(MemoryType::Knowledge), 8)
                    .await;
                context.relevant_docs = self.search_related_docs(query).await;
            }

            Intent::Debugging => {
                // 调试问题：加载错误经验、调试模式
                context.relevant_memories = self
                    .memory_manager
                    .retrieve(&format!("{} error debug fix", query), Some(MemoryType::Experience), 8)
                    .await;
            }

            Intent::Unclear => {
                // 不确定：最小上下文，让 AI 询问用户
                context.relevant_memories = self
                    .memory_manager
                    .retrieve(query, None, 3)
                    .await;
            }
        }

        // 记录检索结果统计
        log::info!(
            "[Perception] 上下文收集完成: {} 条记忆, {} 个文档, {} 个目标",
            context.relevant_memories.len(),
            context.relevant_docs.len(),
            context.active_goals.len()
        );

        context
    }

    /// 获取用户画像摘要
    pub async fn get_user_profile_summary(&self) -> String {
        self.load_user_profile()
            .await
            .unwrap_or_else(|| "暂无用户画像".to_string())
    }

    /// 获取相关记忆
    pub async fn get_relevant_memories(&self, query: &str, limit: usize) -> Vec<Memory> {
        self.memory_manager.retrieve(query, None, limit).await
    }

    /// 获取活跃目标
    pub async fn get_active_goals(&self) -> Vec<GoalInfo> {
        self.load_active_goals().await
    }

    /// 分析用户意图
    pub(crate) async fn analyze_intent(&self, query: &str) -> Intent {
        let query_lower = query.to_lowercase();

        // 问候意图检测
        let greetings = ["你好", "您好", "hi", "hello", "hey", "在吗", "在不在"];
        if greetings.iter().any(|g| query_lower.contains(g)) && query.len() < 50 {
            return Intent::Greeting;
        }

        // 代码任务检测
        let code_keywords = [
            "代码", "程序", "函数", "类", "bug", "错误", "优化", "重构", "写个", "实现",
            "code", "program", "function", "class", "bug", "error", "optimize", "refactor",
            "implement", "write a", "fix",
        ];
        if code_keywords.iter().any(|k| query_lower.contains(k)) {
            return Intent::CodeTask;
        }

        // 文件操作检测
        let file_keywords = [
            "文件", "读取", "写入", "创建", "删除", "目录", "文件夹",
            "file", "read", "write", "create", "delete", "directory", "folder",
        ];
        if file_keywords.iter().any(|k| query_lower.contains(k)) {
            return Intent::FileOperation;
        }

        // 继续项目检测
        let continue_keywords = [
            "继续", "上次", "之前", "刚才", "resume", "continue", "previous", "last time",
        ];
        if continue_keywords.iter().any(|k| query_lower.contains(k)) {
            return Intent::ContinueProject;
        }

        // 搜索查询检测
        let search_keywords = [
            "搜索", "查找", "查询", "是什么", "什么是", "怎么", "如何",
            "search", "find", "look for", "what is", "how to",
        ];
        if search_keywords.iter().any(|k| query_lower.contains(k)) {
            return Intent::SearchQuery;
        }

        // 计划任务检测
        let planning_keywords = [
            "计划", "安排", "组织", "规划", "目标", "任务",
            "plan", "schedule", "organize", "goal", "task",
        ];
        if planning_keywords.iter().any(|k| query_lower.contains(k)) {
            return Intent::Planning;
        }

        // 学习理解检测
        let learning_keywords = [
            "解释", "说明", "教程", "学习", "理解", "什么是",
            "explain", "tutorial", "learn", "understand", "what is",
        ];
        if learning_keywords.iter().any(|k| query_lower.contains(k)) {
            return Intent::Learning;
        }

        // 调试问题检测
        let debug_keywords = [
            "调试", "报错", "异常", "崩溃", "不工作", "失败",
            "debug", "error", "exception", "crash", "not working", "failed",
        ];
        if debug_keywords.iter().any(|k| query_lower.contains(k)) {
            return Intent::Debugging;
        }

        // 默认不确定
        Intent::Unclear
    }

    /// 加载用户画像
    async fn load_user_profile(&self) -> Option<String> {
        let prefs = self.memory_manager.get_all_preferences().await;

        if prefs.is_empty() {
            return None;
        }

        let profile = prefs
            .iter()
            .map(|p| format!("- {}: {} (置信度: {:.0}%)", p.preference_key, p.preference_value, p.confidence * 100.0))
            .collect::<Vec<_>>()
            .join("\n");

        Some(format!("用户偏好:\n{}", profile))
    }

    /// 加载指定文档
    async fn load_documents(&self, doc_names: &[&str]) -> Vec<DocumentContent> {
        let mut docs = Vec::new();
        let docs_path = self.docs_path.read().await;

        for name in doc_names {
            let path = docs_path.join(name);
            if let Ok(content) = tokio::fs::read_to_string(&path).await {
                docs.push(DocumentContent {
                    name: name.to_string(),
                    content,
                    relevance_score: 1.0,
                });
            }
        }

        docs
    }

    /// 加载所有项目文档
    async fn load_all_project_docs(&self) -> Vec<DocumentContent> {
        let docs_path = self.docs_path.read().await;
        let mut docs = Vec::new();

        if let Ok(entries) = tokio::fs::read_dir(&*docs_path).await {
            let mut entries = entries;
            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                if path.extension().map(|e| e == "md").unwrap_or(false) {
                    if let Ok(content) = tokio::fs::read_to_string(&path).await {
                        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                            docs.push(DocumentContent {
                                name: name.to_string(),
                                content,
                                relevance_score: 0.8,
                            });
                        }
                    }
                }
            }
        }

        docs
    }

    /// 搜索相关文档
    async fn search_related_docs(&self, query: &str) -> Vec<DocumentContent> {
        let all_docs = self.load_all_project_docs().await;
        let query_lower = query.to_lowercase();
        let keywords: HashSet<String> = query_lower
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        let mut scored_docs: Vec<(DocumentContent, f32)> = all_docs
            .into_iter()
            .map(|doc| {
                let content_lower = doc.content.to_lowercase();
                let matches = keywords
                    .iter()
                    .filter(|kw| content_lower.contains(&kw.to_lowercase()))
                    .count() as f32;
                let score = matches / keywords.len().max(1) as f32;
                (doc, score)
            })
            .filter(|(_, score)| *score > 0.0)
            .collect();

        scored_docs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        scored_docs
            .into_iter()
            .take(3)
            .map(|(mut doc, score)| {
                doc.relevance_score = score;
                doc
            })
            .collect()
    }

    /// 查找相关文件
    async fn find_related_files(&self, _query: &str) -> Vec<String> {
        // 简单实现：返回当前工作目录的文件列表
        self.list_working_directory().await
    }

    /// 列出工作目录
    async fn list_working_directory(&self) -> Vec<String> {
        let current_dir = std::env::current_dir().ok();

        if let Some(dir) = current_dir {
            if let Ok(entries) = tokio::fs::read_dir(dir).await {
                let mut entries = entries;
                let mut files = Vec::new();

                while let Ok(Some(entry)) = entries.next_entry().await {
                    if let Ok(metadata) = entry.metadata().await {
                        let name = entry.file_name().to_string_lossy().to_string();
                        if metadata.is_dir() {
                            files.push(format!("{}/", name));
                        } else {
                            files.push(name);
                        }
                    }
                }

                return files;
            }
        }

        Vec::new()
    }

    /// 加载活跃目标
    async fn load_active_goals(&self) -> Vec<GoalInfo> {
        // 优先使用 GoalTracker
        if let Some(ref tracker) = self.goal_tracker {
            let goals = tracker.get_active_goals().await;
            return goals.into_iter().map(|g| GoalInfo {
                id: g.id,
                description: g.description,
                progress: g.progress,
                priority: format!("{:?}", g.priority),
                status: "active".to_string(),
            }).collect();
        }

        // 降级：从记忆中检索
        let goal_memories = self
            .memory_manager
            .retrieve("active goal task objective", None, 10)
            .await;

        goal_memories
            .into_iter()
            .filter(|m| m.tags.contains(&"goal".to_string()))
            .map(|m| GoalInfo {
                id: m.id.clone(),
                description: m.content.clone(),
                progress: m
                    .metadata
                    .get("progress")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0) as f32,
                priority: format!("{:?}", m.importance),
                status: "active".to_string(),
            })
            .collect()
    }

    /// 获取记忆管理器
    pub fn memory_manager(&self) -> Arc<MemoryManager> {
        self.memory_manager.clone()
    }

    /// 获取任务管理器
    pub fn task_manager(&self) -> Arc<TaskManager> {
        self.task_manager.clone()
    }

    /// 获取目标追踪器
    pub fn goal_tracker(&self) -> Option<Arc<GoalTracker>> {
        self.goal_tracker.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intent_detection() {
        // 测试意图检测逻辑
        let test_cases = vec![
            ("你好", Intent::Greeting),
            ("帮我写个函数", Intent::CodeTask),
            ("读取文件", Intent::FileOperation),
            ("继续上次的工作", Intent::ContinueProject),
            ("搜索相关信息", Intent::SearchQuery),
        ];

        // 注意：这里只是示例，实际测试需要初始化 PerceptionEngine
        for (query, expected) in test_cases {
            println!("查询 '{}' 应该匹配 {:?}", query, expected);
        }
    }
}
