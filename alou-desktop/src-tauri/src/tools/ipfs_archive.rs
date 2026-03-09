//! IPFS 存档工具
//!
//! 自动将 skills、聊天记录、工作记忆等存档到 IPFS

use super::{ToolExecutor, ToolMetadata, ToolResult, ToolError, ExecutionContext, ToolCategory, ToolStatus, ToolPriority};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// IPFS 存档工具
pub struct IpfsArchiveTool {
    metadata: ToolMetadata,
}

/// 存档类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArchiveType {
    /// Skills
    Skills,
    /// 聊天记录
    ChatHistory,
    /// 工作记忆
    WorkingMemory,
    /// 计划任务
    Plans,
    /// 待办事项
    Todos,
    /// 自定义文件
    CustomFile,
    /// 目录
    Directory,
}

/// 存档结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveResult {
    /// CID
    pub cid: String,
    /// 大小
    pub size: u64,
    /// 存档类型
    pub archive_type: String,
    /// 名称
    pub name: String,
    /// 时间戳
    pub timestamp: i64,
    /// IPNS 名称（如果发布了）
    pub ipns_name: Option<String>,
}

/// 存档元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveMetadata {
    /// 存档ID
    pub id: String,
    /// 存档类型
    pub archive_type: ArchiveType,
    /// 名称
    pub name: String,
    /// 描述
    pub description: String,
    /// 标签
    pub tags: Vec<String>,
    /// 创建时间
    pub created_at: i64,
    /// 关联的会话ID
    pub session_id: Option<String>,
    /// 关联的Agent ID
    pub agent_id: Option<String>,
    /// 额外的元数据
    pub extra: HashMap<String, serde_json::Value>,
}

impl IpfsArchiveTool {
    /// 创建新的 IPFS 存档工具
    pub fn new() -> Self {
        Self {
            metadata: ToolMetadata {
                id: "ipfs_archive".to_string(),
                name: "IPFS Archive Tool".to_string(),
                description: "自动将数据存档到 IPFS，支持 skills、聊天记录、工作记忆等".to_string(),
                category: ToolCategory::FileSystem,
                priority: ToolPriority::Medium,
                status: ToolStatus::Available,
                version: "1.0.0".to_string(),
                author: "Alou Team".to_string(),
                created_at: chrono::Utc::now().timestamp(),
                updated_at: chrono::Utc::now().timestamp(),
                dependencies: vec!["ipfs".to_string()],
                platforms: vec!["windows".to_string(), "macos".to_string(), "linux".to_string()],
                permissions: vec!["read".to_string(), "write".to_string()],
                tags: vec!["ipfs".to_string(), "archive".to_string(), "storage".to_string()],
            },
        }
    }

    /// 存档数据到 IPFS
    async fn archive_data(
        &self,
        data: serde_json::Value,
        archive_type: ArchiveType,
        name: String,
        description: String,
        metadata: Option<ArchiveMetadata>,
        pin: bool,
    ) -> Result<ArchiveResult, String> {
        use crate::ipfs_commands::add_did_document_to_ipfs;
        use crate::utils::default_ipfs_api_url;

        let api_url = default_ipfs_api_url();
        let timestamp = chrono::Utc::now().timestamp();

        // 构建存档包
        let archive_package = serde_json::json!({
            "version": "1.0",
            "archive_type": match &archive_type {
                ArchiveType::Skills => "skills",
                ArchiveType::ChatHistory => "chat_history",
                ArchiveType::WorkingMemory => "working_memory",
                ArchiveType::Plans => "plans",
                ArchiveType::Todos => "todos",
                ArchiveType::CustomFile => "custom_file",
                ArchiveType::Directory => "directory",
            },
            "name": name,
            "description": description,
            "timestamp": timestamp,
            "metadata": metadata,
            "data": data,
        });

        // 上传到 IPFS
        let result = add_did_document_to_ipfs(&api_url, &archive_package).await
            .map_err(|e| format!("Failed to archive to IPFS: {}", e))?;

        // 如果需要，固定(pin)内容
        if pin {
            // 调用 IPFS pin API
            let _ = self.pin_cid(&result.cid).await;
        }

        Ok(ArchiveResult {
            cid: result.cid,
            size: result.size.as_ref().and_then(|s| s.parse().ok()).unwrap_or(0),
            archive_type: match archive_type {
                ArchiveType::Skills => "skills".to_string(),
                ArchiveType::ChatHistory => "chat_history".to_string(),
                ArchiveType::WorkingMemory => "working_memory".to_string(),
                ArchiveType::Plans => "plans".to_string(),
                ArchiveType::Todos => "todos".to_string(),
                ArchiveType::CustomFile => "custom_file".to_string(),
                ArchiveType::Directory => "directory".to_string(),
            },
            name,
            timestamp,
            ipns_name: None,
        })
    }

    /// 固定 CID
    async fn pin_cid(&self, cid: &str) -> Result<(), String> {
        use crate::utils::{default_ipfs_api_url, normalize_base_url};
        use reqwest::Client;

        let api_url = default_ipfs_api_url();
        let endpoint = format!("{}/api/v0/pin/add", normalize_base_url(&api_url));

        let client = Client::new();
        let response = client
            .post(&endpoint)
            .query(&[("arg", cid)])
            .send()
            .await
            .map_err(|e| format!("Pin request failed: {}", e))?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(format!("Pin failed: {}", response.status()))
        }
    }

    /// 从 IPFS 检索存档
    async fn retrieve_archive(&self, cid: &str) -> Result<serde_json::Value, String> {
        use crate::utils::{default_ipfs_api_url, normalize_base_url};
        use reqwest::Client;

        let api_url = default_ipfs_api_url();
        let endpoint = format!("{}/api/v0/cat", normalize_base_url(&api_url));

        let client = Client::new();
        let response = client
            .post(&endpoint)
            .query(&[("arg", cid)])
            .send()
            .await
            .map_err(|e| format!("Retrieve request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Retrieve failed: {}", response.status()));
        }

        let data = response.json::<serde_json::Value>().await
            .map_err(|e| format!("Failed to parse archive data: {}", e))?;

        Ok(data)
    }

    /// 列出已固定的存档
    async fn list_pinned(&self, _archive_type: Option<ArchiveType>) -> Result<Vec<ArchiveResult>, String> {
        // 这里应该从本地存储或IPFS获取已存档的列表
        // 简化实现，返回空列表
        Ok(vec![])
    }

    /// 发布到 IPNS
    async fn publish_to_ipns(&self, cid: &str, key_name: Option<String>) -> Result<String, String> {
        use crate::utils::{default_ipfs_api_url, normalize_base_url};
        use reqwest::Client;
        use serde::Deserialize;

        #[derive(Deserialize)]
        struct IpnsPublishResponse {
            #[serde(rename = "Name")]
            name: String,
        }

        let api_url = default_ipfs_api_url();
        let endpoint = format!("{}/api/v0/name/publish", normalize_base_url(&api_url));

        let client = Client::new();
        let mut request = client
            .post(&endpoint)
            .query(&[("arg", cid)]);

        if let Some(key) = key_name {
            request = request.query(&[("key", key)]);
        }

        let response = request
            .send()
            .await
            .map_err(|e| format!("IPNS publish request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("IPNS publish failed: {}", response.status()));
        }

        let result: IpnsPublishResponse = response.json().await
            .map_err(|e| format!("Failed to parse IPNS response: {}", e))?;

        Ok(result.name)
    }
}

#[async_trait]
impl ToolExecutor for IpfsArchiveTool {
    fn metadata(&self) -> &ToolMetadata {
        &self.metadata
    }

    async fn execute(&self, args: serde_json::Value, _context: &ExecutionContext) -> Result<ToolResult, ToolError> {
        let action = args.get("action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArguments("Missing 'action' field".to_string()))?;

        match action {
            "archive" => {
                let data = args.get("data")
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'data' field".to_string()))?
                    .clone();

                let archive_type = match args.get("archive_type").and_then(|v| v.as_str()) {
                    Some("skills") => ArchiveType::Skills,
                    Some("chat_history") => ArchiveType::ChatHistory,
                    Some("working_memory") => ArchiveType::WorkingMemory,
                    Some("plans") => ArchiveType::Plans,
                    Some("todos") => ArchiveType::Todos,
                    Some("custom_file") => ArchiveType::CustomFile,
                    Some("directory") => ArchiveType::Directory,
                    _ => ArchiveType::CustomFile,
                };

                let name = args.get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unnamed")
                    .to_string();

                let description = args.get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let pin = args.get("pin").and_then(|v| v.as_bool()).unwrap_or(true);

                let result = self.archive_data(data, archive_type, name, description, None, pin).await
                    .map_err(|e| ToolError::ExecutionFailed(e))?;

                Ok(ToolResult {
                    success: true,
                    data: serde_json::json!(result),
                    error: None,
                    execution_time_ms: 0,
                    output: Some(format!("Archived to IPFS: {}", result.cid)),
                    warnings: vec![],
                    context: None,
                })
            }

            "retrieve" => {
                let cid = args.get("cid")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'cid' field".to_string()))?;

                let data = self.retrieve_archive(cid).await
                    .map_err(|e| ToolError::ExecutionFailed(e))?;

                Ok(ToolResult {
                    success: true,
                    data,
                    error: None,
                    execution_time_ms: 0,
                    output: Some(format!("Retrieved from IPFS: {}", cid)),
                    warnings: vec![],
                    context: None,
                })
            }

            "publish_ipns" => {
                let cid = args.get("cid")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'cid' field".to_string()))?;

                let key_name = args.get("key_name").and_then(|v| v.as_str()).map(|s| s.to_string());

                let ipns_name = self.publish_to_ipns(cid, key_name).await
                    .map_err(|e| ToolError::ExecutionFailed(e))?;

                Ok(ToolResult {
                    success: true,
                    data: serde_json::json!({
                        "cid": cid,
                        "ipns_name": ipns_name,
                    }),
                    error: None,
                    execution_time_ms: 0,
                    output: Some(format!("Published to IPNS: {}", ipns_name)),
                    warnings: vec![],
                    context: None,
                })
            }

            _ => Err(ToolError::InvalidArguments(format!("Unknown action: {}", action))),
        }
    }

    async fn validate_args(&self, args: &serde_json::Value) -> Result<(), ToolError> {
        if !args.is_object() {
            return Err(ToolError::InvalidArguments("Arguments must be an object".to_string()));
        }
        if args.get("action").is_none() {
            return Err(ToolError::InvalidArguments("Missing required field: action".to_string()));
        }
        Ok(())
    }

    fn help(&self) -> String {
        r#"IPFS Archive Tool

Archive data to IPFS, including skills, chat history, working memory, etc.

Actions:
  - archive: Archive data to IPFS
    params: {
      "data": <JSON data>,
      "archive_type": "skills|chat_history|working_memory|plans|todos|custom_file",
      "name": "archive name",
      "description": "archive description",
      "pin": true
    }
  
  - retrieve: Retrieve archive from IPFS
    params: { "cid": "Qm..." }
  
  - publish_ipns: Publish CID to IPNS
    params: { "cid": "Qm...", "key_name": "optional_key" }

Examples:

Archive skills:
{
  "action": "archive",
  "archive_type": "skills",
  "name": "My Skills Backup",
  "data": { "skills": [...] }
}

Archive chat history:
{
  "action": "archive",
  "archive_type": "chat_history",
  "name": "Chat Session 123",
  "data": { "messages": [...] }
}"#.to_string()
    }
}
