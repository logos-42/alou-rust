//! 群聊协调器 - 智能体群聊协作核心
//!
//! 实现功能：
//! - 消息过滤（不显示工具调用，只显示最终回复）
//! - 回复次数限制（防止无限回复）
//! - 智能体信息感知
//! - 针对性回复逻辑
//! - 与 Ralph Loop 集成

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::Utc;
use uuid::Uuid;
use log::info;

/// 默认最大回复次数
const DEFAULT_MAX_REPLIES_PER_MESSAGE: u32 = 3;
/// 默认最大消息历史数
const DEFAULT_MAX_MESSAGE_HISTORY: usize = 100;

/// 群聊消息类型
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum GroupMessageType {
    /// 文本消息
    Text,
    /// 命令
    Command,
    /// 系统消息
    System,
    /// 工具调用（不显示）
    ToolCall,
    /// 工具结果（不显示）
    ToolResult,
    /// 智能体响应
    AgentResponse,
    /// 智能体加入
    AgentJoined,
    /// 智能体离开
    AgentLeft,
}

impl Default for GroupMessageType {
    fn default() -> Self {
        Self::Text
    }
}

/// 群聊消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupChatMessage {
    /// 消息ID
    pub id: String,
    /// 群聊ID
    pub group_id: String,
    /// 发送者ID
    pub sender_id: String,
    /// 发送者名称
    pub sender_name: String,
    /// 消息内容
    pub content: String,
    /// 消息类型
    pub message_type: GroupMessageType,
    /// 时间戳
    pub timestamp: i64,
    /// 是否是工具调用
    pub is_tool_call: bool,
    /// 是否是中间步骤
    pub is_intermediate_step: bool,
    /// @提及的智能体ID列表
    pub mentioned_agent_ids: Vec<String>,
    /// 是否已处理（过滤后显示）
    pub processed: bool,
}

impl GroupChatMessage {
    /// 创建新文本消息
    pub fn new(group_id: String, sender_id: String, sender_name: String, content: String) -> Self {
        Self {
            id: format!("msg-{}", Uuid::new_v4().to_string().split('-').next().unwrap()),
            group_id,
            sender_id,
            sender_name,
            content,
            message_type: GroupMessageType::Text,
            timestamp: Utc::now().timestamp_millis(),
            is_tool_call: false,
            is_intermediate_step: false,
            mentioned_agent_ids: Vec::new(),
            processed: false,
        }
    }

    /// 创建工具调用消息
    pub fn new_tool_call(group_id: String, sender_id: String, sender_name: String, content: String) -> Self {
        Self {
            id: format!("msg-{}", Uuid::new_v4().to_string().split('-').next().unwrap()),
            group_id,
            sender_id,
            sender_name,
            content,
            message_type: GroupMessageType::ToolCall,
            timestamp: Utc::now().timestamp_millis(),
            is_tool_call: true,
            is_intermediate_step: true,
            mentioned_agent_ids: Vec::new(),
            processed: false,
        }
    }

    /// 创建系统消息
    pub fn new_system(group_id: String, content: String) -> Self {
        Self {
            id: format!("msg-{}", Uuid::new_v4().to_string().split('-').next().unwrap()),
            group_id,
            sender_id: "system".to_string(),
            sender_name: "系统".to_string(),
            content,
            message_type: GroupMessageType::System,
            timestamp: Utc::now().timestamp_millis(),
            is_tool_call: false,
            is_intermediate_step: false,
            mentioned_agent_ids: Vec::new(),
            processed: true,
        }
    }

    /// 检查消息是否应该显示（过滤工具调用和中间步骤）
    pub fn should_display(&self) -> bool {
        // 过滤工具调用消息
        if self.is_tool_call {
            return false;
        }
        // 过滤中间步骤
        if self.is_intermediate_step {
            return false;
        }
        // 过滤工具调用类型的消息
        if matches!(self.message_type, GroupMessageType::ToolCall | GroupMessageType::ToolResult) {
            return false;
        }
        // 已经处理过的消息不再显示
        if self.processed {
            return false;
        }
        true
    }

    /// 标记为已处理
    pub fn mark_processed(&mut self) {
        self.processed = true;
    }
}

/// 智能体信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    /// 智能体ID
    pub id: String,
    /// 智能体名称
    pub name: String,
    /// 智能体描述
    pub description: Option<String>,
    /// 身份ID
    pub identity_id: Option<String>,
    /// 身份描述/角色
    pub identity_description: Option<String>,
    /// Session 信息
    pub session: Option<String>,
    /// 技能标签
    pub skills: Vec<String>,
    /// 是否在线
    pub online: bool,
    /// 最后活跃时间
    pub last_active: i64,
    /// 加入群聊时间
    pub joined_at: i64,
    /// 已发送消息数
    pub message_count: u32,
    /// 已回复次数
    pub reply_count: u32,
    /// 历史消息统计
    pub history_stats: AgentHistoryStats,
}

/// 智能体历史统计
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentHistoryStats {
    /// 总消息数
    pub total_messages: u32,
    /// 总回复数
    pub total_replies: u32,
    /// 首次发言时间
    pub first_message_at: Option<i64>,
    /// 最后发言时间
    pub last_message_at: Option<i64>,
    /// 活跃天数
    pub active_days: u32,
}

/// 智能体详细信息（用于搜索和展示）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDetailedInfo {
    /// 基本信息
    pub basic: AgentInfo,
    /// 最近消息摘要
    pub recent_messages_summary: Vec<String>,
    /// 协作历史
    pub collaboration_history: Vec<CollaborationRecord>,
}

/// 协作记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationRecord {
    /// 协作时间
    pub timestamp: i64,
    /// 协作类型
    pub collab_type: String,
    /// 参与的智能体ID
    pub participants: Vec<String>,
    /// 协作内容摘要
    pub summary: String,
}

/// 智能体身份信息（用于搜索结果）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentIdentityInfo {
    /// 智能体ID
    pub id: String,
    /// 智能体名称
    pub name: String,
    /// 身份ID
    pub identity_id: Option<String>,
    /// 身份描述
    pub identity_description: Option<String>,
    /// Session
    pub session: Option<String>,
    /// 是否在线
    pub online: bool,
    /// 技能列表
    pub skills: Vec<String>,
}

/// 智能体任务匹配
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTaskFit {
    /// 智能体信息
    pub agent_info: AgentInfo,
    /// 技能匹配度 (0.0 - 1.0)
    pub skill_match_score: f32,
    /// 相关性原因
    pub relevance_reason: String,
}

impl AgentInfo {
    pub fn new(id: String, name: String) -> Self {
        let now = Utc::now().timestamp_millis();
        Self {
            id,
            name,
            description: None,
            identity_id: None,
            identity_description: None,
            session: None,
            skills: Vec::new(),
            online: true,
            last_active: now,
            joined_at: now,
            message_count: 0,
            reply_count: 0,
            history_stats: AgentHistoryStats {
                total_messages: 0,
                total_replies: 0,
                first_message_at: None,
                last_message_at: None,
                active_days: 1,
            },
        }
    }

    /// 完整信息创建
    pub fn with_full_info(
        id: String,
        name: String,
        identity_id: Option<String>,
        identity_description: Option<String>,
        session: Option<String>,
    ) -> Self {
        let now = Utc::now().timestamp_millis();
        Self {
            id,
            name,
            description: None,
            identity_id,
            identity_description,
            session,
            skills: Vec::new(),
            online: true,
            last_active: now,
            joined_at: now,
            message_count: 0,
            reply_count: 0,
            history_stats: AgentHistoryStats {
                total_messages: 0,
                total_replies: 0,
                first_message_at: None,
                last_message_at: None,
                active_days: 1,
            },
        }
    }
}

/// 群聊协调器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupCoordinatorConfig {
    /// 最大回复次数（每个消息）
    pub max_replies_per_message: u32,
    /// 最大消息历史数
    pub max_message_history: usize,
    /// 是否启用智能体响应
    pub enable_agent_response: bool,
    /// 是否显示工具调用（调试用）
    pub show_tool_calls: bool,
}

impl Default for GroupCoordinatorConfig {
    fn default() -> Self {
        Self {
            max_replies_per_message: DEFAULT_MAX_REPLIES_PER_MESSAGE,
            max_message_history: DEFAULT_MAX_MESSAGE_HISTORY,
            enable_agent_response: true,
            show_tool_calls: false,
        }
    }
}

/// 群聊协调器 - 核心组件
pub struct GroupCoordinator {
    /// 配置
    config: GroupCoordinatorConfig,
    /// 群聊消息历史 (group_id -> messages)
    group_messages: Arc<RwLock<HashMap<String, Vec<GroupChatMessage>>>>,
    /// 智能体信息 (agent_id -> info)
    agents: Arc<RwLock<HashMap<String, AgentInfo>>>,
    /// 消息回复计数 (message_id -> count)
    reply_counts: Arc<RwLock<HashMap<String, u32>>>,
    /// 当前群聊ID
    current_group_id: Arc<RwLock<Option<String>>>,
}

impl GroupCoordinator {
    /// 创建新的协调器
    pub fn new(config: Option<GroupCoordinatorConfig>) -> Self {
        Self {
            config: config.unwrap_or_default(),
            group_messages: Arc::new(RwLock::new(HashMap::new())),
            agents: Arc::new(RwLock::new(HashMap::new())),
            reply_counts: Arc::new(RwLock::new(HashMap::new())),
            current_group_id: Arc::new(RwLock::new(None)),
        }
    }

    /// 切换当前群聊
    pub async fn set_current_group(&self, group_id: String) {
        let group_id_clone = group_id.clone();
        let mut current = self.current_group_id.write().await;
        *current = Some(group_id_clone.clone());
        
        // 初始化群聊消息列表
        let mut messages = self.group_messages.write().await;
        if !messages.contains_key(&group_id) {
            messages.insert(group_id, Vec::new());
        }
        
        info!("[GroupCoordinator] 切换到群聊: {}", group_id_clone);
    }

    /// 注册智能体（完整信息）
    pub async fn register_agent_full(
        &self,
        agent_id: String,
        name: String,
        identity_id: Option<String>,
        identity_description: Option<String>,
        session: Option<String>,
        skills: Option<Vec<String>>,
    ) {
        let mut agents = self.agents.write().await;
        let now = Utc::now().timestamp_millis();
        let agent = AgentInfo {
            id: agent_id.clone(),
            name: name.clone(),
            description: None,
            identity_id,
            identity_description,
            session,
            skills: skills.unwrap_or_default(),
            online: true,
            last_active: now,
            joined_at: now,
            message_count: 0,
            reply_count: 0,
            history_stats: AgentHistoryStats {
                total_messages: 0,
                total_replies: 0,
                first_message_at: None,
                last_message_at: None,
                active_days: 1,
            },
        };
        agents.insert(agent_id.clone(), agent);
        info!("[GroupCoordinator] 注册智能体: {} ({})", name, agent_id);
    }

    /// 注册智能体（简单信息）
    pub async fn register_agent(&self, agent_id: String, name: String, description: Option<String>) {
        let mut agents = self.agents.write().await;
        let now = Utc::now().timestamp_millis();
        let agent = AgentInfo {
            id: agent_id.clone(),
            name: name.clone(),
            description,
            identity_id: None,
            identity_description: None,
            session: None,
            skills: Vec::new(),
            online: true,
            last_active: now,
            joined_at: now,
            message_count: 0,
            reply_count: 0,
            history_stats: AgentHistoryStats {
                total_messages: 0,
                total_replies: 0,
                first_message_at: None,
                last_message_at: None,
                active_days: 1,
            },
        };
        agents.insert(agent_id.clone(), agent);
        info!("[GroupCoordinator] 注册智能体: {} ({})", name, agent_id);
    }

    /// 注销智能体
    pub async fn unregister_agent(&self, agent_id: &str) {
        let mut agents = self.agents.write().await;
        if let Some(agent) = agents.get_mut(agent_id) {
            agent.online = false;
            info!("[GroupCoordinator] 智能体下线: {}", agent_id);
        }
    }

    /// 获取所有在线智能体
    pub async fn get_online_agents(&self) -> Vec<AgentInfo> {
        let agents = self.agents.read().await;
        agents.values()
            .filter(|a| a.online)
            .cloned()
            .collect()
    }

    /// 获取所有智能体（包括离线的）
    pub async fn get_all_agents(&self) -> Vec<AgentInfo> {
        let agents = self.agents.read().await;
        agents.values().cloned().collect()
    }

    /// 根据ID获取智能体信息
    pub async fn get_agent_by_id(&self, agent_id: &str) -> Option<AgentInfo> {
        let agents = self.agents.read().await;
        agents.get(agent_id).cloned()
    }

    /// 根据名称搜索智能体
    pub async fn search_agents_by_name(&self, name_query: &str) -> Vec<AgentInfo> {
        let agents = self.agents.read().await;
        let query_lower = name_query.to_lowercase();
        agents.values()
            .filter(|a| a.name.to_lowercase().contains(&query_lower))
            .cloned()
            .collect()
    }

    /// 根据身份ID搜索智能体
    pub async fn search_agents_by_identity(&self, identity_id: &str) -> Vec<AgentInfo> {
        let agents = self.agents.read().await;
        agents.values()
            .filter(|a| a.identity_id.as_deref() == Some(identity_id))
            .cloned()
            .collect()
    }

    /// 根据技能搜索智能体
    pub async fn search_agents_by_skill(&self, skill: &str) -> Vec<AgentInfo> {
        let agents = self.agents.read().await;
        let skill_lower = skill.to_lowercase();
        agents.values()
            .filter(|a| a.skills.iter().any(|s| s.to_lowercase().contains(&skill_lower)))
            .cloned()
            .collect()
    }

    /// 获取最近活跃的智能体
    pub async fn get_recent_agents(&self, limit: usize) -> Vec<AgentInfo> {
        let agents = self.agents.read().await;
        let mut agent_list: Vec<_> = agents.values().cloned().collect();
        agent_list.sort_by(|a, b| b.last_active.cmp(&a.last_active));
        agent_list.truncate(limit);
        agent_list
    }

    /// 在特定时间范围内搜索活跃智能体
    pub async fn get_agents_active_in_range(&self, start_time: i64, end_time: i64) -> Vec<AgentInfo> {
        let agents = self.agents.read().await;
        agents.values()
            .filter(|a| a.last_active >= start_time && a.last_active <= end_time)
            .cloned()
            .collect()
    }

    /// 构建智能体详细信息（用于搜索结果）
    pub async fn get_agent_details(&self, agent_id: &str) -> Option<AgentDetailedInfo> {
        let agents = self.agents.read().await;
        if let Some(agent) = agents.get(agent_id) {
            Some(AgentDetailedInfo {
                basic: agent.clone(),
                recent_messages_summary: Vec::new(),
                collaboration_history: Vec::new(),
            })
        } else {
            None
        }
    }

    /// 添加消息到群聊
    pub async fn add_message(&self, message: GroupChatMessage) {
        let group_id = message.group_id.clone();
        let now = Utc::now().timestamp_millis();
        
        // 更新智能体消息计数
        if !message.sender_id.starts_with("system") {
            let mut agents = self.agents.write().await;
            if let Some(agent) = agents.get_mut(&message.sender_id) {
                agent.message_count += 1;
                agent.last_active = now;
                
                // 更新历史统计
                if agent.history_stats.first_message_at.is_none() {
                    agent.history_stats.first_message_at = Some(now);
                }
                agent.history_stats.last_message_at = Some(now);
                agent.history_stats.total_messages += 1;
            }
        }
        
        // 添加消息到历史
        let mut messages = self.group_messages.write().await;
        let group_messages = messages.entry(group_id).or_insert_with(Vec::new);
        group_messages.push(message);
        
        // 限制消息历史大小
        if group_messages.len() > self.config.max_message_history {
            group_messages.remove(0);
        }
    }

    /// 获取显示的消息（过滤后）
    pub async fn get_displayable_messages(&self, group_id: &str) -> Vec<GroupChatMessage> {
        let messages = self.group_messages.read().await;
        if let Some(group_messages) = messages.get(group_id) {
            group_messages
                .iter()
                .filter(|msg| msg.should_display())
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    /// 获取所有消息（包括未过滤的）
    pub async fn get_all_messages(&self, group_id: &str) -> Vec<GroupChatMessage> {
        let messages = self.group_messages.read().await;
        if let Some(group_messages) = messages.get(group_id) {
            group_messages.clone()
        } else {
            Vec::new()
        }
    }

    /// 检查智能体是否可以回复
    pub async fn can_agent_respond(&self, agent_id: &str, message_id: &str) -> bool {
        if !self.config.enable_agent_response {
            return false;
        }
        
        // 检查智能体是否在线
        let agents = self.agents.read().await;
        if let Some(agent) = agents.get(agent_id) {
            if !agent.online {
                return false;
            }
        } else {
            return false;
        }
        
        // 检查回复次数
        let counts = self.reply_counts.read().await;
        let count = counts.get(message_id).unwrap_or(&0);
        
        *count < self.config.max_replies_per_message
    }

    /// 记录智能体回复
    pub async fn record_agent_response(&self, agent_id: &str, message_id: &str) {
        let now = Utc::now().timestamp_millis();
        
        // 更新回复计数
        let mut counts = self.reply_counts.write().await;
        let count = counts.entry(message_id.to_string()).or_insert(0);
        *count += 1;
        
        // 更新智能体回复计数
        let mut agents = self.agents.write().await;
        if let Some(agent) = agents.get_mut(agent_id) {
            agent.reply_count += 1;
            agent.history_stats.total_replies += 1;
            agent.last_active = now;
        }
        
        info!("[GroupCoordinator] 智能体 {} 回复了消息 {}, 当前回复数: {}", 
            agent_id, message_id, counts.get(message_id).unwrap_or(&0));
    }

    /// 清除群聊历史
    pub async fn clear_group(&self, group_id: &str) {
        let mut messages = self.group_messages.write().await;
        messages.remove(group_id);
        
        let mut counts = self.reply_counts.write().await;
        counts.retain(|k, _| !k.starts_with(group_id));
        
        info!("[GroupCoordinator] 清除群聊历史: {}", group_id);
    }

    /// 构建群聊上下文（用于 Ralph Loop prompt）
    /// 包含所有智能体的详细信息，便于智能体了解群聊成员
    pub async fn build_context_prompt(&self, group_id: &str, exclude_agent_id: Option<&str>) -> String {
        let messages = self.get_displayable_messages(group_id).await;
        let agents = self.get_online_agents().await;
        
        let mut context = String::new();
        
        // 智能体列表（包含详细信息）
        context.push_str("=== 群聊智能体 ===\n");
        for agent in &agents {
            // 排除指定智能体
            if let Some(exclude_id) = exclude_agent_id {
                if agent.id == exclude_id {
                    continue;
                }
            }
            context.push_str(&format!("- 名称: {}\n", agent.name));
            context.push_str(&format!("  ID: {}\n", agent.id));
            if let Some(ref identity_id) = agent.identity_id {
                context.push_str(&format!("  身份ID: {}\n", identity_id));
            }
            if let Some(ref identity_desc) = agent.identity_description {
                context.push_str(&format!("  身份描述: {}\n", identity_desc));
            }
            if let Some(ref session) = agent.session {
                context.push_str(&format!("  Session: {}\n", session));
            }
            if !agent.skills.is_empty() {
                context.push_str(&format!("  技能: {}\n", agent.skills.join(", ")));
            }
            context.push_str(&format!("  状态: {}\n", if agent.online { "在线" } else { "离线" }));
            context.push_str(&format!("  发言数: {}, 回复数: {}\n", agent.message_count, agent.reply_count));
            context.push_str("\n");
        }
        
        // 消息历史
        context.push_str("=== 消息历史 (最近20条) ===\n");
        for msg in messages.iter().rev().take(20) {
            // 排除指定智能体的消息
            if let Some(exclude_id) = exclude_agent_id {
                if msg.sender_id == exclude_id {
                    continue;
                }
            }
            context.push_str(&format!("[{}] {}: {}\n", 
                chrono::DateTime::from_timestamp_millis(msg.timestamp)
                    .map(|dt| dt.format("%H:%M").to_string())
                    .unwrap_or_default(),
                msg.sender_name, 
                msg.content));
        }
        
        context
    }

    /// 构建简洁的智能体列表（用于快速参考）
    pub async fn build_agent_list_prompt(&self, exclude_agent_id: Option<&str>) -> String {
        let agents = self.get_online_agents().await;
        
        let mut context = String::new();
        context.push_str("群聊成员:\n");
        for agent in &agents {
            if let Some(exclude_id) = exclude_agent_id {
                if agent.id == exclude_id {
                    continue;
                }
            }
            context.push_str(&format!("- {} ({})", agent.name, agent.id));
            if let Some(ref identity_desc) = agent.identity_description {
                context.push_str(&format!(" - {}", identity_desc));
            }
            context.push_str("\n");
        }
        
        context
    }

    /// 搜索特定智能体的身份信息（在执行任务前使用）
    pub async fn find_agent_identity(&self, name_query: Option<&str>, identity_id_query: Option<&str>) -> Vec<AgentIdentityInfo> {
        let agents = self.agents.read().await;
        
        let mut results = Vec::new();
        
        for agent in agents.values() {
            let mut matches = false;
            
            // 按名称搜索
            if let Some(name_query) = name_query {
                if agent.name.to_lowercase().contains(&name_query.to_lowercase()) {
                    matches = true;
                }
            }
            
            // 按身份ID搜索
            if let Some(identity_id_query) = identity_id_query {
                if let Some(ref identity_id) = agent.identity_id {
                    if identity_id.to_lowercase().contains(&identity_id_query.to_lowercase()) {
                        matches = true;
                    }
                }
            }
            
            if matches {
                results.push(AgentIdentityInfo {
                    id: agent.id.clone(),
                    name: agent.name.clone(),
                    identity_id: agent.identity_id.clone(),
                    identity_description: agent.identity_description.clone(),
                    session: agent.session.clone(),
                    online: agent.online,
                    skills: agent.skills.clone(),
                });
            }
        }
        
        results
    }

    /// 获取特定时间范围内的智能体活动（用于任务分配）
    pub async fn get_agents_for_task(&self, task_keywords: &[String], time_range_minutes: i64) -> Vec<AgentTaskFit> {
        let now = Utc::now().timestamp_millis();
        let time_threshold = now - (time_range_minutes * 60 * 1000);
        
        let agents = self.agents.read().await;
        let mut task_fits = Vec::new();
        
        for agent in agents.values() {
            if !agent.online {
                continue;
            }
            
            // 检查是否在时间范围内活跃
            if agent.last_active < time_threshold {
                continue;
            }
            
            // 计算任务匹配度
            let skill_match = if task_keywords.is_empty() {
                1.0
            } else {
                let mut match_count = 0;
                for keyword in task_keywords {
                    if agent.skills.iter().any(|s| s.to_lowercase().contains(&keyword.to_lowercase()))
                        || agent.name.to_lowercase().contains(&keyword.to_lowercase()) 
                        || agent.identity_description.as_ref().map(|d| d.to_lowercase().contains(&keyword.to_lowercase())).unwrap_or(false)
                    {
                        match_count += 1;
                    }
                }
                match_count as f32 / task_keywords.len() as f32
            };
            
            task_fits.push(AgentTaskFit {
                agent_info: agent.clone(),
                skill_match_score: skill_match,
                relevance_reason: Self::generate_relevance_reason(agent, task_keywords),
            });
        }
        
        // 按匹配度排序
        task_fits.sort_by(|a, b| b.skill_match_score.partial_cmp(&a.skill_match_score).unwrap());
        
        task_fits
    }
    
    /// 生成相关性原因
    fn generate_relevance_reason(agent: &AgentInfo, keywords: &[String]) -> String {
        let mut reasons = Vec::new();
        
        for keyword in keywords {
            if agent.skills.iter().any(|s| s.to_lowercase().contains(&keyword.to_lowercase())) {
                reasons.push(format!("拥有相关技能: {}", keyword));
            }
            if agent.name.to_lowercase().contains(&keyword.to_lowercase()) {
                reasons.push(format!("名称包含: {}", keyword));
            }
            if let Some(ref desc) = agent.identity_description {
                if desc.to_lowercase().contains(&keyword.to_lowercase()) {
                    reasons.push(format!("身份描述匹配: {}", keyword));
                }
            }
        }
        
        if reasons.is_empty() {
            format!("最近活跃（{}分钟内）", keywords.len())
        } else {
            reasons.join(", ")
        }
    }

    /// 处理新消息（判断是否需要智能体响应）
    pub async fn should_agent_respond(&self, message: &GroupChatMessage) -> bool {
        // 系统消息不需要响应
        if message.message_type == GroupMessageType::System {
            return false;
        }
        
        // 工具调用不需要响应
        if message.is_tool_call || message.is_intermediate_step {
            return false;
        }
        
        // 如果@提及了智能体，需要响应
        if !message.mentioned_agent_ids.is_empty() {
            return true;
        }
        
        // 用户发送的消息需要响应（非智能体发送）
        if !message.sender_id.starts_with("agent_") && !message.sender_id.starts_with("system") {
            return true;
        }
        
        // 智能体发送的消息，检查是否是回复
        if message.message_type == GroupMessageType::AgentResponse {
            return true;
        }
        
        false
    }
}
