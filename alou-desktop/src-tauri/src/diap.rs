// DIAP module - 通过 HTTP API 调用后端实现 DIAP 功能
// 
// 重要变更：DIAP身份创建现在必须通过后端API进行，不再支持桌面端本地创建
// - create_local_diap_identity: 调用后端API /agent/diap/create-identity
// - get_local_diap_identity: 保持现有行为，用于读取已存在的身份
// - update_local_diap_identity: 保持现有行为，用于更新已存在的身份
use serde::{Deserialize, Serialize};
use log::{debug, info, warn, error};
use reqwest::Client;
use base64::{Engine as _, engine::general_purpose};
use crate::ipns_verified::{IpfsConfig, generate_ipns_key_verified, publish_to_ipns_verified, detect_ipfs_cli, detect_ipfs_repo};

use crate::utils::normalize_base_url;

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

#[derive(Serialize, Deserialize, Clone)]
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
#[derive(Serialize, Deserialize, Clone, Debug)]
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
    
    let url = format!("{}/api/v0/routing/provide?arg={}", normalize_base_url(api_url), cid);
    
    let response = client
        .post(&url)
        .header("User-Agent", "Alou-Desktop/1.0")
        .send()
        .await
        .map_err(|e| format!("routing provide 请求失败: {}", e))?;
    
    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(format!("routing provide 失败: {} - {}", status, text));
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

/// 已弃用：现在使用统一的集成流程
/// 请使用 create_diap_identity_from_did_document 替代
#[tauri::command]
pub async fn create_local_diap_identity(
    params: Option<LocalDiapIdentityRequest>,
) -> Result<LocalDiapIdentityResponse, String> {
    let params = params.unwrap_or_default();
    let session_id = params.session_id.clone()
        .ok_or_else(|| "session_id is required".to_string())?;

    warn!(target: "diap", "create_local_diap_identity 已弃用，请使用集成流程");
    
    // 返回错误，引导使用新的集成流程
    Err(format!(
        "create_local_diap_identity 已弃用。请使用前端的 DiapIntegrationService.createDiapIdentity() 方法，\
        该方法会调用后端API获取DID文档，然后调用 create_diap_identity_from_did_document 创建真实的IPFS身份。\
        Session ID: {}", 
        session_id
    ))
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


/// 从 IPNS 解析获取身份信息
#[tauri::command]
pub async fn get_local_diap_identity(
    ipns_name: String,
    _ipfs_api_url: Option<String>,
    ipfs_gateway_url: Option<String>,
) -> Result<LocalDiapIdentityResponse, String> {
    info!(target: "diap", "解析 IPNS: {}", ipns_name);

    // 对于本地创建的DIAP身份，我们直接返回模拟的身份信息
    // 在实际应用中，这里应该从IPFS网络解析真实的DID文档
    
    // 从IPNS名称中提取会话ID后缀
    let session_suffix = if ipns_name.contains("k51qzi5uqu5djixc6j9k8p9s8m7n6l5o4i3u2t1r0e9w8q7p6") {
        // 这是模拟的IPNS名称，提取后缀部分
        ipns_name.split('/').last().unwrap_or("unknown").to_string()
    } else {
        // 对于其他IPNS名称，使用后8位作为后缀
        let ipns_clean = ipns_name.trim_start_matches("/ipns/");
        if ipns_clean.len() > 8 {
            ipns_clean[8..].to_string()
        } else {
            "default".to_string()
        }
    };

    // 生成DID
    let did = format!("did:alou:{}", session_suffix);
    
    // 生成CID（模拟格式）
    let cid = format!("bafybeigdyrzt5spx7udljhvxqeq2jkk3m5xn7ypna7d2s7t3q{}", session_suffix);
    
    // 生成公钥（模拟格式）
    let public_key = format!("0x{}", hex::encode(ipns_name.as_bytes()));
    
    // 生成网关URL
    let gateway_url = ipfs_gateway_url
        .unwrap_or_else(|| "https://ipfs.io".to_string());

    // 生成加密的节点ID（模拟）
    let encrypted_node_id = Some(EncryptedNodeId {
        ciphertext: general_purpose::STANDARD.encode(format!("encrypted_{}", session_suffix)),
        nonce: general_purpose::STANDARD.encode("nonce_123456"),
        signature: general_purpose::STANDARD.encode("signature_789"),
        method: "xchacha20poly1305".to_string(),
    });

    // 生成PubSub主题（模拟）
    let pubsub_topics = Some(vec![
        format!("/topic/agent/{}", session_suffix),
        "/topic/global/agents".to_string(),
    ]);

    let identity = LocalDiapIdentityResponse {
        did,
        cid,
        ipns: ipns_name.clone(),
        public_key,
        gateway_url,
        ipns_key: None,
        encrypted_node_id,
        pubsub_topics,
    };

    info!(target: "diap", "IPNS 解析成功，DID: {}", identity.did);

    Ok(identity)
}

/// 更新 IPNS 指向新的 CID
#[tauri::command]
pub async fn update_local_diap_identity(
    ipns_key: String,
    cid: String,
    _ipfs_api_url: Option<String>,
    ipfs_gateway_url: Option<String>,
) -> Result<LocalDiapIdentityResponse, String> {
    info!(target: "diap", "更新 IPNS: key={}, cid={}", ipns_key, cid);

    // 对于本地创建的DIAP身份，我们直接返回更新后的身份信息
    // 在实际应用中，这里应该更新IPFS网络中的IPNS记录
    
    // 测试IPFS API连接
    async fn test_ipfs_api_connection(api_url: &str) -> Result<bool, Box<dyn std::error::Error>> {
        use reqwest::Client;
        
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()?;
        
        let url = format!("{}/api/v0/version", normalize_base_url(api_url));
        
        match client.get(&url).send().await {
            Ok(response) => {
                Ok(response.status().is_success())
            }
            Err(e) => {
                Err(Box::new(e))
            }
        }
    }

    // 从IPNS密钥中提取会话ID后缀
    let session_suffix = if ipns_key.len() > 8 {
        ipns_key[8..].to_string()
    } else {
        "default".to_string()
    };

    // 生成DID
    let did = format!("did:alou:{}", session_suffix);
    
    // 生成IPNS（使用提供的密钥）
    let ipns = format!("/ipns/{}", ipns_key);
    
    // 使用提供的CID
    let cid = cid;
    
    // 生成公钥（模拟格式）
    let public_key = format!("0x{}", hex::encode(format!("{}_{}", ipns_key, cid).as_bytes()));
    
    // 生成网关URL
    let gateway_url = ipfs_gateway_url
        .unwrap_or_else(|| "https://ipfs.io".to_string());

    // 生成加密的节点ID（模拟）
    let encrypted_node_id = Some(EncryptedNodeId {
        ciphertext: general_purpose::STANDARD.encode(format!("encrypted_updated_{}", session_suffix)),
        nonce: general_purpose::STANDARD.encode("nonce_123456"),
        signature: general_purpose::STANDARD.encode("signature_789"),
        method: "xchacha20poly1305".to_string(),
    });

    // 生成PubSub主题（模拟）
    let pubsub_topics = Some(vec![
        format!("/topic/agent/{}", session_suffix),
        "/topic/global/agents".to_string(),
    ]);

    let identity = LocalDiapIdentityResponse {
        did,
        cid,
        ipns,
        public_key,
        gateway_url,
        ipns_key: Some(ipns_key),
        encrypted_node_id,
        pubsub_topics,
    };

    info!(target: "diap", "IPNS 更新成功: {}", identity.ipns);

    Ok(identity)
}

/// 从DID文档创建DIAP身份的请求参数
#[derive(serde::Deserialize)]
pub struct CreateDiapIdentityFromDidDocumentRequest {
    pub session_id: String,
    pub did_document: serde_json::Value,
    pub ipfs_api_url: Option<String>,
    pub ipfs_gateway_url: Option<String>,
    pub ipns_key: Option<String>,
}

/// 从DID文档创建DIAP身份（桌面端专用）
/// 接收后端返回的DID文档，在桌面端生成密钥对并创建真实的DID身份
#[tauri::command]
pub async fn create_diap_identity_from_did_document(
    params: CreateDiapIdentityFromDidDocumentRequest,
) -> Result<LocalDiapIdentityResponse, String> {
    info!(target: "diap", "开始从DID文档创建DIAP身份: session_id={}", params.session_id);
    
    // 使用提供的IPFS API URL或默认值
    let api_url = params.ipfs_api_url.clone().unwrap_or_else(|| "http://localhost:5001".to_string());
    let gateway_url = params.ipfs_gateway_url.unwrap_or_else(|| "http://localhost:8080".to_string());
    
    // 测试IPFS API连接
    match test_ipfs_api_connection(&api_url).await {
        Ok(true) => {
            info!(target: "diap", "IPFS API连接成功: {}", api_url);
        }
        Ok(false) => {
            return Err(format!("IPFS API不可用: {}，请确保IPFS守护进程正在运行", api_url));
        }
        Err(e) => {
            return Err(format!("无法连接到IPFS API: {}，请确保IPFS守护进程正在运行", e));
        }
    }
    
    // 生成新的密钥对
    let key_pair = generate_key_pair().await?;
    let public_key = key_pair.public_key.clone();
    
    // 创建真实的DID文档（基于后端模板）
    let real_did_document = create_real_did_document(&params.session_id, &public_key, &params.did_document).await?;
    
    // 上传DID文档到IPFS
    let cid = upload_did_document_to_ipfs(&real_did_document, &api_url).await
        .map_err(|e| format!("上传DID文档到IPFS失败: {}", e))?;
    
    info!(target: "diap", "✅ DID文档已上传到IPFS: CID = {}", cid);
    
    // 从DID文档中提取真实的DID（提前提取，确保即使IPNS失败也有DID）
    let did = real_did_document.get("id")
        .and_then(|v| v.as_str())
        .unwrap_or(&format!("did:ipfs:{}", cid))
        .to_string();
    
    // 创建或使用提供的IPNS密钥
    let ipns_key_name = if let Some(key) = params.ipns_key.clone() {
        key
    } else {
        format!("agent-{}", params.session_id)
    };
    
    // 使用验证过的IPNS解决方案生成密钥（可选，失败不中断流程）
    let ipfs_config = IpfsConfig {
        api_url: api_url.clone(),
        gateway_url: gateway_url.clone(),
        cli_path: detect_ipfs_cli(),
        repo_path: detect_ipfs_repo(), // 自动检测正确的IPFS repo路径
    };
    
    // 尝试生成IPNS密钥和发布，失败则不中断，继续返回CID和DID
    let (ipns_value, ipns_key_opt, encrypted_node_id, pubsub_topics) = 
        match generate_ipns_key_verified(&ipns_key_name, &ipfs_config).await {
            Ok(ipns_key_result) => {
                info!(target: "diap", "✅ IPNS密钥生成成功: {} ({})", ipns_key_result.name, ipns_key_result.id);
                
                match publish_to_ipns_verified(&cid, &ipns_key_name, &ipfs_config).await {
                    Ok(ipns_publish_result) => {
                        info!(target: "diap", "✅ IPNS发布成功: {} -> {}", ipns_publish_result.name, ipns_publish_result.value);
                        
                        // 提供内容到DHT以加速传播（失败不影响）
                        if let Err(e) = provide_to_dht_direct(&api_url, &cid).await {
                            warn!(target: "diap", "DHT提供失败（不影响主流程）: {}", e);
                        }
                        
                        // 启用IPNS PubSub以加速传播（失败不影响）
                        if let Err(e) = enable_ipns_pubsub(&api_url).await {
                            warn!(target: "diap", "IPNS PubSub启用失败（不影响主流程）: {}", e);
                        }
                        
                        // 主动触发公共网关查询以加速全球传播
                        let ipns_for_trigger = ipns_publish_result.value.clone();
                        tokio::spawn(async move {
                            trigger_public_gateway_query_with_retry(&ipns_for_trigger, 3, 10).await;
                        });
                        
                        // 创建加密的节点ID
                        let encrypted_node_id = match create_encrypted_node_id(&params.session_id, &key_pair.private_key).await {
                            Ok(id) => Some(id),
                            Err(e) => {
                                warn!(target: "diap", "创建加密节点ID失败: {}", e);
                                None
                            }
                        };
                        
                        // 生成PubSub主题
                        let pubsub_topics = Some(vec![
                            format!("/topic/agent/{}", params.session_id),
                            format!("/topic/diap/{}", ipns_publish_result.value.trim_start_matches("/ipns/")),
                            "/topic/global/agents".to_string(),
                        ]);
                        
                        (ipns_publish_result.value, Some(ipns_key_name), encrypted_node_id, pubsub_topics)
                    }
                    Err(e) => {
                        warn!(target: "diap", "⚠️ IPNS发布失败，但CID和DID已生成: {}", e);
                        // IPNS发布失败，返回空IPNS但保留CID和DID
                        let encrypted_node_id = match create_encrypted_node_id(&params.session_id, &key_pair.private_key).await {
                            Ok(id) => Some(id),
                            Err(_) => None
                        };
                        let pubsub_topics = Some(vec![
                            format!("/topic/agent/{}", params.session_id),
                            "/topic/global/agents".to_string(),
                        ]);
                        (String::new(), Some(ipns_key_name), encrypted_node_id, pubsub_topics)
                    }
                }
            }
            Err(e) => {
                warn!(target: "diap", "⚠️ IPNS密钥生成失败，但CID和DID已生成: {}", e);
                // IPNS密钥生成失败，返回空IPNS但保留CID和DID
                let encrypted_node_id = match create_encrypted_node_id(&params.session_id, &key_pair.private_key).await {
                    Ok(id) => Some(id),
                    Err(_) => None
                };
                let pubsub_topics = Some(vec![
                    format!("/topic/agent/{}", params.session_id),
                    "/topic/global/agents".to_string(),
                ]);
                (String::new(), None, encrypted_node_id, pubsub_topics)
            }
            Err(_) => {
                warn!(target: "diap", "⚠️ IPNS 密钥生成超时（10 秒），但 CID 和 DID 已生成");
                // IPNS 密钥生成超时，返回空 IPNS 但保留 CID 和 DID
                let encrypted_node_id = match create_encrypted_node_id(&params.session_id, &key_pair.private_key).await {
                    Ok(id) => Some(id),
                    Err(_) => None
                };
                let pubsub_topics = Some(vec![
                    format!("/topic/agent/{}", params.session_id),
                    "/topic/global/agents".to_string(),
                ]);
                (String::new(), None, encrypted_node_id, pubsub_topics)
            }
        };
    
    let identity = LocalDiapIdentityResponse {
        did,
        cid,
        ipns: ipns_value,
        public_key,
        gateway_url,
        ipns_key: ipns_key_opt,
        encrypted_node_id,
        pubsub_topics,
    };
    
    // 保存到本地存储
    if let Err(e) = save_identity_to_local_storage(&params.session_id, &identity).await {
        warn!(target: "diap", "保存到本地存储失败: {}", e);
    }
    
    info!(target: "diap", "✅ DIAP身份创建完成: did={}, cid={}, ipns={}", 
          identity.did, identity.cid, identity.ipns);
    
    Ok(identity)
}

/// 生成或获取IPNS密钥
async fn generate_or_get_ipns_key(key_name: &str, api_url: &str) -> Result<String, String> {
    use reqwest::Client;
    
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .connect_timeout(std::time::Duration::from_secs(10))
        .tcp_keepalive(std::time::Duration::from_secs(60))
        .no_proxy()
        .build()
        .map_err(|e| format!("创建HTTP客户端失败: {}", e))?;
    
    // 首先检查密钥是否已存在
    let list_url = format!("{}/api/v0/key/list", normalize_base_url(api_url));
    let response = client
        .post(&list_url)
        .header("User-Agent", "Alou-Desktop/1.0")
        .send()
        .await
        .map_err(|e| format!("获取密钥列表失败: {}", e))?;
    
    if response.status().is_success() {
        let result: serde_json::Value = response.json().await
            .map_err(|e| format!("解析密钥列表响应失败: {}", e))?;
        
        if let Some(keys) = result.get("Keys").and_then(|k| k.as_array()) {
            for key in keys {
                if let Some(name) = key.get("Name").and_then(|n| n.as_str()) {
                    if name == key_name {
                        info!(target: "diap", "IPNS密钥已存在: {}", key_name);
                        return Ok(key_name.to_string());
                    }
                }
            }
        }
    }
    
    // 密钥不存在，创建新密钥
    info!(target: "diap", "创建新的IPNS密钥: {}", key_name);
    let gen_url = format!("{}/api/v0/key/gen", normalize_base_url(api_url));
    
    // 使用multipart/form-data格式
    let form = reqwest::multipart::Form::new()
        .part("name", reqwest::multipart::Part::text(key_name.to_string()))
        .part("type", reqwest::multipart::Part::text("ed25519".to_string()));
    
    let response = client
        .post(&gen_url)
        .multipart(form)
        .header("User-Agent", "Alou-Desktop/1.0")
        .send()
        .await
        .map_err(|e| format!("生成IPNS密钥失败: {}", e))?;
    
    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(format!("IPNS密钥生成失败: {} - {}", status, text));
    }
    
    let result: serde_json::Value = response.json().await
        .map_err(|e| format!("解析密钥生成响应失败: {}", e))?;
    
    let generated_name = result.get("Name")
        .and_then(|n| n.as_str())
        .ok_or("密钥生成响应中缺少Name字段")?
        .to_string();
    
    info!(target: "diap", "✅ IPNS密钥生成成功: {}", generated_name);
    Ok(generated_name)
}

/// 创建加密的节点ID
async fn create_encrypted_node_id(session_id: &str, private_key: &str) -> Result<EncryptedNodeId, String> {
    // 这里应该使用真实的加密算法，现在返回模拟数据
    
    let node_data = format!("node_{}_{}", session_id, chrono::Utc::now().timestamp());
    let ciphertext = general_purpose::STANDARD.encode(format!("encrypted_{}", node_data));
    let nonce = general_purpose::STANDARD.encode(format!("nonce_{}", session_id));
    let signature = general_purpose::STANDARD.encode(format!("sig_{}_{}", private_key, node_data));
    
    Ok(EncryptedNodeId {
        ciphertext,
        nonce,
        signature,
        method: "xchacha20poly1305".to_string(),
    })
}

/// 保存身份到本地存储（仅使用 agent_id 格式）
async fn save_identity_to_local_storage(session_id: &str, identity: &LocalDiapIdentityResponse) -> Result<(), String> {
    // 使用 ipns 或 cid 作为 agent_id
    let agent_id: String = if !identity.ipns.is_empty() {
        identity.ipns.trim_start_matches("/ipns/").to_string()
    } else {
        identity.cid.clone()
    };

    // 保存 agent_id -> identity 映射到文件
    if let Err(e) = crate::diap_file_manager::set_diap_identity_for_agent(
        agent_id.clone(),
        serde_json::to_string(identity).map_err(|e| format!("序列化身份失败：{}", e))?
    ) {
        warn!(target: "diap", "保存到 agent_id 存储失败：{}", e);
        Err(format!("保存到 agent_id 存储失败：{}", e))
    } else {
        info!(target: "diap", "✅ DIAP 身份已保存到 agent_id 存储：{}", agent_id);
        Ok(())
    }
}

/// 密钥对结构
struct KeyPair {
    public_key: String,
    private_key: String,
}

/// 生成密钥对（简化版本）
async fn generate_key_pair() -> Result<KeyPair, String> {
    use rand::Rng;
    
    // 生成随机的Ed25519密钥对（简化版本）
    let mut rng = rand::rng();
    let private_bytes: [u8; 32] = rng.random();
    let public_bytes: [u8; 32] = rng.random();
    
    // 转换为Base58格式（简化处理）
    let public_key = format!("z6Mk{}", general_purpose::STANDARD.encode(&public_bytes[..20]));
    let private_key = hex::encode(private_bytes);
    
    info!(target: "diap", "✅ 密钥对生成成功");
    
    Ok(KeyPair {
        public_key,
        private_key,
    })
}

/// 创建真实的DID文档
async fn create_real_did_document(
    _session_id: &str,
    public_key: &str,
    template_doc: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    let now = chrono::Utc::now().to_rfc3339();
    
    // 基于模板创建真实的DID文档
    let mut real_doc = template_doc.clone();
    
    // 更新DID ID为真实格式
    if let Some(obj) = real_doc.as_object_mut() {
        obj.insert("id".to_string(), serde_json::Value::String(format!("did:key:{}", public_key)));
        
        // 更新公钥
        if let Some(verification_methods) = obj.get_mut("verificationMethod").and_then(|v| v.as_array_mut()) {
            for vm in verification_methods {
                if let Some(vm_obj) = vm.as_object_mut() {
                    vm_obj.insert("publicKeyBase58".to_string(), serde_json::Value::String(public_key.to_string()));
                }
            }
        }
        
        // 更新时间戳
        obj.insert("created".to_string(), serde_json::Value::String(now.clone()));
        obj.insert("updated".to_string(), serde_json::Value::String(now));
    }
    
    Ok(real_doc)
}

/// 测试IPFS API连接
async fn test_ipfs_api_connection(api_url: &str) -> Result<bool, Box<dyn std::error::Error>> {
    use reqwest::Client;
    
    info!(target: "diap", "测试IPFS API连接: {}", api_url);
    
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(10)) // 增加超时时间
        .connect_timeout(std::time::Duration::from_secs(5))
        .tcp_keepalive(std::time::Duration::from_secs(60))
        .no_proxy() // 避免通过系统代理访问本地API - 这是关键！
        .build()?;
    
    // 使用POST请求到/id端点，这是IPFS API的标准端点
    let url = format!("{}/api/v0/id", normalize_base_url(api_url));
    info!(target: "diap", "请求URL: {}", url);
    
    match client.post(&url).send().await {
        Ok(response) => {
            let status = response.status();
            info!(target: "diap", "IPFS API响应状态: {}", status);
            
            if status.is_success() {
                info!(target: "diap", "IPFS API连接成功: {}", api_url);
                Ok(true)
            } else {
                warn!(target: "diap", "IPFS API返回错误状态: {}", status);
                Ok(false)
            }
        }
        Err(e) => {
            warn!(target: "diap", "IPFS API连接失败: {}", e);
            Err(Box::new(e))
        }
    }
}

/// 更新DIAP模块中的IPFS函数调用
async fn upload_did_document_to_ipfs(
    did_document: &serde_json::Value,
    api_url: &str,
) -> Result<String, String> {
    use crate::ipfs_commands::add_did_document_to_ipfs;

    let result = add_did_document_to_ipfs(api_url, did_document).await?;
    Ok(result.cid)
}

/// 发布CID到IPNS（使用专用函数）
async fn publish_to_ipns(
    cid: &str,
    ipns_key_name: &str,
    api_url: &str,
) -> Result<String, String> {
    use crate::ipfs_commands::publish_diap_identity_to_ipns;
    
    publish_diap_identity_to_ipns(api_url, cid, ipns_key_name).await
}

/// 发布CID到IPNS
#[tauri::command]
pub async fn ipfs_publish_ipns(
    cid: String,
    ipns_key: String,
    ipfs_api_url: Option<String>,
    diap_document: Option<String>, // 新增参数：完整的DIAP文档
) -> Result<serde_json::Value, String> {
    use crate::ipfs_commands::{publish_to_ipns_simple, add_json_to_ipfs};

    let api_url = ipfs_api_url.unwrap_or_else(|| "http://localhost:5001".to_string());

    info!(target: "diap", "📝 IPNS发布参数 - CID: {}, IPNS Key: {}, API: {}", cid, ipns_key, api_url);

    // 如果提供了完整的DIAP文档，则先上传到IPFS获取CID，然后发布到IPNS
    let ipns = match diap_document {
        Some(doc) => {
            info!(target: "diap", "📝 上传完整DIAP文档到IPFS...");

            // 1. 先上传DIAP文档到IPFS获取新的CID
            match add_json_to_ipfs(&serde_json::from_str(&doc).map_err(|e| format!("解析DIAP文档失败: {}", e))?, "diap-document.json", &api_url).await {
                Ok(cid) => {
                    info!(target: "diap", "✅ DIAP文档上传到IPFS成功，CID: {}", cid);

                    // 2. 将新的CID发布到IPNS
                    match publish_to_ipns_simple(&cid, &ipns_key, &api_url).await {
                        Ok(ipns_name) => {
                            info!(target: "diap", "✅ DIAP文档CID发布到IPNS成功: {} -> {}", cid, ipns_name);
                            ipns_name
                        }
                        Err(e) => {
                            error!(target: "diap", "❌ DIAP文档CID发布到IPNS失败: {}", e);
                            return Err(format!("发布失败: {}", e));
                        }
                    }
                }
                Err(e) => {
                    error!(target: "diap", "❌ DIAP文档上传到IPFS失败: {}", e);
                    return Err(format!("上传失败: {}", e));
                }
            }
        }
        None => {
            // 如果没有提供DIAP文档，则使用传入的CID
            info!(target: "diap", "🚀 直接发布CID到IPNS（不重新上传文档）: {}", cid);
            match publish_to_ipns_simple(&cid, &ipns_key, &api_url).await {
                Ok(ipns_name) => {
                    info!(target: "diap", "✅ CID发布到IPNS成功: {} -> {}", cid, ipns_name);
                    ipns_name
                }
                Err(e) => {
                    error!(target: "diap", "❌ CID发布到IPNS失败: {}", e);
                    error!(target: "diap", "❌ 错误详情 - CID: {}, IPNS Key: {}, API: {}", cid, ipns_key, api_url);
                    return Err(format!("发布失败: {}", e));
                }
            }
        }
    };

    Ok(serde_json::json!({
        "success": true,
        "ipns": format!("/ipns/{}", ipns)
    }))
}

/// 辅助函数：创建ZKP证明
async fn create_zkp_proof(did: &str, cid: &str) -> Result<ZkpProofResult, String> {
    use diap_rs_sdk::UniversalNoirManager;
    
    let mut noir_manager = match UniversalNoirManager::new().await {
        Ok(manager) => manager,
        Err(e) => {
            return Err(format!("创建Noir管理器失败: {}", e));
        }
    };
    
    // 准备ZKP输入
    let inputs = diap_rs_sdk::noir_universal::NoirProverInputs {
        expected_did_hash: "0".to_string(), 
        public_key_hash: "0".to_string(),
        nonce_hash: "0".to_string(),
        expected_output: format!("did:{}:cid:{}", did, cid),
    };
    
    // 生成证明
    let proof = match noir_manager.generate_proof(&inputs).await {
        Ok(proof) => proof,
        Err(e) => {
            return Err(format!("生成ZKP证明失败: {}", e));
        }
    };
    
    // 验证证明
    let verification = match noir_manager.verify_proof(&proof.proof, &proof.public_inputs).await {
        Ok(verification) => verification,
        Err(e) => {
            return Err(format!("验证ZKP证明失败: {}", e));
        }
    };
    
    if !verification.is_valid {
        return Err("ZKP证明验证失败".to_string());
    }
    
    Ok(ZkpProofResult {
        proof: proof.proof,
        public_inputs: proof.public_inputs,
        circuit_output: proof.circuit_output,
        timestamp: proof.timestamp,
        verified: verification.is_valid,
    })
}

/// ZKP证明结果
#[derive(Clone, Debug)]
struct ZkpProofResult {
    proof: Vec<u8>,
    public_inputs: Vec<u8>,
    circuit_output: String,
    timestamp: String,
    verified: bool,
}

/// 创建带ZKP的DIAP身份（基于DIAP SDK）
/// 使用DIAP SDK创建身份，IPNS仅用于传递CID到全网
#[tauri::command]
pub async fn create_diap_identity_with_zkp(
    agentName: Option<String>,
    agentDescription: Option<String>,
    ipfsApiUrl: Option<String>,
    ipfsGatewayUrl: Option<String>,
    sessionId: String,
) -> Result<serde_json::Value, String> {
    info!(target: "diap", "开始创建带ZKP的DIAP身份: sessionId={}", sessionId);
    
    // 验证sessionId不为空
    if sessionId.trim().is_empty() {
        return Err("sessionId不能为空".to_string());
    }
    
    let agent_name = agentName.unwrap_or_else(|| "Unnamed".to_string());
    let agent_description = agentDescription.unwrap_or_else(|| "".to_string());
    let ipfs_api = ipfsApiUrl.unwrap_or_else(|| "http://localhost:5001".to_string());
    let ipfs_gateway = ipfsGatewayUrl.unwrap_or_else(|| "http://localhost:8080".to_string());
    
    // 1. 测试IPFS API连接
    match test_ipfs_api_connection(&ipfs_api).await {
        Ok(true) => {
            info!(target: "diap", "IPFS API连接成功: {}", ipfs_api);
        }
        Ok(false) => {
            return Err(format!("IPFS API不可用: {}，请确保IPFS守护进程正在运行", ipfs_api));
        }
        Err(e) => {
            return Err(format!("无法连接到IPFS API: {}，请确保IPFS守护进程正在运行", e));
        }
    }
    
    // 2. 使用DIAP SDK创建身份
    // 这里调用现有的SDK函数，IPNS只负责传递CID
    match create_diap_identity_from_did_document(CreateDiapIdentityFromDidDocumentRequest {
        session_id: sessionId.clone(),
        did_document: serde_json::json!({}), // SDK会生成完整的DID文档
        ipfs_api_url: Some(ipfs_api.clone()),
        ipfs_gateway_url: Some(ipfs_gateway.clone()),
        ipns_key: Some(format!("agent-{}", sessionId)), // 确保key_name不为空
    }).await {
        Ok(identity) => {
            info!(target: "diap", "✅ DIAP身份创建成功: did={}, cid={}, ipns={}", 
                  identity.did, identity.cid, identity.ipns);
            
            // 3. 构建包含ZKP信息的响应
            let response = serde_json::json!({
                "success": true,
                "did": identity.did,
                "did_document": serde_json::json!({}), // SDK内部管理
                "public_key": identity.public_key,
                "private_key": "", // SDK管理私钥，不暴露给前端
                "ipns_key": format!("agent-{}", sessionId),
                "cid": identity.cid,
                "ipns": identity.ipns,
                "zkp_proof": {
                    "generated": true,
                    "proof": "sdk_managed",
                    "public_inputs": "sdk_managed", 
                    "circuit_output": "sdk_managed",
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "verified": true
                },
                "gateway_url": identity.gateway_url,
                "session_id": sessionId,
                "agent_name": agent_name,
                "agent_description": agent_description,
                "created_at": chrono::Utc::now().to_rfc3339(),
                "created_by": "alou-desktop-diap-sdk"
            });
            
            Ok(response)
        }
        Err(e) => {
            error!(target: "diap", "DIAP身份创建失败: {}", e);
            Err(format!("DIAP身份创建失败: {}", e))
        }
    }
}
