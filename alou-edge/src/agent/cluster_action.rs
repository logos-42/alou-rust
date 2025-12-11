use crate::storage::kv::KvStore;
use crate::utils::error::{AloudError, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

const CLUSTER_ACTION_TTL_SECONDS: u64 = 24 * 60 * 60; // 24 hours

/// 集群行动执行状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ClusterActionStatus {
    /// 待执行
    Pending,
    /// 执行中
    Running,
    /// 已完成
    Completed,
    /// 执行失败
    Failed,
    /// 已取消
    Cancelled,
}

/// 任务执行状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    /// 待执行
    Pending,
    /// 执行中
    Running,
    /// 已完成
    Completed,
    /// 执行失败
    Failed,
    /// 已跳过（条件分支）
    Skipped,
}

/// 任务依赖类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DependencyType {
    /// 顺序执行：任务 A 完成后执行任务 B
    Sequential,
    /// 并行执行：任务 A 和 B 可以同时执行
    Parallel,
    /// 条件分支：根据任务 A 的结果决定执行任务 B 或 C
    Conditional {
        condition: String, // 条件表达式或描述
    },
}

/// 任务依赖关系
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDependency {
    /// 依赖的任务 ID
    pub depends_on: String,
    /// 依赖类型
    pub dependency_type: DependencyType,
    /// 条件值（用于条件分支）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition_value: Option<Value>,
}

/// 智能体分配
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentAssignment {
    /// 智能体 ID（session_id）
    pub agent_id: String,
    /// 智能体名称
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_name: Option<String>,
    /// 智能体模式（agent/alou）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_mode: Option<String>,
    /// 分配的任务 ID
    pub task_id: String,
    /// 分配时间
    pub assigned_at: i64,
}

/// 单个任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// 任务 ID
    pub task_id: String,
    /// 任务类型（query, transaction, calculation, etc.）
    pub task_type: String,
    /// 任务描述
    pub description: String,
    /// 输入参数
    pub input: Value,
    /// 输出结果
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<Value>,
    /// 任务状态
    pub status: TaskStatus,
    /// 依赖关系
    #[serde(default)]
    pub dependencies: Vec<TaskDependency>,
    /// 分配的智能体
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assigned_agent: Option<AgentAssignment>,
    /// 创建时间
    pub created_at: i64,
    /// 开始执行时间
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<i64>,
    /// 完成时间
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<i64>,
    /// 错误信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// 重试次数
    #[serde(default)]
    pub retry_count: u32,
    /// 最大重试次数
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
    /// 元数据
    #[serde(default)]
    pub metadata: Value,
}

fn default_max_retries() -> u32 {
    3
}

impl Task {
    pub fn new(
        task_type: String,
        description: String,
        input: Value,
    ) -> Self {
        let task_id = format!("task_{}", Uuid::new_v4().to_string().replace("-", ""));
        Self {
            task_id,
            task_type,
            description,
            input,
            output: None,
            status: TaskStatus::Pending,
            dependencies: Vec::new(),
            assigned_agent: None,
            created_at: crate::utils::time::now_timestamp(),
            started_at: None,
            completed_at: None,
            error: None,
            retry_count: 0,
            max_retries: 3,
            metadata: Value::Null,
        }
    }

    /// 检查是否可以执行（依赖是否满足）
    pub fn can_execute(&self, completed_tasks: &[&str]) -> bool {
        if self.status != TaskStatus::Pending {
            return false;
        }

        // 检查所有依赖是否完成
        for dep in &self.dependencies {
            match dep.dependency_type {
                DependencyType::Sequential | DependencyType::Conditional { .. } => {
                    if !completed_tasks.contains(&dep.depends_on.as_str()) {
                        return false;
                    }
                }
                DependencyType::Parallel => {
                    // 并行依赖不需要等待
                }
            }
        }

        true
    }

    /// 标记任务开始执行
    pub fn mark_started(&mut self) {
        self.status = TaskStatus::Running;
        self.started_at = Some(crate::utils::time::now_timestamp());
    }

    /// 标记任务完成
    pub fn mark_completed(&mut self, output: Value) {
        self.status = TaskStatus::Completed;
        self.output = Some(output);
        self.completed_at = Some(crate::utils::time::now_timestamp());
    }

    /// 标记任务失败
    pub fn mark_failed(&mut self, error: String) {
        self.status = TaskStatus::Failed;
        self.error = Some(error);
        self.completed_at = Some(crate::utils::time::now_timestamp());
    }

    /// 增加重试次数
    pub fn increment_retry(&mut self) {
        self.retry_count += 1;
    }

    /// 检查是否可以重试
    pub fn can_retry(&self) -> bool {
        self.status == TaskStatus::Failed && self.retry_count < self.max_retries
    }
}

/// 执行计划
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    /// 计划 ID
    pub plan_id: String,
    /// 任务列表
    pub tasks: Vec<Task>,
    /// 执行顺序（任务 ID 列表）
    pub execution_order: Vec<String>,
    /// 并行任务组（每组内的任务可以并行执行）
    pub parallel_groups: Vec<Vec<String>>,
    /// 创建时间
    pub created_at: i64,
}

impl ExecutionPlan {
    pub fn new() -> Self {
        Self {
            plan_id: format!("plan_{}", Uuid::new_v4().to_string().replace("-", "")),
            tasks: Vec::new(),
            execution_order: Vec::new(),
            parallel_groups: Vec::new(),
            created_at: crate::utils::time::now_timestamp(),
        }
    }

    /// 添加任务
    pub fn add_task(&mut self, task: Task) {
        self.tasks.push(task);
    }

    /// 构建执行顺序
    pub fn build_execution_order(&mut self) {
        // 简单的拓扑排序实现
        let mut order = Vec::new();
        let mut completed = std::collections::HashSet::new();
        let mut remaining: Vec<String> = self.tasks.iter().map(|t| t.task_id.clone()).collect();

        while !remaining.is_empty() {
            let mut progress = false;
            let mut ready_tasks = Vec::new();

            for task_id in &remaining {
                if let Some(task) = self.tasks.iter().find(|t| &t.task_id == task_id) {
                    if task.can_execute(&completed.iter().map(|s| s.as_str()).collect::<Vec<_>>()) {
                        ready_tasks.push(task_id.clone());
                        progress = true;
                    }
                }
            }

            if !progress {
                // 无法继续，可能存在循环依赖
                break;
            }

            // 将就绪的任务添加到执行顺序
            for task_id in ready_tasks {
                order.push(task_id.clone());
                completed.insert(task_id.clone());
                remaining.retain(|id| id != &task_id);
            }
        }

        self.execution_order = order;
    }
}

/// 集群行动主体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterAction {
    /// 集群行动 ID
    pub action_id: String,
    /// 行动描述
    pub description: String,
    /// 执行计划
    pub execution_plan: ExecutionPlan,
    /// 参与的智能体列表
    pub agents: Vec<String>,
    /// 群聊主题（PubSub topic）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_topic: Option<String>,
    /// 执行状态
    pub status: ClusterActionStatus,
    /// 创建者（用户或系统）
    pub created_by: String,
    /// 创建时间
    pub created_at: i64,
    /// 开始执行时间
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<i64>,
    /// 完成时间
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<i64>,
    /// 最终结果
    #[serde(skip_serializing_if = "Option::is_none")]
    pub final_result: Option<Value>,
    /// 错误信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// 元数据
    #[serde(default)]
    pub metadata: Value,
}

impl ClusterAction {
    pub fn new(description: String, created_by: String) -> Self {
        let action_id = format!("action_{}", Uuid::new_v4().to_string().replace("-", ""));
        Self {
            action_id,
            description,
            execution_plan: ExecutionPlan::new(),
            agents: Vec::new(),
            group_topic: None,
            status: ClusterActionStatus::Pending,
            created_by,
            created_at: crate::utils::time::now_timestamp(),
            started_at: None,
            completed_at: None,
            final_result: None,
            error: None,
            metadata: Value::Null,
        }
    }

    /// 标记开始执行
    pub fn mark_started(&mut self) {
        self.status = ClusterActionStatus::Running;
        self.started_at = Some(crate::utils::time::now_timestamp());
    }

    /// 标记完成
    pub fn mark_completed(&mut self, result: Value) {
        self.status = ClusterActionStatus::Completed;
        self.final_result = Some(result);
        self.completed_at = Some(crate::utils::time::now_timestamp());
    }

    /// 标记失败
    pub fn mark_failed(&mut self, error: String) {
        self.status = ClusterActionStatus::Failed;
        self.error = Some(error);
        self.completed_at = Some(crate::utils::time::now_timestamp());
    }

    /// 标记取消
    pub fn mark_cancelled(&mut self) {
        self.status = ClusterActionStatus::Cancelled;
        self.completed_at = Some(crate::utils::time::now_timestamp());
    }

    /// 获取所有已完成的任务
    pub fn get_completed_tasks(&self) -> Vec<&Task> {
        self.execution_plan
            .tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Completed)
            .collect()
    }

    /// 获取所有失败的任务
    pub fn get_failed_tasks(&self) -> Vec<&Task> {
        self.execution_plan
            .tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Failed)
            .collect()
    }

    /// 检查是否所有任务都已完成
    pub fn all_tasks_completed(&self) -> bool {
        self.execution_plan
            .tasks
            .iter()
            .all(|t| matches!(t.status, TaskStatus::Completed | TaskStatus::Skipped))
    }
}

/// 任务分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAnalysis {
    /// 是否需要集群行动
    pub needs_cluster_action: bool,
    /// 原因
    pub reason: String,
    /// 建议的智能体数量
    pub suggested_agent_count: usize,
    /// 任务复杂度（1-10）
    pub complexity: u8,
    /// 所需能力列表
    pub required_capabilities: Vec<String>,
}

/// 集群行动管理器
pub struct ClusterActionManager {
    kv: KvStore,
}

impl ClusterActionManager {
    pub fn new(kv: KvStore) -> Self {
        Self { kv }
    }

    /// 创建新的集群行动
    pub async fn create_cluster_action(
        &self,
        description: String,
        created_by: String,
    ) -> Result<ClusterAction> {
        let mut action = ClusterAction::new(description, created_by);
        self.save_action(&action).await?;
        Ok(action)
    }

    /// 分析任务需求，决定是否需要集群行动
    pub async fn analyze_task(
        &self,
        task_description: &str,
        available_agents: &[String],
    ) -> Result<TaskAnalysis> {
        // 简单的启发式分析
        // 实际实现中可以使用 LLM 进行更智能的分析
        
        let keywords = vec!["多个", "同时", "并行", "协作", "分别", "一起"];
        let has_multi_keywords = keywords.iter().any(|kw| task_description.contains(kw));
        
        let needs_cluster = has_multi_keywords || available_agents.len() > 1;
        
        Ok(TaskAnalysis {
            needs_cluster_action: needs_cluster,
            reason: if needs_cluster {
                "任务需要多个智能体协作完成".to_string()
            } else {
                "单智能体即可完成".to_string()
            },
            suggested_agent_count: if needs_cluster {
                available_agents.len().min(3)
            } else {
                1
            },
            complexity: if needs_cluster { 7 } else { 3 },
            required_capabilities: vec!["query".to_string(), "transaction".to_string()],
        })
    }

    /// 将复杂任务分解为子任务
    pub async fn decompose_task(
        &self,
        task_description: &str,
        input: Value,
    ) -> Result<Vec<Task>> {
        // 简单的任务分解逻辑
        // 实际实现中可以使用 LLM 进行智能分解
        
        let mut tasks = Vec::new();
        
        // 检查是否包含查询任务
        if task_description.contains("查询") || task_description.contains("余额") {
            tasks.push(Task::new(
                "query".to_string(),
                "查询余额或数据".to_string(),
                input.clone(),
            ));
        }
        
        // 检查是否包含交易任务
        if task_description.contains("转账") || task_description.contains("交易") {
            tasks.push(Task::new(
                "transaction".to_string(),
                "执行交易".to_string(),
                input.clone(),
            ));
        }
        
        // 如果没有识别到特定任务，创建一个通用任务
        if tasks.is_empty() {
            tasks.push(Task::new(
                "general".to_string(),
                task_description.to_string(),
                input,
            ));
        }
        
        Ok(tasks)
    }

    /// 创建执行计划
    pub async fn create_execution_plan(
        &self,
        tasks: Vec<Task>,
    ) -> Result<ExecutionPlan> {
        let mut plan = ExecutionPlan::new();
        
        for task in tasks {
            plan.add_task(task);
        }
        
        // 构建执行顺序
        plan.build_execution_order();
        
        Ok(plan)
    }

    /// 获取集群行动状态
    pub async fn get_action_status(
        &self,
        action_id: &str,
    ) -> Result<Option<ClusterActionStatus>> {
        if let Some(action) = self.get_action(action_id).await? {
            Ok(Some(action.status))
        } else {
            Ok(None)
        }
    }

    /// 取消执行中的行动
    pub async fn cancel_action(&self, action_id: &str) -> Result<()> {
        if let Some(mut action) = self.get_action(action_id).await? {
            if action.status == ClusterActionStatus::Running {
                action.mark_cancelled();
                self.update_action(&action).await?;
            }
        }
        Ok(())
    }

    /// 保存集群行动
    pub async fn save_action(&self, action: &ClusterAction) -> Result<()> {
        let key = Self::action_key(&action.action_id);
        self.kv
            .put(&key, action, Some(CLUSTER_ACTION_TTL_SECONDS))
            .await?;
        Ok(())
    }

    /// 获取集群行动
    pub async fn get_action(&self, action_id: &str) -> Result<Option<ClusterAction>> {
        let key = Self::action_key(action_id);
        self.kv.get(&key).await
    }

    /// 更新集群行动
    pub async fn update_action(&self, action: &ClusterAction) -> Result<()> {
        self.save_action(action).await
    }

    fn action_key(action_id: &str) -> String {
        format!("cluster_action:{}", action_id)
    }
}

