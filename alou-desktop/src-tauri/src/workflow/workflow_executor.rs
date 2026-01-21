//! 异步工作流执行器
//!
//! 提供异步工作流执行功能，支持并发步骤执行、实时进度更新和错误处理

pub use super::*;

use crate::workflow::{Workflow, WorkflowStep};
use super::{WorkflowStorage, StorageConfig};
use super::WorkflowMonitor;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex, RwLock};
use tokio::time::{sleep, Duration};
use uuid;

/// 异步工作流执行器
pub struct AsyncWorkflowExecutor {
    /// 活跃的执行任务（内存缓存）
    pub active_executions: Arc<RwLock<HashMap<String, WorkflowExecution>>>,
    /// 工作流定义存储
    pub workflows: Arc<RwLock<HashMap<String, Workflow>>>,
    /// 持久化存储
    pub storage: Arc<WorkflowStorage>,
    /// 重试配置
    pub retry_config: RetryConfig,
    /// Ralph Loop配置
    pub ralph_loop_config: RalphLoopConfig,
    /// Ralph Loop执行历史存储
    pub ralph_loop_histories: Arc<RwLock<HashMap<String, RalphLoopExecutionHistory>>>,
    /// 监控器
    pub monitor: Arc<WorkflowMonitor>,
    /// 事件发送器
    pub event_sender: mpsc::UnboundedSender<ExecutionEvent>,
    /// 事件接收器
    pub event_receiver: Arc<Mutex<mpsc::UnboundedReceiver<ExecutionEvent>>>,
    /// 执行任务句柄存储
    pub execution_handles: Arc<RwLock<HashMap<String, tokio::task::JoinHandle<()>>>>,
    /// 桥接管理器
    pub bridge_manager: Arc<crate::bridges::BridgeManager>,
}

impl AsyncWorkflowExecutor {
    /// 创建新的异步工作流执行器
    pub fn new() -> Result<Self, String> {
        let (tx, rx) = mpsc::unbounded_channel();
        let storage = Arc::new(WorkflowStorage::new(StorageConfig::default())?);
        let monitor = Arc::new(WorkflowMonitor::default());
        let bridge_manager = Arc::new(crate::bridges::create_default_bridge_manager());

        Ok(Self {
            active_executions: Arc::new(RwLock::new(HashMap::new())),
            workflows: Arc::new(RwLock::new(HashMap::new())),
            storage,
            retry_config: RetryConfig::default(),
            ralph_loop_config: RalphLoopConfig::default(),
            ralph_loop_histories: Arc::new(RwLock::new(HashMap::new())),
            monitor,
            event_sender: tx,
            event_receiver: Arc::new(Mutex::new(rx)),
            execution_handles: Arc::new(RwLock::new(HashMap::new())),
            bridge_manager,
        })
    }

    /// 注册工作流定义
    pub async fn register_workflow(&self, workflow: Workflow) -> Result<(), String> {
        let mut workflows = self.workflows.write().await;
        workflows.insert(workflow.id.clone(), workflow);
        Ok(())
    }

    /// 启动工作流执行（支持Ralph Loop）
    pub async fn start_execution(
        &self,
        workflow_id: String,
        api_key: String,
        agent_info: Option<serde_json::Value>,
    ) -> Result<String, String> {
        self.start_execution_with_config(workflow_id, api_key, agent_info, None).await
    }

    /// 启动工作流执行（带Ralph Loop配置）
    pub async fn start_execution_with_ralph_loop(
        &self,
        workflow_id: String,
        api_key: String,
        agent_info: Option<serde_json::Value>,
        ralph_config: RalphLoopConfig,
    ) -> Result<String, String> {
        self.start_execution_with_config(workflow_id, api_key, agent_info, Some(ralph_config)).await
    }

    /// 启动工作流执行（内部方法）
    async fn start_execution_with_config(
        &self,
        workflow_id: String,
        api_key: String,
        agent_info: Option<serde_json::Value>,
        ralph_config: Option<RalphLoopConfig>,
    ) -> Result<String, String> {
        // 获取工作流定义
        let workflow = {
            let workflows = self.workflows.read().await;
            workflows.get(&workflow_id).cloned()
                .ok_or_else(|| format!("Workflow '{}' not found", workflow_id))?
        };

        // 创建执行上下文
        let execution_id = format!("exec_{}", uuid::Uuid::new_v4().to_string());
        let execution = WorkflowExecution {
            execution_id: execution_id.clone(),
            workflow_id: workflow_id.clone(),
            status: ExecutionStatus::Pending,
            current_step: None,
            progress: 0.0,
            step_results: HashMap::new(),
            started_at: chrono::Utc::now().timestamp(),
            completed_at: None,
            total_execution_time_ms: None,
            error: None,
        };

        // 存储执行上下文
        {
            let mut executions = self.active_executions.write().await;
            executions.insert(execution_id.clone(), execution.clone());
        }

        // 发送开始事件
        let _ = self.event_sender.send(ExecutionEvent::Started {
            execution_id: execution_id.clone(),
            workflow_id: workflow_id.clone(),
        });

        // 记录执行开始日志
        self.monitor.log_execution_started(&execution).await;

        // 根据Ralph Loop配置选择执行方式
        let executor = Arc::new(self.clone());
        let execution_id_clone = execution_id.clone();
        let ralph_config = ralph_config.unwrap_or_else(|| self.ralph_loop_config.clone());

        let handle = if ralph_config.enabled {
            // 启用Ralph Loop
            tokio::spawn(async move {
                if let Err(e) = executor.execute_workflow_with_ralph_loop(
                    execution_id_clone,
                    workflow,
                    api_key,
                    agent_info,
                    ralph_config
                ).await {
                    eprintln!("Ralph Loop workflow execution failed: {}", e);
                }
            })
        } else {
            // 普通执行
            tokio::spawn(async move {
                if let Err(e) = executor.execute_workflow_async(execution_id_clone, workflow, api_key, agent_info).await {
                    eprintln!("Workflow execution failed: {}", e);
                }
            })
        };

        // 存储任务句柄
        {
            let mut handles = self.execution_handles.write().await;
            handles.insert(execution_id.clone(), handle);
        }

        Ok(execution_id)
    }

    /// 异步执行工作流
    async fn execute_workflow_async(
        &self,
        execution_id: String,
        workflow: Workflow,
        api_key: String,
        agent_info: Option<serde_json::Value>,
    ) -> Result<(), String> {
        // 更新执行状态为运行中
        self.update_execution_status(&execution_id, ExecutionStatus::Running).await;

        // 保存初始状态到存储
        if let Some(execution) = self.active_executions.read().await.get(&execution_id).cloned() {
            if let Err(e) = self.storage.save_execution(&execution).await {
                eprintln!("Failed to save execution state: {}", e);
            }
        }

        let total_steps = workflow.steps.len();
        let mut completed_steps = 0;

        // 构建步骤依赖图
        let dependency_graph = self.build_dependency_graph(&workflow.steps);

        // 执行步骤（简化版本：按顺序执行，实际应该支持并发）
        for step in &workflow.steps {
            // 检查是否有未完成的依赖
            if !self.check_dependencies_completed(&execution_id, &step.depends_on).await {
                // 等待依赖完成（简化实现）
                sleep(Duration::from_millis(100)).await;
                continue;
            }

            // 执行步骤
            let step_result = self.execute_step(&execution_id, step, &api_key, &agent_info).await;

            // 更新步骤结果
            self.update_step_result(&execution_id, step.id.clone(), step_result.clone()).await;

            // 定期保存状态
            if let Some(execution) = self.active_executions.read().await.get(&execution_id).cloned() {
                if let Err(e) = self.storage.save_execution(&execution).await {
                    eprintln!("Failed to save execution state: {}", e);
                }
            }

            // 发送步骤完成事件
            match step_result.status {
                ExecutionStatus::Completed => {
                    let _ = self.event_sender.send(ExecutionEvent::StepCompleted {
                        execution_id: execution_id.clone(),
                        step_result: step_result.clone(),
                    });
                    // 记录步骤完成日志
                    // TODO: 需要传递workflow_id，这里暂时使用空字符串
                    self.monitor.log_step_completed(&execution_id, "", &step_result).await;
                }
                ExecutionStatus::Failed => {
                    let _ = self.event_sender.send(ExecutionEvent::StepFailed {
                        execution_id: execution_id.clone(),
                        step_result: step_result.clone(),
                    });
                    // 记录步骤失败日志
                    // TODO: 需要传递workflow_id，这里暂时使用空字符串
                    self.monitor.log_step_failed(&execution_id, "", &step_result).await;
                    // 步骤失败时继续执行其他步骤（可配置）
                }
                _ => {}
            }

            completed_steps += 1;
            let progress = completed_steps as f32 / total_steps as f32;

            // 发送进度更新事件
            let _ = self.event_sender.send(ExecutionEvent::ProgressUpdated {
                execution_id: execution_id.clone(),
                progress,
                current_step: Some(step.id.clone()),
            });

            // 记录进度更新日志
            // TODO: 需要传递workflow_id，这里暂时使用空字符串
            self.monitor.log_progress_update(&execution_id, "", progress, Some(&step.id)).await;
        }

        // 检查是否所有步骤都成功
        let all_successful = {
            let executions = self.active_executions.read().await;
            if let Some(execution) = executions.get(&execution_id) {
                execution.step_results.values().all(|r| r.status == ExecutionStatus::Completed)
            } else {
                false
            }
        };

        // 完成执行
        let final_status = if all_successful {
            ExecutionStatus::Completed
        } else {
            ExecutionStatus::Failed
        };

        self.complete_execution(&execution_id, final_status).await;

        // 保存最终状态
        if let Some(execution) = self.active_executions.read().await.get(&execution_id).cloned() {
            if let Err(e) = self.storage.save_execution(&execution).await {
                eprintln!("Failed to save final execution state: {}", e);
            }
        }

        Ok(())
    }

    /// 执行单个步骤（带重试机制）
    async fn execute_step(
        &self,
        execution_id: &str,
        step: &WorkflowStep,
        api_key: &str,
        agent_info: &Option<serde_json::Value>,
    ) -> StepResult {
        let start_time = chrono::Utc::now().timestamp();
        let max_retries = self.retry_config.max_retries;

        // 发送步骤开始事件
        let _ = self.event_sender.send(ExecutionEvent::StepStarted {
            execution_id: execution_id.to_string(),
            step_id: step.id.clone(),
        });

        // 记录步骤开始日志
        // TODO: 需要传递workflow_id，这里暂时使用空字符串
        self.monitor.log_step_started(execution_id, "", &step.id).await;

        let mut last_error = None;
        let mut retry_count = 0;

        // 重试循环
        while retry_count <= max_retries {
            let attempt_start = chrono::Utc::now().timestamp();

            match self.execute_step_logic(step, execution_id, api_key, agent_info).await {
                Ok(result) => {
                    // 执行成功
                    return StepResult {
                        step_id: step.id.clone(),
                        status: ExecutionStatus::Completed,
                        result: Some(result),
                        error: None,
                        started_at: start_time,
                        completed_at: Some(chrono::Utc::now().timestamp()),
                        execution_time_ms: Some((chrono::Utc::now().timestamp() - start_time) as u64 * 1000),
                        retry_count,
                        max_retries,
                    };
                }
                Err(error) => {
                    last_error = Some(error.clone());
                    retry_count += 1;

                    if retry_count <= max_retries {
                        // 计算重试延迟
                        let delay_ms = self.calculate_retry_delay(retry_count, self.retry_config.initial_delay_ms);
                        eprintln!("Step '{}' failed (attempt {}/{}), retrying in {}ms: {}",
                                step.id, retry_count, max_retries + 1, delay_ms, error);

                        // 发送重试事件（可以添加到ExecutionEvent枚举中）
                        sleep(Duration::from_millis(delay_ms)).await;
                    }
                }
            }
        }

        // 所有重试都失败了
        StepResult {
            step_id: step.id.clone(),
            status: ExecutionStatus::Failed,
            result: None,
            error: last_error,
            started_at: start_time,
            completed_at: Some(chrono::Utc::now().timestamp()),
            execution_time_ms: Some((chrono::Utc::now().timestamp() - start_time) as u64 * 1000),
            retry_count: max_retries,
            max_retries,
        }
    }


    /// 执行步骤逻辑（使用真实工具执行）
    pub async fn execute_step_logic(
        &self,
        step: &WorkflowStep,
        execution_id: &str,
        _api_key: &str,
        _agent_info: &Option<serde_json::Value>,
    ) -> Result<serde_json::Value, String> {
        // 使用真实的工具桥接执行工具
        let tool_bridge = self.bridge_manager.tool_bridge();

        let request = crate::bridges::ToolCallRequest {
            session_id: format!("workflow_{}", execution_id),
            user_id: None,
            tool_id: step.tool.clone(),
            args: step.args.clone(),
            working_directory: std::env::current_dir()
                .ok()
                .and_then(|p| p.to_str().map(|s| s.to_string())),
            environment: std::env::vars().collect(),
            timeout_seconds: None,
            permissions: vec![],
        };

        match tool_bridge.handle_request(request).await {
            Ok(response) => {
                if response.success {
                    Ok(serde_json::json!({
                        "step_id": step.id,
                        "tool": step.tool,
                        "result": response.result.map(|r| r.data).unwrap_or_else(|| serde_json::json!({"status": "success"})),
                        "message": format!("Step '{}' completed successfully", step.name)
                    }))
                } else {
                    Err(response.error.unwrap_or_else(|| format!("Step '{}' execution failed", step.name)))
                }
            }
            Err(e) => Err(format!("Tool execution error: {}", e))
        }
    }

    /// 构建步骤依赖图
    fn build_dependency_graph(&self, steps: &[WorkflowStep]) -> HashMap<String, Vec<String>> {
        let mut graph = HashMap::new();
        for step in steps {
            graph.insert(step.id.clone(), step.depends_on.clone());
        }
        graph
    }

    /// 检查依赖是否完成
    async fn check_dependencies_completed(&self, execution_id: &str, dependencies: &[String]) -> bool {
        if dependencies.is_empty() {
            return true;
        }

        let executions = self.active_executions.read().await;
        if let Some(execution) = executions.get(execution_id) {
            dependencies.iter().all(|dep_id| {
                execution.step_results.get(dep_id)
                    .map(|result| result.status == ExecutionStatus::Completed)
                    .unwrap_or(false)
            })
        } else {
            false
        }
    }

    /// 更新执行状态
    pub async fn update_execution_status(&self, execution_id: &str, status: ExecutionStatus) {
        let mut executions = self.active_executions.write().await;
        if let Some(execution) = executions.get_mut(execution_id) {
            execution.status = status.clone();
        }
    }

    /// 更新步骤结果
    pub async fn update_step_result(&self, execution_id: &str, step_id: String, result: StepResult) {
        let mut executions = self.active_executions.write().await;
        if let Some(execution) = executions.get_mut(execution_id) {
            execution.step_results.insert(step_id, result);
        }
    }

    /// 完成执行
    pub async fn complete_execution(&self, execution_id: &str, status: ExecutionStatus) {
        let mut executions = self.active_executions.write().await;
        if let Some(execution) = executions.get_mut(execution_id) {
            execution.status = status.clone();
            execution.completed_at = Some(chrono::Utc::now().timestamp());
            execution.total_execution_time_ms = Some(
                (execution.completed_at.unwrap() - execution.started_at) as u64 * 1000
            );

            // 发送完成事件
            match status {
                ExecutionStatus::Completed => {
                    let result = serde_json::json!({
                        "execution_id": execution_id,
                        "status": "completed",
                        "step_results": execution.step_results
                    });
                    let _ = self.event_sender.send(ExecutionEvent::Completed {
                        execution_id: execution_id.to_string(),
                        result,
                    });
                }
                ExecutionStatus::Failed => {
                    let error = execution.error.clone().unwrap_or_else(|| "Execution failed".to_string());
                    let _ = self.event_sender.send(ExecutionEvent::Failed {
                        execution_id: execution_id.to_string(),
                        error,
                    });
                }
                _ => {}
            }
        }

        // 记录执行完成日志
        if let Some(execution) = self.active_executions.read().await.get(execution_id).cloned() {
            self.monitor.log_execution_completed(&execution).await;
        }
    }

    /// 获取执行状态
    pub async fn get_execution(&self, execution_id: &str) -> Result<Option<WorkflowExecution>, String> {
        // 首先检查内存缓存
        {
            let executions = self.active_executions.read().await;
            if let Some(execution) = executions.get(execution_id) {
                return Ok(Some(execution.clone()));
            }
        }

        // 如果内存中没有，尝试从存储加载
        self.storage.load_execution(execution_id).await
    }

    /// 暂停执行
    pub async fn pause_execution(&self, execution_id: &str) -> Result<(), String> {
        self.update_execution_status(execution_id, ExecutionStatus::Paused).await;
        let _ = self.event_sender.send(ExecutionEvent::Paused {
            execution_id: execution_id.to_string(),
        });
        Ok(())
    }

    /// 恢复执行
    pub async fn resume_execution(&self, execution_id: &str) -> Result<(), String> {
        self.update_execution_status(execution_id, ExecutionStatus::Running).await;
        let _ = self.event_sender.send(ExecutionEvent::Resumed {
            execution_id: execution_id.to_string(),
        });
        Ok(())
    }

    /// 取消执行
    pub async fn cancel_execution(&self, execution_id: &str) -> Result<(), String> {
        self.update_execution_status(execution_id, ExecutionStatus::Cancelled).await;
        let _ = self.event_sender.send(ExecutionEvent::Cancelled {
            execution_id: execution_id.to_string(),
        });

        // 取消任务句柄
        let mut handles = self.execution_handles.write().await;
        if let Some(handle) = handles.remove(execution_id) {
            handle.abort();
        }

        Ok(())
    }

    /// 获取下一个事件（用于前端轮询）
    pub async fn next_event(&self) -> Option<ExecutionEvent> {
        let mut receiver = self.event_receiver.lock().await;
        receiver.recv().await
    }

    /// 获取监控器引用
    pub fn monitor(&self) -> &WorkflowMonitor {
        &self.monitor
    }

    /// 设置Ralph Loop配置
    pub fn set_ralph_loop_config(&mut self, config: RalphLoopConfig) {
        self.ralph_loop_config = config;
    }

    /// 获取Ralph Loop配置
    pub fn get_ralph_loop_config(&self) -> &RalphLoopConfig {
        &self.ralph_loop_config
    }

    /// 清理完成的执行
    pub async fn cleanup_completed_executions(&self) {
        let mut executions = self.active_executions.write().await;
        let mut handles = self.execution_handles.write().await;

        let completed_ids: Vec<String> = executions.iter()
            .filter(|(_, exec)| {
                matches!(exec.status, ExecutionStatus::Completed | ExecutionStatus::Failed | ExecutionStatus::Cancelled)
            })
            .map(|(id, _)| id.clone())
            .collect();

        for id in completed_ids {
            executions.remove(&id);
            handles.remove(&id);
        }
    }

}

impl Clone for AsyncWorkflowExecutor {
    fn clone(&self) -> Self {
        // 注意：克隆时不复制接收器，因为它是独占的
        let (tx, rx) = mpsc::unbounded_channel();

        Self {
            active_executions: Arc::clone(&self.active_executions),
            workflows: Arc::clone(&self.workflows),
            storage: Arc::clone(&self.storage),
            retry_config: self.retry_config.clone(),
            ralph_loop_config: self.ralph_loop_config.clone(),
            ralph_loop_histories: Arc::clone(&self.ralph_loop_histories),
            monitor: Arc::clone(&self.monitor),
            event_sender: tx,
            event_receiver: Arc::new(Mutex::new(rx)),
            execution_handles: Arc::clone(&self.execution_handles),
            bridge_manager: Arc::clone(&self.bridge_manager),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ralph_loop_config_default() {
        let config = RalphLoopConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.max_iterations, 10);
        assert!(config.enable_history);
        assert!(config.smart_retry.enabled);
    }

    #[tokio::test]
    async fn test_smart_retry_strategy() {
        let executor = AsyncWorkflowExecutor::new().unwrap();

        // 测试基本的智能重试判断
        let config = RalphLoopConfig {
            smart_retry: SmartRetryStrategy {
                enabled: true,
                error_based_retry: std::collections::HashMap::new(),
                adaptive_retry: false,
                max_consecutive_failures: 3,
                learning_period: 5,
            },
            ..Default::default()
        };

        // 没有历史时应该允许重试
        let should_retry = executor.should_retry_with_smart_strategy("test_exec", &config, 1, "test error").await;
        assert!(should_retry);
    }

    #[tokio::test]
    async fn test_execution_history_creation() {
        let executor = AsyncWorkflowExecutor::new().unwrap();

        // 创建一个执行历史
        let history = RalphLoopExecutionHistory {
            execution_id: "test_exec".to_string(),
            workflow_id: "test_workflow".to_string(),
            total_iterations: 0,
            iterations: Vec::new(),
            started_at: chrono::Utc::now().timestamp(),
            completed_at: None,
            final_status: None,
            total_cost: 0.0,
            total_execution_time_ms: None,
        };

        let mut histories = executor.ralph_loop_histories.write().await;
        histories.insert("test_exec".to_string(), history);

        // 验证历史是否正确存储
        let retrieved = histories.get("test_exec").unwrap();
        assert_eq!(retrieved.execution_id, "test_exec");
        assert_eq!(retrieved.workflow_id, "test_workflow");
    }

    #[tokio::test]
    async fn test_rollback_functionality() {
        let executor = AsyncWorkflowExecutor::new().unwrap();

        // 创建包含多个迭代的历史
        let mut history = RalphLoopExecutionHistory {
            execution_id: "test_exec".to_string(),
            workflow_id: "test_workflow".to_string(),
            total_iterations: 2,
            iterations: vec![
                RalphLoopIterationHistory {
                    iteration: 1,
                    started_at: chrono::Utc::now().timestamp(),
                    completed_at: Some(chrono::Utc::now().timestamp()),
                    result: Some(serde_json::json!({"status": "success"})),
                    error: None,
                    cost: 0.01,
                    execution_time_ms: Some(1000),
                    retry_count: 0,
                },
                RalphLoopIterationHistory {
                    iteration: 2,
                    started_at: chrono::Utc::now().timestamp(),
                    completed_at: Some(chrono::Utc::now().timestamp()),
                    result: Some(serde_json::json!({"status": "success"})),
                    error: None,
                    cost: 0.01,
                    execution_time_ms: Some(1000),
                    retry_count: 0,
                },
            ],
            started_at: chrono::Utc::now().timestamp(),
            completed_at: Some(chrono::Utc::now().timestamp()),
            final_status: Some("completed".to_string()),
            total_cost: 0.02,
            total_execution_time_ms: Some(2000),
        };

        let mut histories = executor.ralph_loop_histories.write().await;
        histories.insert("test_exec".to_string(), history);

        // 测试回滚功能
        let result = executor.rollback_to_iteration("test_exec", 1).await;
        assert!(result.is_ok());
    }
}