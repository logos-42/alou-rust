//! Agent Command 系统 - 主动控制命令
//!
//! Command 允许用户、系统或 AI 主动控制 Agent 行为
//! 支持：停止、重试、暂停、注入消息、强制工具等

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Agent 命令 - 主动控制 Agent 行为
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentCommand {
    // === 流程控制 ===
    /// 停止当前执行
    Stop,
    /// 暂停执行
    Pause,
    /// 恢复执行
    Resume,
    /// 重试最后一次请求
    Retry,

    // === 消息注入 ===
    /// 注入系统消息
    InjectSystemMessage {
        content: String,
    },
    /// 注入用户消息
    InjectUserMessage {
        content: String,
    },
    /// 注入助手消息（用于修正）
    InjectAssistantMessage {
        content: String,
    },

    // === 工具控制 ===
    /// 强制执行特定工具
    ForceTool {
        tool_name: String,
        arguments: Value,
    },
    /// 跳过下一个工具调用
    SkipNextTool,
    /// 清除所有待处理工具
    ClearPendingTools,

    // === 记忆控制 ===
    /// 清除会话历史
    ClearHistory,
    /// 导出会话
    ExportSession,
    /// 导入会话
    ImportSession {
        history: Vec<serde_json::Value>,
    },

    // === 模型控制 ===
    /// 切换 AI 模型
    SwitchModel {
        model_name: String,
    },
    /// 调整温度参数
    SetTemperature {
        temperature: f32,
    },

    // === 工作流控制 ===
    /// 启动工作流
    StartWorkflow {
        workflow_id: String,
        input: Value,
    },
    /// 暂停工作流
    PauseWorkflow {
        execution_id: String,
    },
    /// 取消工作流
    CancelWorkflow {
        execution_id: String,
    },

    // === 调试控制 ===
    /// 启用调试模式
    EnableDebug,
    /// 禁用调试模式
    DisableDebug,
    /// 获取当前状态
    GetStatus,

    // === 自定义命令 ===
    /// 自定义命令（支持扩展）
    Custom {
        name: String,
        params: Value,
    },
}

impl AgentCommand {
    /// 创建停止命令
    pub fn stop() -> Self {
        Self::Stop
    }

    /// 创建重试命令
    pub fn retry() -> Self {
        Self::Retry
    }

    /// 创建暂停命令
    pub fn pause() -> Self {
        Self::Pause
    }

    /// 创建恢复命令
    pub fn resume() -> Self {
        Self::Resume
    }

    /// 创建注入系统消息命令
    pub fn inject_system(content: String) -> Self {
        Self::InjectSystemMessage { content }
    }

    /// 创建注入用户消息命令
    pub fn inject_user(content: String) -> Self {
        Self::InjectUserMessage { content }
    }

    /// 创建强制工具命令
    pub fn force_tool(name: String, args: Value) -> Self {
        Self::ForceTool {
            tool_name: name,
            arguments: args,
        }
    }

    /// 创建切换模型命令
    pub fn switch_model(name: String) -> Self {
        Self::SwitchModel { model_name: name }
    }

    /// 创建自定义命令
    pub fn custom(name: String, params: Value) -> Self {
        Self::Custom { name, params }
    }

    /// 判断是否是终止类命令
    pub fn is_terminating(&self) -> bool {
        matches!(self, Self::Stop | Self::CancelWorkflow { .. })
    }

    /// 判断是否是暂停类命令
    pub fn is_pausing(&self) -> bool {
        matches!(self, Self::Pause | Self::PauseWorkflow { .. })
    }

    /// 判断是否是注入消息类命令
    pub fn is_injection(&self) -> bool {
        matches!(
            self,
            Self::InjectSystemMessage { .. }
                | Self::InjectUserMessage { .. }
                | Self::InjectAssistantMessage { .. }
        )
    }
}

/// Command 队列 - 管理待处理的命令
#[derive(Debug, Default)]
pub struct CommandQueue {
    commands: Vec<AgentCommand>,
    paused: bool,
}

impl CommandQueue {
    /// 创建新的命令队列
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
            paused: false,
        }
    }

    /// 添加命令到队列
    pub fn push(&mut self, command: AgentCommand) {
        self.commands.push(command);
    }

    /// 添加高优先级命令（插入到队首）
    pub fn push_high_priority(&mut self, command: AgentCommand) {
        self.commands.insert(0, command);
    }

    /// 从队列取出下一个命令
    pub fn pop(&mut self) -> Option<AgentCommand> {
        if self.paused {
            return None;
        }
        if !self.commands.is_empty() {
            Some(self.commands.remove(0))
        } else {
            None
        }
    }

    /// 获取队列长度
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    /// 判断队列是否为空
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    /// 暂停命令处理
    pub fn set_paused(&mut self, paused: bool) {
        self.paused = paused;
    }

    /// 是否处于暂停状态
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// 清除所有命令
    pub fn clear(&mut self) {
        self.commands.clear();
    }

    /// 获取所有命令（用于调试）
    pub fn peek_all(&self) -> &[AgentCommand] {
        &self.commands
    }
}

/// Command 执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    pub command: String,
    pub success: bool,
    pub message: Option<String>,
    pub data: Option<Value>,
}

impl CommandResult {
    pub fn success(command: String, message: Option<String>) -> Self {
        Self {
            command,
            success: true,
            message,
            data: None,
        }
    }

    pub fn failure(command: String, message: Option<String>) -> Self {
        Self {
            command,
            success: false,
            message,
            data: None,
        }
    }

    pub fn with_data(mut self, data: Value) -> Self {
        self.data = Some(data);
        self
    }
}

/// Command 执行错误
#[derive(Debug, Clone)]
pub enum CommandError {
    /// 未知命令
    UnknownCommand(String),
    /// 无效参数
    InvalidParameter(String),
    /// 执行失败
    ExecutionFailed(String),
    /// 系统错误
    SystemError(String),
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandError::UnknownCommand(cmd) => write!(f, "Unknown command: {}", cmd),
            CommandError::InvalidParameter(msg) => write!(f, "Invalid parameter: {}", msg),
            CommandError::ExecutionFailed(msg) => write!(f, "Execution failed: {}", msg),
            CommandError::SystemError(msg) => write!(f, "System error: {}", msg),
        }
    }
}

impl std::error::Error for CommandError {}

/// Command 处理器 Trait
#[async_trait::async_trait]
pub trait CommandHandler: Send + Sync {
    /// 处理命令
    async fn handle_command(
        &self,
        command: &AgentCommand,
        ctx: &mut CommandContext,
    ) -> Result<CommandResult, CommandError>;
}

/// Command 上下文
pub struct CommandContext {
    pub session_id: String,
    pub data: Value,
}

impl CommandContext {
    pub fn new(session_id: String) -> Self {
        Self {
            session_id,
            data: Value::Null,
        }
    }

    pub fn with_data(mut self, data: Value) -> Self {
        self.data = data;
        self
    }
}

/// 默认 Command 处理器
pub struct DefaultCommandHandler;

impl DefaultCommandHandler {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DefaultCommandHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl CommandHandler for DefaultCommandHandler {
    async fn handle_command(
        &self,
        command: &AgentCommand,
        _ctx: &mut CommandContext,
    ) -> Result<CommandResult, CommandError> {
        // 默认处理器只记录命令，实际处理由 SessionActor 完成
        Ok(CommandResult::success(
            format!("{:?}", command),
            Some("Command received, will be processed by SessionActor".to_string()),
        ))
    }
}
