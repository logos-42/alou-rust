// DIAP module - 通过 HTTP API 调用后端实现 DIAP 功能
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::utils::normalize_base_url;
use log::{debug, info, warn};
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
    let session_id = params.session_id.clone()
        .ok_or_else(|| "session_id is required".to_string())?;

    info!(target: "diap", "开始创建 DIAP Identity for session: {}", session_id);

    // 调用后端 API 创建 DIAP 身份
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    // 构建请求体
    let request_body = json!({
        "session_id": session_id,
        "agent_name": params.agent_name,
        "agent_description": params.agent_description,
        "ipfs_api_url": params.ipfs_api_url,
        "ipfs_gateway_url": params.ipfs_gateway_url,
        "ipns_key": params.ipns_key,
        "custom_prompt": params.custom_prompt,
        "avatar_cid": params.avatar_cid,
        "mcp_config_cid": params.mcp_config_cid
    });

    // 调用后端 API
    let backend_url = std::env::var("ALOU_EDGE_URL")
        .unwrap_or_else(|_| "https://alou-edge.alou.workers.dev".to_string());

    let response = client
        .post(&format!("{}/agent/diap/create-identity", backend_url))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("Failed to call backend API: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(format!("Backend API error: {} - {}", status, text));
    }

    let response_data: serde_json::Value = response.json().await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    // 解析响应
    let identity: LocalDiapIdentityResponse = serde_json::from_value(response_data["identity"].clone())
        .map_err(|e| format!("Failed to parse identity: {}", e))?;

    info!(target: "diap", "DIAP Identity 创建成功！");
    info!(target: "diap", "  DID: {}", identity.did);
    info!(target: "diap", "  CID: {}", identity.cid);
    info!(target: "diap", "  IPNS: {}", identity.ipns);

    Ok(identity)
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
    ipfs_api_url: Option<String>,
    ipfs_gateway_url: Option<String>,
) -> Result<LocalDiapIdentityResponse, String> {
    info!(target: "diap", "解析 IPNS: {}", ipns_name);

    // 调用后端 API 获取 DIAP 身份
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    // 构建请求体
    let request_body = json!({
        "ipns_name": ipns_name,
        "ipfs_api_url": ipfs_api_url,
        "ipfs_gateway_url": ipfs_gateway_url
    });

    // 调用后端 API
    let backend_url = std::env::var("ALOU_EDGE_URL")
        .unwrap_or_else(|_| "https://alou-edge.alou.workers.dev".to_string());

    let response = client
        .post(&format!("{}/agent/diap/get-identity", backend_url))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("Failed to call backend API: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(format!("Backend API error: {} - {}", status, text));
    }

    let response_data: serde_json::Value = response.json().await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    // 解析响应
    let identity: LocalDiapIdentityResponse = serde_json::from_value(response_data["identity"].clone())
        .map_err(|e| format!("Failed to parse identity: {}", e))?;

    info!(target: "diap", "IPNS 解析成功，DID: {}", identity.did);

    Ok(identity)
}

/// 更新 IPNS 指向新的 CID
#[tauri::command]
pub async fn update_local_diap_identity(
    ipns_key: String,
    cid: String,
    ipfs_api_url: Option<String>,
    ipfs_gateway_url: Option<String>,
) -> Result<LocalDiapIdentityResponse, String> {
    info!(target: "diap", "更新 IPNS: key={}, cid={}", ipns_key, cid);

    // 调用后端 API 更新 DIAP 身份
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    // 构建请求体
    let request_body = json!({
        "ipns_key": ipns_key,
        "cid": cid,
        "ipfs_api_url": ipfs_api_url,
        "ipfs_gateway_url": ipfs_gateway_url
    });

    // 调用后端 API
    let backend_url = std::env::var("ALOU_EDGE_URL")
        .unwrap_or_else(|_| "https://alou-edge.alou.workers.dev".to_string());

    let response = client
        .post(&format!("{}/agent/diap/update-identity", backend_url))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("Failed to call backend API: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(format!("Backend API error: {} - {}", status, text));
    }

    let response_data: serde_json::Value = response.json().await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    // 解析响应
    let identity: LocalDiapIdentityResponse = serde_json::from_value(response_data["identity"].clone())
        .map_err(|e| format!("Failed to parse identity: {}", e))?;

    info!(target: "diap", "IPNS 更新成功: {}", identity.ipns);

    Ok(identity)
}
