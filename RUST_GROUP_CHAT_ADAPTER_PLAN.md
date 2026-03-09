# Rust 群聊适配器实现方案

## 架构设计

将群聊适配器的核心逻辑移到 Rust 后端，前端只做薄薄的调用层。

```
┌─────────────────────────────────────────────────────────┐
│  前端 (TypeScript/React)                                │
│  ├── UI 组件                                            │
│  ├── Hooks (useUnifiedGroupChat)                       │
│  └── adapters/ (薄适配层，只负责 invoke 调用)            │
└─────────────────────────────────────────────────────────┘
                         │ invoke('group_chat_adapter::*')
                         ▼
┌─────────────────────────────────────────────────────────┐
│  Rust 后端 (Tauri Commands)                             │
│  ├── commands/group_chat_adapter.rs (统一入口)          │
│  ├── adapters/trait.rs (Adapter Trait 定义)            │
│  ├── adapters/memory.rs (Memory 适配器)                │
│  ├── adapters/pubsub.rs (PubSub 适配器)                │
│  └── adapters/iroh.rs (Iroh 适配器)                    │
└─────────────────────────────────────────────────────────┘
```

## 优势

1. **核心逻辑在 Rust** - 类型安全、性能更好、易于测试
2. **前端轻量化** - 只负责 UI 和调用转发
3. **代码复用** - Rust 逻辑可被 CLI 和 Desktop 共用
4. **统一错误处理** - 所有错误在 Rust 层标准化

## Rust 实现

### 1. 定义 Adapter Trait

**文件**: `alou-desktop/src-tauri/src/adapters/trait.rs`

```rust
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 群聊模式枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum GroupChatMode {
    Memory,
    PubSub,
    Iroh,
}

/// 统一群聊结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedGroup {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub mode: GroupChatMode,
    pub members: Vec<String>,
    pub created_at: u64,
    pub created_by: Option<String>,
    pub topic: Option<String>,
    pub ticket: Option<String>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// 统一消息结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedMessage {
    pub id: String,
    pub group_id: String,
    pub sender: String,
    pub sender_name: String,
    pub content: String,
    #[serde(rename = "type")]
    pub message_type: String,
    pub timestamp: u64,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// 群聊配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupChatConfig {
    pub name: String,
    pub description: Option<String>,
    pub members: Option<Vec<String>>,
}

/// 群聊适配器 Trait
/// 所有适配器必须实现此接口
#[async_trait]
pub trait GroupChatAdapter: Send + Sync {
    /// 获取适配器模式
    fn mode(&self) -> GroupChatMode;
    
    /// 检查适配器是否可用
    async fn is_available(&self) -> bool;
    
    // ========== 群聊管理 ==========
    
    /// 创建群聊
    async fn create_group(&self, config: GroupChatConfig) -> Result<UnifiedGroup, AdapterError>;
    
    /// 加入群聊
    async fn join_group(&self, group_id: String) -> Result<UnifiedGroup, AdapterError>;
    
    /// 离开群聊
    async fn leave_group(&self, group_id: String) -> Result<(), AdapterError>;
    
    /// 列出所有群聊
    async fn list_groups(&self) -> Result<Vec<UnifiedGroup>, AdapterError>;
    
    /// 获取群聊详情
    async fn get_group_info(&self, group_id: String) -> Result<Option<UnifiedGroup>, AdapterError>;
    
    // ========== 消息通信 ==========
    
    /// 发送消息
    async fn send_message(
        &self,
        group_id: String,
        content: String,
        message_type: Option<String>,
    ) -> Result<UnifiedMessage, AdapterError>;
    
    /// 获取历史消息
    async fn get_history(
        &self,
        group_id: String,
        limit: usize,
    ) -> Result<Vec<UnifiedMessage>, AdapterError>;
    
    /// 订阅群聊消息（返回消息流）
    /// 注意：实际实现中可能需要使用 channel 或回调
    async fn subscribe(
        &self,
        group_id: String,
    ) -> Result<tokio::sync::mpsc::Receiver<UnifiedMessage>, AdapterError>;
}

/// 适配器错误类型
#[derive(Debug, thiserror::Error)]
pub enum AdapterError {
    #[error("群聊不存在：{0}")]
    GroupNotFound(String),
    
    #[error("消息发送失败：{0}")]
    MessageSendFailed(String),
    
    #[error("网络错误：{0}")]
    NetworkError(String),
    
    #[error("IPFS 错误：{0}")]
    IpfsError(String),
    
    #[error("Iroh 错误：{0}")]
    IrohError(String),
    
    #[error("内部错误：{0}")]
    InternalError(String),
    
    #[error("无效参数：{0}")]
    InvalidParams(String),
}

impl serde::Serialize for AdapterError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

/// 适配器工厂
pub struct AdapterFactory {
    adapters: HashMap<GroupChatMode, Box<dyn GroupChatAdapter>>,
}

impl AdapterFactory {
    pub fn new() -> Self {
        let mut factory = Self {
            adapters: HashMap::new(),
        };
        
        // 注册所有适配器
        factory.register(GroupChatMode::Memory, Box::new(MemoryAdapter::new()));
        factory.register(GroupChatMode::PubSub, Box::new(PubSubAdapter::new()));
        factory.register(GroupChatMode::Iroh, Box::new(IrohAdapter::new()));
        
        factory
    }
    
    fn register(&mut self, mode: GroupChatMode, adapter: Box<dyn GroupChatAdapter>) {
        self.adapters.insert(mode, adapter);
    }
    
    pub fn get_adapter(&self, mode: &GroupChatMode) -> Option<&dyn GroupChatAdapter> {
        self.adapters.get(mode).map(|a| a.as_ref())
    }
    
    pub async fn get_best_adapter(&self) -> Option<&dyn GroupChatAdapter> {
        // 优先级：Iroh > PubSub > Memory
        for mode in [&GroupChatMode::Iroh, &GroupChatMode::PubSub, &GroupChatMode::Memory] {
            if let Some(adapter) = self.adapters.get(mode) {
                if adapter.is_available().await {
                    return Some(adapter.as_ref());
                }
            }
        }
        None
    }
    
    pub async fn detect_available_adapters(&self) -> Vec<GroupChatMode> {
        let mut available = Vec::new();
        
        for (mode, adapter) in &self.adapters {
            if adapter.is_available().await {
                available.push(mode.clone());
            }
        }
        
        available
    }
}

impl Default for AdapterFactory {
    fn default() -> Self {
        Self::new()
    }
}
```

### 2. Memory 适配器实现

**文件**: `alou-desktop/src-tauri/src/adapters/memory.rs`

```rust
use super::trait::*;
use async_trait::async_trait;
use std::collections::HashMap;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;

/// Memory 适配器 - 纯内存实现
pub struct MemoryAdapter {
    groups: RwLock<HashMap<String, UnifiedGroup>>,
    messages: RwLock<HashMap<String, Vec<UnifiedMessage>>>,
    subscribers: RwLock<HashMap<String, Vec<mpsc::Sender<UnifiedMessage>>>>,
}

impl MemoryAdapter {
    pub fn new() -> Self {
        Self {
            groups: RwLock::new(HashMap::new()),
            messages: RwLock::new(HashMap::new()),
            subscribers: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl GroupChatAdapter for MemoryAdapter {
    fn mode(&self) -> GroupChatMode {
        GroupChatMode::Memory
    }
    
    async fn is_available(&self) -> bool {
        true // Memory 始终可用
    }
    
    async fn create_group(&self, config: GroupChatConfig) -> Result<UnifiedGroup, AdapterError> {
        let group = UnifiedGroup {
            id: format!("memory_group_{}_{}", Uuid::new_v4(), chrono::Utc::now().timestamp()),
            name: config.name,
            description: config.description,
            mode: GroupChatMode::Memory,
            members: config.members.unwrap_or_default(),
            created_at: chrono::Utc::now().timestamp() as u64,
            created_by: None,
            topic: None,
            ticket: None,
            metadata: None,
        };
        
        self.groups.write().await.insert(group.id.clone(), group.clone());
        self.messages.write().await.insert(group.id.clone(), Vec::new());
        
        Ok(group)
    }
    
    async fn join_group(&self, group_id: String) -> Result<UnifiedGroup, AdapterError> {
        let groups = self.groups.read().await;
        groups.get(&group_id)
            .cloned()
            .ok_or_else(|| AdapterError::GroupNotFound(group_id.clone()))
    }
    
    async fn leave_group(&self, group_id: String) -> Result<(), AdapterError> {
        self.groups.write().await.remove(&group_id);
        self.messages.write().await.remove(&group_id);
        Ok(())
    }
    
    async fn list_groups(&self) -> Result<Vec<UnifiedGroup>, AdapterError> {
        let groups = self.groups.read().await;
        Ok(groups.values().cloned().collect())
    }
    
    async fn get_group_info(&self, group_id: String) -> Result<Option<UnifiedGroup>, AdapterError> {
        let groups = self.groups.read().await;
        Ok(groups.get(&group_id).cloned())
    }
    
    async fn send_message(
        &self,
        group_id: String,
        content: String,
        message_type: Option<String>,
    ) -> Result<UnifiedMessage, AdapterError> {
        // 检查群聊是否存在
        {
            let groups = self.groups.read().await;
            if !groups.contains_key(&group_id) {
                return Err(AdapterError::GroupNotFound(group_id));
            }
        }
        
        let message = UnifiedMessage {
            id: format!("memory_msg_{}_{}", Uuid::new_v4(), chrono::Utc::now().timestamp()),
            group_id: group_id.clone(),
            sender: "user".to_string(),
            sender_name: "User".to_string(),
            content,
            message_type: message_type.unwrap_or_else(|| "chat".to_string()),
            timestamp: chrono::Utc::now().timestamp() as u64,
            metadata: None,
        };
        
        // 保存消息
        {
            let mut messages = self.messages.write().await;
            if let Some(msgs) = messages.get_mut(&group_id) {
                msgs.push(message.clone());
            }
        }
        
        // 通知订阅者
        {
            let subscribers = self.subscribers.read().await;
            if let Some(subs) = subscribers.get(&group_id) {
                for tx in subs {
                    let _ = tx.send(message.clone()).await;
                }
            }
        }
        
        Ok(message)
    }
    
    async fn get_history(
        &self,
        group_id: String,
        limit: usize,
    ) -> Result<Vec<UnifiedMessage>, AdapterError> {
        let messages = self.messages.read().await;
        let msgs = messages.get(&group_id)
            .ok_or_else(|| AdapterError::GroupNotFound(group_id))?;
        
        // 返回最新的 limit 条消息
        let start = msgs.len().saturating_sub(limit);
        Ok(msgs[start..].to_vec())
    }
    
    async fn subscribe(
        &self,
        group_id: String,
    ) -> Result<mpsc::Receiver<UnifiedMessage>, AdapterError> {
        let (tx, rx) = mpsc::channel(100);
        
        let mut subscribers = self.subscribers.write().await;
        subscribers.entry(group_id).or_insert_with(Vec::new).push(tx);
        
        Ok(rx)
    }
}

impl Default for MemoryAdapter {
    fn default() -> Self {
        Self::new()
    }
}
```

### 3. PubSub 适配器实现

**文件**: `alou-desktop/src-tauri/src/adapters/pubsub.rs`

```rust
use super::trait::*;
use async_trait::async_trait;
use serde_json::json;

/// PubSub 适配器 - 调用现有 PubSub 工具
pub struct PubSubAdapter {
    // 可以持有 PubSubTool 的引用
}

impl PubSubAdapter {
    pub fn new() -> Self {
        Self {}
    }
    
    async fn invoke_pubsub_tool(
        &self,
        action: &str,
        args: serde_json::Value,
    ) -> Result<serde_json::Value, AdapterError> {
        // 调用现有的 PubSubTool
        // 这里需要访问 tool registry
        // 简化示例：
        Err(AdapterError::InternalError("PubSub tool not available".into()))
    }
}

#[async_trait]
impl GroupChatAdapter for PubSubAdapter {
    fn mode(&self) -> GroupChatMode {
        GroupChatMode::PubSub
    }
    
    async fn is_available(&self) -> bool {
        // 检查 IPFS 是否可用
        // 可以调用 get_ipfs_info command
        false // 简化示例
    }
    
    async fn create_group(&self, config: GroupChatConfig) -> Result<UnifiedGroup, AdapterError> {
        let result = self.invoke_pubsub_tool(
            "create_group",
            json!({
                "group_name": config.name,
                "description": config.description,
                "members": config.members,
            }),
        ).await?;
        
        // 解析结果为 UnifiedGroup
        serde_json::from_value(result)
            .map_err(|e| AdapterError::InternalError(e.to_string()))
    }
    
    async fn join_group(&self, group_id: String) -> Result<UnifiedGroup, AdapterError> {
        let result = self.invoke_pubsub_tool(
            "join_group",
            json!({ "group_id": group_id }),
        ).await?;
        
        serde_json::from_value(result)
            .map_err(|e| AdapterError::InternalError(e.to_string()))
    }
    
    async fn leave_group(&self, group_id: String) -> Result<(), AdapterError> {
        self.invoke_pubsub_tool(
            "leave_group",
            json!({ "group_id": group_id }),
        ).await?;
        Ok(())
    }
    
    async fn list_groups(&self) -> Result<Vec<UnifiedGroup>, AdapterError> {
        let result = self.invoke_pubsub_tool("list_groups", json!({})).await?;
        
        serde_json::from_value(result)
            .map_err(|e| AdapterError::InternalError(e.to_string()))
    }
    
    async fn get_group_info(&self, group_id: String) -> Result<Option<UnifiedGroup>, AdapterError> {
        let result = self.invoke_pubsub_tool(
            "get_group_info",
            json!({ "group_id": group_id }),
        ).await?;
        
        serde_json::from_value(result)
            .map_err(|e| AdapterError::InternalError(e.to_string()))
    }
    
    async fn send_message(
        &self,
        group_id: String,
        content: String,
        message_type: Option<String>,
    ) -> Result<UnifiedMessage, AdapterError> {
        let result = self.invoke_pubsub_tool(
            "send_group_message",
            json!({
                "group_id": group_id,
                "message": content,
            }),
        ).await?;
        
        serde_json::from_value(result)
            .map_err(|e| AdapterError::InternalError(e.to_string()))
    }
    
    async fn get_history(
        &self,
        group_id: String,
        limit: usize,
    ) -> Result<Vec<UnifiedMessage>, AdapterError> {
        let result = self.invoke_pubsub_tool(
            "get_group_history",
            json!({
                "group_id": group_id,
                "limit": limit,
            }),
        ).await?;
        
        serde_json::from_value(result)
            .map_err(|e| AdapterError::InternalError(e.to_string()))
    }
    
    async fn subscribe(
        &self,
        group_id: String,
    ) -> Result<mpsc::Receiver<UnifiedMessage>, AdapterError> {
        // TODO: 实现真实的 PubSub 订阅
        let (tx, rx) = mpsc::channel(100);
        Ok(rx)
    }
}

impl Default for PubSubAdapter {
    fn default() -> Self {
        Self::new()
    }
}
```

### 4. Iroh 适配器实现

**文件**: `alou-desktop/src-tauri/src/adapters/iroh.rs`

```rust
use super::trait::*;
use async_trait::async_trait;
use serde_json::json;

/// Iroh 适配器 - 调用现有 Iroh 工具
pub struct IrohAdapter {
    // 可以持有 IrohTool 的引用
}

impl IrohAdapter {
    pub fn new() -> Self {
        Self {}
    }
    
    async fn invoke_iroh_tool(
        &self,
        action: &str,
        args: serde_json::Value,
    ) -> Result<serde_json::Value, AdapterError> {
        // 调用现有的 IrohTool
        Err(AdapterError::InternalError("Iroh tool not available".into()))
    }
}

#[async_trait]
impl GroupChatAdapter for IrohAdapter {
    fn mode(&self) -> GroupChatMode {
        GroupChatMode::Iroh
    }
    
    async fn is_available(&self) -> bool {
        // 检查 Iroh 是否可用
        // 调用 get_node_id 并检查返回值
        false // 简化示例，当前是 Mock 实现
    }
    
    async fn create_group(&self, config: GroupChatConfig) -> Result<UnifiedGroup, AdapterError> {
        let result = self.invoke_iroh_tool(
            "create_group",
            json!({
                "group_name": config.name,
                "description": config.description,
                "members": config.members,
            }),
        ).await?;
        
        serde_json::from_value(result)
            .map_err(|e| AdapterError::InternalError(e.to_string()))
    }
    
    async fn join_group(&self, group_id: String) -> Result<UnifiedGroup, AdapterError> {
        let result = self.invoke_iroh_tool(
            "join_group",
            json!({ "group_id": group_id }),
        ).await?;
        
        serde_json::from_value(result)
            .map_err(|e| AdapterError::InternalError(e.to_string()))
    }
    
    async fn leave_group(&self, group_id: String) -> Result<(), AdapterError> {
        self.invoke_iroh_tool(
            "leave_group",
            json!({ "group_id": group_id }),
        ).await?;
        Ok(())
    }
    
    async fn list_groups(&self) -> Result<Vec<UnifiedGroup>, AdapterError> {
        let result = self.invoke_iroh_tool("list_groups", json!({})).await?;
        
        serde_json::from_value(result)
            .map_err(|e| AdapterError::InternalError(e.to_string()))
    }
    
    async fn get_group_info(&self, group_id: String) -> Result<Option<UnifiedGroup>, AdapterError> {
        let result = self.invoke_iroh_tool(
            "get_group_info",
            json!({ "group_id": group_id }),
        ).await?;
        
        serde_json::from_value(result)
            .map_err(|e| AdapterError::InternalError(e.to_string()))
    }
    
    async fn send_message(
        &self,
        group_id: String,
        content: String,
        message_type: Option<String>,
    ) -> Result<UnifiedMessage, AdapterError> {
        let result = self.invoke_iroh_tool(
            "send_group_message",
            json!({
                "group_id": group_id,
                "message": content,
            }),
        ).await?;
        
        serde_json::from_value(result)
            .map_err(|e| AdapterError::InternalError(e.to_string()))
    }
    
    async fn get_history(
        &self,
        group_id: String,
        limit: usize,
    ) -> Result<Vec<UnifiedMessage>, AdapterError> {
        // TODO: 实现 Iroh 历史消息获取
        Ok(Vec::new())
    }
    
    async fn subscribe(
        &self,
        group_id: String,
    ) -> Result<mpsc::Receiver<UnifiedMessage>, AdapterError> {
        // TODO: 实现真实的 Iroh 订阅
        let (tx, rx) = mpsc::channel(100);
        Ok(rx)
    }
}

impl Default for IrohAdapter {
    fn default() -> Self {
        Self::new()
    }
}
```

### 5. Tauri Commands 统一入口

**文件**: `alou-desktop/src-tauri/src/commands/group_chat_adapter.rs`

```rust
use crate::adapters::trait::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// 全局适配器工厂
pub struct GroupChatAdapterState {
    pub factory: AdapterFactory,
    pub current_mode: RwLock<Option<GroupChatMode>>,
}

impl GroupChatAdapterState {
    pub fn new() -> Self {
        Self {
            factory: AdapterFactory::new(),
            current_mode: RwLock::new(None),
        }
    }
}

impl Default for GroupChatAdapterState {
    fn default() -> Self {
        Self::new()
    }
}

// ========== Command 参数结构 ==========

#[derive(Debug, Deserialize)]
pub struct CreateGroupParams {
    pub name: String,
    pub description: Option<String>,
    pub members: Option<Vec<String>>,
    pub mode: Option<GroupChatMode>,
}

#[derive(Debug, Deserialize)]
pub struct SendMessageParams {
    pub group_id: String,
    pub content: String,
    pub message_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GetHistoryParams {
    pub group_id: String,
    pub limit: Option<usize>,
}

// ========== Command 返回结构 ==========

#[derive(Debug, Serialize)]
pub struct CommandResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> CommandResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }
    
    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
        }
    }
}

// ========== Tauri Commands ==========

#[tauri::command]
pub async fn group_chat_create_group(
    state: tauri::State<'_, Arc<RwLock<GroupChatAdapterState>>>,
    params: CreateGroupParams,
) -> Result<CommandResponse<UnifiedGroup>, String> {
    let state = state.read().await;
    
    // 选择适配器
    let adapter = if let Some(mode) = params.mode {
        state.factory.get_adapter(&mode)
    } else {
        state.factory.get_best_adapter().await
    };
    
    let adapter = adapter.ok_or("No available adapter")?;
    
    let config = GroupChatConfig {
        name: params.name,
        description: params.description,
        members: params.members,
    };
    
    match adapter.create_group(config).await {
        Ok(group) => Ok(CommandResponse::success(group)),
        Err(e) => Ok(CommandResponse::error(e.to_string())),
    }
}

#[tauri::command]
pub async fn group_chat_join_group(
    state: tauri::State<'_, Arc<RwLock<GroupChatAdapterState>>>,
    group_id: String,
) -> Result<CommandResponse<UnifiedGroup>, String> {
    let state = state.read().await;
    
    let adapter = state.factory.get_best_adapter().await
        .ok_or("No available adapter")?;
    
    match adapter.join_group(group_id).await {
        Ok(group) => Ok(CommandResponse::success(group)),
        Err(e) => Ok(CommandResponse::error(e.to_string())),
    }
}

#[tauri::command]
pub async fn group_chat_leave_group(
    state: tauri::State<'_, Arc<RwLock<GroupChatAdapterState>>>,
    group_id: String,
) -> Result<CommandResponse<()>, String> {
    let state = state.read().await;
    
    let adapter = state.factory.get_best_adapter().await
        .ok_or("No available adapter")?;
    
    match adapter.leave_group(group_id).await {
        Ok(_) => Ok(CommandResponse::success(())),
        Err(e) => Ok(CommandResponse::error(e.to_string())),
    }
}

#[tauri::command]
pub async fn group_chat_list_groups(
    state: tauri::State<'_, Arc<RwLock<GroupChatAdapterState>>>,
) -> Result<CommandResponse<Vec<UnifiedGroup>>, String> {
    let state = state.read().await;
    
    let adapter = state.factory.get_best_adapter().await
        .ok_or("No available adapter")?;
    
    match adapter.list_groups().await {
        Ok(groups) => Ok(CommandResponse::success(groups)),
        Err(e) => Ok(CommandResponse::error(e.to_string())),
    }
}

#[tauri::command]
pub async fn group_chat_send_message(
    state: tauri::State<'_, Arc<RwLock<GroupChatAdapterState>>>,
    params: SendMessageParams,
) -> Result<CommandResponse<UnifiedMessage>, String> {
    let state = state.read().await;
    
    let adapter = state.factory.get_best_adapter().await
        .ok_or("No available adapter")?;
    
    match adapter.send_message(
        params.group_id,
        params.content,
        params.message_type,
    ).await {
        Ok(message) => Ok(CommandResponse::success(message)),
        Err(e) => Ok(CommandResponse::error(e.to_string())),
    }
}

#[tauri::command]
pub async fn group_chat_get_history(
    state: tauri::State<'_, Arc<RwLock<GroupChatAdapterState>>>,
    params: GetHistoryParams,
) -> Result<CommandResponse<Vec<UnifiedMessage>>, String> {
    let state = state.read().await;
    
    let adapter = state.factory.get_best_adapter().await
        .ok_or("No available adapter")?;
    
    let limit = params.limit.unwrap_or(50);
    
    match adapter.get_history(params.group_id, limit).await {
        Ok(messages) => Ok(CommandResponse::success(messages)),
        Err(e) => Ok(CommandResponse::error(e.to_string())),
    }
}

#[tauri::command]
pub async fn group_chat_detect_available_adapters(
    state: tauri::State<'_, Arc<RwLock<GroupChatAdapterState>>>,
) -> Result<CommandResponse<Vec<GroupChatMode>>, String> {
    let state = state.read().await;
    
    let available = state.factory.detect_available_adapters().await;
    Ok(CommandResponse::success(available))
}

#[tauri::command]
pub async fn group_chat_get_adapter_status(
    state: tauri::State<'_, Arc<RwLock<GroupChatAdapterState>>>,
) -> Result<CommandResponse<serde_json::Value>, String> {
    let state = state.read().await;
    
    let mut status = serde_json::Map::new();
    
    for mode in &[GroupChatMode::Memory, GroupChatMode::PubSub, GroupChatMode::Iroh] {
        if let Some(adapter) = state.factory.get_adapter(mode) {
            status.insert(
                format!("{:?}", mode).to_lowercase(),
                serde_json::json!(adapter.is_available()),
            );
        }
    }
    
    Ok(CommandResponse::success(serde_json::Value::Object(status)))
}
```

### 6. 注册到 Tauri

**文件**: `alou-desktop/src-tauri/src/main.rs`

```rust
// 添加模块声明
mod adapters {
    pub mod trait;
    pub mod memory;
    pub mod pubsub;
    pub mod iroh;
}

mod commands {
    pub mod group_chat_adapter;
}

use adapters::trait::*;
use commands::group_chat_adapter::*;
use std::sync::Arc;
use tokio::sync::RwLock;

fn main() {
    tauri::Builder::default()
        .manage(Arc::new(RwLock::new(GroupChatAdapterState::new())))
        .invoke_handler(tauri::generate_handler![
            // 群聊适配器 commands
            group_chat_create_group,
            group_chat_join_group,
            group_chat_leave_group,
            group_chat_list_groups,
            group_chat_send_message,
            group_chat_get_history,
            group_chat_detect_available_adapters,
            group_chat_get_adapter_status,
            // ... 其他 commands
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

## 前端调用示例

```typescript
import { invoke } from '@tauri-apps/api/core'

// 创建群聊
const result = await invoke('group_chat_create_group', {
  params: {
    name: '测试群聊',
    description: '这是一个测试',
    mode: 'auto' // 或 'memory', 'pubsub', 'iroh'
  }
})

if (result.success) {
  console.log('群聊创建成功:', result.data)
} else {
  console.error('创建失败:', result.error)
}

// 发送消息
const msgResult = await invoke('group_chat_send_message', {
  params: {
    group_id: 'xxx',
    content: 'Hello',
    message_type: 'chat'
  }
})
```

## 迁移步骤

1. **创建 Rust 模块结构**
   - `src-tauri/src/adapters/trait.rs`
   - `src-tauri/src/adapters/memory.rs`
   - `src-tauri/src/adapters/pubsub.rs`
   - `src-tauri/src/adapters/iroh.rs`
   - `src-tauri/src/commands/group_chat_adapter.rs`

2. **实现 Memory 适配器** (最简单，可独立测试)

3. **实现 PubSub 适配器** (集成现有 pubsub_tool)

4. **实现 Iroh 适配器** (集成现有 iroh_tool)

5. **注册 Tauri Commands**

6. **更新前端** (简化为薄调用层)

7. **测试和清理**

## 总结

这个方案将核心逻辑移到 Rust 后端，前端只保留薄薄的调用层。这样做的好处：

- ✅ 核心逻辑在 Rust，类型安全
- ✅ 前端轻量化，易于维护
- ✅ 代码可被 CLI 和 Desktop 共用
- ✅ 统一的错误处理
- ✅ 更好的测试支持

要不要我帮你实现这个 Rust 方案？
