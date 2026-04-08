//! Swarm 协调器 - 基于人月神话的分布式任务协调
//! 
//! 核心设计原则:
//! - 外科手术式团队: Chief Coordinator + 多个独立 Executor
//! - 减少沟通成本: 通过明确的任务边界和消息传递
//! - 概念完整性: 统一的调度策略和状态管理

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use tokio::sync::{mpsc, RwLock, Mutex};
use tokio::time::{interval, Duration, timeout};
use uuid::Uuid;

use crate::swarm::types::*;
use crate::swarm::executor::TaskExecutor;

/// 协调器配置
#[derive(Debug, Clone)]
pub struct CoordinatorConfig {
    /// 最大并发任务数
    pub max_concurrent_tasks: usize,
    /// 任务队列容量
    pub queue_capacity: usize,
    /// 默认任务超时（毫秒）
    pub default_timeout_ms: u64,
    /// 心跳间隔（毫秒）
    pub heartbeat_interval_ms: u64,
    /// 是否启用任务抢占
    pub enable_preemption: bool,
    /// 负载均衡策略
    pub load_balancing_strategy: LoadBalancingStrategy,
}

impl Default for CoordinatorConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: 10,
            queue_capacity: 1000,
            default_timeout_ms: 300_000, // 5分钟
            heartbeat_interval_ms: 5_000, // 5秒
            enable_preemption: false,
            load_balancing_strategy: LoadBalancingStrategy::LeastLoaded,
        }
    }
}

/// 负载均衡策略
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LoadBalancingStrategy {
    /// 轮询
    RoundRobin,
    /// 最少负载
    LeastLoaded,
    /// 能力匹配
    CapabilityMatch,
    /// 随机
    Random,
}

/// Swarm 协调器
pub struct SwarmCoordinator {
    /// 配置
    config: CoordinatorConfig,
    
    /// 任务存储
    tasks: Arc<RwLock<HashMap<String, Task>>>,
    
    /// 任务队列（按优先级排序）
    task_queue: Arc<Mutex<VecDeque<String>>>,
    
    /// 执行器列表
    executors: Arc<RwLock<Vec<Arc<dyn TaskExecutor>>>>,
    
    /// 活跃任务（任务ID -> 执行器ID）
    active_tasks: Arc<RwLock<HashMap<String, String>>>,
    
    /// 事件发送器
    event_sender: mpsc::Sender<SwarmEvent>,
    
    /// 事件接收器（需要保存以维持通道打开）
    #[allow(dead_code)]
    event_receiver: Arc<Mutex<mpsc::Receiver<SwarmEvent>>>,
    
    /// Agent 注册表
    agents: Arc<RwLock<HashMap<String, AgentInfo>>>,
    
    /// Swarm 列表
    swarms: Arc<RwLock<HashMap<String, AgentSwarm>>>,
    
    /// 运行状态
    running: Arc<RwLock<bool>>,
    
    // ==================== 群聊协作相关字段 ====================
    
    /// 群聊协作是否启用
    group_collaboration_enabled: Arc<RwLock<bool>>,
    
    /// 群聊消息（消息ID -> 消息）
    group_messages: Arc<RwLock<HashMap<String, crate::swarm::types::GroupChatMessage>>>,
    
    /// 消息回复计数（消息ID -> 回复数）
    message_reply_counts: Arc<RwLock<HashMap<String, u32>>>,
    
    /// 当前协作轮次
    collaboration_round: Arc<RwLock<u32>>,
    
    /// 活跃的协作消息
    active_collaborations: Arc<RwLock<HashMap<String, crate::swarm::types::GroupAgentCollaboration>>>,
}

/// Swarm 事件
#[derive(Debug, Clone)]
pub enum SwarmEvent {
    /// 任务创建
    TaskCreated { task_id: String, timestamp: i64 },
    /// 任务入队
    TaskQueued { task_id: String, timestamp: i64 },
    /// 任务开始
    TaskStarted { task_id: String, executor_id: String, timestamp: i64 },
    /// 任务进度更新
    TaskProgress { task_id: String, progress: f32, message: Option<String> },
    /// 任务完成
    TaskCompleted { task_id: String, result: TaskResult, timestamp: i64 },
    /// 任务失败
    TaskFailed { task_id: String, error: TaskError, timestamp: i64 },
    /// 任务取消
    TaskCancelled { task_id: String, reason: String, timestamp: i64 },
    /// Agent 加入
    AgentJoined { agent_id: String, agent_info: AgentInfo },
    /// Agent 离开
    AgentLeft { agent_id: String, reason: String },
    /// Agent 状态变更
    AgentStatusChanged { agent_id: String, old_status: AgentStatus, new_status: AgentStatus },
    /// Swarm 形成
    SwarmFormed { swarm_id: String, member_ids: Vec<String> },
    /// Swarm 解散
    SwarmDisbanded { swarm_id: String, reason: String },
    /// 错误
    Error { source: String, message: String },
}

impl SwarmCoordinator {
    /// 创建新的协调器
    pub fn new(config: CoordinatorConfig) -> Self {
        let (event_sender, event_receiver) = mpsc::channel(1000);
        
        Self {
            config,
            tasks: Arc::new(RwLock::new(HashMap::new())),
            task_queue: Arc::new(Mutex::new(VecDeque::new())),
            executors: Arc::new(RwLock::new(Vec::new())),
            active_tasks: Arc::new(RwLock::new(HashMap::new())),
            event_sender,
            event_receiver: Arc::new(Mutex::new(event_receiver)),
            agents: Arc::new(RwLock::new(HashMap::new())),
            swarms: Arc::new(RwLock::new(HashMap::new())),
            running: Arc::new(RwLock::new(false)),
            // 群聊协作相关字段
            group_collaboration_enabled: Arc::new(RwLock::new(false)),
            group_messages: Arc::new(RwLock::new(HashMap::new())),
            message_reply_counts: Arc::new(RwLock::new(HashMap::new())),
            collaboration_round: Arc::new(RwLock::new(0)),
            active_collaborations: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// 使用默认配置创建
    pub fn default() -> Self {
        Self::new(CoordinatorConfig::default())
    }
    
    /// 启动协调器
    pub async fn start(&self) -> Result<(), String> {
        // 检查是否已经在运行
        {
            let running = self.running.read().await;
            if *running {
                return Err("Coordinator is already running".to_string());
            }
        }
        
        // 设置运行状态
        {
            let mut running = self.running.write().await;
            *running = true;
        }
        
        // 启动调度循环
        self.start_scheduler().await;
        
        // 启动心跳循环
        self.start_heartbeat().await;
        
        Ok(())
    }
    
    /// 停止协调器
    pub async fn stop(&self) -> Result<(), String> {
        let mut running = self.running.write().await;
        *running = false;
        Ok(())
    }
    
    /// 创建任务
    pub async fn create_task(
        &self,
        title: impl Into<String>,
        steps: Vec<TaskStep>,
        options: TaskCreateOptions,
    ) -> Result<Task, String> {
        let task_id = format!("task-{}", Uuid::new_v4().to_string().split('-').next().unwrap());
        let now = chrono::Utc::now().timestamp_millis();
        
        let task = Task {
            id: task_id.clone(),
            title: title.into(),
            description: None,
            priority: options.priority.unwrap_or_default(),
            status: TaskStatus::Pending,
            execution_mode: options.execution_mode.unwrap_or_default(),
            max_agents: options.max_agents,
            timeout: options.timeout.or(Some(self.config.default_timeout_ms)),
            retry_count: Some(0),
            retry_delay: Some(1000),
            steps,
            dependencies: None,
            sub_tasks: None,
            assignee_id: None,
            assignee_ids: None,
            required_capabilities: options.required_capabilities,
            context: None,
            parameters: None,
            result: None,
            error: None,
            created_at: now,
            updated_at: now,
            assigned_at: None,
            started_at: None,
            completed_at: None,
            metadata: options.metadata,
        };
        
        // 存储任务
        {
            let mut tasks = self.tasks.write().await;
            tasks.insert(task_id.clone(), task.clone());
        }
        
        // 发送事件
        let _ = self.event_sender.send(SwarmEvent::TaskCreated {
            task_id: task_id.clone(),
            timestamp: now,
        }).await;
        
        // 如果设置了自动执行，则入队
        if options.auto_execute {
            self.enqueue_task(task_id).await?;
        }
        
        Ok(task)
    }
    
    /// 将任务加入队列
    pub async fn enqueue_task(&self, task_id: String) -> Result<(), String> {
        // 检查任务是否存在
        {
            let tasks = self.tasks.read().await;
            if !tasks.contains_key(&task_id) {
                return Err(format!("Task {} not found", task_id));
            }
        }
        
        // 更新任务状态
        {
            let mut tasks = self.tasks.write().await;
            if let Some(task) = tasks.get_mut(&task_id) {
                task.update_status(TaskStatus::Queued);
            }
        }
        
        // 加入队列
        {
            let mut queue = self.task_queue.lock().await;
            if queue.len() >= self.config.queue_capacity {
                return Err("Task queue is full".to_string());
            }
            queue.push_back(task_id.clone());
        }
        
        // 发送事件
        let _ = self.event_sender.send(SwarmEvent::TaskQueued {
            task_id,
            timestamp: chrono::Utc::now().timestamp_millis(),
        }).await;
        
        Ok(())
    }
    
    /// 取消任务
    pub async fn cancel_task(&self, task_id: &str, reason: impl Into<String>) -> Result<(), String> {
        let mut tasks = self.tasks.write().await;
        
        if let Some(task) = tasks.get_mut(task_id) {
            if task.status.is_terminal() {
                return Err(format!("Task {} is already in terminal state", task_id));
            }
            
            task.update_status(TaskStatus::Cancelled);
            
            // 从队列中移除
            drop(tasks);
            {
                let mut queue = self.task_queue.lock().await;
                queue.retain(|id| id != task_id);
            }
            
            // 发送事件
            let _ = self.event_sender.send(SwarmEvent::TaskCancelled {
                task_id: task_id.to_string(),
                reason: reason.into(),
                timestamp: chrono::Utc::now().timestamp_millis(),
            }).await;
            
            Ok(())
        } else {
            Err(format!("Task {} not found", task_id))
        }
    }
    
    /// 获取任务状态
    pub async fn get_task(&self, task_id: &str) -> Option<Task> {
        let tasks = self.tasks.read().await;
        tasks.get(task_id).cloned()
    }
    
    /// 列出所有任务
    pub async fn list_tasks(&self) -> Vec<Task> {
        let tasks = self.tasks.read().await;
        tasks.values().cloned().collect()
    }
    
    /// 列出特定状态的任务
    pub async fn list_tasks_by_status(&self, status: TaskStatus) -> Vec<Task> {
        let tasks = self.tasks.read().await;
        tasks.values()
            .filter(|t| t.status == status)
            .cloned()
            .collect()
    }
    
    /// 注册 Agent
    pub async fn register_agent(&self, agent_info: AgentInfo) -> Result<(), String> {
        let agent_id = agent_info.id.clone();
        
        {
            let mut agents = self.agents.write().await;
            agents.insert(agent_id.clone(), agent_info.clone());
        }
        
        // 发送事件
        let _ = self.event_sender.send(SwarmEvent::AgentJoined {
            agent_id,
            agent_info,
        }).await;
        
        Ok(())
    }
    
    /// 注销 Agent
    pub async fn unregister_agent(&self, agent_id: &str, reason: impl Into<String>) -> Result<(), String> {
        {
            let mut agents = self.agents.write().await;
            agents.remove(agent_id);
        }
        
        // 发送事件
        let _ = self.event_sender.send(SwarmEvent::AgentLeft {
            agent_id: agent_id.to_string(),
            reason: reason.into(),
        }).await;
        
        Ok(())
    }
    
    /// 获取 Agent 信息
    pub async fn get_agent(&self, agent_id: &str) -> Option<AgentInfo> {
        let agents = self.agents.read().await;
        agents.get(agent_id).cloned()
    }
    
    /// 列出所有 Agents
    pub async fn list_agents(&self) -> Vec<AgentInfo> {
        let agents = self.agents.read().await;
        agents.values().cloned().collect()
    }
    
    /// 创建 Swarm
    pub async fn create_swarm(
        &self,
        name: impl Into<String>,
        member_ids: Vec<String>,
        coordination: SwarmCoordination,
    ) -> Result<AgentSwarm, String> {
        let swarm_id = format!("swarm-{}", Uuid::new_v4().to_string().split('-').next().unwrap());
        
        // 收集成员信息
        let mut members = Vec::new();
        let mut capabilities = HashSet::new();
        
        {
            let agents = self.agents.read().await;
            for member_id in &member_ids {
                if let Some(agent) = agents.get(member_id) {
                    members.push(agent.clone());
                    for cap in &agent.capabilities {
                        capabilities.insert(cap.clone());
                    }
                } else {
                    return Err(format!("Agent {} not found", member_id));
                }
            }
        }
        
        let swarm = AgentSwarm {
            id: swarm_id.clone(),
            name: name.into(),
            members,
            capabilities: capabilities.into_iter().collect(),
            status: SwarmStatus::Active,
            coordination,
            created_at: chrono::Utc::now().timestamp_millis(),
        };
        
        {
            let mut swarms = self.swarms.write().await;
            swarms.insert(swarm_id.clone(), swarm.clone());
        }
        
        // 发送事件
        let _ = self.event_sender.send(SwarmEvent::SwarmFormed {
            swarm_id,
            member_ids,
        }).await;
        
        Ok(swarm)
    }
    
    /// 解散 Swarm
    pub async fn disband_swarm(&self, swarm_id: &str, reason: impl Into<String>) -> Result<(), String> {
        {
            let mut swarms = self.swarms.write().await;
            if let Some(swarm) = swarms.get_mut(swarm_id) {
                swarm.status = SwarmStatus::Disbanded;
            } else {
                return Err(format!("Swarm {} not found", swarm_id));
            }
        }
        
        // 发送事件
        let _ = self.event_sender.send(SwarmEvent::SwarmDisbanded {
            swarm_id: swarm_id.to_string(),
            reason: reason.into(),
        }).await;
        
        Ok(())
    }
    
    /// 订阅事件
    pub async fn subscribe_events(&self) -> mpsc::Receiver<SwarmEvent> {
        // 创建新的通道并转发事件
        let (tx, rx) = mpsc::channel(100);
        // 注意：实际实现需要维护订阅者列表并广播事件
        rx
    }
    
    /// 获取队列统计
    pub async fn get_queue_stats(&self) -> QueueStats {
        let tasks = self.tasks.read().await;
        let queue = self.task_queue.lock().await;
        
        let mut by_priority = HashMap::new();
        by_priority.insert(TaskPriority::Critical, 0u32);
        by_priority.insert(TaskPriority::High, 0);
        by_priority.insert(TaskPriority::Medium, 0);
        by_priority.insert(TaskPriority::Low, 0);
        by_priority.insert(TaskPriority::Custom, 0);
        
        for task_id in queue.iter() {
            if let Some(task) = tasks.get(task_id) {
                *by_priority.entry(task.priority.clone()).or_insert(0) += 1;
            }
        }
        
        QueueStats {
            total: queue.len() as u32,
            by_priority: HashMap::new(), // 简化实现
            custom_queues: Vec::new(),
        }
    }
    
    /// 启动调度循环
    async fn start_scheduler(&self) {
        let tasks = self.tasks.clone();
        let queue = self.task_queue.clone();
        let executors = self.executors.clone();
        let active_tasks = self.active_tasks.clone();
        let config = self.config.clone();
        let event_sender = self.event_sender.clone();
        let running = self.running.clone();
        let agents = self.agents.clone();
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_millis(100));
            
            loop {
                interval.tick().await;
                
                // 检查是否还在运行
                {
                    let r = running.read().await;
                    if !*r {
                        break;
                    }
                }
                
                // 检查活跃任务数
                {
                    let active = active_tasks.read().await;
                    if active.len() >= config.max_concurrent_tasks {
                        continue;
                    }
                }
                
                // 获取下一个任务
                let task_id = {
                    let mut q = queue.lock().await;
                    q.pop_front()
                };
                
                if let Some(task_id) = task_id {
                    // 获取任务信息
                    let task = {
                        let tasks_guard = tasks.read().await;
                        tasks_guard.get(&task_id).cloned()
                    };
                    
                    if let Some(mut task) = task {
                        // 选择执行器（Agent）
                        let executor_id = Self::select_executor(
                            &agents,
                            &task,
                            config.load_balancing_strategy,
                        ).await;
                        
                        if let Some(executor_id) = executor_id {
                            // 更新任务状态
                            task.update_status(TaskStatus::Assigned);
                            task.assignee_id = Some(executor_id.clone());
                            
                            {
                                let mut tasks_guard = tasks.write().await;
                                tasks_guard.insert(task_id.clone(), task.clone());
                            }
                            
                            // 记录活跃任务
                            {
                                let mut active = active_tasks.write().await;
                                active.insert(task_id.clone(), executor_id.clone());
                            }
                            
                            // 发送事件
                            let _ = event_sender.send(SwarmEvent::TaskStarted {
                                task_id: task_id.clone(),
                                executor_id: executor_id.clone(),
                                timestamp: chrono::Utc::now().timestamp_millis(),
                            }).await;
                            
                            // 启动任务执行
                            let tasks_clone = tasks.clone();
                            let active_tasks_clone = active_tasks.clone();
                            let event_sender_clone = event_sender.clone();
                            
                            tokio::spawn(async move {
                                Self::execute_task(
                                    task,
                                    tasks_clone,
                                    active_tasks_clone,
                                    event_sender_clone,
                                ).await;
                            });
                        } else {
                            // 没有可用的执行器，放回队列
                            let mut q = queue.lock().await;
                            q.push_front(task_id);
                        }
                    }
                }
            }
        });
    }
    
    /// 选择执行器
    async fn select_executor(
        agents: &Arc<RwLock<HashMap<String, AgentInfo>>>,
        task: &Task,
        strategy: LoadBalancingStrategy,
    ) -> Option<String> {
        let agents_guard = agents.read().await;
        
        let candidates: Vec<&AgentInfo> = agents_guard.values()
            .filter(|a| a.status == AgentStatus::Idle || a.status == AgentStatus::Online)
            .filter(|a| {
                // 检查能力匹配
                if let Some(required) = &task.required_capabilities {
                    required.iter().all(|cap| a.capabilities.contains(cap))
                } else {
                    true
                }
            })
            .collect();
        
        if candidates.is_empty() {
            return None;
        }
        
        match strategy {
            LoadBalancingStrategy::RoundRobin => {
                candidates.first().map(|a| a.id.clone())
            }
            LoadBalancingStrategy::LeastLoaded => {
                candidates.iter()
                    .min_by(|a, b| a.current_load.partial_cmp(&b.current_load).unwrap())
                    .map(|a| a.id.clone())
            }
            LoadBalancingStrategy::CapabilityMatch => {
                // 选择能力匹配度最高的
                candidates.iter()
                    .max_by_key(|a| a.capabilities.len())
                    .map(|a| a.id.clone())
            }
            LoadBalancingStrategy::Random => {
                use rand::seq::SliceRandom;
                candidates.choose(&mut rand::thread_rng()).map(|a| a.id.clone())
            }
        }
    }
    
    /// 执行任务
    async fn execute_task(
        mut task: Task,
        tasks: Arc<RwLock<HashMap<String, Task>>>,
        active_tasks: Arc<RwLock<HashMap<String, String>>>,
        event_sender: mpsc::Sender<SwarmEvent>,
    ) {
        let task_id = task.id.clone();
        let timeout_ms = task.timeout.unwrap_or(300_000);
        
        // 更新状态为执行中
        task.update_status(TaskStatus::InProgress);
        {
            let mut tasks_guard = tasks.write().await;
            tasks_guard.insert(task_id.clone(), task.clone());
        }
        
        // 执行步骤
        let mut steps_completed = 0u32;
        let mut steps_failed = 0u32;
        let start_time = chrono::Utc::now().timestamp_millis();
        
        for step_idx in 0..task.steps.len() {
            let step_id = task.steps[step_idx].id.clone();
            let step_name = task.steps[step_idx].name.clone();
            let is_optional = task.steps[step_idx].optional;
            
            // 更新步骤状态
            {
                let step = &mut task.steps[step_idx];
                step.status = Some(TaskStatus::InProgress);
                step.started_at = Some(chrono::Utc::now().timestamp_millis());
            }
            
            // 发送进度事件
            let progress = ((step_idx as f32) / (task.steps.len() as f32)) * 100.0;
            let _ = event_sender.send(SwarmEvent::TaskProgress {
                task_id: task_id.clone(),
                progress,
                message: Some(format!("Executing step: {}", step_name)),
            }).await;
            
            // 模拟步骤执行（实际实现中调用 Skill）
            let step_result = timeout(
                Duration::from_millis(timeout_ms),
                Self::execute_step(&task.steps[step_idx])
            ).await;
            
            match step_result {
                Ok(Ok(output)) => {
                    let step = &mut task.steps[step_idx];
                    step.status = Some(TaskStatus::Completed);
                    step.result = Some(output);
                    step.completed_at = Some(chrono::Utc::now().timestamp_millis());
                    steps_completed += 1;
                }
                Ok(Err(e)) => {
                    {
                        let step = &mut task.steps[step_idx];
                        step.status = Some(TaskStatus::Failed);
                        step.error = Some(e.clone());
                        step.completed_at = Some(chrono::Utc::now().timestamp_millis());
                    }
                    steps_failed += 1;
                    
                    if !is_optional {
                        // 非可选步骤失败，任务失败
                        task.update_status(TaskStatus::Failed);
                        task.error = Some(TaskError {
                            code: "STEP_FAILED".to_string(),
                            message: e,
                            details: Some(serde_json::json!({"step_id": step_id})),
                            step_id: Some(step_id.clone()),
                            recoverable: false,
                        });
                        
                        let _ = event_sender.send(SwarmEvent::TaskFailed {
                            task_id: task_id.clone(),
                            error: task.error.clone().unwrap(),
                            timestamp: chrono::Utc::now().timestamp_millis(),
                        }).await;
                        
                        break;
                    }
                }
                Err(_) => {
                    // 超时
                    {
                        let step = &mut task.steps[step_idx];
                        step.status = Some(TaskStatus::Timeout);
                    }
                    task.update_status(TaskStatus::Timeout);
                    task.error = Some(TaskError {
                        code: "TIMEOUT".to_string(),
                        message: "Task execution timed out".to_string(),
                        details: None,
                        step_id: Some(step_id.clone()),
                        recoverable: false,
                    });
                    
                    let _ = event_sender.send(SwarmEvent::TaskFailed {
                        task_id: task_id.clone(),
                        error: task.error.clone().unwrap(),
                        timestamp: chrono::Utc::now().timestamp_millis(),
                    }).await;
                    
                    break;
                }
            }
        }
        
        // 如果没有失败，标记为完成
        if task.error.is_none() {
            task.update_status(TaskStatus::Completed);
            
            let duration = chrono::Utc::now().timestamp_millis() - start_time;
            task.result = Some(TaskResult {
                success: true,
                output: serde_json::json!({
                    "steps_completed": steps_completed,
                    "steps_failed": steps_failed,
                }),
                summary: Some(format!("Completed {} steps in {}ms", steps_completed, duration)),
                artifacts: None,
                metrics: Some(TaskMetrics {
                    duration,
                    steps_completed,
                    steps_failed,
                    agents_involved: 1,
                    retries_count: Some(0),
                    cost_estimate: None,
                }),
            });
            
            let _ = event_sender.send(SwarmEvent::TaskCompleted {
                task_id: task_id.clone(),
                result: task.result.clone().unwrap(),
                timestamp: chrono::Utc::now().timestamp_millis(),
            }).await;
        }
        
        // 保存任务状态
        {
            let mut tasks_guard = tasks.write().await;
            tasks_guard.insert(task_id.clone(), task);
        }
        
        // 从活跃任务中移除
        {
            let mut active = active_tasks.write().await;
            active.remove(&task_id);
        }
    }
    
    /// 执行单个步骤（模拟）
    async fn execute_step(step: &TaskStep) -> Result<serde_json::Value, String> {
        // 实际实现中，这里会根据 step.action 调用对应的 Skill
        // 模拟执行时间
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        Ok(serde_json::json!({
            "step": step.name,
            "action": step.action,
            "status": "completed",
        }))
    }
    
    /// 启动心跳循环
    async fn start_heartbeat(&self) {
        let running = self.running.clone();
        let agents = self.agents.clone();
        let interval_ms = self.config.heartbeat_interval_ms;
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_millis(interval_ms));
            
            loop {
                interval.tick().await;
                
                // 检查是否还在运行
                {
                    let r = running.read().await;
                    if !*r {
                        break;
                    }
                }
                
                // 检查 Agent 健康状态
                let mut agents_guard = agents.write().await;
                for agent in agents_guard.values_mut() {
                    // 模拟心跳检测
                    // 实际实现中会发送 ping 并等待 pong
                    if agent.status == AgentStatus::Online && agent.current_load > 0.9 {
                        agent.status = AgentStatus::Busy;
                    }
                }
            }
        });
    }
    
    // ==================== 群聊智能体协作方法 ====================
    
    /// 启用群聊协作
    pub async fn enable_group_collaboration(&self) {
        let mut enabled = self.group_collaboration_enabled.write().await;
        *enabled = true;
        println!("[Coordinator] 群聊协作已启用");
    }
    
    /// 禁用群聊协作
    pub async fn disable_group_collaboration(&self) {
        let mut enabled = self.group_collaboration_enabled.write().await;
        *enabled = false;
        println!("[Coordinator] 群聊协作已禁用");
    }
    
    /// 检查群聊协作是否启用
    pub async fn is_group_collaboration_enabled(&self) -> bool {
        let enabled = self.group_collaboration_enabled.read().await;
        *enabled
    }
    
    /// 添加群聊消息
    pub async fn add_group_message(&self, message: crate::swarm::types::GroupChatMessage) {
        let mut messages = self.group_messages.write().await;
        messages.insert(message.id.clone(), message);
    }
    
    /// 获取群聊消息
    pub async fn get_group_message(&self, message_id: &str) -> Option<crate::swarm::types::GroupChatMessage> {
        let messages = self.group_messages.read().await;
        messages.get(message_id).cloned()
    }
    
    /// 获取所有群聊消息（过滤后，只显示智能体回复）
    pub async fn get_displayable_messages(&self) -> Vec<crate::swarm::types::GroupChatMessage> {
        let messages = self.group_messages.read().await;
        messages.values()
            .filter(|msg| msg.should_display())
            .cloned()
            .collect()
    }
    
    /// 检查智能体是否可以回复
    pub async fn can_agent_respond(&self, agent_id: &str, message_id: &str, max_replies_per_round: u32) -> bool {
        let enabled = self.group_collaboration_enabled.read().await;
        if !*enabled {
            return false;
        }
        
        let counts = self.message_reply_counts.read().await;
        let count = counts.get(message_id).unwrap_or(&0);
        
        *count < max_replies_per_round
    }
    
    /// 记录智能体回复
    pub async fn record_agent_response(&self, agent_id: &str, message_id: &str) {
        let mut counts = self.message_reply_counts.write().await;
        let count = counts.entry(message_id.to_string()).or_insert(0);
        *count += 1;
        
        // 更新协作轮次
        let mut round = self.collaboration_round.write().await;
        *round += 1;
        
        println!("[Coordinator] 智能体 {} 回复了消息 {}, 当前回复数: {}", agent_id, message_id, count);
    }
    
    /// 检查消息是否需要智能体响应
    pub async fn should_agent_respond(&self, message: &crate::swarm::types::GroupChatMessage) -> bool {
        // 系统消息不需要响应
        if message.message_type == crate::swarm::types::GroupMessageType::System {
            return false;
        }
        
        // 检查是否@提及其他智能体
        if !message.mentioned_agent_ids.is_empty() {
            return true;
        }
        
        // 检查是否是用户发送的消息（非智能体）
        if !message.sender_id.starts_with("agent_") {
            return true;
        }
        
        // 智能体发送的消息，检查是否是回复
        if message.collaboration.is_some() {
            return true;
        }
        
        false
    }
    
    /// 获取群聊中的活跃智能体列表
    pub async fn get_active_agents_in_group(&self) -> Vec<AgentInfo> {
        let agents = self.agents.read().await;
        agents.values()
            .filter(|a| a.status == AgentStatus::Online || a.status == AgentStatus::Idle)
            .cloned()
            .collect()
    }
    
    /// 获取适合处理消息的智能体
    pub async fn get_best_agent_for_message(&self, message_content: &str) -> Option<AgentInfo> {
        let agents = self.get_active_agents_in_group().await;
        
        if agents.is_empty() {
            return None;
        }
        
        let content_lower = message_content.to_lowercase();
        
        // 根据消息内容匹配最合适的智能体
        let mut best_agent: Option<(AgentInfo, i32)> = None;
        
        for agent in agents {
            let mut score = 0;
            
            // 关键词匹配
            if content_lower.contains("代码") || content_lower.contains("code") || content_lower.contains("编程") {
                if agent.capabilities.iter().any(|c| c.contains("code")) {
                    score += 10;
                }
            }
            if content_lower.contains("区块链") || content_lower.contains("blockchain") || content_lower.contains("转账") {
                if agent.capabilities.iter().any(|c| c.contains("blockchain")) {
                    score += 10;
                }
            }
            if content_lower.contains("翻译") || content_lower.contains("translate") {
                if agent.capabilities.iter().any(|c| c.contains("translation")) {
                    score += 10;
                }
            }
            if content_lower.contains("分析") || content_lower.contains("analyze") {
                if agent.capabilities.iter().any(|c| c.contains("research")) {
                    score += 10;
                }
            }
            
            // 空闲状态加分
            if agent.status == AgentStatus::Idle {
                score += 5;
            }
            
            // 更新最佳智能体
            if score > 0 {
                match &best_agent {
                    Some((_, best_score)) if score > *best_score => {
                        best_agent = Some((agent, score));
                    }
                    None => {
                        best_agent = Some((agent, score));
                    }
                    _ => {}
                }
            }
        }
        
        best_agent.map(|(agent, _)| agent)
    }
    
    /// 重置消息回复计数
    pub async fn reset_message_reply_counts(&self, message_id: &str) {
        let mut counts = self.message_reply_counts.write().await;
        counts.remove(message_id);
    }
    
    /// 清理过期的群聊消息
    pub async fn cleanup_old_messages(&self, max_messages: usize) {
        let mut messages = self.group_messages.write().await;
        
        if messages.len() > max_messages {
            // 按时间排序，保留最新的
            // 先在独立作用域中收集要保留的键
            let to_keep: Vec<String> = {
                let mut sorted: Vec<_> = messages.iter().collect();
                sorted.sort_by(|a, b| b.1.timestamp.cmp(&a.1.timestamp));
                sorted.iter().take(max_messages).map(|(k, _)| (*k).clone()).collect()
            };
            
            // 现在 messages 的不可变借用已经释放，可以进行可变借用
            messages.retain(|k, _| to_keep.iter().any(|x| x == k));
            
            println!("[Coordinator] 清理群聊消息，保留 {} 条", max_messages);
        }
    }
}

/// 队列统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueStats {
    pub total: u32,
    pub by_priority: HashMap<TaskPriority, u32>,
    pub custom_queues: Vec<CustomQueue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomQueue {
    pub name: String,
    pub count: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_coordinator_creation() {
        let coordinator = SwarmCoordinator::default();
        assert!(coordinator.start().await.is_ok());
        assert!(coordinator.stop().await.is_ok());
    }
    
    #[tokio::test]
    async fn test_task_creation() {
        let coordinator = SwarmCoordinator::default();
        coordinator.start().await.unwrap();
        
        let steps = vec![TaskStep {
            id: "step-1".to_string(),
            name: "Test Step".to_string(),
            description: None,
            action: "test".to_string(),
            parameters: None,
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
        }];
        
        let options = TaskCreateOptions::default();
        let task = coordinator.create_task("Test Task", steps, options).await.unwrap();
        
        assert_eq!(task.title, "Test Task");
        assert_eq!(task.status, TaskStatus::Pending);
        
        coordinator.stop().await.unwrap();
    }
}