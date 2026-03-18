//! 统一 Tool trait 定义
//!
//! 所有工具都必须实现这个 trait，实现跨 Runtime 的工具共享
//!
//! ## 设计说明
//!
//! 项目中有两个 Tool trait：
//!
//! 1. **`Tool` (本文件)** - 简单工具 trait
//!    - 用于 `ToolBus`（媒体工具等轻量级工具）
//!    - 方法签名简单：`execute(&self, args, context)`
//!    - 返回 `Result<Value, String>`
//!
//! 2. **`ToolExecutor`** - 完整工具 trait
//!    - 用于 `ToolRegistry`（核心工具）
//!    - 包含元数据、验证、帮助等完整功能
//!    - 返回 `Result<ToolResult, ToolError>`
//!
//! ## 何时使用哪个？
//!
//! - **简单工具**（如媒体生成）→ 实现 `Tool`
//! - **复杂工具**（如文件系统、Bash）→ 实现 `ToolExecutor`

use serde_json::Value;

/// 工具执行上下文（简化版）
///
/// 注意：`tools/executor.rs` 中有更完整的 `ExecutionContext`
/// 这里使用简化版以减少依赖
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolContext {
    /// 会话 ID
    pub session_id: String,
    /// 用户 ID（可选）
    pub user_id: Option<String>,
    /// 工作目录
    pub working_directory: Option<String>,
    /// 超时时间（秒）
    pub timeout_seconds: Option<u64>,
}

impl Default for ToolContext {
    fn default() -> Self {
        Self {
            session_id: String::new(),
            user_id: None,
            working_directory: None,
            timeout_seconds: Some(30),
        }
    }
}

/// 统一 Tool trait
///
/// 所有工具（无论属于哪个 Runtime）都必须实现这个 trait
#[async_trait::async_trait]
pub trait Tool: Send + Sync {
    /// 工具名称（唯一标识符）
    fn name(&self) -> &str;
    
    /// 工具描述
    fn description(&self) -> &str;
    
    /// 执行工具（带上下文）
    async fn execute(&self, args: Value, context: &ToolContext) -> Result<Value, String>;
    
    /// 执行工具（简化版，无上下文）
    async fn execute_simple(&self, args: Value) -> Result<Value, String> {
        self.execute(args, &ToolContext::default()).await
    }
}
