//! 工作流执行器类型定义
//!
//! 包含所有工作流执行相关的结构体、枚举和类型定义

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 执行状态枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ExecutionStatus {
    /// 等待执行
    Pending,
    /// 正在执行
    Running,
    /// 执行完成
    Completed,
    /// 执行失败
    Failed,
    /// 已暂停
    Paused,
    /// 已取消
    Cancelled,
}

/// 步骤执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    pub step_id: String,
    pub status: ExecutionStatus,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub started_at: i64,
    pub completed_at: Option<i64>,
    pub execution_time_ms: Option<u64>,
    pub retry_count: u32,
    pub max_retries: u32,
}

/// 工作流执行上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowExecution {
    pub execution_id: String,
    pub workflow_id: String,
    pub status: ExecutionStatus,
    pub current_step: Option<String>,
    pub progress: f32, // 0.0 to 1.0
    pub step_results: HashMap<String, StepResult>,
    pub started_at: i64,
    pub completed_at: Option<i64>,
    pub total_execution_time_ms: Option<u64>,
    pub error: Option<String>,
}

/// 执行事件枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionEvent {
    /// 执行开始
    Started { execution_id: String, workflow_id: String },
    /// 步骤开始
    StepStarted { execution_id: String, step_id: String },
    /// 步骤完成
    StepCompleted { execution_id: String, step_result: StepResult },
    /// 步骤失败
    StepFailed { execution_id: String, step_result: StepResult },
    /// 执行进度更新
    ProgressUpdated { execution_id: String, progress: f32, current_step: Option<String> },
    /// 执行完成
    Completed { execution_id: String, result: serde_json::Value },
    /// 执行失败
    Failed { execution_id: String, error: String },
    /// 执行暂停
    Paused { execution_id: String },
    /// 执行恢复
    Resumed { execution_id: String },
    /// 执行取消
    Cancelled { execution_id: String },
}

/// 重试配置
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RetryConfig {
    /// 最大重试次数
    pub max_retries: u32,
    /// 初始重试延迟（毫秒）
    pub initial_delay_ms: u64,
    /// 最大重试延迟（毫秒）
    pub max_delay_ms: u64,
    /// 重试倍数
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay_ms: 1000,
            max_delay_ms: 30000,
            backoff_multiplier: 2.0,
        }
    }
}

/// Ralph Loop 执行历史记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RalphLoopIterationHistory {
    /// 迭代编号
    pub iteration: u32,
    /// 执行开始时间
    pub started_at: i64,
    /// 执行结束时间
    pub completed_at: Option<i64>,
    /// 执行结果
    pub result: Option<serde_json::Value>,
    /// 执行错误
    pub error: Option<String>,
    /// 消耗的成本
    pub cost: f64,
    /// 执行时间（毫秒）
    pub execution_time_ms: Option<u64>,
    /// 重试次数
    pub retry_count: u32,
}

/// Ralph Loop 执行历史
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RalphLoopExecutionHistory {
    /// 执行ID
    pub execution_id: String,
    /// 工作流ID
    pub workflow_id: String,
    /// 总迭代次数
    pub total_iterations: u32,
    /// 历史记录列表
    pub iterations: Vec<RalphLoopIterationHistory>,
    /// 开始时间
    pub started_at: i64,
    /// 结束时间
    pub completed_at: Option<i64>,
    /// 最终状态
    pub final_status: Option<String>,
    /// 总成本
    pub total_cost: f64,
    /// 总执行时间
    pub total_execution_time_ms: Option<u64>,
}

/// 智能重试策略配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartRetryStrategy {
    /// 是否启用智能重试
    pub enabled: bool,
    /// 基于错误类型的重试策略
    pub error_based_retry: std::collections::HashMap<String, RetryConfig>,
    /// 基于历史表现的动态调整
    pub adaptive_retry: bool,
    /// 最大连续失败次数
    pub max_consecutive_failures: u32,
    /// 学习周期（多少次迭代后调整策略）
    pub learning_period: u32,
}

/// Ralph Loop 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RalphLoopConfig {
    /// 是否启用Ralph Loop
    pub enabled: bool,
    /// 最大迭代次数
    pub max_iterations: u32,
    /// 每次迭代间的延迟（毫秒）
    pub iteration_delay_ms: u64,
    /// 完成条件检查函数（返回true表示完成）
    pub completion_checker: Option<String>, // JSONPath或简单的字符串匹配
    /// 最大总执行时间（毫秒）
    pub max_total_time_ms: Option<u64>,
    /// 每次迭代的超时时间（毫秒）
    pub iteration_timeout_ms: u64,
    /// 成本限制（可选）
    pub max_cost: Option<f64>,
    /// 是否启用执行历史记录
    pub enable_history: bool,
    /// 智能重试策略
    pub smart_retry: SmartRetryStrategy,
}

impl Default for RalphLoopConfig {
    fn default() -> Self {
        Self {
            enabled: true,  // 默认启用Ralph Loop
            max_iterations: 50,  // 增加最大迭代次数
            iteration_delay_ms: 500,  // 减少延迟，更快响应
            completion_checker: Some("auto".to_string()),  // AI自动判断完成条件
            max_total_time_ms: Some(1800000), // 30分钟
            iteration_timeout_ms: 120000, // 2分钟
            max_cost: Some(10.0),  // 设置成本限制
            enable_history: true,
            smart_retry: SmartRetryStrategy {
                enabled: true,
                error_based_retry: std::collections::HashMap::new(),
                adaptive_retry: true,
                max_consecutive_failures: 5,  // 增加容忍度
                learning_period: 3,  // 减少学习周期
            },
        }
    }
}