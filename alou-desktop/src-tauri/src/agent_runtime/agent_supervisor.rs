//! Agent Supervisor - Agent 监督器（崩溃恢复）

use crate::agent_runtime::agent_registry::{AgentInfo, AgentConfig};
use crate::agent_runtime::agent_actor::{AgentActor, AgentHandle, ActorMessage};
use crate::agent_runtime::message_bus::GroupMessage;
use tokio::sync::{mpsc, RwLock};
use std::collections::HashMap;

/// 重启策略
#[derive(Clone)]
pub enum RestartPolicy {
    Never,
    Always,
    OnFailure(u32),  // 最大重启次数
}

/// Agent 监督器
pub struct AgentSupervisor {
    actors: RwLock<HashMap<String, AgentHandle>>,
    configs: RwLock<HashMap<String, AgentConfig>>,
    restart_counts: RwLock<HashMap<String, u32>>,
    restart_policy: RestartPolicy,
}

impl AgentSupervisor {
    pub fn new(restart_policy: RestartPolicy) -> Self {
        Self {
            actors: RwLock::new(HashMap::new()),
            configs: RwLock::new(HashMap::new()),
            restart_counts: RwLock::new(HashMap::new()),
            restart_policy,
        }
    }
    
    /// 生成 agent（带监控）
    pub async fn spawn(&self, agent: AgentInfo, config: AgentConfig) {
        let (tx, rx) = mpsc::channel(100);
        
        let handle = AgentHandle::new(tx);
        self.actors.write().await.insert(agent.id.clone(), handle);
        self.configs.write().await.insert(agent.id.clone(), config.clone());
        
        // 启动 actor
        let actor = AgentActor::new(agent.clone(), config.clone(), rx);
        
        // 启动监控 task
        let supervisor = self.clone();
        let agent_id = agent.id.clone();
        
        tokio::spawn(async move {
            loop {
                let result = actor.clone().run().await;
                
                match result {
                    Ok(_) => {
                        log::info!("Agent 正常退出：{}", agent_id);
                        break;
                    }
                    Err(e) => {
                        log::error!("Agent 异常退出：{} - {}", agent_id, e);
                        
                        // 检查重启策略
                        let should_restart = match &supervisor.restart_policy {
                            RestartPolicy::Never => false,
                            RestartPolicy::Always => true,
                            RestartPolicy::OnFailure(max) => {
                                let mut counts = supervisor.restart_counts.write().await;
                                let count = counts.entry(agent_id.clone()).or_insert(0);
                                *count += 1;
                                *count <= max
                            }
                        };
                        
                        if should_restart {
                            log::info!("重启 Agent: {}", agent_id);
                            // 这里需要重新创建 actor
                            // 简化实现：不自动重启，只记录日志
                        } else {
                            log::error!("Agent 重启次数超限，不再重启：{}", agent_id);
                            break;
                        }
                    }
                }
            }
        });
        
        log::info!("Agent 已生成：{} ({})", agent.name, agent.id);
    }
    
    /// 发送消息到 agent
    pub async fn send_to_agent(&self, agent_id: &str, message: GroupMessage, history: Vec<GroupMessage>) -> Result<(), String> {
        let actors = self.actors.read().await;
        let handle = actors.get(agent_id)
            .ok_or_else(|| format!("Agent 不存在：{}", agent_id))?;
        
        handle.send_message(message, history).await
    }
    
    /// 停止 agent
    pub async fn stop(&self, agent_id: &str) -> Result<(), String> {
        let actors = self.actors.read().await;
        let handle = actors.get(agent_id)
            .ok_or_else(|| format!("Agent 不存在：{}", agent_id))?;
        
        handle.stop().await
    }
    
    /// 获取所有活跃的 agent IDs
    pub async fn list_active_agents(&self) -> Vec<String> {
        self.actors.read().await.keys().cloned().collect()
    }
}

impl Clone for AgentSupervisor {
    fn clone(&self) -> Self {
        Self {
            actors: self.actors.clone(),
            configs: self.configs.clone(),
            restart_counts: self.restart_counts.clone(),
            restart_policy: self.restart_policy.clone(),
        }
    }
}
