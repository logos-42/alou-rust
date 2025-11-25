// DIAP module - 使用完整 DIAP SDK
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use diap_rs_sdk::IpfsClient;
use crate::utils::{default_ipfs_api_url, default_ipfs_gateway_url, normalize_base_url};
use log::error;

const IPFS_HTTP_TIMEOUT_SECS: u64 = 90;

#[derive(Default, Deserialize)]
pub struct LocalDiapIdentityRequest {
    pub agent_name: Option<String>,
    pub agent_description: Option<String>,
    pub ipfs_api_url: Option<String>,
    pub ipfs_gateway_url: Option<String>,
    pub ipns_key: Option<String>,
    pub session_id: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct LocalDiapIdentityResponse {
    pub did: String,
    pub cid: String,
    pub ipns: String,
    pub public_key: String,
    pub gateway_url: String,
    pub ipns_key: Option<String>,
}

#[tauri::command]
pub async fn create_local_diap_identity(
    params: Option<LocalDiapIdentityRequest>,
) -> Result<LocalDiapIdentityResponse, String> {
    let params = params.unwrap_or_default();
    let ipfs_api = params
        .ipfs_api_url
        .clone()
        .unwrap_or_else(default_ipfs_api_url);
    let gateway = params
        .ipfs_gateway_url
        .clone()
        .unwrap_or_else(default_ipfs_gateway_url);
    let agent_name = params
        .agent_name
        .clone()
        .unwrap_or_else(|| "Claude Agent SDK".to_string());
    let agent_description = params.agent_description.clone().unwrap_or_default();
    let session_id = params.session_id.clone();

    // 使用 DIAP SDK 创建 IPFS 客户端
    let ipfs_client = IpfsClient::new_with_remote_node(
        ipfs_api.clone(),
        gateway.clone(),
        IPFS_HTTP_TIMEOUT_SECS,
    );

    // 生成 DID
    let did = format!("did:alou:{}", Uuid::new_v4());
    let did_hash = did.split(':').last().unwrap();
    let created = chrono::Utc::now().to_rfc3339();

    // 生成 IPNS key 名称
    let ipns_key_name = if let Some(ref provided_key) = params.ipns_key {
        provided_key.clone()
    } else {
        format!("agent-{}", did_hash)
    };

    // 使用 DIAP SDK 确保 IPNS key 存在
    let ipns_key = ipfs_client
        .ensure_key_exists(&ipns_key_name)
        .await
        .map_err(|e| format!("Failed to ensure IPNS key exists: {}", e))?;

    // 创建 DID 文档，包含元数据
    let mut did_document = json!({
        "@context": ["https://www.w3.org/ns/did/v1"],
        "id": did,
        "created": created,
        "service": [{
            "id": format!("{}#agent", did),
            "type": "AgentEndpoint",
            "serviceEndpoint": {
                "type": "AgentProfile",
                "name": agent_name,
                "description": agent_description,
            }
        }]
    });

    // 添加元数据到 DID 文档
    if let Some(ref sid) = session_id {
        did_document["alou:metadata"] = json!({
            "ipns_key": ipns_key,
            "session_id": sid,
            "created_at": created,
        });
    } else {
        did_document["alou:metadata"] = json!({
            "ipns_key": ipns_key,
            "created_at": created,
        });
    }

    // 序列化 DID 文档
    let doc_json = serde_json::to_string(&did_document)
        .map_err(|e| format!("Failed to serialize DID document: {}", e))?;

    // 使用 DIAP SDK 上传 DID 文档到 IPFS
    let upload_result = ipfs_client
        .upload(&doc_json, "did.json")
        .await
        .map_err(|e| format!("Failed to upload DID document to IPFS: {}", e))?;

    let cid = upload_result.cid;

    // 使用 DIAP SDK 发布到 IPNS
    let ipns_result = ipfs_client
        .publish_ipns(&cid, &ipns_key, "24h", "24h")
        .await
        .map_err(|e| {
            error!(
                target: "diap",
                "IPNS publish failed (cid={}, key={}, api={}): {:#}",
                cid,
                ipns_key,
                ipfs_api,
                e
            );
            format!("Failed to publish to IPNS: {}", e)
        })?;

    let ipns_name = ipns_result.name.clone();
    let ipns_path = if ipns_name.starts_with("/ipns/") {
        ipns_name.clone()
    } else {
        format!("/ipns/{}", ipns_name)
    };

    let gateway_url = format!("{}/ipfs/{}", normalize_base_url(&gateway), cid);
    let public_key = format!("pubkey_{}", ipns_name.trim_start_matches("/ipns/"));

    Ok(LocalDiapIdentityResponse {
        did,
        cid,
        ipns: ipns_path,
        public_key,
        gateway_url,
        ipns_key: Some(ipns_key),
    })
}

/// 从 IPNS 解析获取身份信息
#[tauri::command]
pub async fn get_local_diap_identity(
    ipns_name: String,
    ipfs_api_url: Option<String>,
    ipfs_gateway_url: Option<String>,
) -> Result<LocalDiapIdentityResponse, String> {
    let ipfs_api = ipfs_api_url.unwrap_or_else(default_ipfs_api_url);
    let gateway = ipfs_gateway_url.unwrap_or_else(default_ipfs_gateway_url);

    // 使用 DIAP SDK 创建 IPFS 客户端
    let ipfs_client = IpfsClient::new_with_remote_node(
        ipfs_api.clone(),
        gateway.clone(),
        IPFS_HTTP_TIMEOUT_SECS,
    );

    // 使用 DIAP SDK 解析 IPNS
    let cid = ipfs_client
        .resolve_ipns(&ipns_name)
        .await
        .map_err(|e| format!("Failed to resolve IPNS: {}", e))?;

    // 使用 DIAP SDK 获取 DID 文档
    let did_document_json = ipfs_client
        .get(&cid)
        .await
        .map_err(|e| format!("Failed to get DID document from IPFS: {}", e))?;

    let did_document: serde_json::Value = serde_json::from_str(&did_document_json)
        .map_err(|e| format!("Failed to parse DID document: {}", e))?;

    // 提取身份信息
    let did = did_document
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing 'id' in DID document".to_string())?
        .to_string();

    let metadata = did_document.get("alou:metadata");
    let ipns_key = metadata
        .and_then(|m| m.get("ipns_key"))
        .and_then(|k| k.as_str())
        .map(|s| s.to_string());

    let public_key = if let Some(ipns) = ipns_name.strip_prefix("/ipns/") {
        format!("pubkey_{}", ipns)
    } else {
        format!("pubkey_{}", ipns_name)
    };

    let gateway_url = format!("{}/ipfs/{}", normalize_base_url(&gateway), cid);

    Ok(LocalDiapIdentityResponse {
        did,
        cid,
        ipns: if ipns_name.starts_with("/ipns/") {
            ipns_name
        } else {
            format!("/ipns/{}", ipns_name)
        },
        public_key,
        gateway_url,
        ipns_key,
    })
}

/// 更新 IPNS 指向新的 CID
#[tauri::command]
pub async fn update_local_diap_identity(
    ipns_key: String,
    cid: String,
    ipfs_api_url: Option<String>,
    ipfs_gateway_url: Option<String>,
) -> Result<LocalDiapIdentityResponse, String> {
    let ipfs_api = ipfs_api_url.unwrap_or_else(default_ipfs_api_url);
    let gateway = ipfs_gateway_url.unwrap_or_else(default_ipfs_gateway_url);

    // 使用 DIAP SDK 创建 IPFS 客户端
    let ipfs_client = IpfsClient::new_with_remote_node(
        ipfs_api.clone(),
        gateway.clone(),
        IPFS_HTTP_TIMEOUT_SECS,
    );

    // 使用 DIAP SDK 更新 IPNS
    let ipns_result = ipfs_client
        .publish_ipns(&cid, &ipns_key, "24h", "24h")
        .await
        .map_err(|e| format!("Failed to update IPNS: {}", e))?;

    let ipns_name = ipns_result.name.clone();
    let ipns_path = if ipns_name.starts_with("/ipns/") {
        ipns_name.clone()
    } else {
        format!("/ipns/{}", ipns_name)
    };

    // 获取 DID 文档以提取完整信息
    let did_document_json = ipfs_client
        .get(&cid)
        .await
        .map_err(|e| format!("Failed to get DID document from IPFS: {}", e))?;

    let did_document: serde_json::Value = serde_json::from_str(&did_document_json)
        .map_err(|e| format!("Failed to parse DID document: {}", e))?;

    let did = did_document
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing 'id' in DID document".to_string())?
        .to_string();

    let public_key = format!("pubkey_{}", ipns_name.trim_start_matches("/ipns/"));
    let gateway_url = format!("{}/ipfs/{}", normalize_base_url(&gateway), cid);

    Ok(LocalDiapIdentityResponse {
        did,
        cid,
        ipns: ipns_path,
        public_key,
        gateway_url,
        ipns_key: Some(ipns_key),
    })
}
