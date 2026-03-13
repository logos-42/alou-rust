//! 渐进式迁移指南
//!
//! 本文档说明如何逐步将现有代码迁移到新的 Session Actor 架构
//!
//! ## 迁移策略
//!
//! ### 阶段 1: 并行运行（当前状态）
//! - 旧的 WorkflowState 继续工作
//! - 新的 SessionRouter 已注册但未使用
//! - 两者互不干扰
//!
//! ### 阶段 2: 新功能使用新架构
//! - 新的 agent 对话使用 SessionActor
//! - 旧功能保持不变
//!
//! ### 阶段 3: 迁移现有功能
//! - 逐步将现有功能迁移到新架构
//! - 验证稳定性后移除旧代码
//!
//! ## 使用示例
//!
//! ### 在 Tauri 命令中使用 SessionActor
//!
//! ```rust
//! use crate::runtime::{SessionRouter, SessionMessage, MessageMetadata};
//!
//! #[tauri::command]
//! async fn send_message_to_session(
//!     session_id: String,
//!     content: String,
//!     router: tauri::State<'_, Arc<SessionRouter>>,
//! ) -> Result<String, String> {
//!     // 创建消息
//!     let msg = SessionMessage::UserMessage {
//!         content,
//!         metadata: MessageMetadata::default(),
//!     };
//!
//!     // 获取或创建 Actor
//!     let handle = router.get_or_create(&session_id).await
//!         .map_err(|e| e.to_string())?;
//!
//!     // 发送消息
//!     handle.send(msg).await
//!         .map_err(|e| e.to_string())?;
//!
//!     Ok("消息已发送".to_string())
//! }
//! ```
//!
//! ### 创建独立 session
//!
//! ```rust
//! // 新的 session 完全隔离
//! let handle = router.get_or_create("new_session_id").await?;
//!
//! // 发送用户消息
//! handle.send(SessionMessage::UserMessage {
//!     content: "你好".to_string(),
//!     metadata: MessageMetadata::default(),
//! }).await?;
//! ```

/// 迁移状态跟踪
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MigrationPhase {
    /// 阶段 1: 并行运行
    Phase1_Parallel,
    /// 阶段 2: 新功能使用新架构
    Phase2_NewFeatures,
    /// 阶段 3: 迁移现有功能
    Phase3_Migration,
    /// 阶段 4: 完成（移除旧代码）
    Phase4_Complete,
}

impl Default for MigrationPhase {
    fn default() -> Self {
        // 当前默认阶段 1
        Self::Phase1_Parallel
    }
}
