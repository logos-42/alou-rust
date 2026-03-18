//! Agent 执行器模块
//!
//! 实现了 Ralph Loop 执行架构：
//! - **Perceive**: 感知环境，智能检索上下文
//! - **Reason**: 推理决策，带强制反思层
//! - **Act**: 执行行动，支持工具调用和元行动
//! - **Integrate**: 整合结果，管理循环状态
//!
//! ## 架构图
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                     RalphLoopExecutor                        │
//! ├─────────────┬─────────────┬─────────────┬───────────────────┤
//! │  Perception │  Reasoning  │   Action    │   Integration     │
//! │    Layer    │    Layer    │    Layer    │     Layer         │
//! ├─────────────┼─────────────┼─────────────┼───────────────────┤
//! │ • gather    │ • analyze   │ • tool_call │ • update_state    │
//! │   context   │ • reflect   │ • create_   │ • check_term      │
//! │ • analyze   │ • decide    │   goal      │ • build_summary   │
//! │   intent    │             │ • update_   │                   │
//! │             │             │   goal      │                   │
//! └─────────────┴─────────────┴─────────────┴───────────────────┘
//! ```

pub mod types;
pub mod perception;
pub mod reasoning;
pub mod action;
pub mod integration;
pub mod core;

// 重新导出主要类型
pub use types::{
    Thought, Action, Reflection, InformationAssessment, TaskProgress,
    ActionResult, EnvironmentState, ExecutionResult, ExecutorError,
    ContextDocuments, Task,
};
pub use perception::PerceptionLayer;
pub use reasoning::ReasoningLayer;
pub use action::ActionLayer;
pub use integration::IntegrationLayer;
pub use core::{RalphLoopExecutor, ExecutorCore, RalphLoopExecutorBuilder, TaskFinalResult};
