// DIAP module - 使用完整 DIAP SDK 规范
use serde::{Deserialize, Serialize};
use serde_json::json;

use diap_rs_sdk::{IpfsClient, KeyPair};
use crate::utils::{default_ipfs_api_url, default_ipfs_gateway_url, normalize_base_url};
use log::{debug, error, info, warn};
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

/// 主动触发公共网关查询以加速 IPNS 传播（带重试预热）
/// 每隔指定时间触发一次，共触发指定次数
/// 因为第一次触发时网关可能还没在 DHT 中搜到，第二次请求通常就能命中
async fn trigger_public_gateway_query_with_retry(ipns_path: &str, retry_count: u32, interval_secs: u64) {
    for i in 1..=retry_count {
        info!(target: "diap", "预热触发第 {}/{} 次公共网关查询...", i, retry_count);
        trigger_public_gateway_query(ipns_path).await;
        
        if i < retry_count {
            info!(target: "diap", "等待 {} 秒后进行下一次预热触发...", interval_secs);
            tokio::time::sleep(tokio::time::Duration::from_secs(interval_secs)).await;
        }
    }
    info!(target: "diap", "✅ 公共网关查询预热完成（已触发 {} 次）", retry_count);
}

/// 主动触发公共网关查询以加速 IPNS 传播
/// 当向公共网关发起请求时，这些大型网关会主动去 DHT 中寻找 IPNS 记录
/// 这相当于强制触发了 DHT 的查询过程，可以大大缩短全球生效时间
/// 公共网关通常拥有极高的带宽和海量的 Peer 连接，一旦找到记录会协助缓存并进一步扩散
async fn trigger_public_gateway_query(ipns_path: &str) {
    let public_gateways: Vec<String> = vec![
        "https://gateway.ipfs.io".to_string(),
        "https://ipfs.io".to_string(),
        "https://dweb.link".to_string(),
        "https://cloudflare-ipfs.com".to_string(),
    ];
    
    let client = match Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            warn!(target: "diap", "创建 HTTP 客户端失败，跳过主动触发: {}", e);
            return;
        }
    };
    
    // 保存网关数量用于日志
    let gateway_count = public_gateways.len();
    
    // 并发向所有公共网关发起 HEAD 请求（触发 DHT 查询）
    let trigger_futures: Vec<_> = public_gateways
        .into_iter()
        .map(|gateway| {
            let gateway_url = format!("{}{}", gateway, ipns_path);
            let client_clone = client.clone();
            
            async move {
                // 使用 HEAD 请求触发网关查询（不下载内容，只触发 DHT 查询）
                match client_clone
                    .head(&gateway_url)
                    .timeout(std::time::Duration::from_secs(5))
                    .send()
                    .await
                {
                    Ok(response) => {
                        let status = response.status().as_u16();
                        if status == 200 {
                            info!(target: "diap", "✅ {} 已找到 IPNS 记录", gateway_url);
                        } else if status == 504 || status == 502 || status == 503 {
                            info!(target: "diap", "⏳ {} 正在查询 IPNS 记录（已触发 DHT 查询）", gateway_url);
                        } else {
                            info!(target: "diap", "🔍 {} 已触发查询，状态: {}", gateway_url, status);
                        }
                    }
                    Err(e) => {
                        // 即使失败也视为成功触发（因为网关可能已经开始查询）
                        let error_str = e.to_string();
                        if error_str.contains("timeout") {
                            info!(target: "diap", "⏳ {} 查询超时（但已触发 DHT 查询）", gateway_url);
                        } else {
                            debug!(target: "diap", "🔍 {} 触发查询时出错（可能已开始查询）: {}", gateway_url, e);
                        }
                    }
                }
            }
        })
        .collect();
    
    // 并发执行所有触发请求（不等待全部完成，后台执行）
    tokio::spawn(async move {
        let handles: Vec<_> = trigger_futures.into_iter()
            .map(|f| tokio::spawn(f))
            .collect();
        
        // 等待所有请求完成（但不会阻塞主流程）
        for handle in handles {
            let _ = handle.await;
        }
    });
    
    info!(target: "diap", "✅ 已向 {} 个公共网关发起查询请求，将加速 IPNS 记录传播", gateway_count);
}

/// 检查并启用 IPNS PubSub 以加速 IPNS 记录的传播
/// IPNS PubSub 可以将传播时间从几分钟减少到几秒
/// 注意：IPNS PubSub 需要节点在配置中启用，如果节点不支持会回退到 DHT 传播
async fn enable_ipns_pubsub(api_url: &str) -> Result<(), String> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .no_proxy()
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;
    
    // IPNS PubSub 的工作原理：
    // 1. 当节点启用 PubSub 时，IPNS 记录会通过 PubSub 快速传播（从几分钟减少到几秒）
    // 2. IPNS PubSub 是自动的：如果节点支持，发布 IPNS 时会自动使用 PubSub
    // 3. 我们可以通过检查 IPNS PubSub 订阅列表来判断节点是否支持
    
    // 检查节点是否支持 IPNS PubSub
    let url = format!("{}/api/v0/name/pubsub/subs", normalize_base_url(api_url));
    let response = client
        .post(&url)
        .header("User-Agent", "Alou-Desktop/1.0")
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await;
    
    match response {
        Ok(resp) if resp.status().is_success() => {
            // 节点支持 IPNS PubSub，记录将自动通过 PubSub 快速传播
            info!(target: "diap", "节点支持 IPNS PubSub，IPNS 记录将通过 PubSub 快速传播");
            Ok(())
        }
        _ => {
            // 节点可能不支持 IPNS PubSub 或未启用，将回退到 DHT 传播
            // 这是正常的，DHT 传播仍然有效，只是速度较慢（几分钟到几十分钟）
            Err("节点不支持 IPNS PubSub，将使用 DHT 传播（正常，但速度较慢）".to_string())
        }
    }
}

/// 订阅 IPNS PubSub 主题以实时接收 IPNS 记录更新
/// 这对于两个 Agent 实时对话很重要，可以实时接收对方的 IPNS 记录更新
async fn subscribe_ipns_pubsub(api_url: &str, ipns_key: &str) -> Result<(), String> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .no_proxy()
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    // IPNS PubSub 主题格式是 /ipns/<key>
    let topic = format!("/ipns/{}", ipns_key);
    let url = format!(
        "{}/api/v0/pubsub/sub?arg={}",
        normalize_base_url(api_url),
        urlencoding::encode(&topic)
    );

    let response = client
        .post(&url)
        .header("User-Agent", "Alou-Desktop/1.0")
        .send()
        .await
        .map_err(|e| format!("IPNS PubSub 订阅请求失败: {}", e))?;

    if response.status().is_success() {
        info!(target: "diap", "成功订阅 IPNS PubSub 主题: {}", topic);
        Ok(())
    } else {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        Err(format!("IPNS PubSub 订阅失败: {} - {}", status, text))
    }
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
        .publish_ipns_direct(&cid, &ipns_key, "8760h", "24h")
        .await
    {
        Ok(result) => {
            info!(target: "diap", "✅ IPNS 发布成功（在线模式，已传播到DHT，lifetime=1年）: {}", result.name);
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
            warn!(target: "diap", "降级使用普通 IPNS 发布（allow-offline=true，lifetime=1年）...");
            ipfs_client
                .publish_ipns(&cid, &ipns_key, "8760h", "24h")
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

    // 注意：IPNS 记录的传播是通过 publish_ipns_direct 使用 allow-offline=false 自动完成的
    // IPFS 会自动将 IPNS 记录传播到 DHT 网络，无需手动调用 dht/provide
    // dht/provide API 只能用于 CID，不能用于 IPNS 名称
    // IPNS 记录会在 DHT 网络中自动传播，通常需要几分钟到几十分钟时间
    
    // 尝试启用 IPNS PubSub 以加速 IPNS 记录的传播（从几分钟减少到几秒）
    // 注意：这需要 IPFS 节点支持 PubSub，如果失败也不影响功能（会回退到 DHT 传播）
    info!(target: "diap", "检查 IPNS PubSub 支持以加速传播...");
    if let Err(e) = enable_ipns_pubsub(&ipfs_api).await {
        warn!(target: "diap", "IPNS PubSub 不可用（不影响功能，将使用 DHT 传播）: {}", e);
    }

    // 主动触发公共网关查询（加速传播技巧 + 重试预热）
    // 当向公共网关发起请求时，这些大型网关会主动去 DHT 中寻找 IPNS 记录
    // 这相当于强制触发了 DHT 的查询过程，可以大大缩短全球生效时间
    // 公共网关通常拥有极高的带宽和海量的 Peer 连接，一旦找到记录会协助缓存并进一步扩散
    // 使用重试预热：每隔 30 秒触发一次，共触发 3 次，因为第一次触发时网关可能还没在 DHT 中搜到，第二次请求通常就能命中
    info!(target: "diap", "启动公共网关查询预热（将触发 3 次，间隔 30 秒）...");
    let ipns_path_for_trigger = ipns_path.clone();
    tokio::spawn(async move {
        trigger_public_gateway_query_with_retry(&ipns_path_for_trigger, 3, 30).await;
    });

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

/// 测试 IPNS 在公共网关上的可访问性
/// 使用并发请求和 HEAD 方法优化性能
#[tauri::command]
pub async fn test_ipns_on_public_gateway(ipns_name: String) -> Result<serde_json::Value, String> {
    // 规范化 IPNS 名称
    let ipns_name_clean = ipns_name.trim();
    let ipns_path = if ipns_name_clean.starts_with("/ipns/") {
        ipns_name_clean.to_string()
    } else {
        format!("/ipns/{}", ipns_name_clean)
    };
    
    // 公共网关列表
    let public_gateways: Vec<String> = vec![
        "https://gateway.ipfs.io".to_string(),
        "https://ipfs.io".to_string(),
        "https://dweb.link".to_string(),
        "https://cloudflare-ipfs.com".to_string(),
    ];
    
    // 保存网关数量用于后续日志
    let gateway_count = public_gateways.len();
    
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;
    
    // 创建并发测试任务
    let test_futures: Vec<_> = public_gateways
        .into_iter()
        .map(|gateway| {
            let gateway_url = format!("{}{}", gateway, ipns_path);
            let client_clone = client.clone();
            let gateway_clone = gateway.clone();
            
            async move {
                let gateway = gateway_clone.as_str();
                let client = client_clone;
                let mut gateway_result = serde_json::json!({
                    "gateway": gateway,
                    "url": gateway_url.clone(),
                    "accessible": false,
                    "status": 0,
                    "status_text": "unknown",
                    "error": serde_json::Value::Null
                });
                
                info!(target: "diap", "测试公共网关: {} -> {}", gateway, gateway_url);
                
                // 使用 HEAD 请求（更高效，不下载内容）
                match client
                    .head(&gateway_url)
                    .timeout(std::time::Duration::from_secs(20))
                    .send()
                    .await
                {
                    Ok(response) => {
                        let status = response.status();
                        let status_code = status.as_u16();
                        
                        gateway_result["status"] = serde_json::json!(status_code);
                        
                        match status_code {
                            200 => {
                                gateway_result["accessible"] = serde_json::json!(true);
                                gateway_result["status_text"] = serde_json::json!("已就绪");
                                info!(target: "diap", "✅ {} 可访问，IPNS 记录已就绪", gateway);
                            }
                            404 => {
                                gateway_result["accessible"] = serde_json::json!(false);
                                gateway_result["status_text"] = serde_json::json!("未找到");
                                gateway_result["error"] = serde_json::json!("网关未找到 IPNS 记录，可能尚未传播到此网关");
                                warn!(target: "diap", "⚠️ {} 返回 404，IPNS 记录未找到", gateway);
                            }
                            400 | 403 => {
                                gateway_result["accessible"] = serde_json::json!(false);
                                gateway_result["status_text"] = serde_json::json!("访问被拒绝");
                                gateway_result["error"] = serde_json::json!(format!("网关拒绝访问 (HTTP {})，可能是内容格式错误或被屏蔽", status_code));
                                warn!(target: "diap", "❌ {} 访问被拒绝，状态码: {}", gateway, status_code);
                            }
                            504 | 502 | 503 => {
                                gateway_result["accessible"] = serde_json::json!(false);
                                gateway_result["status_text"] = serde_json::json!("同步中");
                                gateway_result["error"] = serde_json::json!("网关超时或服务不可用，IPNS 记录可能正在同步中，请保持节点在线");
                                warn!(target: "diap", "⚠️ {} 返回 {}，IPNS 记录同步中（请保持节点在线）", gateway, status_code);
                            }
                            _ => {
                                gateway_result["accessible"] = serde_json::json!(false);
                                gateway_result["status_text"] = serde_json::json!("访问受阻");
                                gateway_result["error"] = serde_json::json!(format!("HTTP {}", status_code));
                                warn!(target: "diap", "❌ {} 访问受阻，状态码: {}", gateway, status_code);
                            }
                        }
                    }
                    Err(e) => {
                        gateway_result["accessible"] = serde_json::json!(false);
                        gateway_result["status"] = serde_json::json!(0);
                        
                        // 区分超时和其他网络错误
                        let error_str = e.to_string();
                        if error_str.contains("timeout") || error_str.contains("timed out") {
                            gateway_result["status_text"] = serde_json::json!("同步中 (Timeout)");
                            gateway_result["error"] = serde_json::json!("请求超时，IPNS 记录可能正在同步中，请保持节点在线");
                            warn!(target: "diap", "⏱️ {} 请求超时，IPNS 记录同步中", gateway);
                        } else {
                            gateway_result["status_text"] = serde_json::json!("网络错误");
                            gateway_result["error"] = serde_json::json!(error_str);
                            warn!(target: "diap", "❌ {} 网络错误: {}", gateway, e);
                        }
                    }
                }
                
                gateway_result
            }
        })
        .collect();
    
    // 并发执行所有测试（使用 tokio::spawn 并发执行所有 future）
    let handles: Vec<_> = test_futures.into_iter()
        .map(|f| tokio::spawn(f))
        .collect();
    
    let mut gateway_results = Vec::new();
    for handle in handles {
        match handle.await {
            Ok(result) => gateway_results.push(result),
            Err(e) => {
                warn!(target: "diap", "测试任务执行失败: {}", e);
                gateway_results.push(serde_json::json!({
                    "gateway": "unknown",
                    "url": "",
                    "accessible": false,
                    "status": 0,
                    "status_text": "执行失败",
                    "error": format!("任务执行失败: {}", e)
                }));
            }
        }
    }
    
    // 计算成功数量和同步中的数量
    let accessible_count = gateway_results
        .iter()
        .filter(|g| g["accessible"].as_bool().unwrap_or(false))
        .count();
    
    let syncing_count = gateway_results
        .iter()
        .filter(|g| {
            let status_text = g["status_text"].as_str().unwrap_or("");
            status_text == "同步中" || status_text == "同步中 (Timeout)"
        })
        .count();
    
    // 生成友好的提示消息
    let (message, user_friendly_message) = if accessible_count > 0 {
        (
            format!("IPNS 记录可在 {}/{} 个公共网关上访问", accessible_count, gateway_count),
            None
        )
    } else if syncing_count > 0 {
        (
            format!("IPNS 记录正在同步中（{}/{} 个网关）", syncing_count, gateway_count),
            Some("🌍 正在全球同步节点信息... 您的身份已在本地创建，全球生效可能需要 1-3 分钟。".to_string())
        )
    } else {
        (
            "IPNS 记录尚未在公共网关上可访问，可能需要等待传播（通常需要几分钟到几十分钟）".to_string(),
            Some("🌍 正在全球同步节点信息... 您的身份已在本地创建，全球生效可能需要 1-3 分钟。".to_string())
        )
    };
    
    let mut summary = serde_json::json!({
        "total_gateways": gateway_count,
        "accessible_count": accessible_count,
        "syncing_count": syncing_count,
        "all_accessible": accessible_count == gateway_count,
        "message": message
    });
    
    if let Some(user_msg) = user_friendly_message {
        summary["user_friendly_message"] = serde_json::json!(user_msg);
    }
    
    let results = serde_json::json!({
        "ipns_name": ipns_path,
        "gateways": gateway_results,
        "summary": summary
    });
    
    Ok(results)
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
        .and_then(|svc| {
            svc.get("serviceEndpoint")
                .and_then(|ep| ep.as_object())
                .and_then(|ep_obj| ep_obj.get("pubsubTopics"))
                .and_then(|topics| topics.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|t| t.as_str().map(|s| s.to_string()))
                        .collect::<Vec<String>>()
                })
        })
        .unwrap_or_default();
    
    // 如果解析的身份有 PubSub 主题，尝试订阅 IPNS PubSub 以获取实时更新
    // 这对于两个 Agent 实时对话很重要，可以实时接收对方的 IPNS 记录更新
    if !pubsub_topics.is_empty() {
        info!(target: "diap", "检测到 PubSub 主题，尝试订阅 IPNS PubSub 以增强实时性...");
        let ipns_name_clean = ipns_name.trim();
        let ipns_key = ipns_name_clean.trim_start_matches("/ipns/").to_string();
        let ipfs_api_clone = ipfs_api.clone();
        
        // 在后台订阅 IPNS PubSub（不阻塞主流程）
        tokio::spawn(async move {
            if let Err(e) = subscribe_ipns_pubsub(&ipfs_api_clone, &ipns_key).await {
                warn!(target: "diap", "订阅 IPNS PubSub 失败（不影响功能）: {}", e);
            } else {
                info!(target: "diap", "✅ 已订阅 IPNS PubSub，将实时接收更新");
            }
        });
    }

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
        pubsub_topics: Some(pubsub_topics),
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
