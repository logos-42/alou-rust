//! Agent Scheduler - 主调度器

use std::sync::Arc;
use tokio::sync::mpsc;
use crate::scheduler::types::{AgentTask, SchedulingDecision, DecisionType, AgentState};
use crate::scheduler::queue::TaskQueue;
use crate::scheduler::quota::{QuotaManager, ResourceQuota};

/// Agent Scheduler
pub struct AgentScheduler {
    task_queue: Arc<tokio::sync::Mutex<TaskQueue>>,
    quota_manager: Arc<QuotaManager>,
    result_tx: mpsc::UnboundedSender<TaskResult>,
    task_states: Arc<tokio::sync::Mutex<std::collections::HashMap<String, AgentState>>>,
}

/// 任务结果
#[derive(Debug, Clone)]
pub struct TaskResult {
    pub task_id: String,
    pub session_id: String,
    pub success: bool,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
}

impl AgentScheduler {
    pub fn new(quota: ResourceQuota) -> (Self, mpsc::UnboundedReceiver<TaskResult>) {
        let (result_tx, result_rx) = mpsc::unbounded_channel();
        
        let scheduler = Self {
            task_queue: Arc::new(tokio::sync::Mutex::new(TaskQueue::new(1000))),
            quota_manager: Arc::new(QuotaManager::new(quota)),
            result_tx,
            task_states: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
        };
        
        // 启动调度循环
        let scheduler_clone = scheduler.clone();
        tokio::spawn(async move {
            scheduler_clone.run_scheduler_loop().await;
        });
        
        (scheduler, result_rx)
    }
    
    fn clone(&self) -> Self {
        Self {
            task_queue: self.task_queue.clone(),
            quota_manager: self.quota_manager.clone(),
            result_tx: self.result_tx.clone(),
            task_states: self.task_states.clone(),
        }
    }
    
    /// 提交任务
    pub async fn submit_task(&self, task: AgentTask) -> SchedulingDecision {
        let task_id = task.task_id.clone();
        let session_id = task.session_id.clone();
        
        log::info!(
            "[AgentScheduler] Submitting task {} for session {}",
            task_id,
            session_id
        );
        
        // 1. 检查速率限制
        if !self.quota_manager.check_rate_limit(&session_id) {
            return SchedulingDecision {
                task_id: task_id.clone(),
                decision: DecisionType::Reject { retry_after_secs: Some(60) },
                decided_at: chrono::Utc::now().timestamp(),
                reason: "Rate limit exceeded".to_string(),
            };
        }
        
        // 2. 检查 session 配额
        if !self.quota_manager.check_session_quota(&session_id) {
            return SchedulingDecision {
                task_id: task_id.clone(),
                decision: DecisionType::Reject { retry_after_secs: Some(30) },
                decided_at: chrono::Utc::now().timestamp(),
                reason: "Session quota exceeded".to_string(),
            };
        }
        
        // 3. 记录请求
        self.quota_manager.record_request(&session_id);
        self.quota_manager.increment_session_task(&session_id);
        
        // 4. 更新任务状态
        {
            let mut states = self.task_states.lock().await;
            states.insert(task_id.clone(), AgentState::Idle);
        }
        
        // 5. 入队
        let mut queue = self.task_queue.lock().await;
        if queue.enqueue(task) {
            SchedulingDecision {
                task_id,
                decision: DecisionType::Enqueue { position: queue.len() },
                decided_at: chrono::Utc::now().timestamp(),
                reason: "Task queued successfully".to_string(),
            }
        } else {
            SchedulingDecision {
                task_id,
                decision: DecisionType::Reject { retry_after_secs: Some(10) },
                decided_at: chrono::Utc::now().timestamp(),
                reason: "Queue is full".to_string(),
            }
        }
    }
    
    /// 调度器主循环
    async fn run_scheduler_loop(&self) {
        log::info!("[AgentScheduler] Scheduler loop started");
        
        loop {
            // 1. 检查队列
            let task = {
                let mut queue = self.task_queue.lock().await;
                queue.dequeue()
            };
            
            if let Some(task) = task {
                // 2. 检查是否过期
                if task.is_expired() {
                    log::warn!("[AgentScheduler] Task {} expired, skipping", task.task_id);
                    self.quota_manager.decrement_session_task(&task.session_id);
                    continue;
                }
                
                // 3. 调度执行
                log::info!(
                    "[AgentScheduler] Dispatching task {} (priority={:?})",
                    task.task_id,
                    task.priority
                );
                
                // 更新状态
                {
                    let mut states = self.task_states.lock().await;
                    states.insert(
                        task.task_id.clone(),
                        AgentState::Busy {
                            task_id: task.task_id.clone(),
                            started_at: chrono::Utc::now().timestamp(),
                        },
                    );
                }
                
                // 4. 获取资源许可（根据任务类型）
                let _permit = match task.task_type {
                    crate::scheduler::types::TaskType::AgentChat => {
                        match self.quota_manager.acquire_llm().await {
                            Ok(p) => Some(p),
                            Err(e) => {
                                log::error!("[AgentScheduler] Failed to acquire LLM permit: {}", e);
                                None
                            }
                        }
                    }
                    crate::scheduler::types::TaskType::ToolCall { .. } => {
                        match self.quota_manager.acquire_tool().await {
                            Ok(p) => Some(p),
                            Err(e) => {
                                log::error!("[AgentScheduler] Failed to acquire tool permit: {}", e);
                                None
                            }
                        }
                    }
                    crate::scheduler::types::TaskType::Workflow { .. } => {
                        match self.quota_manager.acquire_workflow().await {
                            Ok(p) => Some(p),
                            Err(e) => {
                                log::error!("[AgentScheduler] Failed to acquire workflow permit: {}", e);
                                None
                            }
                        }
                    }
                    _ => None,
                };
                
                // 5. 更新状态为空闲
                {
                    let mut states = self.task_states.lock().await;
                    states.insert(task.task_id.clone(), AgentState::Idle);
                }
                
                // 6. 减少 session 任务计数
                self.quota_manager.decrement_session_task(&task.session_id);
            }
            
            // 7. 短暂休眠
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }
    
    /// 获取调度器统计
    pub async fn get_stats(&self) -> SchedulerStats {
        let queue = self.task_queue.lock().await;
        let queue_stats = queue.get_stats();
        let quota_stats = self.quota_manager.get_stats();
        
        SchedulerStats {
            pending_tasks: queue_stats.total_tasks,
            by_priority: queue_stats.by_priority,
            active_sessions: quota_stats.active_sessions,
            llm_available: quota_stats.llm_available,
            tool_available: quota_stats.tool_available,
            workflow_available: quota_stats.workflow_available,
        }
    }
    
    /// 取消 session 的所有任务
    pub async fn cancel_session_tasks(&self, session_id: &str) -> usize {
        let mut queue = self.task_queue.lock().await;
        let removed = queue.remove_session_tasks(session_id);
        removed.len()
    }
}

/// 调度器统计
#[derive(Debug, Clone)]
pub struct SchedulerStats {
    pub pending_tasks: usize,
    pub by_priority: [usize; 4],
    pub active_sessions: usize,
    pub llm_available: usize,
    pub tool_available: usize,
    pub workflow_available: usize,
}
