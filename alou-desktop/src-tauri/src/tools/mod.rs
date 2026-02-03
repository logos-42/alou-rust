//! 工具执行器模块
//!
//! 提供丰富的工具生态系统，支持文件操作、终端命令、搜索、计划管理等功能

use std::sync::Arc;

pub mod executor;
pub mod registry;
pub mod bash;
pub mod filesystem;
pub mod search;
pub mod network;
pub mod system;
pub mod plan;
pub mod todolist;
pub mod agent_skills;
pub mod agent_collaboration;
pub mod tool_creation;

// 重新导出核心类型和接口
pub use executor::{ToolExecutor, ToolResult, ToolError, ToolContext};
// ToolCategory is defined in this module, not registry
pub use registry::{ToolRegistry, ToolDefinition};
pub use filesystem::{FileSystemTool, FileOperation};
pub use search::{SearchTool, SearchPattern};
pub use bash::{BashTool, CommandResult};
pub use plan::{PlanTool, TaskPlan, PlanStep};
pub use todolist::{TodoListTool, TodoItem, TodoStatus};
pub use agent_skills::{AgentSkillsTool, AgentSkill, SkillMetadata, SkillExecutionContext, SkillExecutionResult};
pub use agent_collaboration::{AgentCollaborationTool, CollaborationSession, PubSubChatMessage, SessionStatus, MessageType, ParticipantInfo, ParticipantRole};
pub use tool_creation::{ToolCreationTool, ToolDefinition as CreatedToolDefinition, ToolType, ParameterDef, ToolUsageRecord, AgentToolUsageRecord, AgentToolRegistry, DynamicToolExecutor, DynamicToolResult};

// 工具分类枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ToolCategory {
    /// 文件系统操作
    FileSystem,
    /// 搜索和查找
    Search,
    /// 终端命令执行
    Terminal,
    /// 网络操作
    Network,
    /// 系统信息
    System,
    /// 计划和任务管理
    Planning,
    /// 待办事项
    Todo,
    /// Skills 系统
    Skills,
    /// 智能体自动化
    Automation,
    /// 通信协作
    Communication,
    /// 开发工具
    Development,
    /// 其他
    Other,
}

// 工具优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub enum ToolPriority {
    /// 最高优先级
    Critical = 0,
    /// 高优先级
    High = 1,
    /// 中等优先级
    Medium = 2,
    /// 低优先级
    Low = 3,
}

// 工具状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ToolStatus {
    /// 可用
    Available,
    /// 不可用
    Unavailable,
    /// 需要权限
    RequiresPermission,
    /// 正在执行
    Executing,
    /// 出错
    Error,
}

/// 工具元信息
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolMetadata {
    /// 工具ID
    pub id: String,
    /// 工具名称
    pub name: String,
    /// 工具描述
    pub description: String,
    /// 工具分类
    pub category: ToolCategory,
    /// 工具优先级
    pub priority: ToolPriority,
    /// 工具状态
    pub status: ToolStatus,
    /// 工具版本
    pub version: String,
    /// 作者
    pub author: String,
    /// 创建时间
    pub created_at: i64,
    /// 更新时间
    pub updated_at: i64,
    /// 依赖项
    pub dependencies: Vec<String>,
    /// 平台兼容性
    pub platforms: Vec<String>,
    /// 权限要求
    pub permissions: Vec<String>,
}

/// 工具执行上下文
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExecutionContext {
    /// 会话ID
    pub session_id: String,
    /// 用户ID
    pub user_id: Option<String>,
    /// 执行目录
    pub working_directory: Option<String>,
    /// 环境变量
    pub environment: std::collections::HashMap<String, String>,
    /// 超时时间（秒）
    pub timeout_seconds: Option<u64>,
    /// 权限上下文
    pub permissions: Vec<String>,
    /// 执行时间戳
    pub timestamp: i64,
}

/// 工具配置
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolConfig {
    /// 是否启用工具
    pub enabled: bool,
    /// 最大并发执行数
    pub max_concurrent: usize,
    /// 默认超时时间
    pub default_timeout: u64,
    /// 缓存配置
    pub cache_enabled: bool,
    /// 安全模式
    pub safe_mode: bool,
    /// 调试模式
    pub debug_mode: bool,
}

impl Default for ToolConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_concurrent: 5,
            default_timeout: 30,
            cache_enabled: true,
            safe_mode: true,
            debug_mode: false,
        }
    }
}

/// 初始化所有工具
pub async fn initialize_tools() -> Result<ToolRegistry, Box<dyn std::error::Error>> {
    let mut registry = ToolRegistry::new();

    // 注册文件系统工具
    let fs_tool = Arc::new(FileSystemTool::new());
    registry.register(fs_tool).await?;

    // 注册搜索工具
    let search_tool = Arc::new(SearchTool::new());
    registry.register(search_tool).await?;

    // 注册Bash工具
    let bash_tool = Arc::new(BashTool::new());
    registry.register(bash_tool).await?;

    // 注册计划工具
    let plan_tool = Arc::new(PlanTool::new());
    registry.register(plan_tool).await?;

    // 注册待办事项工具
    let todo_tool = Arc::new(TodoListTool::new());
    registry.register(todo_tool).await?;

    // 注册Agent Skills工具（替代旧的Skills工具）
    let agent_skills_tool = Arc::new(AgentSkillsTool::new()?);
    registry.register(agent_skills_tool).await?;

    // 注册智能体协作工具（使用 IPFS PubSub）
    let ipfs_api_url = std::env::var("IPFS_API_URL").unwrap_or_else(|_| "http://127.0.0.1:5001".to_string());
    let agent_collab_tool = Arc::new(AgentCollaborationTool::new(ipfs_api_url));
    registry.register(agent_collab_tool).await?;

    // 注册工具创建和记录工具
    let tool_creation_tool = Arc::new(ToolCreationTool::new());
    registry.register(tool_creation_tool).await?;

    // 注册动态工具执行器
    let dynamic_tool_executor = Arc::new(DynamicToolExecutor::new());
    registry.register(dynamic_tool_executor).await?;

    Ok(registry)
}

/// 获取默认工具配置
pub fn default_tool_config() -> ToolConfig {
    ToolConfig::default()
}

/// 创建默认执行上下文
pub fn create_execution_context(session_id: String) -> ExecutionContext {
    ExecutionContext {
        session_id,
        user_id: None,
        working_directory: std::env::current_dir()
            .ok()
            .and_then(|p| p.to_str().map(|s| s.to_string())),
        environment: std::env::vars().collect(),
        timeout_seconds: Some(30),
        permissions: vec!["read".to_string(), "write".to_string()],
        timestamp: chrono::Utc::now().timestamp(),
    }
}