//! 待办事项管理工具
//!
//! 提供任务列表管理功能

use super::{ToolExecutor, ToolMetadata, ToolCategory, ToolPriority, ToolStatus, ExecutionContext, ToolResult, ToolError};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 待办事项工具
pub struct TodoListTool {
    metadata: ToolMetadata,
    todos: std::sync::Arc<std::sync::Mutex<HashMap<String, TodoItem>>>,
}

impl TodoListTool {
    /// 创建新的待办事项工具
    pub fn new() -> Self {
        Self {
            metadata: ToolMetadata {
                id: "todolist".to_string(),
                name: "Todo List Tool".to_string(),
                description: "Manage todo lists and tasks".to_string(),
                category: ToolCategory::Todo,
                priority: ToolPriority::Medium,
                status: ToolStatus::Available,
                version: "1.0.0".to_string(),
                author: "Alou Team".to_string(),
                created_at: chrono::Utc::now().timestamp(),
                updated_at: chrono::Utc::now().timestamp(),
                dependencies: vec![],
                platforms: vec!["windows".to_string(), "macos".to_string(), "linux".to_string()],
                permissions: vec![],
            },
            todos: std::sync::Arc::new(std::sync::Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl ToolExecutor for TodoListTool {
    fn metadata(&self) -> &ToolMetadata {
        &self.metadata
    }

    async fn execute(
        &self,
        args: serde_json::Value,
        _context: &ExecutionContext,
    ) -> Result<ToolResult, ToolError> {
        let todo_op: TodoOperation = serde_json::from_value(args)
            .map_err(|e| ToolError::InvalidArguments(format!("Invalid arguments: {}", e)))?;

        match todo_op {
            TodoOperation::Create { title, description, priority, due_date } =>
                self.create_todo(title, description, priority, due_date).await,
            TodoOperation::Update { id, title, description, status, priority, due_date } =>
                self.update_todo(id, title, description, status, priority, due_date).await,
            TodoOperation::Delete { id } =>
                self.delete_todo(id).await,
            TodoOperation::Get { id } =>
                self.get_todo(id).await,
            TodoOperation::List { status_filter, priority_filter } =>
                self.list_todos(status_filter, priority_filter).await,
            TodoOperation::Clear { status } =>
                self.clear_todos(status).await,
        }
    }

    async fn validate_args(&self, args: &serde_json::Value) -> Result<(), ToolError> {
        if let Ok(_op) = serde_json::from_value::<TodoOperation>(args.clone()) {
            Ok(())
        } else {
            Err(ToolError::InvalidArguments("Invalid todo operation arguments".to_string()))
        }
    }

    fn help(&self) -> String {
        r#"Todo List Tool - Manage tasks and todo items

Available operations:
- create: Create a new todo item
- update: Update an existing todo item
- delete: Delete a todo item
- get: Get a specific todo item
- list: List todo items with optional filters
- clear: Clear completed or all todo items

Create options:
- title: Task title (required)
- description: Task description
- priority: Priority level (low, medium, high, critical)
- due_date: Due date timestamp

Update options:
- id: Todo item ID (required)
- title: New title
- description: New description
- status: New status (pending, in_progress, completed, cancelled)
- priority: New priority
- due_date: New due date

List options:
- status_filter: Filter by status
- priority_filter: Filter by priority

Clear options:
- status: Status to clear (completed, all)

Example usage:
{
  "operation": "create",
  "title": "Review code changes",
  "description": "Review the latest PR",
  "priority": "high"
}"#.to_string()
    }
}

impl TodoListTool {
    /// 创建待办事项
    async fn create_todo(
        &self,
        title: String,
        description: Option<String>,
        priority: Option<String>,
        due_date: Option<i64>,
    ) -> Result<ToolResult, ToolError> {
        let id = format!("todo_{}", uuid::Uuid::new_v4().to_string());
        let priority = priority.unwrap_or_else(|| "medium".to_string());
        let priority_enum = match priority.as_str() {
            "low" => TodoPriority::Low,
            "medium" => TodoPriority::Medium,
            "high" => TodoPriority::High,
            "critical" => TodoPriority::Critical,
            _ => TodoPriority::Medium,
        };

        let todo = TodoItem {
            id: id.clone(),
            title,
            description,
            status: TodoStatus::Pending,
            priority: priority_enum,
            created_at: chrono::Utc::now().timestamp(),
            updated_at: chrono::Utc::now().timestamp(),
            due_date,
            completed_at: None,
        };

        let mut todos = self.todos.lock().unwrap();
        todos.insert(id.clone(), todo.clone());

        Ok(ToolResult {
            success: true,
            data: serde_json::to_value(&todo).unwrap(),
            error: None,
            execution_time_ms: 0,
            output: Some(format!("Created todo item: {}", id)),
            warnings: vec![],
            context: None,
        })
    }

    /// 更新待办事项
    async fn update_todo(
        &self,
        id: String,
        title: Option<String>,
        description: Option<String>,
        status: Option<String>,
        priority: Option<String>,
        due_date: Option<i64>,
    ) -> Result<ToolResult, ToolError> {
        let mut todos = self.todos.lock().unwrap();

        let todo = todos.get_mut(&id)
            .ok_or_else(|| ToolError::ExecutionFailed(format!("Todo item '{}' not found", id)))?;

        if let Some(title) = title {
            todo.title = title;
        }

        if let Some(description) = description {
            todo.description = Some(description);
        }

        if let Some(status_str) = status {
            todo.status = match status_str.as_str() {
                "pending" => TodoStatus::Pending,
                "in_progress" => TodoStatus::InProgress,
                "completed" => {
                    todo.completed_at = Some(chrono::Utc::now().timestamp());
                    TodoStatus::Completed
                }
                "cancelled" => TodoStatus::Cancelled,
                _ => return Err(ToolError::InvalidArguments("Invalid status".to_string())),
            };
        }

        if let Some(priority_str) = priority {
            todo.priority = match priority_str.as_str() {
                "low" => TodoPriority::Low,
                "medium" => TodoPriority::Medium,
                "high" => TodoPriority::High,
                "critical" => TodoPriority::Critical,
                _ => return Err(ToolError::InvalidArguments("Invalid priority".to_string())),
            };
        }

        if let Some(due_date) = due_date {
            todo.due_date = Some(due_date);
        }

        todo.updated_at = chrono::Utc::now().timestamp();

        let todo_clone = todo.clone();

        Ok(ToolResult {
            success: true,
            data: serde_json::to_value(&todo_clone).unwrap(),
            error: None,
            execution_time_ms: 0,
            output: Some(format!("Updated todo item: {}", id)),
            warnings: vec![],
            context: None,
        })
    }

    /// 删除待办事项
    async fn delete_todo(&self, id: String) -> Result<ToolResult, ToolError> {
        let mut todos = self.todos.lock().unwrap();

        if todos.remove(&id).is_some() {
            Ok(ToolResult {
                success: true,
                data: serde_json::json!({
                    "id": id,
                    "deleted": true
                }),
                error: None,
                execution_time_ms: 0,
                output: Some(format!("Deleted todo item: {}", id)),
                warnings: vec![],
                context: None,
            })
        } else {
            Err(ToolError::ExecutionFailed(format!("Todo item '{}' not found", id)))
        }
    }

    /// 获取待办事项
    async fn get_todo(&self, id: String) -> Result<ToolResult, ToolError> {
        let todos = self.todos.lock().unwrap();

        if let Some(todo) = todos.get(&id) {
            Ok(ToolResult {
                success: true,
                data: serde_json::to_value(todo).unwrap(),
                error: None,
                execution_time_ms: 0,
                output: Some(format!("Retrieved todo item: {}", id)),
                warnings: vec![],
                context: None,
            })
        } else {
            Err(ToolError::ExecutionFailed(format!("Todo item '{}' not found", id)))
        }
    }

    /// 列出待办事项
    async fn list_todos(
        &self,
        status_filter: Option<String>,
        priority_filter: Option<String>,
    ) -> Result<ToolResult, ToolError> {
        let todos = self.todos.lock().unwrap();

        let mut filtered_todos: Vec<&TodoItem> = todos.values().collect();

        // 状态过滤
        if let Some(status_str) = status_filter {
            let target_status = match status_str.as_str() {
                "pending" => TodoStatus::Pending,
                "in_progress" => TodoStatus::InProgress,
                "completed" => TodoStatus::Completed,
                "cancelled" => TodoStatus::Cancelled,
                _ => return Err(ToolError::InvalidArguments("Invalid status filter".to_string())),
            };
            filtered_todos.retain(|t| t.status == target_status);
        }

        // 优先级过滤
        if let Some(priority_str) = priority_filter {
            let target_priority = match priority_str.as_str() {
                "low" => TodoPriority::Low,
                "medium" => TodoPriority::Medium,
                "high" => TodoPriority::High,
                "critical" => TodoPriority::Critical,
                _ => return Err(ToolError::InvalidArguments("Invalid priority filter".to_string())),
            };
            filtered_todos.retain(|t| t.priority == target_priority);
        }

        // 按创建时间排序
        let mut sorted_todos = filtered_todos.into_iter().cloned().collect::<Vec<_>>();
        sorted_todos.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        Ok(ToolResult {
            success: true,
            data: serde_json::json!({
                "todos": sorted_todos,
                "count": sorted_todos.len()
            }),
            error: None,
            execution_time_ms: 0,
            output: Some(format!("Listed {} todo items", sorted_todos.len())),
            warnings: vec![],
            context: None,
        })
    }

    /// 清除待办事项
    async fn clear_todos(&self, status: Option<String>) -> Result<ToolResult, ToolError> {
        let mut todos = self.todos.lock().unwrap();

        let initial_count = todos.len();

        match status.as_deref() {
            Some("completed") => {
                todos.retain(|_, t| t.status != TodoStatus::Completed);
            }
            Some("all") => {
                todos.clear();
            }
            _ => return Err(ToolError::InvalidArguments("Invalid status for clear operation".to_string())),
        }

        let removed_count = initial_count - todos.len();

        Ok(ToolResult {
            success: true,
            data: serde_json::json!({
                "removed_count": removed_count,
                "remaining_count": todos.len()
            }),
            error: None,
            execution_time_ms: 0,
            output: Some(format!("Cleared {} todo items", removed_count)),
            warnings: vec![],
            context: None,
        })
    }
}

/// 待办操作枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "operation")]
pub enum TodoOperation {
    /// 创建待办事项
    Create {
        title: String,
        description: Option<String>,
        priority: Option<String>,
        due_date: Option<i64>,
    },
    /// 更新待办事项
    Update {
        id: String,
        title: Option<String>,
        description: Option<String>,
        status: Option<String>,
        priority: Option<String>,
        due_date: Option<i64>,
    },
    /// 删除待办事项
    Delete {
        id: String,
    },
    /// 获取待办事项
    Get {
        id: String,
    },
    /// 列出待办事项
    List {
        status_filter: Option<String>,
        priority_filter: Option<String>,
    },
    /// 清除待办事项
    Clear {
        status: Option<String>,
    },
}

/// 待办事项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoItem {
    /// 唯一ID
    pub id: String,
    /// 标题
    pub title: String,
    /// 描述
    pub description: Option<String>,
    /// 状态
    pub status: TodoStatus,
    /// 优先级
    pub priority: TodoPriority,
    /// 创建时间
    pub created_at: i64,
    /// 更新时间
    pub updated_at: i64,
    /// 截止时间
    pub due_date: Option<i64>,
    /// 完成时间
    pub completed_at: Option<i64>,
}

/// 待办状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TodoStatus {
    /// 待处理
    Pending,
    /// 进行中
    InProgress,
    /// 已完成
    Completed,
    /// 已取消
    Cancelled,
}

/// 待办优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TodoPriority {
    /// 低优先级
    Low,
    /// 中等优先级
    Medium,
    /// 高优先级
    High,
    /// 紧急优先级
    Critical,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_and_get_todo() {
        let tool = TodoListTool::new();
        let context = ExecutionContext {
            session_id: "test".to_string(),
            user_id: None,
            working_directory: None,
            environment: std::collections::HashMap::new(),
            timeout_seconds: Some(30),
            permissions: vec![],
            timestamp: chrono::Utc::now().timestamp(),
        };

        // 创建待办事项
        let create_args = serde_json::json!({
            "operation": "create",
            "title": "Test Task",
            "description": "A test todo item",
            "priority": "high"
        });

        let result = tool.execute(create_args, &context).await.unwrap();
        assert!(result.success);

        let todo_id = result.data["id"].as_str().unwrap();

        // 获取待办事项
        let get_args = serde_json::json!({
            "operation": "get",
            "id": todo_id
        });

        let result = tool.execute(get_args, &context).await.unwrap();
        assert!(result.success);
        assert_eq!(result.data["title"], "Test Task");
    }

    #[tokio::test]
    async fn test_update_todo_status() {
        let tool = TodoListTool::new();
        let context = ExecutionContext {
            session_id: "test".to_string(),
            user_id: None,
            working_directory: None,
            environment: std::collections::HashMap::new(),
            timeout_seconds: Some(30),
            permissions: vec![],
            timestamp: chrono::Utc::now().timestamp(),
        };

        // 创建待办事项
        let create_args = serde_json::json!({
            "operation": "create",
            "title": "Test Task"
        });

        let result = tool.execute(create_args, &context).await.unwrap();
        let todo_id = result.data["id"].as_str().unwrap();

        // 更新状态为完成
        let update_args = serde_json::json!({
            "operation": "update",
            "id": todo_id,
            "status": "completed"
        });

        let result = tool.execute(update_args, &context).await.unwrap();
        assert!(result.success);
        assert_eq!(result.data["status"], "Completed");
        assert!(result.data["completed_at"].is_number());
    }
}