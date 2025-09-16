use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid::Uuid;
use tracing::{info, warn, error, debug};

use crate::connection_pool::ConnectionPool;
use crate::error::Error;

/// 任务状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    /// 待执行
    Pending,
    /// 执行中
    InProgress,
    /// 已完成
    Completed,
    /// 失败
    Failed(String),
    /// 已取消
    Cancelled,
    /// 等待依赖
    WaitingForDependency(String),
}

/// 任务类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskType {
    /// 工具调用任务
    ToolCall {
        tool_name: String,
        arguments: HashMap<String, serde_json::Value>,
    },
    /// 子任务组
    SubTasks {
        tasks: Vec<String>, // 子任务ID列表
        parallel: bool,     // 是否并行执行
    },
    /// 条件任务
    Conditional {
        condition: String,
        true_task: String,
        false_task: Option<String>,
    },
    /// 循环任务
    Loop {
        condition: String,
        task: String,
        max_iterations: Option<u32>,
    },
    /// 用户交互任务
    UserInteraction {
        prompt: String,
        required: bool,
    },
}

/// 任务定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// 任务ID
    pub id: String,
    /// 任务名称
    pub name: String,
    /// 任务描述
    pub description: String,
    /// 任务类型
    pub task_type: TaskType,
    /// 任务状态
    pub status: TaskStatus,
    /// 依赖任务ID列表
    pub dependencies: Vec<String>,
    /// 任务优先级 (0-10, 数字越大优先级越高)
    pub priority: u8,
    /// 重试次数
    pub retry_count: u32,
    /// 最大重试次数
    pub max_retries: u32,
    /// 创建时间
    pub created_at: u64,
    /// 开始时间
    pub started_at: Option<u64>,
    /// 完成时间
    pub completed_at: Option<u64>,
    /// 任务结果
    pub result: Option<serde_json::Value>,
    /// 错误信息
    pub error: Option<String>,
    /// 任务元数据
    pub metadata: HashMap<String, serde_json::Value>,
}

/// 任务执行上下文
#[derive(Clone)]
pub struct TaskExecutionContext {
    /// 连接池
    pub connection_pool: Arc<ConnectionPool>,
    /// 全局变量
    pub variables: HashMap<String, serde_json::Value>,
    /// 任务结果缓存
    pub task_results: HashMap<String, serde_json::Value>,
}

/// 任务流程定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskFlow {
    /// 流程ID
    pub id: String,
    /// 流程名称
    pub name: String,
    /// 流程描述
    pub description: String,
    /// 任务列表
    pub tasks: Vec<Task>,
    /// 流程状态
    pub status: TaskStatus,
    /// 创建时间
    pub created_at: u64,
    /// 开始时间
    pub started_at: Option<u64>,
    /// 完成时间
    pub completed_at: Option<u64>,
}

/// 任务执行器trait
#[async_trait]
pub trait TaskExecutor: Send + Sync {
    /// 执行单个任务
    async fn execute_task(
        &self,
        task: &Task,
        context: &TaskExecutionContext,
    ) -> Result<serde_json::Value, Error>;
    
    /// 检查任务依赖是否满足
    async fn check_dependencies(
        &self,
        task: &Task,
        context: &TaskExecutionContext,
    ) -> Result<bool, Error>;
    
    /// 处理任务失败
    async fn handle_task_failure(
        &self,
        task: &Task,
        error: Error,
        context: &TaskExecutionContext,
    ) -> Result<(), Error>;
}

/// 任务流程管理器
pub struct TaskFlowManager {
    /// 任务流程存储
    flows: Arc<RwLock<HashMap<String, TaskFlow>>>,
    /// 任务执行器
    executor: Arc<dyn TaskExecutor>,
    /// 连接池
    connection_pool: Arc<ConnectionPool>,
}

impl TaskFlowManager {
    /// 创建新的任务流程管理器
    pub fn new(executor: Arc<dyn TaskExecutor>, connection_pool: Arc<ConnectionPool>) -> Self {
        Self {
            flows: Arc::new(RwLock::new(HashMap::new())),
            executor,
            connection_pool,
        }
    }
    
    /// 创建新的任务流程
    pub async fn create_flow(
        &self,
        name: String,
        description: String,
        tasks: Vec<Task>,
    ) -> Result<String, Error> {
        let flow_id = Uuid::new_v4().to_string();
        let flow = TaskFlow {
            id: flow_id.clone(),
            name,
            description,
            tasks,
            status: TaskStatus::Pending,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            started_at: None,
            completed_at: None,
        };
        
        {
            let mut flows = self.flows.write().await;
            flows.insert(flow_id.clone(), flow);
        }
        
        info!("创建任务流程: {}", flow_id);
        Ok(flow_id)
    }
    
    /// 执行任务流程
    pub async fn execute_flow(&self, flow_id: &str) -> Result<serde_json::Value, Error> {
        let flow = {
            let flows = self.flows.read().await;
            flows.get(flow_id).cloned()
                .ok_or_else(|| Error::Other(format!("任务流程 {} 不存在", flow_id)))?
        };
        
        info!("开始执行任务流程: {}", flow.name);
        
        // 更新流程状态
        {
            let mut flows = self.flows.write().await;
            if let Some(flow) = flows.get_mut(flow_id) {
                flow.status = TaskStatus::InProgress;
                flow.started_at = Some(std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs());
            }
        }
        
        // 创建执行上下文
        let context = TaskExecutionContext {
            connection_pool: self.connection_pool.clone(),
            variables: HashMap::new(),
            task_results: HashMap::new(),
        };
        
        // 执行所有任务
        let results = self.execute_tasks(&flow.tasks, &context).await?;
        
        // 更新流程完成状态
        {
            let mut flows = self.flows.write().await;
            if let Some(flow) = flows.get_mut(flow_id) {
                flow.status = TaskStatus::Completed;
                flow.completed_at = Some(std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs());
            }
        }
        
        info!("任务流程执行完成: {}", flow.name);
        Ok(serde_json::json!({
            "flow_id": flow_id,
            "status": "completed",
            "results": results
        }))
    }
    
    /// 执行任务列表
    async fn execute_tasks(
        &self,
        tasks: &[Task],
        context: &TaskExecutionContext,
    ) -> Result<Vec<serde_json::Value>, Error> {
        let mut results = Vec::new();
        
        for task in tasks {
            debug!("执行任务: {}", task.name);
            
            // 检查依赖
            if !self.executor.check_dependencies(task, context).await? {
                warn!("任务 {} 的依赖未满足，跳过执行", task.name);
                continue;
            }
            
            // 执行任务
            match self.executor.execute_task(task, context).await {
                Ok(result) => {
                    info!("任务 {} 执行成功", task.name);
                    results.push(serde_json::json!({
                        "task_id": task.id,
                        "status": "success",
                        "result": result
                    }));
                }
                Err(e) => {
                    error!("任务 {} 执行失败: {}", task.name, e);
                    results.push(serde_json::json!({
                        "task_id": task.id,
                        "status": "failed",
                        "error": e.to_string()
                    }));
                    
                    // 处理任务失败
                    if let Err(handle_error) = self.executor.handle_task_failure(task, e, context).await {
                        error!("处理任务失败时出错: {}", handle_error);
                    }
                }
            }
        }
        
        Ok(results)
    }
    
    /// 获取任务流程状态
    pub async fn get_flow_status(&self, flow_id: &str) -> Result<TaskStatus, Error> {
        let flows = self.flows.read().await;
        let flow = flows.get(flow_id)
            .ok_or_else(|| Error::Other(format!("任务流程 {} 不存在", flow_id)))?;
        Ok(flow.status.clone())
    }
    
    /// 取消任务流程
    pub async fn cancel_flow(&self, flow_id: &str) -> Result<(), Error> {
        let mut flows = self.flows.write().await;
        if let Some(flow) = flows.get_mut(flow_id) {
            flow.status = TaskStatus::Cancelled;
            flow.completed_at = Some(std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs());
            info!("任务流程 {} 已取消", flow.name);
        }
        Ok(())
    }
    
    /// 列出所有任务流程
    pub async fn list_flows(&self) -> Vec<TaskFlow> {
        let flows = self.flows.read().await;
        flows.values().cloned().collect()
    }
}

/// 默认任务执行器
pub struct DefaultTaskExecutor;

#[async_trait]
impl TaskExecutor for DefaultTaskExecutor {
    async fn execute_task(
        &self,
        task: &Task,
        context: &TaskExecutionContext,
    ) -> Result<serde_json::Value, Error> {
        match &task.task_type {
            TaskType::ToolCall { tool_name, arguments } => {
                self.execute_tool_call(tool_name, arguments, context).await
            }
            TaskType::SubTasks { tasks: _, parallel: _ } => {
                // 子任务执行逻辑
                Ok(serde_json::json!({"message": "子任务执行功能待实现"}))
            }
            TaskType::Conditional { condition: _, true_task: _, false_task: _ } => {
                // 条件任务执行逻辑
                Ok(serde_json::json!({"message": "条件任务执行功能待实现"}))
            }
            TaskType::Loop { condition: _, task: _, max_iterations: _ } => {
                // 循环任务执行逻辑
                Ok(serde_json::json!({"message": "循环任务执行功能待实现"}))
            }
            TaskType::UserInteraction { prompt, required } => {
                // 用户交互任务执行逻辑
                Ok(serde_json::json!({
                    "message": "用户交互任务",
                    "prompt": prompt,
                    "required": required
                }))
            }
        }
    }
    
    async fn check_dependencies(
        &self,
        task: &Task,
        context: &TaskExecutionContext,
    ) -> Result<bool, Error> {
        for dep_id in &task.dependencies {
            if let Some(result) = context.task_results.get(dep_id) {
                // 检查依赖任务是否成功完成
                if let Some(status) = result.get("status") {
                    if status.as_str() != Some("success") {
                        return Ok(false);
                    }
                }
            } else {
                return Ok(false);
            }
        }
        Ok(true)
    }
    
    async fn handle_task_failure(
        &self,
        task: &Task,
        error: Error,
        _context: &TaskExecutionContext,
    ) -> Result<(), Error> {
        warn!("任务 {} 执行失败: {}", task.name, error);
        
        // 这里可以添加失败处理逻辑，比如：
        // 1. 记录错误日志
        // 2. 发送通知
        // 3. 尝试恢复
        // 4. 更新任务状态
        
        Ok(())
    }
}

impl DefaultTaskExecutor {
    /// 执行工具调用任务
    async fn execute_tool_call(
        &self,
        tool_name: &str,
        arguments: &HashMap<String, serde_json::Value>,
        context: &TaskExecutionContext,
    ) -> Result<serde_json::Value, Error> {
        // 根据工具名称确定服务器
        let server_name = self.get_server_for_tool(tool_name);
        
        // 获取连接
        let connection = context.connection_pool.get_connection(&server_name).await?;
        let client = connection.lock().await;
        
        // 调用工具
        let result = client.call_tool(tool_name, serde_json::Value::Object(
            arguments.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
        )).await?;
        
        // 将CallToolResult转换为serde_json::Value
        Ok(serde_json::to_value(result)?)
    }
    
    /// 根据工具名称获取对应的服务器名称
    fn get_server_for_tool(&self, tool_name: &str) -> String {
        // 根据工具名称映射到对应的MCP服务器
        match tool_name {
            // 文件系统工具
            "list_directory" | "read_file" | "write_file" | "get_file_info" | "list_allowed_directories" | "create_directory" | "read_multiple_files" | "edit_file" | "directory_tree" | "move_file" | "search_files" => {
                "filesystem".to_string()
            }
            // 内存工具
            "create_entities" | "create_relations" | "search_nodes" | "read_graph" | "add_observations" | "delete_entities" | "delete_observations" | "delete_relations" => {
                "memory".to_string()
            }
            // 支付工具
            "get_network_info" | "get_supported_tokens" | "create_wallet" | "list_wallets" | "estimate_gas_fees" | "validate_address" | "get_balance" | "send_transaction" | "get_transaction_status" | "set_user_wallet" | "switch_wallet" | "remove_wallet" | "get_wallet_address" => {
                "payment-npm".to_string()
            }
            // 其他工具
            _ => {
                // 默认尝试文件系统服务器
                "filesystem".to_string()
            }
        }
    }
}

/// 任务流程构建器
pub struct TaskFlowBuilder {
    name: String,
    description: String,
    tasks: Vec<Task>,
}

impl TaskFlowBuilder {
    /// 创建新的任务流程构建器
    pub fn new(name: String, description: String) -> Self {
        Self {
            name,
            description,
            tasks: Vec::new(),
        }
    }
    
    /// 添加工具调用任务
    pub fn add_tool_call(
        mut self,
        name: String,
        description: String,
        tool_name: String,
        arguments: HashMap<String, serde_json::Value>,
        dependencies: Vec<String>,
    ) -> Self {
        let task = Task {
            id: Uuid::new_v4().to_string(),
            name,
            description,
            task_type: TaskType::ToolCall { tool_name, arguments },
            status: TaskStatus::Pending,
            dependencies,
            priority: 5,
            retry_count: 0,
            max_retries: 3,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            started_at: None,
            completed_at: None,
            result: None,
            error: None,
            metadata: HashMap::new(),
        };
        self.tasks.push(task);
        self
    }
    
    /// 添加用户交互任务
    pub fn add_user_interaction(
        mut self,
        name: String,
        description: String,
        prompt: String,
        required: bool,
        dependencies: Vec<String>,
    ) -> Self {
        let task = Task {
            id: Uuid::new_v4().to_string(),
            name,
            description,
            task_type: TaskType::UserInteraction { prompt, required },
            status: TaskStatus::Pending,
            dependencies,
            priority: 5,
            retry_count: 0,
            max_retries: 3,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            started_at: None,
            completed_at: None,
            result: None,
            error: None,
            metadata: HashMap::new(),
        };
        self.tasks.push(task);
        self
    }
    
    /// 构建任务流程
    pub fn build(self) -> TaskFlow {
        TaskFlow {
            id: Uuid::new_v4().to_string(),
            name: self.name,
            description: self.description,
            tasks: self.tasks,
            status: TaskStatus::Pending,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            started_at: None,
            completed_at: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_task_flow_builder() {
        let flow = TaskFlowBuilder::new("测试流程".to_string(), "测试任务流程构建器".to_string())
            .add_tool_call(
                "列出目录".to_string(),
                "列出当前目录内容".to_string(),
                "list_directory".to_string(),
                HashMap::new(),
                Vec::new(),
            )
            .build();
        
        assert_eq!(flow.name, "测试流程");
        assert_eq!(flow.tasks.len(), 1);
        assert_eq!(flow.tasks[0].name, "列出目录");
    }
}
