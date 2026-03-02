//! Agent Swarm 核心类型定义
//! 
//! 与 alou-desktop/src/types/tasks.ts 保持兼容的 Rust 实现

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

// ========== 枚举类型 ==========

/// 任务优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskPriority {
    /// 关键任务，立即执行
    Critical,
    /// 高优先级，优先执行
    High,
    /// 中等优先级，正常执行
    Medium,
    /// 低优先级，空闲时执行
    Low,
    /// 自定义优先级
    Custom,
}

impl Default for TaskPriority {
    fn default() -> Self {
        TaskPriority::Medium
    }
}

/// 任务状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    /// 待处理
    Pending,
    /// 已入队
    Queued,
    /// 已分配
    Assigned,
    /// 执行中
    InProgress,
    /// 已暂停
    Paused,
    /// 已完成
    Completed,
    /// 失败
    Failed,
    /// 已取消
    Cancelled,
    /// 超时
    Timeout,
}

impl Default for TaskStatus {
    fn default() -> Self {
        TaskStatus::Pending
    }
}

impl TaskStatus {
    /// 是否为终态
    pub fn is_terminal(&self) -> bool {
        matches!(self, TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled | TaskStatus::Timeout)
    }
    
    /// 是否可以执行
    pub fn is_executable(&self) -> bool {
        matches!(self, TaskStatus::Pending | TaskStatus::Queued | TaskStatus::Paused)
    }
}

/// 执行模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionMode {
    /// 顺序执行
    Sequential,
    /// 并行执行
    Parallel,
    /// Agent 群协作
    Swarm,
    /// 工作流模式
    Workflow,
}

impl Default for ExecutionMode {
    fn default() -> Self {
        ExecutionMode::Sequential
    }
}

/// Agent 状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentStatus {
    /// 在线
    Online,
    /// 离线
    Offline,
    /// 忙碌
    Busy,
    /// 空闲
    Idle,
    /// 错误
    Error,
}

impl Default for AgentStatus {
    fn default() -> Self {
        AgentStatus::Idle
    }
}

/// Swarm 协作模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SwarmCoordinationMode {
    /// 领导者 - 跟随者模式
    LeaderFollower,
    /// 点对点模式
    PeerToPeer,
    /// 群体智能模式
    HiveMind,
}

impl Default for SwarmCoordinationMode {
    fn default() -> Self {
        SwarmCoordinationMode::LeaderFollower
    }
}

/// 消息优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessagePriority {
    /// 紧急
    Urgent,
    /// 普通
    Normal,
    /// 低
    Low,
}

impl Default for MessagePriority {
    fn default() -> Self {
        MessagePriority::Normal
    }
}

/// Swarm 状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SwarmStatus {
    /// 形成中
    Forming,
    /// 活跃
    Active,
    /// 暂停
    Paused,
    /// 解散中
    Disbanding,
    /// 已解散
    Disbanded,
}

impl Default for SwarmStatus {
    fn default() -> Self {
        SwarmStatus::Forming
    }
}

/// Swarm 消息类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SwarmMessageType {
    /// 任务公告
    TaskAnnouncement,
    /// 进度更新
    ProgressUpdate,
    /// 求助请求
    HelpRequest,
    /// 结果分享
    ResultShare,
    /// 任务分配
    TaskAssignment,
    /// 状态同步
    StatusSync,
}

/// 进度事件类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProgressEventType {
    /// 步骤开始
    StepStart,
    /// 步骤完成
    StepComplete,
    /// 步骤失败
    StepFail,
    /// Agent 加入
    AgentJoin,
    /// Agent 离开
    AgentLeave,
    /// 进度更新
    ProgressUpdate,
    /// 错误发生
    ErrorOccurred,
    /// 任务暂停
    TaskPaused,
    /// 任务恢复
    TaskResumed,
}

// ========== 核心结构体 ==========

/// 任务定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// 任务唯一 ID
    pub id: String,
    /// 任务标题
    pub title: String,
    /// 任务描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// 优先级
    #[serde(default)]
    pub priority: TaskPriority,
    /// 当前状态
    #[serde(default)]
    pub status: TaskStatus,
    
    // 执行配置
    /// 执行模式
    #[serde(default)]
    pub execution_mode: ExecutionMode,
    /// 最大 Agent 数量（Swarm 模式）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_agents: Option<u32>,
    /// 超时时间（毫秒）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
    /// 重试次数
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_count: Option<u32>,
    /// 重试间隔（毫秒）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_delay: Option<u64>,
    
    // 任务内容
    /// 任务步骤
    #[serde(default)]
    pub steps: Vec<TaskStep>,
    /// 依赖的任务 ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependencies: Option<Vec<String>>,
    /// 子任务 ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_tasks: Option<Vec<String>>,
    
    // Agent 分配
    /// 分配的 Agent ID（单一）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee_id: Option<String>,
    /// 分配的 Agent IDs（Swarm）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee_ids: Option<Vec<String>>,
    /// 所需能力
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_capabilities: Option<Vec<String>>,
    
    // 上下文和参数
    /// 任务上下文
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<HashMap<String, serde_json::Value>>,
    /// 输入参数
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<HashMap<String, serde_json::Value>>,
    
    // 执行结果
    /// 执行结果
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<TaskResult>,
    /// 错误信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<TaskError>,
    
    // 时间戳 (使用毫秒时间戳与 TypeScript 兼容)
    /// 创建时间
    pub created_at: i64,
    /// 更新时间
    pub updated_at: i64,
    /// 分配时间
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assigned_at: Option<i64>,
    /// 开始时间
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<i64>,
    /// 完成时间
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<i64>,
    
    // 元数据
    /// 元数据
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<TaskMetadata>,
}

impl Task {
    /// 创建新任务
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        let now = chrono::Utc::now().timestamp_millis();
        Self {
            id: id.into(),
            title: title.into(),
            description: None,
            priority: TaskPriority::default(),
            status: TaskStatus::default(),
            execution_mode: ExecutionMode::default(),
            max_agents: None,
            timeout: None,
            retry_count: None,
            retry_delay: None,
            steps: Vec::new(),
            dependencies: None,
            sub_tasks: None,
            assignee_id: None,
            assignee_ids: None,
            required_capabilities: None,
            context: None,
            parameters: None,
            result: None,
            error: None,
            created_at: now,
            updated_at: now,
            assigned_at: None,
            started_at: None,
            completed_at: None,
            metadata: None,
        }
    }
    
    /// 计算执行时长（毫秒）
    pub fn duration(&self) -> Option<i64> {
        match (self.started_at, self.completed_at) {
            (Some(start), Some(end)) => Some(end - start),
            (Some(start), None) => Some(chrono::Utc::now().timestamp_millis() - start),
            _ => None,
        }
    }
    
    /// 更新状态
    pub fn update_status(&mut self, status: TaskStatus) {
        let now = chrono::Utc::now().timestamp_millis();
        self.status = status;
        self.updated_at = now;
        
        match status {
            TaskStatus::Assigned => self.assigned_at = Some(now),
            TaskStatus::InProgress if self.started_at.is_none() => self.started_at = Some(now),
            TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled | TaskStatus::Timeout => {
                self.completed_at = Some(now);
            }
            _ => {}
        }
    }
    
    /// 检查是否可以执行（依赖都已完成）
    pub fn is_ready(&self, completed_task_ids: &[String]) -> bool {
        if !self.status.is_executable() {
            return false;
        }
        
        if let Some(deps) = &self.dependencies {
            deps.iter().all(|dep| completed_task_ids.contains(dep))
        } else {
            true
        }
    }
}

/// 任务步骤
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStep {
    /// 步骤 ID
    pub id: String,
    /// 步骤名称
    pub name: String,
    /// 步骤描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    
    // 执行内容
    /// 动作名称（Skill 或 Tool）
    pub action: String,
    /// 动作参数
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<HashMap<String, serde_json::Value>>,
    
    // 依赖和条件
    /// 依赖的步骤 ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub depends_on: Option<Vec<String>>,
    /// 执行条件（表达式）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
    
    // 执行控制
    /// 是否可以并行
    #[serde(default)]
    pub parallel: bool,
    /// 是否可选（失败不影响整体）
    #[serde(default)]
    pub optional: bool,
    /// 重试次数
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_count: Option<u32>,
    
    // 执行结果
    /// 步骤状态
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<TaskStatus>,
    /// 步骤结果
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    /// 错误信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    
    // 时间戳
    /// 开始时间
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<i64>,
    /// 完成时间
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<i64>,
}

/// 任务结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    /// 是否成功
    pub success: bool,
    /// 输出数据
    pub output: serde_json::Value,
    /// 结果摘要
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    /// 生成的文件/资源
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifacts: Option<Vec<String>>,
    /// 执行指标
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metrics: Option<TaskMetrics>,
}

/// 任务执行指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskMetrics {
    /// 执行时长（毫秒）
    pub duration: i64,
    /// 完成的步骤数
    pub steps_completed: u32,
    /// 失败的步骤数
    pub steps_failed: u32,
    /// 参与的 Agent 数量
    pub agents_involved: u32,
    /// 重试次数
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retries_count: Option<u32>,
    /// 成本估算（如 API 调用费用）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost_estimate: Option<f64>,
}

/// 任务错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskError {
    /// 错误代码
    pub code: String,
    /// 错误消息
    pub message: String,
    /// 详细信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
    /// 失败的步骤 ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step_id: Option<String>,
    /// 是否可恢复
    #[serde(default)]
    pub recoverable: bool,
}

/// 任务元数据
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaskMetadata {
    /// 任务来源（用户/AI/系统）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// 标签
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    /// 所属分组
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    /// 父任务 ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_task_id: Option<String>,
    /// 相关任务 ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_tasks: Option<Vec<String>>,
    /// 创建者
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
    /// 备注
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

/// Agent 信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    /// Agent ID
    pub id: String,
    /// Agent 名称
    pub name: String,
    /// 显示名称
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// 描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// 能力列表
    #[serde(default)]
    pub capabilities: Vec<String>,
    /// 当前状态
    #[serde(default)]
    pub status: AgentStatus,
    /// 当前负载 (0-1)
    #[serde(default)]
    pub current_load: f32,
    /// 专长领域
    #[serde(skip_serializing_if = "Option::is_none")]
    pub specialization: Option<String>,
    /// 历史表现评分
    #[serde(skip_serializing_if = "Option::is_none")]
    pub performance_score: Option<f32>,
    /// 当前任务 ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_task_id: Option<String>,
    /// 已完成任务数
    #[serde(default)]
    pub completed_tasks: u32,
    /// 失败任务数
    #[serde(default)]
    pub failed_tasks: u32,
}

/// Agent Swarm
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSwarm {
    /// Swarm ID
    pub id: String,
    /// Swarm 名称
    pub name: String,
    /// 成员列表
    #[serde(default)]
    pub members: Vec<AgentInfo>,
    /// 集体能力
    #[serde(default)]
    pub capabilities: Vec<String>,
    /// Swarm 状态
    #[serde(default)]
    pub status: SwarmStatus,
    /// 协作配置
    pub coordination: SwarmCoordination,
    /// 创建时间
    pub created_at: i64,
}

/// Swarm 协作配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmCoordination {
    /// 协作模式
    #[serde(default)]
    pub mode: SwarmCoordinationMode,
    /// 通信协议
    pub communication_protocol: String,
    /// 是否需要共识
    #[serde(default)]
    pub consensus_required: bool,
    /// 领导者 ID（如果是 leader-follower 模式）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub leader_id: Option<String>,
    /// 决策策略
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decision_strategy: Option<String>,
}

impl Default for SwarmCoordination {
    fn default() -> Self {
        Self {
            mode: SwarmCoordinationMode::default(),
            communication_protocol: "message-bus".to_string(),
            consensus_required: false,
            leader_id: None,
            decision_strategy: Some("leader-decides".to_string()),
        }
    }
}

/// Swarm 消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmMessage {
    /// 消息 ID
    pub id: String,
    /// 消息类型
    #[serde(rename = "type")]
    pub msg_type: SwarmMessageType,
    /// 发送者 Agent ID
    pub from: String,
    /// 接收者 Agent IDs（可选，广播时为空）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<Vec<String>>,
    /// 相关任务 ID
    pub task_id: String,
    /// 消息内容
    pub content: serde_json::Value,
    /// 时间戳
    pub timestamp: i64,
    /// 优先级
    #[serde(default)]
    pub priority: MessagePriority,
}

/// 进度追踪
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressTracker {
    /// 任务 ID
    pub task_id: String,
    /// 总体进度 (0-100)
    pub overall_progress: f32,
    /// 各步骤进度
    #[serde(default)]
    pub step_progress: HashMap<String, f32>,
    /// 完成的步骤数
    pub completed_steps: u32,
    /// 总步骤数
    pub total_steps: u32,
    /// 活跃的 Agent 数量
    pub active_agents: u32,
    /// 已用时间（毫秒）
    pub elapsed_time: i64,
    /// 预计剩余时间（毫秒）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_remaining: Option<i64>,
    /// 进度事件
    #[serde(default)]
    pub events: Vec<ProgressEvent>,
}

/// 进度事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressEvent {
    /// 时间戳
    pub timestamp: i64,
    /// 事件类型
    #[serde(rename = "type")]
    pub event_type: ProgressEventType,
    /// 步骤 ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step_id: Option<String>,
    /// Agent ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    /// 详细信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

/// 依赖图
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyGraph {
    /// 节点（任务/步骤 ID）
    #[serde(default)]
    pub nodes: Vec<String>,
    /// 边（依赖关系）
    #[serde(default)]
    pub edges: Vec<DependencyEdge>,
    /// 关键路径
    #[serde(default)]
    pub critical_path: Vec<String>,
    /// 可并行执行的组
    #[serde(default)]
    pub parallel_groups: Vec<Vec<String>>,
    /// 预估总时长
    pub estimated_duration: i64,
}

/// 依赖边
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyEdge {
    /// 前置任务
    pub from: String,
    /// 后置任务
    pub to: String,
    /// 依赖类型
    #[serde(default = "default_required")]
    pub edge_type: String,
}

fn default_required() -> String {
    "required".to_string()
}

/// 并发配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConcurrencyConfig {
    /// 最大并发任务数
    pub max_concurrent_tasks: u32,
    /// 最大并发步骤数
    pub max_concurrent_steps: u32,
    /// 资源限制
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_limits: Option<ResourceLimits>,
    /// 速率限制
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate_limiting: Option<RateLimiting>,
}

/// 资源限制
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// 内存限制（MB）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory: Option<u32>,
    /// CPU 使用限制（百分比）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu: Option<u32>,
    /// 网络请求限制
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<u32>,
}

/// 速率限制
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimiting {
    /// 每秒请求数限制
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requests_per_second: Option<u32>,
    /// 突发容量
    #[serde(skip_serializing_if = "Option::is_none")]
    pub burst_size: Option<u32>,
}

/// 任务创建选项
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaskCreateOptions {
    /// 执行模式
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_mode: Option<ExecutionMode>,
    /// 优先级
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<TaskPriority>,
    /// 最大 Agent 数量
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_agents: Option<u32>,
    /// 超时时间
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
    /// 所需能力
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_capabilities: Option<Vec<String>>,
    /// 元数据
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<TaskMetadata>,
    /// 是否立即执行
    #[serde(default)]
    pub auto_execute: bool,
}

/// 任务执行选项
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaskExecutionOptions {
    /// 分配的 Agent IDs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee_ids: Option<Vec<String>>,
    /// 自定义上下文
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<HashMap<String, serde_json::Value>>,
}

// ========== 类型转换实现 ==========

impl TaskResult {
    /// 创建成功结果
    pub fn success(output: impl Serialize) -> Result<Self, serde_json::Error> {
        Ok(Self {
            success: true,
            output: serde_json::to_value(output)?,
            summary: None,
            artifacts: None,
            metrics: None,
        })
    }
    
    /// 创建失败结果
    pub fn failure(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            success: false,
            output: serde_json::Value::Null,
            summary: None,
            artifacts: None,
            metrics: None,
        }
    }
}

impl TaskError {
    /// 创建错误
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: None,
            step_id: None,
            recoverable: false,
        }
    }
    
    /// 设置可恢复
    pub fn recoverable(mut self) -> Self {
        self.recoverable = true;
        self
    }
    
    /// 添加详情
    pub fn with_details(mut self, details: impl Serialize) -> Result<Self, serde_json::Error> {
        self.details = Some(serde_json::to_value(details)?);
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_task_creation() {
        let task = Task::new("task-001", "测试任务");
        assert_eq!(task.id, "task-001");
        assert_eq!(task.title, "测试任务");
        assert_eq!(task.status, TaskStatus::Pending);
    }
    
    #[test]
    fn test_task_serialization() {
        let task = Task::new("task-001", "测试任务");
        let json = serde_json::to_string(&task).unwrap();
        assert!(json.contains("task-001"));
        assert!(json.contains("测试任务"));
    }
    
    #[test]
    fn test_task_status_is_terminal() {
        assert!(TaskStatus::Completed.is_terminal());
        assert!(TaskStatus::Failed.is_terminal());
        assert!(!TaskStatus::Pending.is_terminal());
        assert!(!TaskStatus::InProgress.is_terminal());
    }
}