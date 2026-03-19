//! 执行器核心类型定义
//!
//! 包含 Agent Reasoning Loop 所需的所有数据结构

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::agent::ai_client::AiMessage;
use crate::agent::perception::RetrievedContext;
pub use crate::tasks::types::Task;

/// 🔥 上下文文档缓存
#[derive(Debug, Clone, Default)]
pub struct ContextDocuments {
    pub soul: Option<String>,
    pub tasks: Option<Vec<Task>>,
    pub memory: Option<String>,
}

/// 环境状态（Perceive 层输出）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentState {
    pub task_id: String,
    pub messages: Vec<AiMessage>,
    pub iteration_count: u32,
    pub tool_call_count: u32,
    pub available_tools: Vec<String>,
    /// 智能感知层检索到的上下文
    pub retrieved_context: RetrievedContext,
    /// 🔥 文档缓存（SOUL.md, TASKS.md, MEMORY.md）
    #[serde(skip)]
    pub context_documents: ContextDocuments,
    /// 🔥 前端传入的系统提示（优先使用）
    #[serde(skip)]
    pub system_prompt: Option<String>,
}

/// 推理结果（Reason 层输出 - LLM 单次调用）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thought {
    /// 分析过程
    pub analysis: String,
    /// 强制反思
    pub reflection: Option<Reflection>,
    /// 下一步行动
    pub action: Action,
}

/// 反思层结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reflection {
    /// 信息充分性评估
    pub information_assessment: InformationAssessment,
    /// 任务进度评估
    pub task_progress: TaskProgress,
    /// 决策置信度 (0.0-1.0)
    pub confidence: f32,
    /// 决策理由
    pub reasoning: String,
    /// 建议的下一步
    pub suggested_next_step: String,
}

/// 信息充分性评估
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InformationAssessment {
    /// 状态："sufficient" | "insufficient" | "uncertain"
    pub status: String,
    /// 缺少的信息
    #[serde(default)]
    pub missing_info: Vec<String>,
    /// 建议："proceed" | "gather_more" | "ask_user"
    pub suggestion: String,
}

/// 任务进度评估
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskProgress {
    /// 整体进度 (0.0-1.0)
    pub overall_progress: f32,
    /// 已完成的步骤
    #[serde(default)]
    pub completed_steps: Vec<String>,
    /// 待处理的步骤
    #[serde(default)]
    pub pending_steps: Vec<String>,
    /// 预计剩余迭代次数
    pub estimated_remaining_iterations: u32,
}

impl Default for InformationAssessment {
    fn default() -> Self {
        Self {
            status: "uncertain".to_string(),
            missing_info: vec![],
            suggestion: "proceed".to_string(),
        }
    }
}

impl Default for TaskProgress {
    fn default() -> Self {
        Self {
            overall_progress: 0.0,
            completed_steps: vec![],
            pending_steps: vec![],
            estimated_remaining_iterations: 0,
        }
    }
}

impl Default for Reflection {
    fn default() -> Self {
        Self {
            information_assessment: InformationAssessment::default(),
            task_progress: TaskProgress::default(),
            confidence: 0.5,
            reasoning: String::new(),
            suggested_next_step: String::new(),
        }
    }
}

/// 行动类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Action {
    /// 调用工具（支持并发）
    ToolCall {
        tool: String,
        args: Value,
        id: String,
    },
    /// 完成任务
    Complete(String),
    /// 继续思考
    Continue,
    /// 失败
    Fail(String),
    /// 🔥 新增：询问用户（信息不足时）
    AskUser { question: String },
    /// 🔥 新增：创建目标
    CreateGoal { description: String, priority: String },
    /// 🔥 新增：更新目标进度
    UpdateGoal { goal_id: String, progress: f32 },
}

/// 行动执行结果
#[derive(Debug, Clone)]
pub struct ActionResult {
    pub action_id: String,
    pub tool_name: String,
    pub success: bool,
    pub output: Option<Value>,
    pub error: Option<String>,
}

/// 执行结果
#[derive(Debug)]
pub enum ExecutionResult {
    /// 已完成
    Completed(String),
    /// 需要更多迭代
    NeedsMoreIterations,
    /// 失败
    Failed(String),
}

/// 执行器错误
#[derive(Debug)]
pub enum ExecutorError {
    TaskNotFound(String),
    AiError(String),
    ToolError(String),
    InternalError(String),
}

impl std::fmt::Display for ExecutorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutorError::TaskNotFound(msg) => write!(f, "任务未找到: {}", msg),
            ExecutorError::AiError(msg) => write!(f, "AI 错误: {}", msg),
            ExecutorError::ToolError(msg) => write!(f, "工具错误: {}", msg),
            ExecutorError::InternalError(msg) => write!(f, "内部错误: {}", msg),
        }
    }
}

impl std::error::Error for ExecutorError {}
