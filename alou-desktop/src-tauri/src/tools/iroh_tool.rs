use crate::tools::{ToolExecutor, ToolResult, ToolError, ToolCategory};
use crate::tools::ExecutionContext;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
#[serde(tag = "action")]
pub enum IrohOperation {
    /// 创建一个新的文档用于数据共享
    #[serde(rename = "create_doc")]
    CreateDoc {
        /// 可选的文档名称
        name: Option<String>,
    },
    
    /// 打开一个已存在的文档
    #[serde(rename = "open_doc")]
    OpenDoc {
        /// 文档ID
        doc_id: String,
    },
    
    /// 向文档添加键值对
    #[serde(rename = "set")]
    Set {
        /// 文档ID
        doc_id: String,
        /// 键
        key: String,
        /// 值
        value: String,
    },
    
    /// 从文档获取值
    #[serde(rename = "get")]
    Get {
        /// 文档ID
        doc_id: String,
        /// 键
        key: String,
    },
    
    /// 列出文档中的所有条目
    #[serde(rename = "list_entries")]
    ListEntries {
        /// 文档ID
        doc_id: String,
    },
    
    /// 获取当前节点的对等ID
    #[serde(rename = "get_node_id")]
    GetNodeId {},
    
    /// 连接到远程对等节点
    #[serde(rename = "connect_to_node")]
    ConnectToNode {
        /// 对等节点ID
        peer_id: String,
        /// 可选地址
        addr: Option<String>,
    },
    
    /// 共享文档票证以便其他节点加入
    #[serde(rename = "share_doc_ticket")]
    ShareDocTicket {
        /// 文档ID
        doc_id: String,
    },
    
    /// 创建 P2P 群聊（基于 Iroh 文档）
    #[serde(rename = "create_group")]
    CreateGroup {
        /// 群聊名称
        group_name: String,
        /// 群聊描述 (可选)
        description: Option<String>,
        /// 初始成员列表 (可选)
        members: Option<Vec<String>>,
    },
    
    /// 加入 P2P 群聊
    #[serde(rename = "join_group")]
    JoinGroup {
        /// 群聊 ID 或票据
        group_id: String,
    },
    
    /// 离开 P2P 群聊
    #[serde(rename = "leave_group")]
    LeaveGroup {
        /// 群聊 ID
        group_id: String,
    },
    
    /// 发送 P2P 群聊消息
    #[serde(rename = "send_group_message")]
    SendGroupMessage {
        /// 群聊 ID
        group_id: String,
        /// 消息内容
        message: String,
    },
    
    /// 获取 P2P 群聊信息
    #[serde(rename = "get_group_info")]
    GetGroupInfo {
        /// 群聊 ID
        group_id: String,
    },
    
    /// 列出所有 P2P 群聊
    #[serde(rename = "list_groups")]
    ListGroups {},
}

#[derive(Debug, Clone)]
pub struct IrohTool {
    id: String,
    name: String,
    description: String,
    category: ToolCategory,
}

impl IrohTool {
    pub fn new() -> Self {
        Self {
            id: "iroh".to_string(),
            name: "Iroh Tool".to_string(),
            description: "Iroh P2P networking and data synchronization tool".to_string(),
            category: ToolCategory::Communication,
        }
    }
}

#[derive(Serialize, Debug)]
pub struct IrohToolResult {
    pub success: bool,
    pub message: String,
    pub data: Option<Value>,
    pub output: Option<String>,
}

// 模拟 iroh 客户端结构
pub struct IrohClient {
    // 在实际实现中，这里会有一个到 iroh 节点的实际连接
    node_id: String,
    docs: HashMap<String, HashMap<String, String>>,
    groups: HashMap<String, IrohGroupInfo>,
    group_messages: HashMap<String, Vec<IrohGroupMessage>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct IrohGroupInfo {
    group_id: String,
    group_name: String,
    ticket: String,
    description: Option<String>,
    members: Vec<String>,
    created_at: i64,
    created_by: String,
}

#[derive(Clone, Debug, Serialize)]
struct IrohGroupMessage {
    id: String,
    group_id: String,
    sender: String,
    content: String,
    timestamp: i64,
}

impl IrohClient {
    pub fn new() -> Self {
        Self {
            node_id: "fake_node_id_12345".to_string(),
            docs: HashMap::new(),
            groups: HashMap::new(),
            group_messages: HashMap::new(),
        }
    }

    pub async fn create_doc(&mut self, name: Option<String>) -> Result<String, String> {
        // 生成一个模拟的文档ID
        let doc_id = format!("doc_{}", uuid::Uuid::new_v4().to_string()[..8].to_uppercase());
        
        // 如果提供了名称，则可以将其存储为元数据
        let mut doc_data = HashMap::new();
        if let Some(doc_name) = name {
            doc_data.insert("_meta_name".to_string(), doc_name);
        }
        
        self.docs.insert(doc_id.clone(), doc_data);
        
        Ok(doc_id)
    }

    pub async fn open_doc(&mut self, doc_id: &str) -> Result<(), String> {
        if !self.docs.contains_key(doc_id) {
            return Err(format!("Document {} not found", doc_id));
        }
        Ok(())
    }

    pub async fn set(&mut self, doc_id: &str, key: String, value: String) -> Result<(), String> {
        if let Some(doc) = self.docs.get_mut(doc_id) {
            doc.insert(key, value);
            Ok(())
        } else {
            Err(format!("Document {} not found", doc_id))
        }
    }

    pub async fn get(&self, doc_id: &str, key: &str) -> Result<Option<String>, String> {
        if let Some(doc) = self.docs.get(doc_id) {
            Ok(doc.get(key).cloned())
        } else {
            Err(format!("Document {} not found", doc_id))
        }
    }

    pub async fn list_entries(&self, doc_id: &str) -> Result<Vec<(String, String)>, String> {
        if let Some(doc) = self.docs.get(doc_id) {
            let entries: Vec<(String, String)> = doc
                .iter()
                .filter(|(k, _)| !k.starts_with('_')) // 过滤掉元数据
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            Ok(entries)
        } else {
            Err(format!("Document {} not found", doc_id))
        }
    }

    pub fn get_node_id(&self) -> &str {
        &self.node_id
    }

    pub async fn connect_to_node(&self, peer_id: &str, addr: Option<&str>) -> Result<(), String> {
        // 模拟连接过程
        println!("Connecting to peer: {}, addr: {:?}", peer_id, addr);
        Ok(())
    }

    pub async fn share_doc_ticket(&self, doc_id: &str) -> Result<String, String> {
        if !self.docs.contains_key(doc_id) {
            return Err(format!("Document {} not found", doc_id));
        }
        
        // 生成一个模拟的票证
        Ok(format!("ticket_for_{}", doc_id))
    }
    
    // ============ P2P 群聊相关方法 ============
    
    pub async fn create_group(
        &mut self,
        group_name: String,
        description: Option<String>,
        members: Option<Vec<String>>,
    ) -> Result<IrohGroupInfo, String> {
        let group_id = format!("iroh_group_{}", uuid::Uuid::new_v4().to_string()[..8].to_uppercase());
        
        let group_info = IrohGroupInfo {
            group_id: group_id.clone(),
            group_name: group_name.clone(),
            ticket: format!("iroh://{}", group_id),
            description: description.clone(),
            members: members.unwrap_or_default(),
            created_at: chrono::Utc::now().timestamp(),
            created_by: self.node_id.clone(),
        };
        
        // 存储群聊信息
        self.groups.insert(group_id.clone(), group_info.clone());
        
        // 创建对应的文档用于存储群聊消息
        let doc_id = self.create_doc(Some(format!("group_{}", group_id))).await?;
        
        // 初始化群聊消息列表
        self.group_messages.insert(group_id.clone(), Vec::new());
        
        Ok(group_info)
    }
    
    pub async fn join_group(&mut self, group_id: &str) -> Result<IrohGroupInfo, String> {
        // 如果是票据格式，提取 group_id
        let actual_group_id = if group_id.starts_with("iroh://") {
            group_id.strip_prefix("iroh://").unwrap_or(group_id)
        } else {
            group_id
        };
        
        if let Some(group) = self.groups.get(actual_group_id) {
            Ok(group.clone())
        } else {
            Err(format!("Group '{}' not found", group_id))
        }
    }
    
    pub async fn leave_group(&mut self, group_id: &str) -> Result<(), String> {
        if self.groups.contains_key(group_id) {
            self.groups.remove(group_id);
            self.group_messages.remove(group_id);
            Ok(())
        } else {
            Err(format!("Group '{}' not found", group_id))
        }
    }
    
    pub async fn send_group_message(
        &mut self,
        group_id: &str,
        message: String,
    ) -> Result<String, String> {
        // 确保群聊存在
        if !self.groups.contains_key(group_id) {
            return Err(format!("Group '{}' not found", group_id));
        }
        
        let group_msg = IrohGroupMessage {
            id: format!("iroh_msg_{}", uuid::Uuid::new_v4().to_string()[..8].to_uppercase()),
            group_id: group_id.to_string(),
            sender: self.node_id.clone(),
            content: message.clone(),
            timestamp: chrono::Utc::now().timestamp(),
        };
        
        // 添加到群聊消息历史
        self.group_messages.entry(group_id.to_string()).or_insert_with(Vec::new).push(group_msg.clone());
        
        Ok(group_msg.id)
    }
    
    pub async fn get_group_info(&self, group_id: &str) -> Result<IrohGroupInfo, String> {
        self.groups.get(group_id)
            .cloned()
            .ok_or_else(|| format!("Group '{}' not found", group_id))
    }
    
    pub async fn list_groups(&self) -> Vec<IrohGroupInfo> {
        self.groups.values().cloned().collect()
    }
}

impl IrohTool {
    async fn execute_impl(&self, args: Value) -> Result<IrohToolResult, Box<dyn std::error::Error>> {
        let operation: IrohOperation = serde_json::from_value(args)?;

        // 在实际实现中，这里会连接到真实的 iroh 节点
        // 现在我们使用模拟客户端
        let mut client = IrohClient::new();

        match operation {
            IrohOperation::CreateDoc { name } => {
                match client.create_doc(name).await {
                    Ok(doc_id) => {
                        Ok(IrohToolResult {
                            success: true,
                            message: format!("Successfully created document: {}", doc_id),
                            data: Some(serde_json::json!({
                                "doc_id": doc_id
                            })),
                            output: Some(format!("Created new document with ID: {}", doc_id)),
                        })
                    }
                    Err(e) => {
                        Ok(IrohToolResult {
                            success: false,
                            message: format!("Failed to create document: {}", e),
                            data: None,
                            output: Some(format!("Error creating document: {}", e)),
                        })
                    }
                }
            }

            IrohOperation::OpenDoc { doc_id } => {
                match client.open_doc(&doc_id).await {
                    Ok(_) => {
                        Ok(IrohToolResult {
                            success: true,
                            message: format!("Successfully opened document: {}", doc_id),
                            data: Some(serde_json::json!({
                                "doc_id": doc_id
                            })),
                            output: Some(format!("Opened document: {}", doc_id)),
                        })
                    }
                    Err(e) => {
                        Ok(IrohToolResult {
                            success: false,
                            message: format!("Failed to open document: {}", e),
                            data: None,
                            output: Some(format!("Error opening document: {}", e)),
                        })
                    }
                }
            }

            IrohOperation::Set { doc_id, key, value } => {
                match client.set(&doc_id, key.clone(), value.clone()).await {
                    Ok(_) => {
                        Ok(IrohToolResult {
                            success: true,
                            message: format!("Successfully set key '{}' in document '{}'", key, doc_id),
                            data: Some(serde_json::json!({
                                "doc_id": doc_id,
                                "key": key,
                                "value": value
                            })),
                            output: Some(format!("Set key '{}' to value '{}' in document '{}'", key, value, doc_id)),
                        })
                    }
                    Err(e) => {
                        Ok(IrohToolResult {
                            success: false,
                            message: format!("Failed to set key in document: {}", e),
                            data: None,
                            output: Some(format!("Error setting key in document: {}", e)),
                        })
                    }
                }
            }

            IrohOperation::Get { doc_id, key } => {
                match client.get(&doc_id, &key).await {
                    Ok(Some(value)) => {
                        Ok(IrohToolResult {
                            success: true,
                            message: format!("Successfully retrieved value for key '{}' in document '{}'", key, doc_id),
                            data: Some(serde_json::json!({
                                "doc_id": doc_id,
                                "key": key,
                                "value": value
                            })),
                            output: Some(format!("Retrieved value for key '{}': {}", key, value)),
                        })
                    }
                    Ok(None) => {
                        Ok(IrohToolResult {
                            success: true,
                            message: format!("Key '{}' not found in document '{}'", key, doc_id),
                            data: Some(serde_json::json!({
                                "doc_id": doc_id,
                                "key": key,
                                "value": null
                            })),
                            output: Some(format!("Key '{}' not found in document '{}'", key, doc_id)),
                        })
                    }
                    Err(e) => {
                        Ok(IrohToolResult {
                            success: false,
                            message: format!("Failed to get value for key: {}", e),
                            data: None,
                            output: Some(format!("Error getting value for key: {}", e)),
                        })
                    }
                }
            }

            IrohOperation::ListEntries { doc_id } => {
                match client.list_entries(&doc_id).await {
                    Ok(entries) => {
                        let entries_json: Vec<Value> = entries
                            .into_iter()
                            .map(|(k, v)| serde_json::json!({"key": k, "value": v}))
                            .collect();

                        Ok(IrohToolResult {
                            success: true,
                            message: format!("Successfully listed entries in document '{}'", doc_id),
                            data: Some(serde_json::json!({
                                "doc_id": doc_id,
                                "entries": entries_json
                            })),
                            output: Some(format!("Found {} entries in document '{}'", entries_json.len(), doc_id)),
                        })
                    }
                    Err(e) => {
                        Ok(IrohToolResult {
                            success: false,
                            message: format!("Failed to list entries: {}", e),
                            data: None,
                            output: Some(format!("Error listing entries: {}", e)),
                        })
                    }
                }
            }

            IrohOperation::GetNodeId {} => {
                let node_id = client.get_node_id();

                Ok(IrohToolResult {
                    success: true,
                    message: format!("Current node ID: {}", node_id),
                    data: Some(serde_json::json!({
                        "node_id": node_id
                    })),
                    output: Some(format!("Current Iroh node ID: {}", node_id)),
                })
            }

            IrohOperation::ConnectToNode { peer_id, addr } => {
                match client.connect_to_node(&peer_id, addr.as_deref()).await {
                    Ok(_) => {
                        Ok(IrohToolResult {
                            success: true,
                            message: format!("Successfully connected to peer: {}", peer_id),
                            data: Some(serde_json::json!({
                                "peer_id": peer_id,
                                "addr": addr
                            })),
                            output: Some(format!("Connected to peer: {}", peer_id)),
                        })
                    }
                    Err(e) => {
                        Ok(IrohToolResult {
                            success: false,
                            message: format!("Failed to connect to peer: {}", e),
                            data: None,
                            output: Some(format!("Error connecting to peer: {}", e)),
                        })
                    }
                }
            }

            IrohOperation::ShareDocTicket { doc_id } => {
                match client.share_doc_ticket(&doc_id).await {
                    Ok(ticket) => {
                        Ok(IrohToolResult {
                            success: true,
                            message: format!("Successfully generated ticket for document: {}", doc_id),
                            data: Some(serde_json::json!({
                                "doc_id": doc_id,
                                "ticket": ticket
                            })),
                            output: Some(format!("Generated ticket for document '{}': {}", doc_id, ticket)),
                        })
                    }
                    Err(e) => {
                        Ok(IrohToolResult {
                            success: false,
                            message: format!("Failed to generate ticket: {}", e),
                            data: None,
                            output: Some(format!("Error generating ticket: {}", e)),
                        })
                    }
                }
            }
            
            // ============ P2P 群聊操作 ============
            
            IrohOperation::CreateGroup { group_name, description, members } => {
                match client.create_group(group_name.clone(), description.clone(), members.clone()).await {
                    Ok(group) => {
                        Ok(IrohToolResult {
                            success: true,
                            message: format!("Successfully created P2P group: {}", group.group_name),
                            data: Some(serde_json::json!({
                                "group_id": group.group_id,
                                "group_name": group.group_name,
                                "ticket": group.ticket,
                                "description": group.description,
                                "members": group.members,
                                "created_at": group.created_at
                            })),
                            output: Some(format!("Created P2P group '{}' with ticket: {}", group.group_name, group.ticket)),
                        })
                    }
                    Err(e) => {
                        Ok(IrohToolResult {
                            success: false,
                            message: format!("Failed to create group: {}", e),
                            data: None,
                            output: Some(format!("Error creating group: {}", e)),
                        })
                    }
                }
            }
            
            IrohOperation::JoinGroup { group_id } => {
                match client.join_group(&group_id).await {
                    Ok(group) => {
                        Ok(IrohToolResult {
                            success: true,
                            message: format!("Successfully joined P2P group: {}", group.group_name),
                            data: Some(serde_json::json!({
                                "group_id": group.group_id,
                                "group_name": group.group_name,
                                "ticket": group.ticket,
                                "members": group.members
                            })),
                            output: Some(format!("Joined P2P group: {}", group.group_name)),
                        })
                    }
                    Err(e) => {
                        Ok(IrohToolResult {
                            success: false,
                            message: format!("Failed to join group: {}", e),
                            data: None,
                            output: Some(format!("Error joining group: {}", e)),
                        })
                    }
                }
            }
            
            IrohOperation::LeaveGroup { group_id } => {
                match client.leave_group(&group_id).await {
                    Ok(_) => {
                        Ok(IrohToolResult {
                            success: true,
                            message: format!("Successfully left P2P group: {}", group_id),
                            data: Some(serde_json::json!({
                                "group_id": group_id
                            })),
                            output: Some(format!("Left P2P group: {}", group_id)),
                        })
                    }
                    Err(e) => {
                        Ok(IrohToolResult {
                            success: false,
                            message: format!("Failed to leave group: {}", e),
                            data: None,
                            output: Some(format!("Error leaving group: {}", e)),
                        })
                    }
                }
            }
            
            IrohOperation::SendGroupMessage { group_id, message } => {
                match client.send_group_message(&group_id, message.clone()).await {
                    Ok(message_id) => {
                        Ok(IrohToolResult {
                            success: true,
                            message: format!("Successfully sent message to P2P group"),
                            data: Some(serde_json::json!({
                                "group_id": group_id,
                                "message_id": message_id,
                                "message": message
                            })),
                            output: Some(format!("Sent message to P2P group '{}': {}", group_id, message)),
                        })
                    }
                    Err(e) => {
                        Ok(IrohToolResult {
                            success: false,
                            message: format!("Failed to send message: {}", e),
                            data: None,
                            output: Some(format!("Error sending message: {}", e)),
                        })
                    }
                }
            }
            
            IrohOperation::GetGroupInfo { group_id } => {
                match client.get_group_info(&group_id).await {
                    Ok(group) => {
                        Ok(IrohToolResult {
                            success: true,
                            message: format!("Successfully got P2P group info: {}", group.group_name),
                            data: Some(serde_json::json!({
                                "group_id": group.group_id,
                                "group_name": group.group_name,
                                "ticket": group.ticket,
                                "description": group.description,
                                "members": group.members,
                                "created_at": group.created_at,
                                "created_by": group.created_by
                            })),
                            output: Some(format!("P2P group '{}': {} members", group.group_name, group.members.len())),
                        })
                    }
                    Err(e) => {
                        Ok(IrohToolResult {
                            success: false,
                            message: format!("Failed to get group info: {}", e),
                            data: None,
                            output: Some(format!("Error getting group info: {}", e)),
                        })
                    }
                }
            }
            
            IrohOperation::ListGroups {} => {
                let groups = client.list_groups().await;
                let groups_json: Vec<Value> = groups.iter().map(|g| {
                    serde_json::json!({
                        "group_id": g.group_id,
                        "group_name": g.group_name,
                        "ticket": g.ticket,
                        "members": g.members,
                        "created_at": g.created_at
                    })
                }).collect();
                
                Ok(IrohToolResult {
                    success: true,
                    message: format!("Successfully listed {} P2P groups", groups_json.len()),
                    data: Some(serde_json::json!({
                        "groups": groups_json,
                        "count": groups_json.len()
                    })),
                    output: Some(format!("Found {} P2P groups", groups_json.len())),
                })
            }
        }
    }
}

#[async_trait::async_trait]
impl ToolExecutor for IrohTool {
    fn metadata(&self) -> &crate::tools::ToolMetadata {
        // 由于 metadata 返回的是 &ToolMetadata，我们需要静态存储或使用 lazy_static
        // 为了简化，我们暂时返回一个基本的 metadata
        // 在实际实现中，这应该是一个静态或缓存的值
        use std::sync::OnceLock;
        static METADATA: OnceLock<crate::tools::ToolMetadata> = OnceLock::new();

        METADATA.get_or_init(|| crate::tools::ToolMetadata {
            id: "iroh".to_string(),
            name: self.name.clone(),
            description: self.description.clone(),
            category: self.category,
            priority: crate::tools::ToolPriority::Medium,
            status: crate::tools::ToolStatus::Available,
            version: "1.0.0".to_string(),
            author: "Alou Team".to_string(),
            created_at: 1700000000, // 示例时间戳
            updated_at: 1700000000,
            dependencies: vec!["iroh".to_string()],
            platforms: vec!["windows".to_string(), "macos".to_string(), "linux".to_string()],
            permissions: vec!["network".to_string(), "read".to_string(), "write".to_string()],
        })
    }

    async fn execute(&self, args: Value, _context: &ExecutionContext) -> Result<ToolResult, ToolError> {
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
                    output: Some(format!("Error executing Iroh tool: {}", e)),
                    warnings: vec![],
                    context: None,
                })
            }
        }
    }

    async fn validate_args(&self, args: &Value) -> Result<(), ToolError> {
        // 尝试解析操作以验证参数
        match serde_json::from_value::<IrohOperation>(args.clone()) {
            Ok(_) => Ok(()),
            Err(e) => Err(ToolError::InvalidArguments(e.to_string())),
        }
    }

    fn help(&self) -> String {
        "Iroh P2P networking and data synchronization tool. Actions: create_doc, open_doc, set, get, list_entries, get_node_id, connect_to_node, share_doc_ticket, create_group, join_group, leave_group, send_group_message, get_group_info, list_groups".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_create_doc() {
        let tool = IrohTool::new();
        let args = json!({
            "action": "create_doc",
            "name": "test_doc"
        });

        let context = ExecutionContext {
            session_id: "test_session".to_string(),
            user_id: None,
            working_directory: None,
            environment: std::collections::HashMap::new(),
            timeout_seconds: Some(30),
            permissions: vec!["read".to_string(), "write".to_string(), "network".to_string()],
            timestamp: chrono::Utc::now().timestamp(),
        };

        let result = tool.execute(args, &context).await.unwrap();
        assert!(result.success);
        assert!(result.data["doc_id"].is_string());
        assert!(result.output.is_some());
        assert!(result.output.as_ref().unwrap().contains("Created new document"));
    }

    #[tokio::test]
    async fn test_set_and_get() {
        let tool = IrohTool::new();
        let context = ExecutionContext {
            session_id: "test_session".to_string(),
            user_id: None,
            working_directory: None,
            environment: std::collections::HashMap::new(),
            timeout_seconds: Some(30),
            permissions: vec!["read".to_string(), "write".to_string(), "network".to_string()],
            timestamp: chrono::Utc::now().timestamp(),
        };

        // 创建文档
        let create_args = json!({
            "action": "create_doc",
            "name": "test_doc"
        });
        let create_result = tool.execute(create_args, &context).await.unwrap();
        assert!(create_result.success);

        if let Some(doc_id) = create_result.data["doc_id"].as_str() {
            // 设置键值对
            let set_args = json!({
                "action": "set",
                "doc_id": doc_id,
                "key": "test_key",
                "value": "test_value"
            });
            let set_result = tool.execute(set_args, &context).await.unwrap();
            assert!(set_result.success);

            // 获取值
            let get_args = json!({
                "action": "get",
                "doc_id": doc_id,
                "key": "test_key"
            });
            let get_result = tool.execute(get_args, &context).await.unwrap();
            assert!(get_result.success);
            assert_eq!(get_result.data["value"].as_str().unwrap(), "test_value");
        }
    }
}