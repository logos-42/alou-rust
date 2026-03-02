//! Task 执行器 - 负责任务的实际执行
//! 
//! 基于人月神话的执行原则:
//! - 原子性: 每个执行单元要么成功要么失败
//! - 隔离性: 执行环境相互隔离
//! - 幂等性: 重复执行结果一致

use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use tokio::sync::RwLock;
use tokio::time::{Duration, timeout};

use crate::swarm::types::*;
use crate::swarm::skill_registry::SkillRegistry;

/// 任务执行器 trait
#[async_trait]
pub trait TaskExecutor: Send + Sync {
    /// 获取执行器 ID
    fn id(&self) -> &str;
    
    /// 获取执行器能力
    fn capabilities(&self) -> &[String];
    
    /// 获取当前负载 (0-1)
    async fn current_load(&self) -> f32;
    
    /// 检查是否能执行特定任务
    async fn can_execute(&self, task: &Task) -> bool;
    
    /// 执行任务
    async fn execute(&self, task: &Task) -> Result<TaskResult, TaskError>;
    
    /// 取消任务
    async fn cancel(&self, task_id: &str) -> Result<(), String>;
    
    /// 获取执行器状态
    async fn status(&self) -> AgentStatus;
}

/// 本地执行器 - 在当前进程中执行任务
pub struct LocalExecutor {
    /// 执行器 ID
    id: String,
    /// 执行器名称
    name: String,
    /// 能力列表
    capabilities: Vec<String>,
    /// 当前负载
    load: Arc<RwLock<f32>>,
    /// Skill 注册表
    skill_registry: Arc<SkillRegistry>,
    /// 活跃任务
    active_tasks: Arc<RwLock<HashMap<String, TaskHandle>>>,
}

/// 任务句柄
#[derive(Debug, Clone)]
struct TaskHandle {
    task_id: String,
    started_at: i64,
}

impl LocalExecutor {
    /// 创建新的本地执行器
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        capabilities: Vec<String>,
        skill_registry: Arc<SkillRegistry>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            capabilities,
            load: Arc::new(RwLock::new(0.0)),
            skill_registry,
            active_tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// 创建默认执行器
    pub fn default_with_registry(skill_registry: Arc<SkillRegistry>) -> Self {
        Self::new(
            format!("executor-{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap()),
            "Default Local Executor",
            vec![
                "bash".to_string(),
                "python".to_string(),
                "file_read".to_string(),
                "file_write".to_string(),
            ],
            skill_registry,
        )
    }
    
    /// 更新负载
    async fn update_load(&self, delta: f32) {
        let mut load = self.load.write().await;
        *load = (*load + delta).clamp(0.0, 1.0);
    }
    
    /// 执行单个步骤
    async fn execute_step(
        &self,
        step: &TaskStep,
        task_context: &TaskContext,
    ) -> Result<serde_json::Value, TaskError> {
        let start_time = chrono::Utc::now().timestamp_millis();
        
        // 检查条件
        if let Some(condition) = &step.condition {
            if !self.evaluate_condition(condition, task_context).await {
                return Ok(serde_json::json!({"skipped": true, "reason": "condition_not_met"}));
            }
        }
        
        // 检查依赖
        if let Some(depends_on) = &step.depends_on {
            for dep_id in depends_on {
                // 检查依赖步骤是否完成
                // 实际实现中需要从 task_context 获取步骤状态
            }
        }
        
        // 执行动作
        let result = match step.action.as_str() {
            "bash" => self.execute_bash_step(step).await,
            "python" => self.execute_python_step(step).await,
            "node" | "javascript" => self.execute_node_step(step).await,
            "file_read" => self.execute_file_read_step(step).await,
            "file_write" => self.execute_file_write_step(step).await,
            "web_search" => self.execute_web_search_step(step).await,
            "skill" => self.execute_skill_step(step).await,
            _ => {
                // 尝试从 Skill Registry 执行
                self.execute_skill_step(step).await
            }
        };
        
        let duration = chrono::Utc::now().timestamp_millis() - start_time;
        
        match result {
            Ok(output) => {
                Ok(serde_json::json!({
                    "success": true,
                    "output": output,
                    "duration_ms": duration,
                }))
            }
            Err(e) => {
                if step.optional {
                    // 可选步骤失败不影响整体
                    Ok(serde_json::json!({
                        "success": false,
                        "error": e.message,
                        "optional": true,
                        "duration_ms": duration,
                    }))
                } else {
                    Err(e)
                }
            }
        }
    }
    
    /// 评估条件表达式
    async fn evaluate_condition(&self, condition: &str, _context: &TaskContext) -> bool {
        // 简单的条件评估实现
        // 实际实现中可以使用表达式引擎如 evalexpr
        condition == "true" || condition.is_empty()
    }
    
    /// 执行 Bash 步骤
    async fn execute_bash_step(&self, step: &TaskStep) -> Result<serde_json::Value, TaskError> {
        use tokio::process::Command;
        
        let command = step.parameters
            .as_ref()
            .and_then(|p| p.get("command"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| TaskError::new(
                "MISSING_COMMAND",
                "Bash step requires 'command' parameter"
            ))?;
        
        let output = Command::new("bash")
            .arg("-c")
            .arg(command)
            .output()
            .await
            .map_err(|e| TaskError::new(
                "EXECUTION_FAILED",
                format!("Failed to execute bash command: {}", e)
            ))?;
        
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            Ok(serde_json::json!({
                "stdout": stdout.to_string(),
                "stderr": String::from_utf8_lossy(&output.stderr).to_string(),
                "exit_code": 0,
            }))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(TaskError::new(
                "COMMAND_FAILED",
                format!("Command failed with exit code {:?}: {}", 
                    output.status.code(), stderr)
            ))
        }
    }
    
    /// 执行 Python 步骤
    async fn execute_python_step(&self, step: &TaskStep) -> Result<serde_json::Value, TaskError> {
        use tokio::process::Command;
        
        let code = step.parameters
            .as_ref()
            .and_then(|p| p.get("code"))
            .and_then(|v| v.as_str())
            .or_else(|| {
                step.parameters
                    .as_ref()
                    .and_then(|p| p.get("script"))
                    .and_then(|v| v.as_str())
            });
        
        if let Some(code) = code {
            let output = Command::new("python3")
                .arg("-c")
                .arg(code)
                .output()
                .await
                .map_err(|e| TaskError::new(
                    "EXECUTION_FAILED",
                    format!("Failed to execute Python code: {}", e)
                ))?;
            
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                // 尝试解析为 JSON
                match serde_json::from_str::<serde_json::Value>(&stdout) {
                    Ok(json) => Ok(json),
                    Err(_) => Ok(serde_json::json!({"output": stdout.to_string()})),
                }
            } else {
                Err(TaskError::new(
                    "PYTHON_ERROR",
                    String::from_utf8_lossy(&output.stderr).to_string()
                ))
            }
        } else {
            Err(TaskError::new(
                "MISSING_CODE",
                "Python step requires 'code' or 'script' parameter"
            ))
        }
    }
    
    /// 执行 Node.js 步骤
    async fn execute_node_step(&self, step: &TaskStep) -> Result<serde_json::Value, TaskError> {
        use tokio::process::Command;
        
        let code = step.parameters
            .as_ref()
            .and_then(|p| p.get("code"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| TaskError::new(
                "MISSING_CODE",
                "Node step requires 'code' parameter"
            ))?;
        
        let output = Command::new("node")
            .arg("-e")
            .arg(code)
            .output()
            .await
            .map_err(|e| TaskError::new(
                "EXECUTION_FAILED",
                format!("Failed to execute Node.js code: {}", e)
            ))?;
        
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            match serde_json::from_str::<serde_json::Value>(&stdout) {
                Ok(json) => Ok(json),
                Err(_) => Ok(serde_json::json!({"output": stdout.to_string()})),
            }
        } else {
            Err(TaskError::new(
                "NODE_ERROR",
                String::from_utf8_lossy(&output.stderr).to_string()
            ))
        }
    }
    
    /// 执行文件读取步骤
    async fn execute_file_read_step(&self, step: &TaskStep) -> Result<serde_json::Value, TaskError> {
        use tokio::fs;
        
        let path = step.parameters
            .as_ref()
            .and_then(|p| p.get("path"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| TaskError::new(
                "MISSING_PATH",
                "File read step requires 'path' parameter"
            ))?;
        
        let content = fs::read_to_string(path)
            .await
            .map_err(|e| TaskError::new(
                "READ_FAILED",
                format!("Failed to read file {}: {}", path, e)
            ))?;
        
        Ok(serde_json::json!({
            "path": path,
            "content": content,
            "size": content.len(),
        }))
    }
    
    /// 执行文件写入步骤
    async fn execute_file_write_step(&self, step: &TaskStep) -> Result<serde_json::Value, TaskError> {
        use tokio::fs;
        
        let path = step.parameters
            .as_ref()
            .and_then(|p| p.get("path"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| TaskError::new(
                "MISSING_PATH",
                "File write step requires 'path' parameter"
            ))?;
        
        let content = step.parameters
            .as_ref()
            .and_then(|p| p.get("content"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| TaskError::new(
                "MISSING_CONTENT",
                "File write step requires 'content' parameter"
            ))?;
        
        fs::write(path, content)
            .await
            .map_err(|e| TaskError::new(
                "WRITE_FAILED",
                format!("Failed to write file {}: {}", path, e)
            ))?;
        
        Ok(serde_json::json!({
            "path": path,
            "bytes_written": content.len(),
            "success": true,
        }))
    }
    
    /// 执行网络搜索步骤
    async fn execute_web_search_step(&self, step: &TaskStep) -> Result<serde_json::Value, TaskError> {
        let query = step.parameters
            .as_ref()
            .and_then(|p| p.get("query"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| TaskError::new(
                "MISSING_QUERY",
                "Web search step requires 'query' parameter"
            ))?;
        
        // 实际实现中会调用搜索 API
        // 这里返回模拟结果
        Ok(serde_json::json!({
            "query": query,
            "results": [
                {"title": "Result 1", "url": "https://example.com/1"},
                {"title": "Result 2", "url": "https://example.com/2"},
            ],
            "note": "This is a simulated search result"
        }))
    }
    
    /// 执行 Skill 步骤
    async fn execute_skill_step(&self, step: &TaskStep) -> Result<serde_json::Value, TaskError> {
        let skill_name = step.action.clone();
        let input = step.parameters.clone().unwrap_or_default();
        
        match self.skill_registry.execute(&skill_name, input).await {
            Ok(result) => Ok(result),
            Err(e) => Err(TaskError::new(
                "SKILL_EXECUTION_FAILED",
                format!("Failed to execute skill {}: {}", skill_name, e)
            )),
        }
    }
}

#[async_trait]
impl TaskExecutor for LocalExecutor {
    fn id(&self) -> &str {
        &self.id
    }
    
    fn capabilities(&self) -> &[String] {
        &self.capabilities
    }
    
    async fn current_load(&self) -> f32 {
        *self.load.read().await
    }
    
    async fn can_execute(&self, task: &Task) -> bool {
        // 检查能力匹配
        if let Some(required) = &task.required_capabilities {
            for cap in required {
                if !self.capabilities.contains(cap) {
                    return false;
                }
            }
        }
        
        // 检查负载
        let load = self.current_load().await;
        load < 0.9
    }
    
    async fn execute(&self, task: &Task) -> Result<TaskResult, TaskError> {
        // 更新负载
        self.update_load(0.3).await;
        
        // 添加到活跃任务
        {
            let mut active = self.active_tasks.write().await;
            active.insert(
                task.id.clone(),
                TaskHandle {
                    task_id: task.id.clone(),
                    started_at: chrono::Utc::now().timestamp_millis(),
                }
            );
        }
        
        let start_time = chrono::Utc::now().timestamp_millis();
        let timeout_ms = task.timeout.unwrap_or(300_000);
        
        // 创建任务上下文
        let task_context = TaskContext {
            task_id: task.id.clone(),
            workspace: format!("/tmp/agent-tasks/{}", task.id),
            input: task.parameters.clone().unwrap_or_default(),
            temp: HashMap::new(),
        };
        
        // 执行任务
        let result = timeout(
            Duration::from_millis(timeout_ms),
            self.execute_task_internal(task, &task_context)
        ).await;
        
        // 从活跃任务移除
        {
            let mut active = self.active_tasks.write().await;
            active.remove(&task.id);
        }
        
        // 更新负载
        self.update_load(-0.3).await;
        
        match result {
            Ok(Ok(task_result)) => Ok(task_result),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(TaskError::new(
                "TIMEOUT",
                format!("Task execution timed out after {}ms", timeout_ms)
            )),
        }
    }
    
    async fn cancel(&self, task_id: &str) -> Result<(), String> {
        // 从活跃任务中移除
        let mut active = self.active_tasks.write().await;
        if active.remove(task_id).is_some() {
            self.update_load(-0.3).await;
            Ok(())
        } else {
            Err(format!("Task {} not found in active tasks", task_id))
        }
    }
    
    async fn status(&self) -> AgentStatus {
        let load = self.current_load().await;
        let active_count = self.active_tasks.read().await.len();
        
        if load > 0.9 {
            AgentStatus::Busy
        } else if active_count > 0 {
            AgentStatus::Online
        } else {
            AgentStatus::Idle
        }
    }
}

impl LocalExecutor {
    /// 内部任务执行
    async fn execute_task_internal(
        &self,
        task: &Task,
        context: &TaskContext,
    ) -> Result<TaskResult, TaskError> {
        let mut steps_completed = 0u32;
        let mut steps_failed = 0u32;
        let mut step_results = Vec::new();
        
        for step in &task.steps {
            match self.execute_step(step, context).await {
                Ok(result) => {
                    steps_completed += 1;
                    step_results.push(result);
                }
                Err(e) => {
                    steps_failed += 1;
                    if !step.optional {
                        return Err(e);
                    }
                }
            }
        }
        
        let duration = chrono::Utc::now().timestamp_millis() - 
            task.started_at.unwrap_or(task.created_at);
        
        Ok(TaskResult {
            success: steps_failed == 0 || steps_completed > 0,
            output: serde_json::json!({
                "steps": step_results,
                "completed": steps_completed,
                "failed": steps_failed,
            }),
            summary: Some(format!(
                "Completed {}/{} steps in {}ms",
                steps_completed,
                task.steps.len(),
                duration
            )),
            artifacts: None,
            metrics: Some(TaskMetrics {
                duration,
                steps_completed,
                steps_failed,
                agents_involved: 1,
                retries_count: task.retry_count,
                cost_estimate: None,
            }),
        })
    }
}

/// 任务上下文
#[derive(Debug, Clone)]
pub struct TaskContext {
    pub task_id: String,
    pub workspace: String,
    pub input: HashMap<String, serde_json::Value>,
    pub temp: HashMap<String, serde_json::Value>,
}

/// 远程执行器 - 通过 RPC 调用远程 Agent
pub struct RemoteExecutor {
    id: String,
    endpoint: String,
    capabilities: Vec<String>,
}

impl RemoteExecutor {
    pub fn new(id: impl Into<String>, endpoint: impl Into<String>, capabilities: Vec<String>) -> Self {
        Self {
            id: id.into(),
            endpoint: endpoint.into(),
            capabilities,
        }
    }
}

#[async_trait]
impl TaskExecutor for RemoteExecutor {
    fn id(&self) -> &str {
        &self.id
    }
    
    fn capabilities(&self) -> &[String] {
        &self.capabilities
    }
    
    async fn current_load(&self) -> f32 {
        // 远程调用获取负载
        0.5 // 模拟
    }
    
    async fn can_execute(&self, task: &Task) -> bool {
        if let Some(required) = &task.required_capabilities {
            for cap in required {
                if !self.capabilities.contains(cap) {
                    return false;
                }
            }
        }
        true
    }
    
    async fn execute(&self, task: &Task) -> Result<TaskResult, TaskError> {
        // 远程 RPC 调用
        // 实际实现中使用 gRPC 或 HTTP
        Err(TaskError::new(
            "NOT_IMPLEMENTED",
            "Remote execution not yet implemented"
        ))
    }
    
    async fn cancel(&self, _task_id: &str) -> Result<(), String> {
        Err("Remote cancellation not yet implemented".to_string())
    }
    
    async fn status(&self) -> AgentStatus {
        AgentStatus::Online
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_step(action: &str, params: Option<HashMap<String, serde_json::Value>>) -> TaskStep {
        TaskStep {
            id: format!("step-{}", uuid::Uuid::new_v4()),
            name: "Test Step".to_string(),
            description: None,
            action: action.to_string(),
            parameters: params,
            depends_on: None,
            condition: None,
            parallel: false,
            optional: false,
            retry_count: None,
            status: None,
            result: None,
            error: None,
            started_at: None,
            completed_at: None,
        }
    }
    
    #[tokio::test]
    async fn test_executor_creation() {
        let registry = Arc::new(SkillRegistry::new());
        let executor = LocalExecutor::default_with_registry(registry);
        
        assert!(!executor.id().is_empty());
        assert!(!executor.capabilities().is_empty());
        assert_eq!(executor.current_load().await, 0.0);
    }
}