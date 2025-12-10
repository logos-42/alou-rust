// DIAP module - 使用完整 DIAP SDK 规范
use serde::{Deserialize, Serialize};
use serde_json::json;

use diap_rs_sdk::{IpfsClient, KeyPair};
use crate::utils::{default_ipfs_api_url, default_ipfs_gateway_url, normalize_base_url};
use log::{error, info, warn};
use reqwest::Client;

const IPFS_HTTP_TIMEOUT_SECS: u64 = 90;

#[derive(Default, Deserialize)]
pub struct LocalDiapIdentityRequest {
    pub agent_name: Option<String>,
    pub agent_description: Option<String>,
    pub ipfs_api_url: Option<String>,
    pub ipfs_gateway_url: Option<String>,
    pub ipns_key: Option<String>,
    pub session_id: Option<String>,
    pub custom_prompt: Option<String>,
    pub avatar_cid: Option<String>,
    pub mcp_config_cid: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct LocalDiapIdentityResponse {
    pub did: String,
    pub cid: String,
    pub ipns: String,
    pub public_key: String,
    pub gateway_url: String,
    pub ipns_key: Option<String>,
    /// 加密的节点标识信息
    pub encrypted_node_id: Option<EncryptedNodeId>,
    /// PubSub 主题配置
    pub pubsub_topics: Option<Vec<String>>,
}

/// 加密的节点标识
#[derive(Serialize, Clone, Debug)]
pub struct EncryptedNodeId {
    /// 加密后的数据（Base64）
    pub ciphertext: String,
    /// Nonce（Base64）
    pub nonce: String,
    /// 签名（Base64）
    pub signature: String,
    /// 加密方法
    pub method: String,
}

/// 直接调用 IPFS API 提供内容到 DHT（桌面版专用，不依赖 SDK）
async fn provide_to_dht_direct(api_url: &str, cid: &str) -> Result<(), String> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .no_proxy()
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;
    
    let url = format!("{}/api/v0/dht/provide?arg={}", normalize_base_url(api_url), cid);
    
    let response = client
        .post(&url)
        .header("User-Agent", "Alou-Desktop/1.0")
        .send()
        .await
        .map_err(|e| format!("DHT provide 请求失败: {}", e))?;
    
    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(format!("DHT provide 失败: {} - {}", status, text));
    }
    
    info!(target: "diap", "✅ 成功提供内容到 DHT: {}", cid);
    Ok(())
}

// 注意：publish_ipns_direct 函数已移除，现在使用 SDK 的 publish_ipns_direct 方法

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
    let custom_prompt = params.custom_prompt.clone();
    let avatar_cid = params.avatar_cid.clone();
    let mcp_config_cid = params.mcp_config_cid.clone();

    info!(target: "diap", "开始创建 DIAP Identity...");

    // 使用 DIAP SDK 创建 IPFS 客户端
    let ipfs_client = IpfsClient::new_with_remote_node(
        ipfs_api.clone(),
        gateway.clone(),
        IPFS_HTTP_TIMEOUT_SECS,
    );

    // 使用 DIAP SDK 生成密钥对
    let keypair = KeyPair::generate()
        .map_err(|e| format!("Failed to generate key pair: {}", e))?;
    
    let did = keypair.did.clone();
    let did_hash = did.split(':').last().unwrap_or(&did);
    let created = chrono::Utc::now().to_rfc3339();

    info!(target: "diap", "生成的 DID: {}", did);

    // 生成 IPNS key 名称
    let ipns_key_name = if let Some(ref provided_key) = params.ipns_key {
        provided_key.clone()
    } else {
        format!("agent-{}", &did_hash[..12.min(did_hash.len())])
    };

    // 使用 DIAP SDK 确保 IPNS key 存在
    let ipns_key = ipfs_client
        .ensure_key_exists(&ipns_key_name)
        .await
        .map_err(|e| format!("Failed to ensure IPNS key exists: {}", e))?;

    // 生成 PubSub 主题
    let pubsub_auth_topic = format!("diap/auth/{}", &did_hash[..16.min(did_hash.len())]);
    let pubsub_msg_topic = format!("diap/msg/{}", &did_hash[..16.min(did_hash.len())]);
    let pubsub_topics = vec![pubsub_auth_topic.clone(), pubsub_msg_topic.clone()];

    // 公钥的 multibase 编码（z 前缀 + base58btc）
    let public_key_multibase = format!("z{}", bs58::encode(&keypair.public_key).into_string());

    // 创建完整的 DID 文档
    let mut did_document = json!({
        "@context": [
            "https://www.w3.org/ns/did/v1",
            "https://w3id.org/security/suites/ed25519-2020/v1"
        ],
        "id": did,
        "created": created,
        "verificationMethod": [{
            "id": format!("{}#keys-1", did),
            "type": "Ed25519VerificationKey2020",
            "controller": did,
            "publicKeyMultibase": public_key_multibase
        }],
        "authentication": [format!("{}#keys-1", did)],
        "assertionMethod": [format!("{}#keys-1", did)],
        "service": [{
            "id": format!("{}#agent", did),
            "type": "AgentEndpoint",
            "serviceEndpoint": {
                "type": "AgentProfile",
                "name": agent_name,
                "description": agent_description,
                "agent_type": "claude_agent_sdk"
            },
            "pubsubTopics": pubsub_topics.clone(),
            "networkAddresses": []
        }]
    });

    // 添加可选的服务端点信息
    if let Some(services) = did_document.get_mut("service").and_then(|s| s.as_array_mut()) {
        if let Some(agent_service) = services.get_mut(0) {
            if let Some(endpoint) = agent_service.get_mut("serviceEndpoint") {
                if let Some(avatar) = &avatar_cid {
                    endpoint["avatar_cid"] = json!(avatar);
                }
                if let Some(mcp) = &mcp_config_cid {
                    endpoint["mcp_config_cid"] = json!(mcp);
                }
                if let Some(prompt) = &custom_prompt {
                    endpoint["custom_prompt"] = json!(prompt);
                }
            }
        }

        // 添加 PubSub 认证服务端点
        services.push(json!({
            "id": format!("{}#pubsub-auth", did),
            "type": "PubSubAuth",
            "serviceEndpoint": {
                "topic": pubsub_auth_topic,
                "protocol": "gossipsub"
            }
        }));
    }

    // 添加元数据
    did_document["alou:metadata"] = if let Some(ref sid) = session_id {
        json!({
            "ipns_key": ipns_key,
            "session_id": sid,
            "created_at": created,
            "sdk_version": "0.2.10",
            "pubsub_enabled": true
        })
    } else {
        json!({
            "ipns_key": ipns_key,
            "created_at": created,
            "sdk_version": "0.2.10",
            "pubsub_enabled": true
        })
    };

    // 生成简化的加密节点标识（使用密钥派生的伪节点ID）
    let encrypted_node_id = generate_encrypted_node_id(&keypair)?;

    // 将加密节点标识添加到 DID 文档
    if let Some(services) = did_document.get_mut("service").and_then(|s| s.as_array_mut()) {
        services.push(json!({
            "id": format!("{}#encrypted-node-id", did),
            "type": "EncryptedNodeId",
            "serviceEndpoint": {
                "ciphertext": encrypted_node_id.ciphertext,
                "nonce": encrypted_node_id.nonce,
                "signature": encrypted_node_id.signature,
                "method": encrypted_node_id.method
            }
        }));
    }

    // 序列化 DID 文档
    let doc_json = serde_json::to_string_pretty(&did_document)
        .map_err(|e| format!("Failed to serialize DID document: {}", e))?;

    info!(target: "diap", "DID 文档已创建，正在上传到 IPFS...");

    // 使用 DIAP SDK 上传 DID 文档到 IPFS
    let upload_result = ipfs_client
        .upload(&doc_json, "did.json")
        .await
        .map_err(|e| format!("Failed to upload DID document to IPFS: {}", e))?;

    let cid = upload_result.cid;
    info!(target: "diap", "DID 文档已上传，CID: {}", cid);

    // 主动提供数据到 DHT，确保数据可以被其他节点发现
    info!(target: "diap", "正在提供数据到 DHT...");
    if let Err(e) = provide_to_dht_direct(&ipfs_api, &cid).await {
        warn!(
            target: "diap",
            "DHT provide 失败（不影响上传，数据仍会被pin）: {}",
            e
        );
    } else {
        info!(target: "diap", "✅ 数据已提供到 DHT，CID: {}", cid);
    }

    // 使用 SDK 的 publish_ipns_direct 方法发布 IPNS（使用 allow-offline=false 确保在 DHT 中传播）
    info!(target: "diap", "正在发布 IPNS 记录（确保在线传播）...");
    let ipns_result = match ipfs_client
        .publish_ipns_direct(&cid, &ipns_key, "24h", "24h")
        .await
    {
        Ok(result) => {
            info!(target: "diap", "✅ IPNS 发布成功（在线模式，已传播到DHT）: {}", result.name);
            result
        }
        Err(e) => {
            error!(
                target: "diap",
                "IPNS publish_direct failed (cid={}, key={}, api={}): {}",
                cid,
                ipns_key,
                ipfs_api,
                e
            );
            // 如果直接发布失败，降级使用普通 publish_ipns（allow-offline=true）
            warn!(target: "diap", "降级使用普通 IPNS 发布（allow-offline=true）...");
            ipfs_client
                .publish_ipns(&cid, &ipns_key, "24h", "24h")
                .await
                .map_err(|e| format!("SDK IPNS 发布也失败: {}", e))?
        }
    };
    
    let ipns_name = ipns_result.name;

    let ipns_path = if ipns_name.starts_with("/ipns/") {
        ipns_name.clone()
    } else {
        format!("/ipns/{}", ipns_name)
    };

    // 主动提供 IPNS 记录到 DHT，确保 IPNS 可以被其他节点发现
    info!(target: "diap", "正在提供 IPNS 记录到 DHT...");
    let ipns_name_for_dht = ipns_name.trim_start_matches("/ipns/");
    if let Err(e) = provide_to_dht_direct(&ipfs_api, ipns_name_for_dht).await {
        warn!(
            target: "diap",
            "IPNS DHT provide 失败（不影响发布）: {}",
            e
        );
    } else {
        info!(target: "diap", "✅ IPNS 记录已提供到 DHT，IPNS: {}", ipns_path);
    }

    let gateway_url = format!("{}/ipfs/{}", normalize_base_url(&gateway), cid);

    info!(target: "diap", "DIAP Identity 创建成功！");
    info!(target: "diap", "  DID: {}", did);
    info!(target: "diap", "  CID: {}", cid);
    info!(target: "diap", "  IPNS: {}", ipns_path);

    Ok(LocalDiapIdentityResponse {
        did,
        cid,
        ipns: ipns_path,
        public_key: public_key_multibase,
        gateway_url,
        ipns_key: Some(ipns_key),
        encrypted_node_id: Some(encrypted_node_id),
        pubsub_topics: Some(pubsub_topics),
    })
}

/// 生成加密的节点标识
fn generate_encrypted_node_id(keypair: &KeyPair) -> Result<EncryptedNodeId, String> {
    use sha2::{Sha256, Digest};
    use base64::{engine::general_purpose::STANDARD, Engine};
    
    // 从公钥派生一个伪节点ID（32字节）
    let mut hasher = Sha256::new();
    hasher.update(&keypair.public_key);
    hasher.update(b"DIAP_NODE_ID_V1");
    let node_id = hasher.finalize();
    
    // 生成随机 nonce（12字节，用于 AES-GCM）
    let mut nonce = [0u8; 12];
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;
    nonce[..8].copy_from_slice(&timestamp.to_le_bytes());
    
    // 简化的"加密"：XOR with key-derived mask（用于演示，生产环境应使用真正的 AES-GCM）
    let mut key_hasher = Sha256::new();
    key_hasher.update(&keypair.private_key);
    key_hasher.update(b"DIAP_ENC_KEY_V1");
    let enc_key = key_hasher.finalize();
    
    let mut ciphertext = node_id.to_vec();
    for (i, byte) in ciphertext.iter_mut().enumerate() {
        *byte ^= enc_key[i % 32];
    }
    
    // 生成签名
    let mut sig_data = Vec::new();
    sig_data.extend_from_slice(&ciphertext);
    sig_data.extend_from_slice(&nonce);
    
    let mut sig_hasher = Sha256::new();
    sig_hasher.update(&keypair.private_key);
    sig_hasher.update(&sig_data);
    let signature = sig_hasher.finalize();
    
    Ok(EncryptedNodeId {
        ciphertext: STANDARD.encode(&ciphertext),
        nonce: STANDARD.encode(&nonce),
        signature: STANDARD.encode(&signature),
        method: "XOR-SHA256-V1".to_string(), // 简化版本标识
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

    info!(target: "diap", "解析 IPNS: {}", ipns_name);

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

    info!(target: "diap", "IPNS 解析成功，CID: {}", cid);

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

    // 提取公钥
    let public_key = did_document
        .get("verificationMethod")
        .and_then(|vm| vm.as_array())
        .and_then(|arr| arr.first())
        .and_then(|m| m.get("publicKeyMultibase"))
        .and_then(|pk| pk.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            if let Some(ipns) = ipns_name.strip_prefix("/ipns/") {
                format!("pubkey_{}", ipns)
            } else {
                format!("pubkey_{}", ipns_name)
            }
        });

    // 提取 PubSub 主题
    let pubsub_topics = did_document
        .get("service")
        .and_then(|s| s.as_array())
        .and_then(|arr| {
            arr.iter()
                .find(|svc| svc.get("type").and_then(|t| t.as_str()) == Some("AgentEndpoint"))
        })
        .and_then(|svc| svc.get("pubsubTopics"))
        .and_then(|topics| topics.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect::<Vec<_>>()
        });

    // 提取加密节点标识
    let encrypted_node_id = did_document
        .get("service")
        .and_then(|s| s.as_array())
        .and_then(|arr| {
            arr.iter()
                .find(|svc| svc.get("type").and_then(|t| t.as_str()) == Some("EncryptedNodeId"))
        })
        .and_then(|svc| svc.get("serviceEndpoint"))
        .and_then(|ep| {
            Some(EncryptedNodeId {
                ciphertext: ep.get("ciphertext")?.as_str()?.to_string(),
                nonce: ep.get("nonce")?.as_str()?.to_string(),
                signature: ep.get("signature")?.as_str()?.to_string(),
                method: ep.get("method")?.as_str()?.to_string(),
            })
        });

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
        encrypted_node_id,
        pubsub_topics,
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

    info!(target: "diap", "更新 IPNS: key={}, cid={}", ipns_key, cid);

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

    let public_key = did_document
        .get("verificationMethod")
        .and_then(|vm| vm.as_array())
        .and_then(|arr| arr.first())
        .and_then(|m| m.get("publicKeyMultibase"))
        .and_then(|pk| pk.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("pubkey_{}", ipns_name.trim_start_matches("/ipns/")));

    let pubsub_topics = did_document
        .get("service")
        .and_then(|s| s.as_array())
        .and_then(|arr| {
            arr.iter()
                .find(|svc| svc.get("type").and_then(|t| t.as_str()) == Some("AgentEndpoint"))
        })
        .and_then(|svc| svc.get("pubsubTopics"))
        .and_then(|topics| topics.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect::<Vec<_>>()
        });

    let encrypted_node_id = did_document
        .get("service")
        .and_then(|s| s.as_array())
        .and_then(|arr| {
            arr.iter()
                .find(|svc| svc.get("type").and_then(|t| t.as_str()) == Some("EncryptedNodeId"))
        })
        .and_then(|svc| svc.get("serviceEndpoint"))
        .and_then(|ep| {
            Some(EncryptedNodeId {
                ciphertext: ep.get("ciphertext")?.as_str()?.to_string(),
                nonce: ep.get("nonce")?.as_str()?.to_string(),
                signature: ep.get("signature")?.as_str()?.to_string(),
                method: ep.get("method")?.as_str()?.to_string(),
            })
        });

    let gateway_url = format!("{}/ipfs/{}", normalize_base_url(&gateway), cid);

    info!(target: "diap", "IPNS 更新成功: {}", ipns_path);

    Ok(LocalDiapIdentityResponse {
        did,
        cid,
        ipns: ipns_path,
        public_key,
        gateway_url,
        ipns_key: Some(ipns_key),
        encrypted_node_id,
        pubsub_topics,
    })
}
