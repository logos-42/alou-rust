//! 统一群聊适配器实现
//!
//! 核心逻辑：管理三种模式的群聊存储和操作

use crate::tools::adapters::types::*;
use crate::tools::{ToolExecutor, ToolResult, ToolError, ToolCategory, ToolMetadata, ToolStatus, ToolPriority, ExecutionContext};
use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::OnceLock;
use serde::Deserialize;
use chrono::Utc;
use uuid::Uuid;

/// 统一的群聊适配器
pub struct GroupAdapter {
    memory_groups: Mutex<HashMap<String, UnifiedGroup>>,
    memory_messages: Mutex<HashMap<String, Vec<UnifiedMessage>>>,
    pubsub_groups: Mutex<HashMap<String, UnifiedGroup>>,
    pubsub_messages: Mutex<HashMap<String, Vec<UnifiedMessage>>>,
    iroh_groups: Mutex<HashMap<String, UnifiedGroup>>,
    iroh_messages: Mutex<HashMap<String, Vec<UnifiedMessage>>>,
    local_identity: Mutex<Option<AgentInfo>>,
}

impl GroupAdapter {
    pub fn new() -> Self {
        Self {
            memory_groups: Mutex::new(HashMap::new()),
            memory_messages: Mutex::new(HashMap::new()),
            pubsub_groups: Mutex::new(HashMap::new()),
            pubsub_messages: Mutex::new(HashMap::new()),
            iroh_groups: Mutex::new(HashMap::new()),
            iroh_messages: Mutex::new(HashMap::new()),
            local_identity: Mutex::new(None),
        }
    }

    pub fn set_local_identity(&self, identity: AgentInfo) {
        if let Ok(mut local) = self.local_identity.lock() {
            *local = Some(identity);
        }
    }

    pub fn get_local_identity(&self) -> Option<AgentInfo> {
        self.local_identity.lock().ok().and_then(|local| local.clone())
    }

    pub fn create_group(&self, config: GroupChatConfig) -> Result<UnifiedGroup, String> {
        let mode = config.mode.unwrap_or(GroupChatMode::Memory);
        let mut group = UnifiedGroup::new(&config.name, mode);
        
        if let Some(desc) = config.description {
            group.description = Some(desc);
        }
        
        if let Some(members) = config.members {
            group.members = members;
        }

        match mode {
            GroupChatMode::Memory => {
                let mut groups = self.memory_groups.lock().map_err(|e| format!("Lock error: {}", e))?;
                groups.insert(group.id.clone(), group.clone());
                self.memory_messages.lock().map_err(|e| format!("Lock error: {}", e))?
                    .insert(group.id.clone(), Vec::new());
            },
            GroupChatMode::Pubsub => {
                let mut groups = self.pubsub_groups.lock().map_err(|e| format!("Lock error: {}", e))?;
                group.topic = Some(format!("diap/group/{}", group.id));
                groups.insert(group.id.clone(), group.clone());
                self.pubsub_messages.lock().map_err(|e| format!("Lock error: {}", e))?
                    .insert(group.id.clone(), Vec::new());
            },
            GroupChatMode::Iroh => {
                let mut groups = self.iroh_groups.lock().map_err(|e| format!("Lock error: {}", e))?;
                group.ticket = Some(format!("iroh://{}", group.id));
                group.topic = Some(format!("iroh/group/{}", group.id));
                groups.insert(group.id.clone(), group.clone());
                self.iroh_messages.lock().map_err(|e| format!("Lock error: {}", e))?
                    .insert(group.id.clone(), Vec::new());
            },
        }

        let _ = self.add_system_message(&group.id, format!("群聊 \"{}\" 已创建", config.name), MessageType::System, mode);
        Ok(group)
    }

    pub fn join_group(&self, group_id: &str) -> Result<UnifiedGroup, String> {
        if let Ok(groups) = self.memory_groups.lock() {
            if let Some(group) = groups.get(group_id) {
                return self.do_join_group(group.clone());
            }
        }
        if let Ok(groups) = self.pubsub_groups.lock() {
            if let Some(group) = groups.get(group_id) {
                return self.do_join_group(group.clone());
            }
        }
        if let Ok(groups) = self.iroh_groups.lock() {
            if let Some(group) = groups.get(group_id) {
                return self.do_join_group(group.clone());
            }
        }
        Err(format!("Group not found: {}", group_id))
    }

    fn do_join_group(&self, mut group: UnifiedGroup) -> Result<UnifiedGroup, String> {
        let identity = self.get_local_identity().unwrap_or(AgentInfo::new("anonymous", "Anonymous"));
        if !group.members.iter().any(|m| m.id == identity.id) {
            group.members.push(identity.clone());
        }

        let mode = group.mode;
        match mode {
            GroupChatMode::Memory => {
                let mut groups = self.memory_groups.lock().map_err(|e| format!("Lock error: {}", e))?;
                groups.insert(group.id.clone(), group.clone());
            },
            GroupChatMode::Pubsub => {
                let mut groups = self.pubsub_groups.lock().map_err(|e| format!("Lock error: {}", e))?;
                groups.insert(group.id.clone(), group.clone());
            },
            GroupChatMode::Iroh => {
                let mut groups = self.iroh_groups.lock().map_err(|e| format!("Lock error: {}", e))?;
                groups.insert(group.id.clone(), group.clone());
            },
        }

        let _ = self.add_system_message(&group.id, format!("{} joined", identity.name), MessageType::Join, mode);
        Ok(group)
    }

    pub fn leave_group(&self, group_id: &str) -> Result<(), String> {
        let (group, mode) = self.find_group(group_id)?;
        
        if let Some(identity) = self.get_local_identity() {
            let _ = self.add_system_message(&group.id, format!("{} left", identity.name), MessageType::Leave, mode);
            let mut updated_group = group.clone();
            updated_group.members.retain(|m| m.id != identity.id);

            match mode {
                GroupChatMode::Memory => {
                    let mut groups = self.memory_groups.lock().map_err(|e| format!("Lock error: {}", e))?;
                    groups.insert(updated_group.id.clone(), updated_group);
                },
                GroupChatMode::Pubsub => {
                    let mut groups = self.pubsub_groups.lock().map_err(|e| format!("Lock error: {}", e))?;
                    groups.insert(updated_group.id.clone(), updated_group);
                },
                GroupChatMode::Iroh => {
                    let mut groups = self.iroh_groups.lock().map_err(|e| format!("Lock error: {}", e))?;
                    groups.insert(updated_group.id.clone(), updated_group);
                },
            }
        }
        Ok(())
    }

    pub fn list_groups(&self) -> Result<Vec<UnifiedGroup>, String> {
        let mut all_groups = Vec::new();
        if let Ok(groups) = self.memory_groups.lock() {
            all_groups.extend(groups.values().cloned().collect::<Vec<_>>());
        }
        if let Ok(groups) = self.pubsub_groups.lock() {
            all_groups.extend(groups.values().cloned().collect::<Vec<_>>());
        }
        if let Ok(groups) = self.iroh_groups.lock() {
            all_groups.extend(groups.values().cloned().collect::<Vec<_>>());
        }
        Ok(all_groups)
    }

    pub fn get_group_info(&self, group_id: &str) -> Result<UnifiedGroup, String> {
        self.find_group(group_id).map(|(g, _)| g)
    }

    pub fn send_message(&self, group_id: &str, content: &str, message_type: Option<MessageType>) -> Result<UnifiedMessage, String> {
        let (_group, mode) = self.find_group(group_id)?;
        let sender = self.get_local_identity().unwrap_or(AgentInfo::new("anonymous", "Anonymous"));
        
        let mut message = UnifiedMessage::new(group_id, sender, content);
        if let Some(msg_type) = message_type {
            message.message_type = msg_type;
        }

        match mode {
            GroupChatMode::Memory => {
                let mut messages = self.memory_messages.lock().map_err(|e| format!("Lock error: {}", e))?;
                messages.entry(group_id.to_string()).or_insert_with(Vec::new).push(message.clone());
            },
            GroupChatMode::Pubsub => {
                let mut messages = self.pubsub_messages.lock().map_err(|e| format!("Lock error: {}", e))?;
                messages.entry(group_id.to_string()).or_insert_with(Vec::new).push(message.clone());
            },
            GroupChatMode::Iroh => {
                let mut messages = self.iroh_messages.lock().map_err(|e| format!("Lock error: {}", e))?;
                messages.entry(group_id.to_string()).or_insert_with(Vec::new).push(message.clone());
            },
        }
        Ok(message)
    }

    pub fn get_history(&self, group_id: &str, limit: Option<usize>) -> Result<Vec<UnifiedMessage>, String> {
        let (group, mode) = self.find_group(group_id)?;
        let limit = limit.unwrap_or(50);

        let messages = match mode {
            GroupChatMode::Memory => self.memory_messages.lock(),
            GroupChatMode::Pubsub => self.pubsub_messages.lock(),
            GroupChatMode::Iroh => self.iroh_messages.lock(),
        }.map_err(|e| format!("Lock error: {}", e))?;

        let msgs = messages.get(&group.id).cloned().unwrap_or_default();
        let start = if msgs.len() > limit { msgs.len() - limit } else { 0 };
        Ok(msgs[start..].to_vec())
    }

    pub fn detect_available_modes(&self) -> Vec<GroupChatMode> {
        vec![GroupChatMode::Memory]
    }

    fn find_group(&self, group_id: &str) -> Result<(UnifiedGroup, GroupChatMode), String> {
        if let Ok(groups) = self.memory_groups.lock() {
            if let Some(group) = groups.get(group_id) {
                return Ok((group.clone(), GroupChatMode::Memory));
            }
        }
        if let Ok(groups) = self.pubsub_groups.lock() {
            if let Some(group) = groups.get(group_id) {
                return Ok((group.clone(), GroupChatMode::Pubsub));
            }
        }
        if let Ok(groups) = self.iroh_groups.lock() {
            if let Some(group) = groups.get(group_id) {
                return Ok((group.clone(), GroupChatMode::Iroh));
            }
        }
        Err(format!("Group not found: {}", group_id))
    }

    fn add_system_message(&self, group_id: &str, content: String, msg_type: MessageType, mode: GroupChatMode) -> Result<UnifiedMessage, String> {
        let message = UnifiedMessage {
            id: format!("sys_{}", Uuid::new_v4()),
            group_id: group_id.to_string(),
            message_type: msg_type,
            sender: AgentInfo::new("system", "System"),
            content,
            timestamp: Utc::now().timestamp_millis(),
            metadata: Some([("is_system".to_string(), serde_json::json!(true))].into_iter().collect()),
            delivered: Some(true),
        };

        match mode {
            GroupChatMode::Memory => {
                let mut messages = self.memory_messages.lock().map_err(|e| format!("Lock error: {}", e))?;
                messages.entry(group_id.to_string()).or_insert_with(Vec::new).push(message.clone());
            },
            GroupChatMode::Pubsub => {
                let mut messages = self.pubsub_messages.lock().map_err(|e| format!("Lock error: {}", e))?;
                messages.entry(group_id.to_string()).or_insert_with(Vec::new).push(message.clone());
            },
            GroupChatMode::Iroh => {
                let mut messages = self.iroh_messages.lock().map_err(|e| format!("Lock error: {}", e))?;
                messages.entry(group_id.to_string()).or_insert_with(Vec::new).push(message.clone());
            },
        }
        Ok(message)
    }
}

impl Default for GroupAdapter {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tauri 工具操作
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(tag = "action")]
pub enum GroupAdapterOperation {
    #[serde(rename = "set_local_identity")]
    SetLocalIdentity { id: String, name: String, did: Option<String>, avatar: Option<String> },
    #[serde(rename = "create_group")]
    CreateGroup { name: String, description: Option<String>, members: Option<Vec<AgentInfo>>, mode: Option<GroupChatMode>, topic: Option<String> },
    #[serde(rename = "join_group")]
    JoinGroup { group_id: String },
    #[serde(rename = "leave_group")]
    LeaveGroup { group_id: String },
    #[serde(rename = "list_groups")]
    ListGroups {},
    #[serde(rename = "get_group_info")]
    GetGroupInfo { group_id: String },
    #[serde(rename = "send_message")]
    SendMessage { group_id: String, content: String, message_type: Option<String> },
    #[serde(rename = "get_history")]
    GetHistory { group_id: String, limit: Option<usize> },
    #[serde(rename = "detect_available_modes")]
    DetectAvailableModes {},
}

struct ExecuteResult {
    success: bool,
    data: Option<serde_json::Value>,
    message: String,
    output: Option<String>,
}

#[async_trait::async_trait]
impl ToolExecutor for GroupAdapter {
    fn metadata(&self) -> &ToolMetadata {
        static METADATA: OnceLock<ToolMetadata> = OnceLock::new();
        METADATA.get_or_init(|| ToolMetadata {
            id: "group_adapter".to_string(),
            name: "Group Chat Adapter".to_string(),
            description: "统一群聊适配器".to_string(),
            category: ToolCategory::Communication,
            priority: ToolPriority::Medium,
            status: ToolStatus::Available,
            version: "1.0.0".to_string(),
            author: "Alou Team".to_string(),
            created_at: 1700000000,
            updated_at: 1700000000,
            dependencies: vec![],
            platforms: vec!["windows".to_string(), "macos".to_string(), "linux".to_string()],
            permissions: vec!["network".to_string()],
                tags: vec![],
            })
    }

    async fn execute(&self, args: serde_json::Value, _context: &ExecutionContext) -> Result<ToolResult, ToolError> {
        let start_time = std::time::Instant::now();
        
        match self.execute_impl(args).await {
            Ok(result) => {
                Ok(ToolResult {
                    success: result.success,
                    data: result.data.unwrap_or(serde_json::Value::Null),
                    error: if result.success { None } else { Some(result.message) },
                    execution_time_ms: start_time.elapsed().as_millis() as u64,
                    output: result.output,
                    warnings: vec![],
                    context: None,
                })
            }
            Err(e) => {
                Ok(ToolResult {
                    success: false,
                    data: serde_json::Value::Null,
                    error: Some(e.to_string()),
                    execution_time_ms: start_time.elapsed().as_millis() as u64,
                    output: Some(format!("Error: {}", e)),
                    warnings: vec![],
                    context: None,
                })
            }
        }
    }

    async fn validate_args(&self, args: &serde_json::Value) -> Result<(), ToolError> {
        match serde_json::from_value::<GroupAdapterOperation>(args.clone()) {
            Ok(_) => Ok(()),
            Err(e) => Err(ToolError::InvalidArguments(e.to_string())),
        }
    }

    fn help(&self) -> String {
        "统一群聊适配器工具".to_string()
    }
}

impl GroupAdapter {
    async fn execute_impl(&self, params: serde_json::Value) -> Result<ExecuteResult, String> {
        let op: GroupAdapterOperation = serde_json::from_value(params).map_err(|e| format!("Parse error: {}", e))?;

        match op {
            GroupAdapterOperation::SetLocalIdentity { id, name, did, avatar } => {
                let identity = AgentInfo { id, name, did, avatar, mode: Some("user".to_string()) };
                self.set_local_identity(identity);
                Ok(ExecuteResult { success: true, data: Some(serde_json::json!({"status": "ok"})), message: String::new(), output: Some("Identity set".to_string()) })
            },
            GroupAdapterOperation::CreateGroup { name, description, members, mode, topic } => {
                let config = GroupChatConfig { name, description, members, mode, topic };
                match self.create_group(config) {
                    Ok(group) => Ok(ExecuteResult { success: true, data: Some(serde_json::to_value(group).unwrap()), message: String::new(), output: Some("Created".to_string()) }),
                    Err(e) => Err(e),
                }
            },
            GroupAdapterOperation::JoinGroup { group_id } => match self.join_group(&group_id) {
                Ok(group) => Ok(ExecuteResult { success: true, data: Some(serde_json::to_value(group).unwrap()), message: String::new(), output: Some("Joined".to_string()) }),
                Err(e) => Err(e),
            },
            GroupAdapterOperation::LeaveGroup { group_id } => match self.leave_group(&group_id) {
                Ok(_) =>                 Ok(ExecuteResult { success: true, data: Some(serde_json::json!({"status": "ok"})), message: String::new(), output: Some("Left".to_string()) }),
                Err(e) => Err(e),
            },
            GroupAdapterOperation::ListGroups {} => match self.list_groups() {
                Ok(groups) => Ok(ExecuteResult { success: true, data: Some(serde_json::to_value(groups).unwrap()), message: String::new(), output: Some("Listed".to_string()) }),
                Err(e) => Err(e),
            },
            GroupAdapterOperation::GetGroupInfo { group_id } => match self.get_group_info(&group_id) {
                Ok(group) => Ok(ExecuteResult { success: true, data: Some(serde_json::to_value(group).unwrap()), message: String::new(), output: Some("Got info".to_string()) }),
                Err(e) => Err(e),
            },
            GroupAdapterOperation::SendMessage { group_id, content, message_type } => {
                let msg_type = message_type.as_ref().map(|s| MessageType::from(s.as_str()));
                match self.send_message(&group_id, &content, msg_type) {
                    Ok(message) => Ok(ExecuteResult { success: true, data: Some(serde_json::to_value(message).unwrap()), message: String::new(), output: Some("Sent".to_string()) }),
                    Err(e) => Err(e),
                }
            },
            GroupAdapterOperation::GetHistory { group_id, limit } => match self.get_history(&group_id, limit) {
                Ok(messages) => Ok(ExecuteResult { success: true, data: Some(serde_json::to_value(messages).unwrap()), message: String::new(), output: Some("Got history".to_string()) }),
                Err(e) => Err(e),
            },
            GroupAdapterOperation::DetectAvailableModes {} => {
                let modes = self.detect_available_modes();
                Ok(ExecuteResult { success: true, data: Some(serde_json::to_value(modes).unwrap()), message: String::new(), output: Some("Detected".to_string()) })
            },
        }
    }
}
