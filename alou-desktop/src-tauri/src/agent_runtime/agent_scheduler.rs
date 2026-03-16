//! Agent Scheduler - Agent 调度器
//!
//! 解决系统空转问题，实现主动式 Agent 调度

use std::sync::Arc;
use tokio::time::{interval, Duration};
use tokio::sync::RwLock;
use futures::future::join_all;

/// Agent Trait - 所有可被调度的 Agent 必须实现
#[async_trait::async_trait]
pub trait SchedulableAgent: Send + Sync {
    async fn tick(&self) -> TickResult;
    fn is_active(&self) -> bool;
    fn agent_id(&self) -> &str;
}

/// Tick 结果
#[derive(Debug, Clone)]
pub struct TickResult {
    pub agent_id: String,
    pub success: bool,
    pub tasks_spawned: u32,
    pub message: Option<String>,
}

impl TickResult {
    pub fn success(agent_id: String) -> Self {
        Self {
            agent_id,
            success: true,
            tasks_spawned: 0,
            message: None,
        }
    }
    
    pub fn with_tasks(agent_id: String, count: u32) -> Self {
        Self {
            agent_id,
            success: true,
            tasks_spawned: count,
            message: None,
        }
    }
    
    pub fn failure(agent_id: String, message: String) -> Self {
        Self {
            agent_id,
            success: false,
            tasks_spawned: 0,
            message: Some(message),
        }
    }
}

/// Agent 调度器
pub struct AgentScheduler {
    agents: RwLock<Vec<Arc<dyn SchedulableAgent>>>,
    tick_interval_ms: u64,
    is_running: RwLock<bool>,
}

impl AgentScheduler {
    pub fn new(tick_interval_ms: u64) -> Self {
        Self {
            agents: RwLock::new(Vec::new()),
            tick_interval_ms,
            is_running: RwLock::new(false),
        }
    }
    
    pub async fn register(&self, agent: Arc<dyn SchedulableAgent>) {
        log::info!("注册 Agent 到调度器：{}", agent.agent_id());
        self.agents.write().await.push(agent);
    }
    
    pub async fn unregister(&self, agent_id: &str) {
        let mut agents = self.agents.write().await;
        agents.retain(|a| a.agent_id() != agent_id);
        log::info!("从调度器注销 Agent: {}", agent_id);
    }
    
    pub async fn agent_count(&self) -> usize {
        self.agents.read().await.len()
    }
    
    pub async fn run(&self) {
        let mut running = self.is_running.write().await;
        if *running {
            log::warn!("Agent Scheduler 已在运行中");
            return;
        }
        *running = true;
        drop(running);
        
        log::info!("Agent Scheduler 启动，tick 间隔：{}ms", self.tick_interval_ms);
        
        let mut ticker = interval(Duration::from_millis(self.tick_interval_ms));
        
        loop {
            ticker.tick().await;
            
            {
                let running = self.is_running.read().await;
                if !*running {
                    log::info!("Agent Scheduler 停止");
                    break;
                }
            }
            
            self.tick_all().await;
        }
    }
    
    pub async fn stop(&self) {
        let mut running = self.is_running.write().await;
        *running = false;
        log::info!("正在停止 Agent Scheduler...");
    }
    
    async fn tick_all(&self) {
        let agents = self.agents.read().await;
        
        let active_agents: Vec<_> = agents
            .iter()
            .filter(|a| a.is_active())
            .collect();
        
        if active_agents.is_empty() {
            return;
        }
        
        log::debug!("Tick {} 个活跃 Agent", active_agents.len());
        
        let futures = active_agents.iter().map(|agent| {
            async move {
                let result = agent.tick().await;
                
                if !result.success {
                    log::warn!(
                        "Agent tick 失败：{} - {:?}",
                        result.agent_id,
                        result.message
                    );
                }
                
                if result.tasks_spawned > 0 {
                    log::debug!(
                        "Agent {} 生成 {} 个任务",
                        result.agent_id,
                        result.tasks_spawned
                    );
                }
                
                result
            }
        });
        
        let results = join_all(futures).await;
        
        let total_tasks: u32 = results.iter().map(|r| r.tasks_spawned).sum();
        let failures: usize = results.iter().filter(|r| !r.success).count();
        
        if total_tasks > 0 || failures > 0 {
            log::info!(
                "Scheduler tick 完成：生成 {} 个任务，{} 个失败",
                total_tasks,
                failures
            );
        }
    }
    
    pub async fn tick_now(&self) -> Vec<TickResult> {
        let agents = self.agents.read().await;
        
        let futures = agents.iter().map(|agent| {
            async move {
                if agent.is_active() {
                    agent.tick().await
                } else {
                    TickResult::success(agent.agent_id().to_string())
                }
            }
        });
        
        join_all(futures).await
    }
}

impl Default for AgentScheduler {
    fn default() -> Self {
        Self::new(1000)
    }
}
