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
use log::{info, warn, error};

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
    /// 是否在线
    pub online: bool,
    /// 最后活跃时间
    pub last_active: i64,
    /// 已发送消息数
    pub message_count: u32,
    /// 已回复次数
    pub reply_count: u32,
}

impl AgentInfo {
    pub fn new(id: String, name: String) -> Self {
        Self {
            id,
            name,
            description: None,
            online: true,
            last_active: Utc::now().timestamp_millis(),
            message_count: 0,
            reply_count: 0,
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
        let mut current = self.current_group_id.write().await;
        *current = Some(group_id.clone());
        
        // 初始化群聊消息列表
        let mut messages = self.group_messages.write().await;
        if !messages.contains_key(&group_id) {
            messages.insert(group_id, Vec::new());
        }
        
        info!("[GroupCoordinator] 切换到群聊: {}", group_id);
    }

    /// 注册智能体
    pub async fn register_agent(&self, agent_id: String, name: String, description: Option<String>) {
        let mut agents = self.agents.write().await;
        let agent = AgentInfo {
            id: agent_id.clone(),
            name: name.clone(),
            description,
            online: true,
            last_active: Utc::now().timestamp_millis(),
            message_count: 0,
            reply_count: 0,
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

    /// 添加消息到群聊
    pub async fn add_message(&self, message: GroupChatMessage) {
        let group_id = message.group_id.clone();
        
        // 更新智能体消息计数
        if !message.sender_id.starts_with("system") {
            let mut agents = self.agents.write().await;
            if let Some(agent) = agents.get_mut(&message.sender_id) {
                agent.message_count += 1;
                agent.last_active = Utc::now().timestamp_millis();
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
        // 更新回复计数
        let mut counts = self.reply_counts.write().await;
        let count = counts.entry(message_id.to_string()).or_insert(0);
        *count += 1;
        
        // 更新智能体回复计数
        let mut agents = self.agents.write().await;
        if let Some(agent) = agents.get_mut(agent_id) {
            agent.reply_count += 1;
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
    pub async fn build_context_prompt(&self, group_id: &str, exclude_agent_id: Option<&str>) -> String {
        let messages = self.get_displayable_messages(group_id).await;
        let agents = self.get_online_agents().await;
        
        let mut context = String::new();
        
        // 智能体列表
        context.push_str("=== 群聊智能体 ===\n");
        for agent in &agents {
            context.push_str(&format!("- {} ({.name, agent.id));
        }
        context.push('\n})\n", agent');
        
        // 消息历史
        context.push_str("=== 消息历史 ===\n");
        for msg in messages.iter().rev().take(20) {
            // 排除指定智能体的消息
            if let Some(exclude_id) = exclude_agent_id {
                if msg.sender_id == exclude_id {
                    continue;
                }
            }
            context.push_str(&format!("[{}]: {}\n", msg.sender_name, msg.content));
        }
        
        context
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
        
        // 用户发送的消息需要响应
        if
        
        // 用户发送的消息需要响应（非智能体发送）
        if !message.sender_id.starts_with("agent_") {
            return true;
        }
        
        // 智能体发送的消息，检查是否是回复
        if message.message_type == GroupMessageType::AgentResponse {
            return true;
        }
        
        false
    }
}
