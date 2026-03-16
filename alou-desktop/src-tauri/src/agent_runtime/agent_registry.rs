//! Agent Registry - Agent 注册表（原子更新索引）

use std::collections::HashMap;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

/// 群聊订阅配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupSubscription {
    pub group_id: String,
    pub mode: GroupChatMode,
    pub auto_reply: bool,
}

/// 群聊模式
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum GroupChatMode {
    PubSub,
    Iroh,
    Memory,
}

impl Default for GroupChatMode {
    fn default() -> Self {
        GroupChatMode::Memory
    }
}

/// Agent 信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub capabilities: Vec<String>,
    pub groups: Vec<String>,
    pub script_path: String,
    pub enabled: bool,
    pub config: AgentConfig,
    pub subscribed_groups: Vec<GroupSubscription>,  // 新增：群聊订阅
}

/// Agent 配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentConfig {
    pub mention_only: bool,
    pub auto_reply: bool,
    pub reply_delay_ms: u64,
    pub max_context_messages: usize,
    pub custom_prompt: Option<String>,
}

/// Agent 注册表
pub struct AgentRegistry {
    agents: RwLock<HashMap<String, AgentInfo>>,
    index_capability: RwLock<HashMap<String, Vec<String>>>,
    index_group: RwLock<HashMap<String, Vec<String>>>,
}

impl AgentRegistry {
    pub fn new() -> Self {
        Self {
            agents: RwLock::new(HashMap::new()),
            index_capability: RwLock::new(HashMap::new()),
            index_group: RwLock::new(HashMap::new()),
        }
    }

    /// 原子注册（避免索引不一致）
    pub async fn register(&self, agent: AgentInfo) {
        let mut agents = self.agents.write().await;
        let agent_id = agent.id.clone();

        // 如果是更新，先删除旧索引
        if let Some(old_agent) = agents.get(&agent_id) {
            self._remove_indexes(old_agent).await;
        }

        // 插入 agent
        agents.insert(agent_id.clone(), agent.clone());
        drop(agents);

        // 构建新索引
        self._add_indexes(&agent).await;

        log::info!("Agent 注册：{} ({})", agent.name, agent_id);
    }

    /// 注销 agent
    pub async fn unregister(&self, agent_id: &str) {
        let mut agents = self.agents.write().await;
        if let Some(agent) = agents.remove(agent_id) {
            drop(agents);
            self._remove_indexes(&agent).await;
            log::info!("Agent 注销：{} ({})", agent.name, agent_id);
        }
    }

    async fn _remove_indexes(&self, agent: &AgentInfo) {
        // 删除能力索引
        for cap in &agent.capabilities {
            let mut index = self.index_capability.write().await;
            if let Some(agents) = index.get_mut(cap) {
                agents.retain(|id| id != &agent.id);
                if agents.is_empty() {
                    index.remove(cap);
                }
            }
        }

        // 删除群聊索引
        for group in &agent.groups {
            let mut index = self.index_group.write().await;
            if let Some(agents) = index.get_mut(group) {
                agents.retain(|id| id != &agent.id);
                if agents.is_empty() {
                    index.remove(group);
                }
            }
        }
    }

    async fn _add_indexes(&self, agent: &AgentInfo) {
        // 添加能力索引
        for cap in &agent.capabilities {
            let mut index = self.index_capability.write().await;
            index.entry(cap.clone())
                .or_insert_with(Vec::new)
                .push(agent.id.clone());
        }

        // 添加群聊索引
        for group in &agent.groups {
            let mut index = self.index_group.write().await;
            index.entry(group.clone())
                .or_insert_with(Vec::new)
                .push(agent.id.clone());
        }
    }

    pub async fn get(&self, agent_id: &str) -> Option<AgentInfo> {
        let agents = self.agents.read().await;
        agents.get(agent_id).cloned()
    }

    pub async fn get_by_capability(&self, capability: &str) -> Vec<AgentInfo> {
        let index = self.index_capability.read().await;
        let agents = self.agents.read().await;

        index.get(capability)
            .map(|ids| ids.iter().filter_map(|id| agents.get(id).cloned()).collect())
            .unwrap_or_default()
    }

    pub async fn get_by_group(&self, group_id: &str) -> Vec<AgentInfo> {
        let index = self.index_group.read().await;
        let agents = self.agents.read().await;

        index.get(group_id)
            .map(|ids| ids.iter().filter_map(|id| agents.get(id).cloned()).collect())
            .unwrap_or_default()
    }

    pub async fn list_all(&self) -> Vec<AgentInfo> {
        let agents = self.agents.read().await;
        agents.values().cloned().collect()
    }

    pub async fn update_enabled(&self, agent_id: &str, enabled: bool) {
        let mut agents = self.agents.write().await;
        if let Some(agent) = agents.get_mut(agent_id) {
            agent.enabled = enabled;
            log::info!("Agent {} 启用状态：{}", agent_id, if enabled { "启用" } else { "禁用" });
        }
    }
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for AgentRegistry {
    fn clone(&self) -> Self {
        Self::new()
    }
}
