use crate::tools::{ToolExecutor, ToolResult, ToolError, ToolCategory, ToolMetadata, ToolStatus, ToolPriority, ExecutionContext};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};
use tokio::sync::RwLock;
use bytes::Bytes;
use tokio::sync::Mutex;
use iroh_gossip::api::{GossipSender, GossipReceiver, Event};
use iroh_gossip::TopicId;
use iroh_gossip::net::Gossip;
use iroh_gossip::net::GOSSIP_ALPN;
use iroh::Endpoint;
use iroh::protocol::Router;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
#[serde(tag = "action")]
pub enum IrohOperation {
    #[serde(rename = "create_doc")]
    CreateDoc { name: Option<String> },
    
    #[serde(rename = "open_doc")]
    OpenDoc { doc_id: String },
    
    #[serde(rename = "set")]
    Set { doc_id: String, key: String, value: String },
    
    #[serde(rename = "get")]
    Get { doc_id: String, key: String },
    
    #[serde(rename = "list_entries")]
    ListEntries { doc_id: String },
    
    #[serde(rename = "get_node_id")]
    GetNodeId {},
    
    #[serde(rename = "connect_to_node")]
    ConnectToNode { peer_id: String, addr: Option<String> },
    
    #[serde(rename = "share_doc_ticket")]
    ShareDocTicket { doc_id: String },
    
    #[serde(rename = "create_group")]
    CreateGroup {
        group_name: String,
        description: Option<String>,
        members: Option<Vec<String>>,
    },
    
    #[serde(rename = "join_group")]
    JoinGroup { group_id: String },
    
    #[serde(rename = "leave_group")]
    LeaveGroup { group_id: String },
    
    #[serde(rename = "send_group_message")]
    SendGroupMessage { group_id: String, message: String },
    
    #[serde(rename = "get_group_info")]
    GetGroupInfo { group_id: String },
    
    #[serde(rename = "list_groups")]
    ListGroups {},
    
    #[serde(rename = "get_group_messages")]
    GetGroupMessages { group_id: String, limit: Option<usize> },
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IrohGroupInfo {
    pub group_id: String,
    pub group_name: String,
    pub ticket: String,
    pub description: Option<String>,
    pub members: Vec<String>,
    pub created_at: i64,
    pub created_by: String,
    pub topic_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IrohGroupMessage {
    pub id: String,
    pub group_id: String,
    pub sender: String,
    pub sender_name: Option<String>,
    pub content: String,
    pub timestamp: i64,
}

pub struct IrohClient {
    node_id: String,
    endpoint: Option<Endpoint>,
    gossip: Option<Gossip>,
    router: Option<Router>,
    docs: HashMap<String, HashMap<String, String>>,
    groups: HashMap<String, IrohGroupInfo>,
    group_messages: HashMap<String, Vec<IrohGroupMessage>>,
    group_senders: HashMap<String, GossipSender>,
    group_receivers: HashMap<String, Arc<Mutex<GossipReceiver>>>,
    data_dir: Option<PathBuf>,
}

impl IrohClient {
    pub fn new() -> Self {
        let node_id = format!("iroh_node_{}", &uuid::Uuid::new_v4().to_string()[..8]);
        Self {
            node_id,
            endpoint: None,
            gossip: None,
            router: None,
            docs: HashMap::new(),
            groups: HashMap::new(),
            group_messages: HashMap::new(),
            group_senders: HashMap::new(),
            group_receivers: HashMap::new(),
            data_dir: None,
        }
    }

    pub fn with_data_dir(mut self, data_dir: PathBuf) -> Self {
        self.data_dir = Some(data_dir);
        self
    }
    
    pub async fn init(&mut self) -> Result<(), String> {
        if self.endpoint.is_some() {
            return Ok(());
        }

        let endpoint = Endpoint::builder()
            .bind()
            .await
            .map_err(|e| format!("Failed to create endpoint: {}", e))?;

        let node_id = endpoint.id().to_string();
        log::info!("Iroh endpoint initialized with node_id: {}", node_id);
        self.node_id = node_id;

        let gossip = Gossip::builder()
            .spawn(endpoint.clone());

        let router = Router::builder(endpoint.clone())
            .accept(GOSSIP_ALPN, gossip.clone())
            .spawn();

        self.endpoint = Some(endpoint);
        self.gossip = Some(gossip);
        self.router = Some(router);

        // Load persisted groups from disk
        if let Err(e) = self.load_groups_from_disk().await {
            log::warn!("Failed to load groups from disk: {}", e);
        }

        Ok(())
    }
    
    fn ensure_initialized(&self) -> Result<(), String> {
        if self.endpoint.is_none() {
            return Err("Iroh not initialized. Call init() first.".to_string());
        }
        Ok(())
    }

    /// Get the data directory for persistence
    fn get_data_dir(&self) -> Option<PathBuf> {
        self.data_dir.clone().or_else(|| {
            // Default to app data directory
            dirs::data_local_dir().map(|p| p.join("alou").join("iroh"))
        })
    }

    /// Save groups to disk
    async fn save_groups_to_disk(&self) -> Result<(), String> {
        let data_dir = match self.get_data_dir() {
            Some(dir) => dir,
            None => return Ok(()), // No data dir, skip persistence
        };

        // Create directory if it doesn't exist
        if let Err(e) = std::fs::create_dir_all(&data_dir) {
            log::warn!("Failed to create data directory: {}", e);
            return Ok(());
        }

        let groups_file = data_dir.join("iroh_groups.json");
        let messages_file = data_dir.join("iroh_messages.json");

        // Save groups
        let groups_json = serde_json::to_string_pretty(&self.groups).unwrap_or_default();
        if let Err(e) = tokio::fs::write(&groups_file, &groups_json).await {
            log::warn!("Failed to save groups: {}", e);
        }

        // Save messages
        let messages_json = serde_json::to_string_pretty(&self.group_messages).unwrap_or_default();
        if let Err(e) = tokio::fs::write(&messages_file, &messages_json).await {
            log::warn!("Failed to save messages: {}", e);
        }

        log::info!("Saved {} groups to disk", self.groups.len());
        Ok(())
    }

    /// Load groups from disk
    async fn load_groups_from_disk(&mut self) -> Result<(), String> {
        let data_dir = match self.get_data_dir() {
            Some(dir) => dir,
            None => return Ok(()), // No data dir, skip persistence
        };

        let groups_file = data_dir.join("iroh_groups.json");
        let messages_file = data_dir.join("iroh_messages.json");

        // Load groups
        if groups_file.exists() {
            match tokio::fs::read_to_string(&groups_file).await {
                Ok(content) => {
                    match serde_json::from_str(&content) {
                        Ok(loaded_groups) => {
                            self.groups = loaded_groups;
                            log::info!("Loaded {} groups from disk", self.groups.len());
                        }
                        Err(e) => log::warn!("Failed to parse groups file: {}", e),
                    }
                }
                Err(e) => log::warn!("Failed to read groups file: {}", e),
            }
        }

        // Load messages
        if messages_file.exists() {
            match tokio::fs::read_to_string(&messages_file).await {
                Ok(content) => {
                    match serde_json::from_str(&content) {
                        Ok(loaded_messages) => {
                            self.group_messages = loaded_messages;
                            log::info!("Loaded messages for {} groups from disk", self.group_messages.len());
                        }
                        Err(e) => log::warn!("Failed to parse messages file: {}", e),
                    }
                }
                Err(e) => log::warn!("Failed to read messages file: {}", e),
            }
        }

        Ok(())
    }

    pub fn get_node_id(&self) -> &str {
        &self.node_id
    }
    
    pub async fn create_doc(&mut self, name: Option<String>) -> Result<String, String> {
        self.ensure_initialized()?;
        
        let doc_id = format!("doc_{}", &uuid::Uuid::new_v4().to_string()[..8].to_uppercase());
        
        let mut doc_data = HashMap::new();
        if let Some(doc_name) = name {
            doc_data.insert("_meta_name".to_string(), doc_name);
        }
        
        self.docs.insert(doc_id.clone(), doc_data);
        
        log::info!("Created doc: {}", doc_id);
        Ok(doc_id)
    }
    
    pub async fn open_doc(&mut self, doc_id: &str) -> Result<(), String> {
        self.ensure_initialized()?;
        
        if !self.docs.contains_key(doc_id) {
            return Err(format!("Document {} not found", doc_id));
        }
        Ok(())
    }
    
    pub async fn set(&mut self, doc_id: &str, key: String, value: String) -> Result<(), String> {
        self.ensure_initialized()?;
        
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
                .filter(|(k, _)| !k.starts_with('_'))
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            Ok(entries)
        } else {
            Err(format!("Document {} not found", doc_id))
        }
    }
    
    pub async fn connect_to_node(&self, peer_id: &str, addr: Option<&str>) -> Result<(), String> {
        self.ensure_initialized()?;
        
        log::info!("Connecting to peer: {}, addr: {:?}", peer_id, addr);
        Ok(())
    }
    
    pub async fn share_doc_ticket(&self, doc_id: &str) -> Result<String, String> {
        if !self.docs.contains_key(doc_id) {
            return Err(format!("Document {} not found", doc_id));
        }
        
        Ok(format!("iroh://doc/{}", doc_id))
    }
    
    fn create_topic_id(group_id: &str) -> TopicId {
        let hash = blake3::hash(group_id.as_bytes());
        let bytes: [u8; 32] = hash.as_bytes()[..32].try_into().unwrap();
        TopicId::from_bytes(bytes)
    }
    
    pub async fn create_group(
        &mut self,
        group_name: String,
        description: Option<String>,
        _members: Option<Vec<String>>,
    ) -> Result<IrohGroupInfo, String> {
        self.ensure_initialized()?;

        let gossip = self.gossip.as_ref().ok_or("Gossip not initialized")?;

        let group_id = format!("iroh_group_{}", &uuid::Uuid::new_v4().to_string()[..8].to_uppercase());
        let topic_id = Self::create_topic_id(&group_id);

        let topic = gossip.subscribe(topic_id, vec![])
            .await
            .map_err(|e| format!("Failed to subscribe to topic: {}", e))?;

        let (sender, receiver) = topic.split();

        self.group_senders.insert(group_id.clone(), sender);
        self.group_receivers.insert(group_id.clone(), Arc::new(Mutex::new(receiver)));

        let topic_id_str = hex::encode(topic_id.as_bytes());

        let group_info = IrohGroupInfo {
            group_id: group_id.clone(),
            group_name: group_name.clone(),
            ticket: format!("iroh://{}", group_id),
            description: description.clone(),
            members: vec![self.node_id.clone()],
            created_at: chrono::Utc::now().timestamp(),
            created_by: self.node_id.clone(),
            topic_id: topic_id_str.clone(),
        };

        self.groups.insert(group_id.clone(), group_info.clone());

        let messages = self.group_messages.entry(group_id.clone()).or_insert_with(Vec::new);
        let sys_msg = IrohGroupMessage {
            id: format!("sys_{}", &uuid::Uuid::new_v4().to_string()[..8]),
            group_id: group_id.clone(),
            sender: self.node_id.clone(),
            sender_name: Some("System".to_string()),
            content: format!("群聊 \"{}\" 已创建", group_name),
            timestamp: chrono::Utc::now().timestamp(),
        };
        messages.push(sys_msg);

        // Save to disk for persistence
        if let Err(e) = self.save_groups_to_disk().await {
            log::warn!("Failed to save group to disk: {}", e);
        }

        let group_id_clone = group_id.clone();
        let receiver = self.group_receivers.get(&group_id[..]).cloned().unwrap();
        tokio::spawn(async move {
            let mut receiver_guard = receiver.lock().await;
            while let Some(event_result) = futures_lite::StreamExt::next(&mut *receiver_guard).await {
                match event_result {
                    Ok(event) => {
                        match event {
                            Event::Received(msg) => {
                                log::debug!("Received message in group {}: {:?}", group_id_clone, msg.content);
                            }
                            Event::NeighborUp(peer) => {
                                log::debug!("Peer joined group {}: {:?}", group_id_clone, peer);
                            }
                            Event::NeighborDown(peer) => {
                                log::debug!("Peer left group {}: {:?}", group_id_clone, peer);
                            }
                            Event::Lagged => {
                                log::warn!("Lagged behind in group {}", group_id_clone);
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Gossip event error in group {}: {}", group_id_clone, e);
                        break;
                    }
                }
            }
        });

        log::info!("Created Iroh group: {} with topic: {}", group_id, topic_id_str);

        Ok(group_info)
    }
    
    pub async fn join_group(&mut self, group_id_or_ticket: &str) -> Result<IrohGroupInfo, String> {
        self.ensure_initialized()?;

        let gossip = self.gossip.as_ref().ok_or("Gossip not initialized")?;

        let group_id = if group_id_or_ticket.starts_with("iroh://") {
            group_id_or_ticket.strip_prefix("iroh://").unwrap_or(group_id_or_ticket)
        } else {
            group_id_or_ticket
        };

        if let Some(group) = self.groups.get(group_id) {
            return Ok(group.clone());
        }

        let topic_id = Self::create_topic_id(group_id);

        let topic = gossip.subscribe(topic_id, vec![])
            .await
            .map_err(|e| format!("Failed to subscribe to topic: {}", e))?;

        let (sender, receiver) = topic.split();

        self.group_senders.insert(group_id.to_string(), sender);
        self.group_receivers.insert(group_id.to_string(), Arc::new(Mutex::new(receiver)));

        let topic_id_str = hex::encode(topic_id.as_bytes());

        let group_info = IrohGroupInfo {
            group_id: group_id.to_string(),
            group_name: format!("Group {}", group_id),
            ticket: format!("iroh://{}", group_id),
            description: None,
            members: vec![self.node_id.clone()],
            created_at: chrono::Utc::now().timestamp(),
            created_by: self.node_id.clone(),
            topic_id: topic_id_str,
        };

        self.groups.insert(group_id.to_string(), group_info.clone());

        let messages = self.group_messages.entry(group_id.to_string()).or_insert_with(Vec::new);
        let join_msg = IrohGroupMessage {
            id: format!("sys_{}", &uuid::Uuid::new_v4().to_string()[..8]),
            group_id: group_id.to_string(),
            sender: self.node_id.clone(),
            sender_name: Some("System".to_string()),
            content: "新成员已加入群聊".to_string(),
            timestamp: chrono::Utc::now().timestamp(),
        };
        messages.push(join_msg);

        // Save to disk for persistence
        if let Err(e) = self.save_groups_to_disk().await {
            log::warn!("Failed to save group to disk: {}", e);
        }

        let group_id_clone = group_id.to_string();
        let receiver = self.group_receivers.get(&group_id[..]).cloned().unwrap();
        tokio::spawn(async move {
            let mut receiver_guard = receiver.lock().await;
            while let Some(event_result) = futures_lite::StreamExt::next(&mut *receiver_guard).await {
                match event_result {
                    Ok(event) => {
                        match event {
                            Event::Received(msg) => {
                                log::debug!("Received message in group {}: {:?}", group_id_clone, msg.content);
                            }
                            Event::NeighborUp(peer) => {
                                log::debug!("Peer joined group {}: {:?}", group_id_clone, peer);
                            }
                            Event::NeighborDown(peer) => {
                                log::debug!("Peer left group {}: {:?}", group_id_clone, peer);
                            }
                            Event::Lagged => {
                                log::warn!("Lagged behind in group {}", group_id_clone);
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Gossip event error in group {}: {}", group_id_clone, e);
                        break;
                    }
                }
            }
        });

        log::info!("Joined Iroh group: {}", group_id);

        Ok(group_info)
    }
    
    pub async fn leave_group(&mut self, group_id: &str) -> Result<(), String> {
        self.ensure_initialized()?;

        if self.groups.contains_key(group_id) {
            self.groups.remove(group_id);
            self.group_messages.remove(group_id);
            self.group_senders.remove(group_id);
            self.group_receivers.remove(group_id);
            log::info!("Left Iroh group: {}", group_id);
            
            // Save to disk after removing group
            if let Err(e) = self.save_groups_to_disk().await {
                log::warn!("Failed to save groups to disk: {}", e);
            }
            
            Ok(())
        } else {
            Err(format!("Group '{}' not found", group_id))
        }
    }
    
    pub async fn send_group_message(
        &mut self,
        group_id: &str,
        message: String,
        _sender_name: Option<String>,
    ) -> Result<String, String> {
        self.ensure_initialized()?;
        
        if !self.groups.contains_key(group_id) {
            return Err(format!("Group '{}' not found", group_id));
        }
        
        let sender = self.group_senders.get(group_id)
            .ok_or("No sender for group")?;
        
        let msg = IrohGroupMessage {
            id: format!("msg_{}", &uuid::Uuid::new_v4().to_string()[..8].to_uppercase()),
            group_id: group_id.to_string(),
            sender: self.node_id.clone(),
            sender_name: None,
            content: message.clone(),
            timestamp: chrono::Utc::now().timestamp(),
        };
        
        let msg_bytes = serde_json::to_vec(&msg)
            .map_err(|e| format!("Failed to serialize message: {}", e))?;

        let _ = sender.broadcast(Bytes::from(msg_bytes))
            .await
            .map_err(|e| format!("Failed to broadcast message: {}", e))?;

        self.group_messages.entry(group_id.to_string()).or_insert_with(Vec::new).push(msg.clone());

        // Save messages to disk for persistence
        if let Err(e) = self.save_groups_to_disk().await {
            log::warn!("Failed to save message to disk: {}", e);
        }

        log::debug!("Sent message to group {}: {}", group_id, msg.id);

        Ok(msg.id)
    }
    
    pub async fn get_group_info(&self, group_id: &str) -> Result<IrohGroupInfo, String> {
        self.groups.get(group_id)
            .cloned()
            .ok_or_else(|| format!("Group '{}' not found", group_id))
    }
    
    pub async fn list_groups(&self) -> Vec<IrohGroupInfo> {
        self.groups.values().cloned().collect()
    }
    
    pub async fn get_group_messages(&self, group_id: &str, limit: Option<usize>) -> Result<Vec<IrohGroupMessage>, String> {
        let messages = self.group_messages.get(group_id)
            .ok_or_else(|| format!("Group '{}' not found", group_id))?;
        
        let limit = limit.unwrap_or(50);
        let start = if messages.len() > limit { messages.len() - limit } else { 0 };
        
        Ok(messages[start..].to_vec())
    }
}

impl Default for IrohClient {
    fn default() -> Self {
        Self::new()
    }
}

impl IrohTool {
    async fn execute_impl(&self, args: Value) -> Result<IrohToolResult, Box<dyn std::error::Error>> {
        static CLIENT: std::sync::OnceLock<Arc<RwLock<IrohClient>>> = std::sync::OnceLock::new();
        
        // Initialize client with data directory on first use
        let client = CLIENT.get_or_init(|| {
            let data_dir = dirs::data_local_dir()
                .map(|p| p.join("alou").join("iroh"))
                .unwrap_or_else(|| std::path::PathBuf::from("./alou-iroh"));
            
            let client = IrohClient::new().with_data_dir(data_dir);
            Arc::new(RwLock::new(client))
        });

        let mut client = client.write().await;

        let operation: IrohOperation = serde_json::from_value(args)?;
        
        match operation {
            IrohOperation::GetNodeId {} => {
                let node_id = client.get_node_id();
                Ok(IrohToolResult {
                    success: true,
                    message: format!("Node ID: {}", node_id),
                    data: Some(serde_json::json!({ "node_id": node_id })),
                    output: Some(node_id.to_string()),
                })
            },
            
            IrohOperation::CreateDoc { name } => {
                let doc_id = client.create_doc(name).await?;
                Ok(IrohToolResult {
                    success: true,
                    message: format!("Document created: {}", doc_id),
                    data: Some(serde_json::json!({ "doc_id": doc_id })),
                    output: Some(doc_id),
                })
            },
            
            IrohOperation::OpenDoc { doc_id } => {
                client.open_doc(&doc_id).await?;
                Ok(IrohToolResult {
                    success: true,
                    message: format!("Document opened: {}", doc_id),
                    data: None,
                    output: None,
                })
            },
            
            IrohOperation::Set { doc_id, key, value } => {
                client.set(&doc_id, key, value).await?;
                Ok(IrohToolResult {
                    success: true,
                    message: "Value set successfully".to_string(),
                    data: None,
                    output: None,
                })
            },
            
            IrohOperation::Get { doc_id, key } => {
                let value = client.get(&doc_id, &key).await?;
                Ok(IrohToolResult {
                    success: true,
                    message: format!("Value: {:?}", value),
                    data: Some(serde_json::json!({ "value": value })),
                    output: value,
                })
            },
            
            IrohOperation::ListEntries { doc_id } => {
                let entries = client.list_entries(&doc_id).await?;
                Ok(IrohToolResult {
                    success: true,
                    message: format!("Found {} entries", entries.len()),
                    data: Some(serde_json::json!({ "entries": entries })),
                    output: None,
                })
            },
            
            IrohOperation::ConnectToNode { peer_id, addr } => {
                client.connect_to_node(&peer_id, addr.as_deref()).await?;
                Ok(IrohToolResult {
                    success: true,
                    message: format!("Connected to {}", peer_id),
                    data: None,
                    output: None,
                })
            },
            
            IrohOperation::ShareDocTicket { doc_id } => {
                let ticket = client.share_doc_ticket(&doc_id).await?;
                Ok(IrohToolResult {
                    success: true,
                    message: format!("Ticket: {}", ticket),
                    data: Some(serde_json::json!({ "ticket": ticket })),
                    output: Some(ticket),
                })
            },
            
            IrohOperation::CreateGroup { group_name, description, members } => {
                client.init().await?;
                
                let group = client.create_group(group_name, description, members).await?;
                Ok(IrohToolResult {
                    success: true,
                    message: format!("Group created: {}", group.group_id),
                    data: Some(serde_json::json!({
                        "group_id": group.group_id,
                        "ticket": group.ticket,
                        "topic_id": group.topic_id,
                    })),
                    output: Some(group.ticket),
                })
            },
            
            IrohOperation::JoinGroup { group_id } => {
                client.init().await?;
                
                let group = client.join_group(&group_id).await?;
                Ok(IrohToolResult {
                    success: true,
                    message: format!("Joined group: {}", group.group_id),
                    data: Some(serde_json::json!({
                        "group_id": group.group_id,
                        "ticket": group.ticket,
                        "topic_id": group.topic_id,
                    })),
                    output: Some(group.group_id),
                })
            },
            
            IrohOperation::LeaveGroup { group_id } => {
                client.leave_group(&group_id).await?;
                Ok(IrohToolResult {
                    success: true,
                    message: format!("Left group: {}", group_id),
                    data: None,
                    output: None,
                })
            },
            
            IrohOperation::SendGroupMessage { group_id, message } => {
                let msg_id = client.send_group_message(&group_id, message, None).await?;
                Ok(IrohToolResult {
                    success: true,
                    message: format!("Message sent: {}", msg_id),
                    data: Some(serde_json::json!({ "message_id": msg_id })),
                    output: Some(msg_id),
                })
            },
            
            IrohOperation::GetGroupInfo { group_id } => {
                let group = client.get_group_info(&group_id).await?;
                Ok(IrohToolResult {
                    success: true,
                    message: format!("Group info: {}", group.group_name),
                    data: Some(serde_json::json!(group)),
                    output: None,
                })
            },
            
            IrohOperation::ListGroups {} => {
                let groups = client.list_groups().await;
                Ok(IrohToolResult {
                    success: true,
                    message: format!("{} groups", groups.len()),
                    data: Some(serde_json::json!({ "groups": groups })),
                    output: None,
                })
            },
            
            IrohOperation::GetGroupMessages { group_id, limit } => {
                let messages = client.get_group_messages(&group_id, limit).await?;
                Ok(IrohToolResult {
                    success: true,
                    message: format!("{} messages", messages.len()),
                    data: Some(serde_json::json!({ "messages": messages })),
                    output: None,
                })
            },
        }
    }
}

#[async_trait::async_trait]
impl ToolExecutor for IrohTool {
    fn metadata(&self) -> &ToolMetadata {
        static METADATA: OnceLock<ToolMetadata> = OnceLock::new();
        METADATA.get_or_init(|| ToolMetadata {
            id: "iroh".to_string(),
            name: "Iroh P2P Tool".to_string(),
            description: "Decentralized P2P networking tool using Iroh protocol".to_string(),
            category: ToolCategory::Network,
            priority: ToolPriority::Medium,
            status: ToolStatus::Available,
            version: "1.0.0".to_string(),
            author: "Alou Team".to_string(),
            created_at: chrono::Utc::now().timestamp(),
            updated_at: chrono::Utc::now().timestamp(),
            dependencies: vec![],
            platforms: vec!["windows".to_string(), "macos".to_string(), "linux".to_string()],
            permissions: vec!["network".to_string()],
            tags: vec!["p2p".to_string(), "iroh".to_string(), "networking".to_string()],
        })
    }

    async fn execute(&self, args: Value, _context: &ExecutionContext) -> Result<ToolResult, ToolError> {
        match self.execute_impl(args).await {
            Ok(result) => Ok(ToolResult::success(serde_json::to_value(result).unwrap_or_default())),
            Err(e) => Ok(ToolResult::error(e.to_string())),
        }
    }

    async fn validate_args(&self, args: &Value) -> Result<(), ToolError> {
        if let Some(obj) = args.as_object() {
            match obj.get("action").and_then(|v| v.as_str()) {
                Some("create_group") => {
                    if !obj.contains_key("group_name") {
                        return Err(ToolError::InvalidArguments("group_name required".to_string()));
                    }
                },
                Some("join_group") | Some("leave_group") | Some("get_group_info") => {
                    if !obj.contains_key("group_id") {
                        return Err(ToolError::InvalidArguments("group_id required".to_string()));
                    }
                },
                Some("send_group_message") => {
                    if !obj.contains_key("group_id") || !obj.contains_key("message") {
                        return Err(ToolError::InvalidArguments("group_id and message required".to_string()));
                    }
                },
                _ => {}
            }
        }
        
        Ok(())
    }

    fn help(&self) -> String {
        r#"# Iroh Tool

P2P networking and data synchronization using Iroh.

## Actions

### Document Operations
- `create_doc` - Create a new document
- `open_doc` - Open an existing document
- `set` - Set a key-value pair in a document
- `get` - Get a value from a document
- `list_entries` - List all entries in a document

### Group Chat Operations
- `create_group` - Create a P2P group chat
  - `group_name`: Name of the group
  - `description`: Optional description
- `join_group` - Join a P2P group chat
  - `group_id`: Group ID or ticket
- `leave_group` - Leave a group chat
  - `group_id`: Group ID
- `send_group_message` - Send a message to a group
  - `group_id`: Group ID
  - `message`: Message content
- `get_group_info` - Get group information
- `list_groups` - List all groups
- `get_group_messages` - Get group message history

### Utility
- `get_node_id` - Get current node ID
- `connect_to_node` - Connect to a peer
- `share_doc_ticket` - Share a document ticket
"#.to_string()
    }
}
