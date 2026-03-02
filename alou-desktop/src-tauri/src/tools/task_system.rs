//! 任务系统核心模块
//!
//! 提供任务定义、调度、执行和 Agent Swarm 协作功能
//! 支持并行执行、优先级队列和分布式协作

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;
use chrono::Utc;

/// 任务优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskPriority {
    /// 关键任务，立即执行
    Critical = 0,
    /// 高优先级，优先执行
    High = 1,
    /// 中等优先级，正常执行
    Medium = 2,
    /// 低优先级，空闲时执行
    Low = 3,
    /// 自定义优先级
    Custom = 4,
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

/// 任务执行模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

/// Agent 状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

/// Agent 协作模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SwarmCoordinationMode {
    /// 领导者 - 跟随者模式
    LeaderFollower,
    /// 点对点模式
    PeerToPeer,
    /// 群体智能模式
    HiveMind,
}

/// 任务步骤
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStep {
    /// 步骤 ID
    pub id: String,
    /// 步骤名称
    pub name: String,
    /// 步骤描述
    pub description: Option<String>,
    /// 动作名称（Skill 或 Tool）
    pub action: String,
    /// 动作参数
    pub parameters: Option<Value>,
    /// 依赖的步骤 ID
    pub depends_on: Option<Vec<String>>,
    /// 执行条件（表达式）
    pub condition: Option<String>,
    /// 是否可以并行
    pub parallel: Option<bool>,
    /// 是否可选（失败不影响整体）
    pub optional: Option<bool>,
    /// 重试次数
    pub retry_count: Option<u32>,
    /// 步骤状态
    pub status: Option<TaskStatus>,
    /// 步骤结果
    pub result: Option<Value>,
    /// 错误信息
    pub error: Option<String>,
    /// 开始时间
    pub started_at: Option<i64>,
    /// 完成时间
    pub completed_at: Option<i64>,
}

impl TaskStep {
    /// 创建新步骤
    pub fn new(id: String, name: String, action: String) -> Self {
        Self {
            id,
            name,
            description: None,
            action,
            parameters: None,
            depends_on: None,
            condition: None,
            parallel: None,
            optional: None,
            retry_count: None,
            status: None,
            result: None,
            error: None,
            started_at: None,
            completed_at: None,
        }
    }
}

/// 任务执行指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskMetrics {
    /// 执行时长（毫秒）
    pub duration: u64,
    /// 完成的步骤数
    pub steps_completed: u32,
    /// 失败的步骤数
    pub steps_failed: u32,
    /// 参与的 Agent 数量
    pub agents_involved: u32,
    /// 重试次数
    pub retries_count: u32,
    /// 成本估算
    pub cost_estimate: Option<f64>,
}

impl Default for TaskMetrics {
    fn default() -> Self {
        Self {
            duration: 0,
            steps_completed: 0,
            steps_failed: 0,
            agents_involved: 0,
            retries_count: 0,
            cost_estimate: None,
        }
    }
}

/// 任务错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskError {
    /// 错误代码
    pub code: String,
    /// 错误消息
    pub message: String,
    /// 详细信息
    pub details: Option<Value>,
    /// 失败的步骤 ID
    pub step_id: Option<String>,
    /// 是否可恢复
    pub recoverable: bool,
}

/// 任务结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    /// 是否成功
    pub success: bool,
    /// 输出数据
    pub output: Option<Value>,
    /// 结果摘要
    pub summary: Option<String>,
    /// 生成的文件/资源
    pub artifacts: Option<Vec<String>>,
    /// 执行指标
    pub metrics: Option<TaskMetrics>,
}

/// Agent 信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    /// Agent ID
    pub id: String,
    /// Agent 名称
    pub name: String,
    /// 显示名称
    pub display_name: Option<String>,
    /// 描述
    pub description: Option<String>,
    /// 能力列表
    pub capabilities: Vec<String>,
    /// 当前状态
    pub status: AgentStatus,
    /// 当前负载 (0-1)
    pub current_load: f64,
    /// 专长领域
    pub specialization: Option<String>,
    /// 历史表现评分
    pub performance_score: Option<f64>,
    /// 当前任务 ID
    pub current_task_id: Option<String>,
    /// 已完成任务数
    pub completed_tasks: u32,
    /// 失败任务数
    pub failed_tasks: u32,
}

impl AgentInfo {
    /// 创建新 Agent 信息
    pub fn new(id: String, name: String, capabilities: Vec<String>) -> Self {
        Self {
            id,
            name,
            display_name: None,
            description: None,
            capabilities,
            status: AgentStatus::Idle,
            current_load: 0.0,
            specialization: None,
            performance_score: None,
            current_task_id: None,
            completed_tasks: 0,
            failed_tasks: 0,
        }
    }
}

/// Agent Swarm
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSwarm {
    /// Swarm ID
    pub id: String,
    /// Swarm 名称
    pub name: String,
    /// 成员列表
    pub members: Vec<AgentInfo>,
    /// 集体能力
    pub capabilities: Vec<String>,
    /// Swarm 状态
    pub status: SwarmStatus,
    /// 协作配置
    pub coordination: SwarmCoordination,
    /// 创建时间
    pub created_at: i64,
}

/// Swarm 状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SwarmStatus {
    Forming,
    Active,
    Paused,
    Disbanding,
    Disbanded,
}

/// Swarm 协作配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmCoordination {
    /// 协作模式
    pub mode: SwarmCoordinationMode,
    /// 通信协议
    pub communication_protocol: String,
    /// 是否需要共识
    pub consensus_required: bool,
    /// 领导者 ID
    pub leader_id: Option<String>,
    /// 决策策略
    pub decision_strategy: Option<String>,
}

/// 任务定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// 任务唯一 ID
    pub id: String,
    /// 任务标题
    pub title: String,
    /// 任务描述
    pub description: String,
    /// 优先级
    pub priority: TaskPriority,
    /// 当前状态
    pub status: TaskStatus,
    /// 执行模式
    pub execution_mode: ExecutionMode,
    /// 最大 Agent 数量（Swarm 模式）
    pub max_agents: Option<u32>,
    /// 超时时间（毫秒）
    pub timeout: Option<u64>,
    /// 重试次数
    pub retry_count: Option<u32>,
    /// 重试间隔（毫秒）
    pub retry_delay: Option<u64>,
    /// 任务步骤
    pub steps: Vec<TaskStep>,
    /// 依赖的任务 ID
    pub dependencies: Option<Vec<String>>,
    /// 子任务 ID
    pub sub_tasks: Option<Vec<String>>,
    /// 分配的 Agent ID（单一）
    pub assignee_id: Option<String>,
    /// 分配的 Agent IDs（Swarm）
    pub assignee_ids: Option<Vec<String>>,
    /// 所需能力
    pub required_capabilities: Option<Vec<String>>,
    /// 任务上下文
    pub context: Option<Value>,
    /// 输入参数
    pub parameters: Option<Value>,
    /// 执行结果
    pub result: Option<TaskResult>,
    /// 错误信息
    pub error: Option<TaskError>,
    /// 创建时间
    pub created_at: i64,
    /// 更新时间
    pub updated_at: i64,
    /// 分配时间
    pub assigned_at: Option<i64>,
    /// 开始时间
    pub started_at: Option<i64>,
    /// 完成时间
    pub completed_at: Option<i64>,
    /// 元数据
    pub metadata: Option<Value>,
}

impl Task {
    /// 创建新任务
    pub fn new(
        title: String,
        description: String,
        priority: TaskPriority,
        execution_mode: ExecutionMode,
    ) -> Self {
        let now = Utc::now().timestamp();
        Self {
            id: format!("task_{}", Uuid::new_v4().to_string().replace("-", "")),
            title,
            description,
            priority,
            status: TaskStatus::Pending,
            execution_mode,
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

    /// 添加步骤
    pub fn add_step(&mut self, step: TaskStep) {
        self.steps.push(step);
        self.updated_at = Utc::now().timestamp();
    }

    /// 更新状态
    pub fn update_status(&mut self, status: TaskStatus) {
        self.status = status;
        self.updated_at = Utc::now().timestamp();
    }
}

/// 任务统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStats {
    /// 总任务数
    pub total: u32,
    /// 按状态分类
    pub by_status: HashMap<String, u32>,
    /// 按优先级分类
    pub by_priority: HashMap<String, u32>,
    /// 按执行模式分类
    pub by_execution_mode: HashMap<String, u32>,
    /// 完成率
    pub completion_rate: f64,
    /// 平均执行时长（毫秒）
    pub average_duration: f64,
    /// 今日任务数
    pub today_tasks: u32,
    /// 进行中任务数
    pub in_progress_tasks: u32,
}

impl Default for TaskStats {
    fn default() -> Self {
        Self {
            total: 0,
            by_status: HashMap::new(),
            by_priority: HashMap::new(),
            by_execution_mode: HashMap::new(),
            completion_rate: 0.0,
            average_duration: 0.0,
            today_tasks: 0,
            in_progress_tasks: 0,
        }
    }
}

/// 任务队列管理器
pub struct TaskQueueManager {
    /// 任务队列
    queue: Vec<Task>,
    /// 历史记录
    history: Vec<Task>,
    /// 按状态索引
    by_status: HashMap<TaskStatus, Vec<String>>,
    /// 按优先级索引
    by_priority: HashMap<TaskPriority, Vec<String>>,
    /// Agents 注册表
    agents: HashMap<String, AgentInfo>,
    /// Swarms 注册表
    swarms: HashMap<String, AgentSwarm>,
    /// 任务 -Agent 分配历史
    assignment_history: HashMap<String, Vec<String>>,
    /// 存储路径
    storage_path: std::path::PathBuf,
}

impl TaskQueueManager {
    /// 创建新的任务队列管理器
    pub fn new(storage_path: Option<std::path::PathBuf>) -> Result<Self, Box<dyn std::error::Error>> {
        use std::path::PathBuf;
        use std::fs;

        let path = storage_path.unwrap_or_else(|| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".alou")
                .join("tasks")
        });

        // 确保目录存在
        if !path.exists() {
            fs::create_dir_all(&path)?;
        }

        let mut manager = Self {
            queue: Vec::new(),
            history: Vec::new(),
            by_status: HashMap::new(),
            by_priority: HashMap::new(),
            agents: HashMap::new(),
            swarms: HashMap::new(),
            assignment_history: HashMap::new(),
            storage_path: path,
        };

        manager.load_from_disk()?;
        Ok(manager)
    }

    /// 注册 Agent
    pub fn register_agent(&mut self, agent: AgentInfo) {
        self.agents.insert(agent.id.clone(), agent);
    }

    /// 注销 Agent
    pub fn unregister_agent(&mut self, agent_id: &str) {
        self.agents.remove(agent_id);
    }

    /// 创建 Swarm
    pub fn create_swarm(
        &mut self,
        name: String,
        member_ids: Vec<String>,
        mode: SwarmCoordinationMode,
    ) -> Option<String> {
        use uuid::Uuid;

        let mut members = Vec::new();
        let mut capabilities = std::collections::HashSet::new();

        for id in &member_ids {
            if let Some(agent) = self.agents.get(id) {
                members.push(agent.clone());
                for cap in &agent.capabilities {
                    capabilities.insert(cap.clone());
                }
            }
        }

        if members.is_empty() {
            return None;
        }

        let swarm_id = format!("swarm_{}", Uuid::new_v4().to_string().replace("-", ""));
        let now = Utc::now().timestamp();

        // 先计算 leader_id（在 members 移动之前）
        let leader_id = if mode == SwarmCoordinationMode::LeaderFollower {
            members.first().map(|m| m.id.clone())
        } else {
            None
        };

        let decision_strategy = match mode {
            SwarmCoordinationMode::LeaderFollower => "leader-decides".to_string(),
            SwarmCoordinationMode::HiveMind => "unanimous".to_string(),
            SwarmCoordinationMode::PeerToPeer => "majority".to_string(),
        };

        let swarm = AgentSwarm {
            id: swarm_id.clone(),
            name,
            members,
            capabilities: capabilities.into_iter().collect(),
            status: SwarmStatus::Active,
            coordination: SwarmCoordination {
                mode,
                communication_protocol: "pubsub".to_string(),
                consensus_required: mode == SwarmCoordinationMode::HiveMind,
                leader_id,
                decision_strategy: Some(decision_strategy),
            },
            created_at: now,
        };

        self.swarms.insert(swarm_id.clone(), swarm);
        Some(swarm_id)
    }

    /// 添加任务
    pub async fn add_task(&mut self, mut task: Task) -> String {
        task.status = TaskStatus::Queued;
        task.updated_at = Utc::now().timestamp();
        let task_id = task.id.clone();

        // 添加到队列
        self.queue.push(task);

        // 更新索引
        self.by_status
            .entry(TaskStatus::Queued)
            .or_insert_with(Vec::new)
            .push(task_id.clone());

        self.by_priority
            .entry(TaskPriority::Medium)
            .or_insert_with(Vec::new)
            .push(task_id.clone());

        // 按优先级排序
        self.sort_by_priority();

        // 保存到磁盘
        self.save_to_disk().await;

        task_id
    }

    /// 获取下一个任务
    pub async fn next_task(&mut self, executor: Option<&str>) -> Option<Task> {
        // 过滤出待执行的任务
        let candidates: Vec<usize> = self
            .queue
            .iter()
            .enumerate()
            .filter(|(_, task)| task.status == TaskStatus::Queued || task.status == TaskStatus::Pending)
            .filter(|(_, task)| match executor {
                Some(e) => task.assignee_id.as_deref() == Some(e),
                None => true,
            })
            .map(|(idx, _)| idx)
            .collect();

        if candidates.is_empty() {
            return None;
        }

        // 选择优先级最高的任务
        let best_idx = candidates.iter().min_by_key(|&&idx| self.queue[idx].priority).copied()?;
        let mut task = self.queue.remove(best_idx);
        task.update_status(TaskStatus::InProgress);

        Some(task)
    }

    /// 为任务分配 Agents
    pub fn assign_agents_to_task(&mut self, task_id: &str, agent_ids: Vec<String>) -> bool {
        if let Some(task) = self.queue.iter_mut().find(|t| t.id == task_id) {
            task.assignee_ids = Some(agent_ids.clone());
            task.assigned_at = Some(Utc::now().timestamp());
            task.update_status(TaskStatus::Assigned);
            
            self.assignment_history.insert(task_id.to_string(), agent_ids);
            return true;
        }
        false
    }

    /// 获取所有 Agents
    pub fn get_agents(&self) -> Vec<AgentInfo> {
        self.agents.values().cloned().collect()
    }

    /// 获取在线 Agents
    pub fn get_online_agents(&self) -> Vec<AgentInfo> {
        self.agents
            .values()
            .filter(|a| a.status == AgentStatus::Online || a.status == AgentStatus::Idle)
            .cloned()
            .collect()
    }

    /// 根据能力筛选 Agents
    pub fn get_agents_by_capabilities(&self, capabilities: &[String]) -> Vec<AgentInfo> {
        self.agents
            .values()
            .filter(|a| capabilities.iter().all(|cap| a.capabilities.contains(cap)))
            .cloned()
            .collect()
    }

    /// 获取任务列表
    pub fn list_tasks(&self, filter: Option<TaskStatus>) -> Vec<Task> {
        match filter {
            Some(status) => self.queue.iter().filter(|t| t.status == status).cloned().collect(),
            None => self.queue.clone(),
        }
    }

    /// 获取任务统计
    pub fn stats(&self) -> TaskStats {
        let mut stats = TaskStats::default();
        
        for task in &self.queue {
            stats.total += 1;
            *stats.by_status.entry(format!("{:?}", task.status)).or_insert(0) += 1;
            *stats.by_priority.entry(format!("{:?}", task.priority)).or_insert(0) += 1;
            *stats.by_execution_mode.entry(format!("{:?}", task.execution_mode)).or_insert(0) += 1;
            
            if task.status == TaskStatus::InProgress {
                stats.in_progress_tasks += 1;
            }
        }

        stats
    }

    /// 按优先级排序
    fn sort_by_priority(&mut self) {
        self.queue.sort_by(|a, b| a.priority.cmp(&b.priority));
    }

    /// 保存到磁盘
    async fn save_to_disk(&self) {
        use std::fs;

        // 保存队列
        let queue_path = self.storage_path.join("queue.json");
        if let Ok(content) = serde_json::to_string_pretty(&self.queue) {
            let _ = fs::write(&queue_path, content);
        }

        // 保存历史
        let history_path = self.storage_path.join("history.json");
        if let Ok(content) = serde_json::to_string_pretty(&self.history) {
            let _ = fs::write(&history_path, content);
        }
    }

    /// 从磁盘加载
    fn load_from_disk(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        use std::fs;

        let queue_path = self.storage_path.join("queue.json");
        if queue_path.exists() {
            let content = fs::read_to_string(&queue_path)?;
            self.queue = serde_json::from_str(&content)?;
        }

        let history_path = self.storage_path.join("history.json");
        if history_path.exists() {
            let content = fs::read_to_string(&history_path)?;
            self.history = serde_json::from_str(&content)?;
        }

        Ok(())
    }
}

/// 任务执行引擎
pub struct TaskExecutionEngine {
    /// 活跃的任务执行
    active_executions: Arc<RwLock<HashMap<String, TaskExecution>>>,
    /// 最大并发任务数
    max_concurrent_tasks: u32,
    /// 最大并发步骤数
    max_concurrent_steps: u32,
}

impl TaskExecutionEngine {
    /// 创建新的执行引擎
    pub fn new(max_concurrent_tasks: u32, max_concurrent_steps: u32) -> Self {
        Self {
            active_executions: Arc::new(RwLock::new(HashMap::new())),
            max_concurrent_tasks,
            max_concurrent_steps,
        }
    }

    /// 执行任务
    pub async fn execute_task(&self, task: Task, agents: Vec<AgentInfo>) -> TaskResult {
        let execution = TaskExecution::new(task, agents);
        
        {
            let mut executions = self.active_executions.write().await;
            executions.insert(execution.task_id.clone(), execution);
        }

        // 实际执行逻辑应该在 TaskExecution 中实现
        // 这里简化处理
        TaskResult {
            success: true,
            output: None,
            summary: Some("任务执行完成".to_string()),
            artifacts: None,
            metrics: Some(TaskMetrics::default()),
        }
    }

    /// 暂停任务
    pub async fn pause_task(&self, task_id: &str) -> bool {
        let mut executions = self.active_executions.write().await;
        if let Some(execution) = executions.get_mut(task_id) {
            execution.pause();
            return true;
        }
        false
    }

    /// 恢复任务
    pub async fn resume_task(&self, task_id: &str) -> bool {
        let mut executions = self.active_executions.write().await;
        if let Some(execution) = executions.get_mut(task_id) {
            execution.resume();
            return true;
        }
        false
    }

    /// 取消任务
    pub async fn cancel_task(&self, task_id: &str) -> bool {
        let mut executions = self.active_executions.write().await;
        executions.remove(task_id);
        true
    }
}

/// 单个任务的执行实例
pub struct TaskExecution {
    task_id: String,
    agents: Vec<AgentInfo>,
    paused: bool,
    cancelled: bool,
}

impl TaskExecution {
    pub fn new(task: Task, agents: Vec<AgentInfo>) -> Self {
        Self {
            task_id: task.id,
            agents,
            paused: false,
            cancelled: false,
        }
    }

    pub fn pause(&mut self) {
        self.paused = true;
    }

    pub fn resume(&mut self) {
        self.paused = false;
    }

    pub fn cancel(&mut self) {
        self.cancelled = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_task_creation() {
        let task = Task::new(
            "测试任务".to_string(),
            "这是一个测试任务".to_string(),
            TaskPriority::Medium,
            ExecutionMode::Sequential,
        );
        
        assert!(!task.id.is_empty());
        assert_eq!(task.status, TaskStatus::Pending);
    }

    #[tokio::test]
    async fn test_agent_registration() {
        let mut manager = TaskQueueManager::new(None).unwrap();
        
        let agent = AgentInfo::new(
            "agent_1".to_string(),
            "Test Agent".to_string(),
            vec!["chat".to_string(), "search".to_string()],
        );
        
        manager.register_agent(agent);
        assert_eq!(manager.get_agents().len(), 1);
    }

    #[tokio::test]
    async fn test_swarm_creation() {
        let mut manager = TaskQueueManager::new(None).unwrap();
        
        // 注册 Agents
        manager.register_agent(AgentInfo::new(
            "agent_1".to_string(),
            "Agent 1".to_string(),
            vec!["chat".to_string()],
        ));
        manager.register_agent(AgentInfo::new(
            "agent_2".to_string(),
            "Agent 2".to_string(),
            vec!["search".to_string()],
        ));
        
        // 创建 Swarm
        let swarm_id = manager.create_swarm(
            "测试 Swarm".to_string(),
            vec!["agent_1".to_string(), "agent_2".to_string()],
            SwarmCoordinationMode::PeerToPeer,
        );
        
        assert!(swarm_id.is_some());
        assert_eq!(manager.swarms.len(), 1);
    }
}
