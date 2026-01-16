//! 计划和任务管理工具
//!
//! 提供任务规划、项目管理、待办事项管理等功能

use super::{ToolExecutor, ToolMetadata, ToolResult, ToolError, ExecutionContext, ToolCategory, ToolStatus, ToolPriority};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// 任务状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    /// 待处理
    Pending,
    /// 进行中
    InProgress,
    /// 已完成
    Completed,
    /// 已暂停
    Paused,
    /// 已取消
    Cancelled,
    /// 已失败
    Failed,
}

impl TaskStatus {
    /// 获取状态字符串表示
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskStatus::Pending => "pending",
            TaskStatus::InProgress => "in_progress",
            TaskStatus::Completed => "completed",
            TaskStatus::Paused => "paused",
            TaskStatus::Cancelled => "cancelled",
            TaskStatus::Failed => "failed",
        }
    }
}

/// 任务优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TaskPriority {
    /// 低优先级
    Low = 1,
    /// 中等优先级
    Medium = 2,
    /// 高优先级
    High = 3,
    /// 紧急优先级
    Urgent = 4,
}

impl TaskPriority {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskPriority::Low => "low",
            TaskPriority::Medium => "medium",
            TaskPriority::High => "high",
            TaskPriority::Urgent => "urgent",
        }
    }
}

/// 计划步骤
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanStep {
    /// 步骤ID
    pub id: String,
    /// 步骤名称
    pub name: String,
    /// 步骤描述
    pub description: String,
    /// 依赖的步骤ID列表
    pub dependencies: Vec<String>,
    /// 预计持续时间（分钟）
    pub estimated_duration: Option<u64>,
    /// 分配的资源
    pub resources: Vec<String>,
    /// 状态
    pub status: TaskStatus,
    /// 进度 (0.0-1.0)
    pub progress: f64,
    /// 创建时间
    pub created_at: i64,
    /// 更新时间
    pub updated_at: i64,
}

/// 任务计划
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskPlan {
    /// 计划ID
    pub id: String,
    /// 计划名称
    pub name: String,
    /// 计划描述
    pub description: String,
    /// 目标
    pub goal: String,
    /// 步骤列表
    pub steps: Vec<PlanStep>,
    /// 总体状态
    pub status: TaskStatus,
    /// 总体进度 (0.0-1.0)
    pub progress: f64,
    /// 优先级
    pub priority: TaskPriority,
    /// 截止时间
    pub deadline: Option<i64>,
    /// 标签
    pub tags: Vec<String>,
    /// 创建者
    pub creator: String,
    /// 创建时间
    pub created_at: i64,
    /// 更新时间
    pub updated_at: i64,
}

/// 待办事项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoItem {
    /// 事项ID
    pub id: String,
    /// 标题
    pub title: String,
    /// 描述
    pub description: Option<String>,
    /// 状态
    pub status: TaskStatus,
    /// 优先级
    pub priority: TaskPriority,
    /// 截止时间
    pub due_date: Option<i64>,
    /// 标签
    pub tags: Vec<String>,
    /// 相关计划ID
    pub plan_id: Option<String>,
    /// 相关步骤ID
    pub step_id: Option<String>,
    /// 创建时间
    pub created_at: i64,
    /// 更新时间
    pub updated_at: i64,
    /// 完成时间
    pub completed_at: Option<i64>,
}

/// 计划工具
pub struct PlanTool {
    metadata: ToolMetadata,
    /// 存储的计划
    plans: Arc<Mutex<HashMap<String, TaskPlan>>>,
    /// 存储的待办事项
    todos: Arc<Mutex<HashMap<String, TodoItem>>>,
}

impl PlanTool {
    /// 创建新的计划工具
    pub fn new() -> Self {
        Self {
            metadata: ToolMetadata {
                id: "plan".to_string(),
                name: "Plan & Task Management Tool".to_string(),
                description: "创建和管理任务计划、项目规划和待办事项".to_string(),
                category: ToolCategory::Planning,
                priority: ToolPriority::High,
                status: ToolStatus::Available,
                version: "1.0.0".to_string(),
                author: "Alou Team".to_string(),
                created_at: chrono::Utc::now().timestamp(),
                updated_at: chrono::Utc::now().timestamp(),
                dependencies: vec![],
                platforms: vec!["windows".to_string(), "macos".to_string(), "linux".to_string()],
                permissions: vec!["read".to_string(), "write".to_string()],
            },
            plans: Arc::new(Mutex::new(HashMap::new())),
            todos: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 创建任务计划
    async fn create_plan(&self, name: String, description: String, goal: String, steps_data: Vec<serde_json::Value>) -> Result<TaskPlan, ToolError> {
        let plan_id = format!("plan_{}", uuid::Uuid::new_v4().to_string());
        let now = chrono::Utc::now().timestamp();

        // 解析步骤数据
        let mut steps = Vec::new();
        for (index, step_data) in steps_data.iter().enumerate() {
            let step = PlanStep {
                id: format!("{}_step_{}", plan_id, index),
                name: step_data.get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unnamed Step")
                    .to_string(),
                description: step_data.get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                dependencies: step_data.get("dependencies")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter()
                        .filter_map(|v| v.as_str())
                        .map(|s| s.to_string())
                        .collect())
                    .unwrap_or_default(),
                estimated_duration: step_data.get("estimated_duration")
                    .and_then(|v| v.as_u64()),
                resources: step_data.get("resources")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter()
                        .filter_map(|v| v.as_str())
                        .map(|s| s.to_string())
                        .collect())
                    .unwrap_or_default(),
                status: TaskStatus::Pending,
                progress: 0.0,
                created_at: now,
                updated_at: now,
            };
            steps.push(step);
        }

        let plan = TaskPlan {
            id: plan_id.clone(),
            name,
            description,
            goal,
            steps,
            status: TaskStatus::Pending,
            progress: 0.0,
            priority: TaskPriority::Medium,
            deadline: None,
            tags: vec![],
            creator: "system".to_string(),
            created_at: now,
            updated_at: now,
        };

        // 存储计划
        let mut plans = self.plans.lock().await;
        plans.insert(plan_id, plan.clone());

        Ok(plan)
    }

    /// 更新计划步骤状态
    async fn update_step_status(&self, plan_id: &str, step_id: &str, status: TaskStatus, progress: Option<f64>) -> Result<(), ToolError> {
        let mut plans = self.plans.lock().await;
        let plan = plans.get_mut(plan_id)
            .ok_or_else(|| ToolError::InvalidArguments(format!("Plan '{}' not found", plan_id)))?;

        let step = plan.steps.iter_mut()
            .find(|s| s.id == step_id)
            .ok_or_else(|| ToolError::InvalidArguments(format!("Step '{}' not found in plan '{}'", step_id, plan_id)))?;

        step.status = status;
        if let Some(progress) = progress {
            step.progress = progress.clamp(0.0, 1.0);
        }
        step.updated_at = chrono::Utc::now().timestamp();

        // 更新整体计划状态和进度
        self.update_plan_status(plan);

        Ok(())
    }

    /// 更新计划整体状态
    fn update_plan_status(&self, plan: &mut TaskPlan) {
        let total_steps = plan.steps.len();
        if total_steps == 0 {
            plan.status = TaskStatus::Completed;
            plan.progress = 1.0;
            return;
        }

        let completed_steps = plan.steps.iter()
            .filter(|s| s.status == TaskStatus::Completed)
            .count();
        let in_progress_steps = plan.steps.iter()
            .filter(|s| s.status == TaskStatus::InProgress)
            .count();

        plan.progress = completed_steps as f64 / total_steps as f64;

        if completed_steps == total_steps {
            plan.status = TaskStatus::Completed;
        } else if in_progress_steps > 0 {
            plan.status = TaskStatus::InProgress;
        } else {
            plan.status = TaskStatus::Pending;
        }

        plan.updated_at = chrono::Utc::now().timestamp();
    }

    /// 创建待办事项
    async fn create_todo(&self, title: String, description: Option<String>, priority: Option<TaskPriority>) -> Result<TodoItem, ToolError> {
        let todo_id = format!("todo_{}", uuid::Uuid::new_v4().to_string());
        let now = chrono::Utc::now().timestamp();

        let todo = TodoItem {
            id: todo_id.clone(),
            title,
            description,
            status: TaskStatus::Pending,
            priority: priority.unwrap_or(TaskPriority::Medium),
            due_date: None,
            tags: vec![],
            plan_id: None,
            step_id: None,
            created_at: now,
            updated_at: now,
            completed_at: None,
        };

        // 存储待办事项
        let mut todos = self.todos.lock().await;
        todos.insert(todo_id, todo.clone());

        Ok(todo)
    }

    /// 更新待办事项状态
    async fn update_todo_status(&self, todo_id: &str, status: TaskStatus) -> Result<(), ToolError> {
        let mut todos = self.todos.lock().await;
        let todo = todos.get_mut(todo_id)
            .ok_or_else(|| ToolError::InvalidArguments(format!("Todo item '{}' not found", todo_id)))?;

        todo.status = status;
        todo.updated_at = chrono::Utc::now().timestamp();

        if status == TaskStatus::Completed {
            todo.completed_at = Some(chrono::Utc::now().timestamp());
        }

        Ok(())
    }

    /// 获取计划列表
    async fn list_plans(&self) -> Vec<TaskPlan> {
        let plans = self.plans.lock().await;
        plans.values().cloned().collect()
    }

    /// 获取待办事项列表
    async fn list_todos(&self, status_filter: Option<TaskStatus>) -> Vec<TodoItem> {
        let todos = self.todos.lock().await;
        if let Some(status) = status_filter {
            todos.values()
                .filter(|todo| todo.status == status)
                .cloned()
                .collect()
        } else {
            todos.values().cloned().collect()
        }
    }

    /// 分析任务依赖关系
    fn analyze_dependencies(&self, plan: &TaskPlan) -> Result<Vec<String>, ToolError> {
        let mut execution_order = Vec::new();
        let mut completed = std::collections::HashSet::new();
        let mut in_progress = std::collections::HashSet::new();

        // 拓扑排序
        loop {
            let mut added = false;

            for step in &plan.steps {
                if completed.contains(&step.id) {
                    continue;
                }

                // 检查依赖是否都已完成
                let dependencies_satisfied = step.dependencies.iter()
                    .all(|dep_id| completed.contains(dep_id));

                if dependencies_satisfied && !in_progress.contains(&step.id) {
                    execution_order.push(step.id.clone());
                    in_progress.insert(step.id.clone());
                    added = true;
                }
            }

            if !added {
                break;
            }

            // 模拟执行（标记为已完成）
            for step_id in &in_progress.clone() {
                completed.insert(step_id.clone());
            }
            in_progress.clear();
        }

        Ok(execution_order)
    }

    /// 生成任务执行建议
    fn generate_execution_suggestions(&self, plan: &TaskPlan) -> Vec<String> {
        let mut suggestions = Vec::new();

        // 检查依赖关系
        if let Ok(execution_order) = self.analyze_dependencies(plan) {
            if execution_order.len() != plan.steps.len() {
                suggestions.push("⚠️ 检测到循环依赖，请检查步骤依赖关系".to_string());
            }
        }

        // 检查时间估计
        let total_estimated_time: u64 = plan.steps.iter()
            .filter_map(|s| s.estimated_duration)
            .sum();
        if total_estimated_time > 480 { // 超过8小时
            suggestions.push(format!("⚠️ 计划总预计时间较长: {} 分钟", total_estimated_time));
        }

        // 检查优先级
        if plan.priority == TaskPriority::Urgent && plan.deadline.is_none() {
            suggestions.push("💡 紧急任务建议设置截止时间".to_string());
        }

        // 检查资源分配
        let steps_without_resources: Vec<_> = plan.steps.iter()
            .filter(|s| s.resources.is_empty())
            .map(|s| s.name.as_str())
            .collect();
        if !steps_without_resources.is_empty() {
            suggestions.push(format!("💡 以下步骤缺少资源分配: {}", steps_without_resources.join(", ")));
        }

        suggestions
    }
}

#[async_trait]
impl ToolExecutor for PlanTool {
    fn metadata(&self) -> &ToolMetadata {
        &self.metadata
    }

    async fn execute(&self, args: serde_json::Value, _context: &ExecutionContext) -> Result<ToolResult, ToolError> {
        let action = args.get("action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArguments("Missing 'action' field".to_string()))?;

        match action {
            "create_plan" => {
                let name = args.get("name")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'name' field".to_string()))?
                    .to_string();

                let description = args.get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let goal = args.get("goal")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'goal' field".to_string()))?
                    .to_string();

                let steps = args.get("steps")
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'steps' field".to_string()))?
                    .clone();

                let plan = self.create_plan(name, description, goal, steps).await?;
                let suggestions = self.generate_execution_suggestions(&plan);

                Ok(ToolResult {
                    success: true,
                    data: serde_json::json!({
                        "plan": plan,
                        "execution_order": self.analyze_dependencies(&plan).unwrap_or_default(),
                        "suggestions": suggestions
                    }),
                    error: None,
                    execution_time_ms: 0,
                    output: Some(format!("Successfully created plan '{}'", plan.name)),
                    warnings: vec![],
                    context: None,
                })
            }

            "update_step" => {
                let plan_id = args.get("plan_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'plan_id' field".to_string()))?;

                let step_id = args.get("step_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'step_id' field".to_string()))?;

                let status_str = args.get("status")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'status' field".to_string()))?;

                let status = match status_str {
                    "pending" => TaskStatus::Pending,
                    "in_progress" => TaskStatus::InProgress,
                    "completed" => TaskStatus::Completed,
                    "paused" => TaskStatus::Paused,
                    "cancelled" => TaskStatus::Cancelled,
                    "failed" => TaskStatus::Failed,
                    _ => return Err(ToolError::InvalidArguments(format!("Invalid status: {}", status_str))),
                };

                let progress = args.get("progress").and_then(|v| v.as_f64());

                self.update_step_status(plan_id, step_id, status, progress).await?;

                Ok(ToolResult {
                    success: true,
                    data: serde_json::json!({
                        "plan_id": plan_id,
                        "step_id": step_id,
                        "status": status.as_str(),
                        "progress": progress
                    }),
                    error: None,
                    execution_time_ms: 0,
                    output: Some(format!("Successfully updated step {} in plan {}", step_id, plan_id)),
                    warnings: vec![],
                    context: None,
                })
            }

            "create_todo" => {
                let title = args.get("title")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'title' field".to_string()))?
                    .to_string();

                let description = args.get("description")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                let priority = args.get("priority")
                    .and_then(|v| v.as_str())
                    .map(|p| match p {
                        "low" => TaskPriority::Low,
                        "medium" => TaskPriority::Medium,
                        "high" => TaskPriority::High,
                        "urgent" => TaskPriority::Urgent,
                        _ => TaskPriority::Medium,
                    });

                let todo = self.create_todo(title, description, priority).await?;

                Ok(ToolResult {
                    success: true,
                    data: serde_json::json!({
                        "todo": todo
                    }),
                    error: None,
                    execution_time_ms: 0,
                    output: Some(format!("Successfully created todo item '{}'", todo.title)),
                    warnings: vec![],
                    context: None,
                })
            }

            "update_todo" => {
                let todo_id = args.get("todo_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'todo_id' field".to_string()))?;

                let status_str = args.get("status")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'status' field".to_string()))?;

                let status = match status_str {
                    "pending" => TaskStatus::Pending,
                    "in_progress" => TaskStatus::InProgress,
                    "completed" => TaskStatus::Completed,
                    "paused" => TaskStatus::Paused,
                    "cancelled" => TaskStatus::Cancelled,
                    "failed" => TaskStatus::Failed,
                    _ => return Err(ToolError::InvalidArguments(format!("Invalid status: {}", status_str))),
                };

                self.update_todo_status(todo_id, status).await?;

                Ok(ToolResult {
                    success: true,
                    data: serde_json::json!({
                        "todo_id": todo_id,
                        "status": status.as_str()
                    }),
                    error: None,
                    execution_time_ms: 0,
                    output: Some(format!("Successfully updated todo item {}", todo_id)),
                    warnings: vec![],
                    context: None,
                })
            }

            "list_plans" => {
                let plans = self.list_plans().await;
                Ok(ToolResult {
                    success: true,
                    data: serde_json::json!({
                        "plans": plans
                    }),
                    error: None,
                    execution_time_ms: 0,
                    output: Some(format!("Found {} plans", plans.len())),
                    warnings: vec![],
                    context: None,
                })
            }

            "list_todos" => {
                let status_filter = args.get("status_filter")
                    .and_then(|v| v.as_str())
                    .map(|s| match s {
                        "pending" => TaskStatus::Pending,
                        "in_progress" => TaskStatus::InProgress,
                        "completed" => TaskStatus::Completed,
                        "paused" => TaskStatus::Paused,
                        "cancelled" => TaskStatus::Cancelled,
                        "failed" => TaskStatus::Failed,
                        _ => TaskStatus::Pending,
                    });

                let todos = self.list_todos(status_filter).await;
                Ok(ToolResult {
                    success: true,
                    data: serde_json::json!({
                        "todos": todos,
                        "status_filter": status_filter.map(|s| s.as_str())
                    }),
                    error: None,
                    execution_time_ms: 0,
                    output: Some(format!("Found {} todo items", todos.len())),
                    warnings: vec![],
                    context: None,
                })
            }

            _ => Err(ToolError::InvalidArguments(format!("Unknown action: {}", action))),
        }
    }

    async fn validate_args(&self, args: &serde_json::Value) -> Result<(), ToolError> {
        if !args.is_object() {
            return Err(ToolError::InvalidArguments("Arguments must be an object".to_string()));
        }

        if args.get("action").is_none() {
            return Err(ToolError::InvalidArguments("Missing required field: action".to_string()));
        }

        let action = args.get("action").and_then(|v| v.as_str()).unwrap();

        match action {
            "create_plan" => {
                if args.get("name").is_none() {
                    return Err(ToolError::InvalidArguments("Missing required field: name".to_string()));
                }
                if args.get("goal").is_none() {
                    return Err(ToolError::InvalidArguments("Missing required field: goal".to_string()));
                }
                if args.get("steps").is_none() {
                    return Err(ToolError::InvalidArguments("Missing required field: steps".to_string()));
                }
            }
            "update_step" => {
                if args.get("plan_id").is_none() {
                    return Err(ToolError::InvalidArguments("Missing required field: plan_id".to_string()));
                }
                if args.get("step_id").is_none() {
                    return Err(ToolError::InvalidArguments("Missing required field: step_id".to_string()));
                }
                if args.get("status").is_none() {
                    return Err(ToolError::InvalidArguments("Missing required field: status".to_string()));
                }
            }
            "create_todo" => {
                if args.get("title").is_none() {
                    return Err(ToolError::InvalidArguments("Missing required field: title".to_string()));
                }
            }
            "update_todo" => {
                if args.get("todo_id").is_none() {
                    return Err(ToolError::InvalidArguments("Missing required field: todo_id".to_string()));
                }
                if args.get("status").is_none() {
                    return Err(ToolError::InvalidArguments("Missing required field: status".to_string()));
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn help(&self) -> String {
        r#"Plan & Task Management Tool

Create and manage task plans, project workflows, and todo items.

Actions:
  - create_plan: Create a new task plan
  - update_step: Update plan step status and progress
  - create_todo: Create a new todo item
  - update_todo: Update todo item status
  - list_plans: List all plans
  - list_todos: List todo items (optionally filtered by status)

Examples:

Create Plan:
{
  "action": "create_plan",
  "name": "Website Redesign",
  "description": "Complete website redesign project",
  "goal": "Modern, responsive website",
  "steps": [
    {
      "name": "Design Mockups",
      "description": "Create wireframes and visual designs",
      "dependencies": [],
      "estimated_duration": 240,
      "resources": ["designer", "figma"]
    },
    {
      "name": "Frontend Development",
      "description": "Implement responsive frontend",
      "dependencies": ["Design Mockups"],
      "estimated_duration": 480,
      "resources": ["developer", "react"]
    }
  ]
}

Create Todo:
{
  "action": "create_todo",
  "title": "Review pull request",
  "description": "Review and approve the latest PR",
  "priority": "high"
}

Update Step:
{
  "action": "update_step",
  "plan_id": "plan_123",
  "step_id": "step_1",
  "status": "completed",
  "progress": 1.0
}"#
        .to_string()
    }
}