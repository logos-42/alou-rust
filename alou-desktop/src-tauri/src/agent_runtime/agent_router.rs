//! Agent Router - Agent 路由器（三种路由模式）

use crate::agent_runtime::agent_registry::{AgentInfo, AgentRegistry};
use crate::agent_runtime::message_bus::GroupMessage;
use std::sync::Arc;
use tokio::sync::RwLock;
use regex::Regex;

/// Agent 路由器
pub struct AgentRouter {
    registry: Arc<AgentRegistry>,
    mention_pattern: RwLock<Option<Regex>>,
}

impl AgentRouter {
    pub fn new(registry: Arc<AgentRegistry>) -> Self {
        Self {
            registry,
            mention_pattern: RwLock::new(None),
        }
    }
    
    /// 初始化提及模式
    async fn init_mention_pattern(&self) {
        let mut pattern = self.mention_pattern.write().await;
        *pattern = Regex::new(r"@(\w+)").ok();
    }
    
    /// 路由消息到目标 agent
    /// 
    /// 返回应该接收此消息的 agent ID 列表
    pub async fn route(&self, message: &GroupMessage) -> Vec<String> {
        // 确保提及模式已初始化
        if self.mention_pattern.read().await.is_none() {
            self.init_mention_pattern().await;
        }
        
        let pattern = self.mention_pattern.read().await;
        
        // 1. 检查是否被 @ 提及（名称路由）
        if let Some(ref regex) = *pattern {
            for captures in regex.captures_iter(&message.content) {
                if let Some(mentioned) = captures.get(1) {
                    let mentioned_name = mentioned.as_str();
                    
                    // 查找名称匹配的 agent
                    for agent in self.registry.list_all().await {
                        if agent.name == mentioned_name || agent.display_name == mentioned_name {
                            if agent.enabled {
                                return vec![agent.id.clone()];
                            }
                        }
                    }
                }
            }
        }
        
        // 2. 检查是否需要特定能力（能力路由）
        if let Some(capability) = self.detect_capability(&message.content) {
            let agents = self.registry.get_by_capability(&capability).await;
            let enabled_agents: Vec<String> = agents
                .iter()
                .filter(|a| a.enabled)
                .map(|a| a.id.clone())
                .collect();
            
            if !enabled_agents.is_empty() {
                return enabled_agents;
            }
        }
        
        // 3. 广播到群聊所有 agent（广播路由）
        let agents = self.registry.get_by_group(&message.group_id).await;
        agents
            .iter()
            .filter(|a| a.enabled)
            .map(|a| a.id.clone())
            .collect()
    }
    
    /// 检测消息需要的能力
    fn detect_capability(&self, content: &str) -> Option<String> {
        let content_lower = content.to_lowercase();
        
        // 简单的关键词匹配
        if content_lower.contains("翻译") || content_lower.contains("translate") {
            return Some("translate".to_string());
        }
        if content_lower.contains("搜索") || content_lower.contains("search") {
            return Some("search".to_string());
        }
        if content_lower.contains("计算") || content_lower.contains("calculate") {
            return Some("calculate".to_string());
        }
        if content_lower.contains("写作") || content_lower.contains("write") {
            return Some("write".to_string());
        }
        if content_lower.contains("代码") || content_lower.contains("code") {
            return Some("code".to_string());
        }
        
        None
    }
    
    /// 获取群聊中的所有 agent
    pub async fn get_group_agents(&self, group_id: &str) -> Vec<AgentInfo> {
        self.registry.get_by_group(group_id).await
    }
    
    /// 注册 agent 到群聊
    pub async fn add_agent_to_group(&self, agent_id: &str, group_id: &str) {
        let mut agents = self.registry.agents.write().await;
        if let Some(agent) = agents.get_mut(agent_id) {
            if !agent.groups.contains(&group_id.to_string()) {
                agent.groups.push(group_id.to_string());
            }
        }
    }
}
