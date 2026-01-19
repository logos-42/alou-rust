// IPFS commands module - 专用于DIAP身份创建的IPFS操作
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use reqwest::{multipart::Form, multipart::Part, Client};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::utils::{default_ipfs_api_url, normalize_base_url};

/// 创建配置好的 IPFS HTTP 客户端
fn create_ipfs_client() -> Client {
    Client::builder()
        .http1_only() // 强制使用 HTTP/1.1，因为 IPFS 可能不支持 HTTP/2
        .timeout(std::time::Duration::from_secs(60)) // 增加超时时间，IPNS发布需要更长时间
        .connect_timeout(std::time::Duration::from_secs(10))
        .tcp_keepalive(std::time::Duration::from_secs(60))
        .no_proxy() // 避免通过系统代理访问本地 API
        .build()
        .unwrap_or_else(|_| Client::new()) // 如果构建失败，回退到默认客户端
}

#[derive(Serialize)]
pub struct IpfsAddResult {
    pub cid: String,
    pub size: Option<String>,
    pub name: Option<String>,
}

#[derive(Deserialize)]
struct IpfsAddApiResponse {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Hash")]
    hash: String,
    #[serde(rename = "Size")]
    size: String,
}

#[derive(Deserialize)]
struct IpnsPublishResponse {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Value")]
    value: String,
}

/// 专用于DID文档的IPFS上传函数
pub async fn add_did_document_to_ipfs(
    api_url: &str,
    did_document: &serde_json::Value,
) -> Result<IpfsAddResult, String> {
    let endpoint = format!("{}/api/v0/add", normalize_base_url(api_url));
    
    // 序列化DID文档为JSON字符串
    let json_str = serde_json::to_string_pretty(did_document)
        .map_err(|e| format!("序列化DID文档失败: {}", e))?;
    
    let bytes = json_str.as_bytes().to_vec();
    let file_name = format!("did_document_{}.json", Uuid::new_v4());
    
    let part = Part::bytes(bytes)
        .file_name(file_name.clone())
        .mime_str("application/json")
        .map_err(|e| format!("创建文件部分失败: {}", e))?;
    
    let form = Form::new().part("file", part);

    let client = create_ipfs_client();
    let response = client
        .post(&endpoint)
        .header("User-Agent", "Alou-Desktop/1.0")
        .query(&[("pin", "true"), ("wrap-with-directory", "false")])
        .multipart(form)
        .send()
        .await
        .map_err(|e| {
            format!(
                "IPFS DID文档上传请求失败: {}. 请确保 IPFS 节点正在运行 (API: {})",
                e, api_url
            )
        })?;

    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|e| format!("IPFS DID文档上传响应错误: {}", e))?;

    if !status.is_success() {
        let status_code = status.as_u16();
        let error_msg = if text.is_empty() {
            "响应为空".to_string()
        } else {
            text.trim().to_string()
        };
        
        if status_code == 502 {
            return Err(format!(
                "IPFS API 服务尚未就绪 (502 Bad Gateway). IPFS 节点正在启动中，请等待几秒钟后重试。"
            ));
        }
        
        return Err(format!(
            "IPFS DID文档上传失败，状态码 {}: {}. 请检查 IPFS 节点 (API: {})",
            status_code,
            error_msg,
            api_url
        ));
    }

    if text.trim().is_empty() {
        return Err(format!(
            "IPFS DID文档上传响应为空. 请检查 IPFS 节点 (API: {})",
            api_url
        ));
    }

    // 解析IPFS响应（可能是多行JSON）
    let last_line = text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .last()
        .ok_or_else(|| {
            format!(
                "IPFS DID文档上传响应中没有有效内容. 原始响应: {}",
                if text.len() > 200 {
                    format!("{}...", &text[..200])
                } else {
                    text.clone()
                }
            )
        })?;

    let parsed: IpfsAddApiResponse = serde_json::from_str(last_line)
        .map_err(|e| {
            format!(
                "无法解析 IPFS DID文档上传响应: {}. 响应内容: {}",
                e,
                if last_line.len() > 200 {
                    format!("{}...", &last_line[..200])
                } else {
                    last_line.to_string()
                }
            )
        })?;

    println!("✅ DID文档已上传到IPFS: CID = {}, 大小 = {}", parsed.hash, parsed.size);

    Ok(IpfsAddResult {
        cid: parsed.hash,
        size: Some(parsed.size),
        name: Some(parsed.name),
    })
}

/// 专用于DIAP身份的IPNS发布函数
pub async fn publish_diap_identity_to_ipns(
    api_url: &str,
    cid: &str,
    ipns_key_name: &str,
) -> Result<String, String> {
    let endpoint = format!("{}/api/v0/name/publish", normalize_base_url(api_url));
    
    let client = create_ipfs_client();
    
    // 构建请求参数
    let params = [
        ("arg", cid),
        ("key", ipns_key_name),
        ("lifetime", "24h"), // 24小时生命周期
        ("ttl", "1h"),       // 1小时TTL
        ("resolve", "true"), // 立即解析验证
    ];
    
    println!("🚀 开始发布DIAP身份到IPNS: CID={}, Key={}", cid, ipns_key_name);
    
    let response = client
        .post(&endpoint)
        .form(&params)
        .header("User-Agent", "Alou-Desktop/1.0")
        .send()
        .await
        .map_err(|e| format!("IPNS发布请求失败: {}", e))?;

    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|e| format!("IPNS发布响应错误: {}", e))?;

    if !status.is_success() {
        let status_code = status.as_u16();
        let error_msg = if text.is_empty() {
            "响应为空".to_string()
        } else {
            text.trim().to_string()
        };
        
        return Err(format!(
            "IPNS发布失败，状态码 {}: {}. 请检查IPNS密钥是否存在 (API: {})",
            status_code,
            error_msg,
            api_url
        ));
    }

    let parsed: IpnsPublishResponse = serde_json::from_str(&text)
        .map_err(|e| {
            format!(
                "无法解析IPNS发布响应: {}. 响应内容: {}",
                e,
                if text.len() > 200 {
                    format!("{}...", &text[..200])
                } else {
                    text.clone()
                }
            )
        })?;

    let ipns_name = format!("/ipns/{}", parsed.name);
    
    println!("✅ DIAP身份已发布到IPNS: {} -> {}", ipns_name, parsed.value);

    Ok(ipns_name)
}

pub async fn add_bytes_to_ipfs(
    api_url: &str,
    bytes: Vec<u8>,
    file_name: Option<String>,
) -> Result<IpfsAddResult, String> {
    let endpoint = format!("{}/api/v0/add", normalize_base_url(api_url));
    let name = file_name.unwrap_or_else(|| format!("payload-{}.bin", Uuid::new_v4()));
    let part = Part::bytes(bytes).file_name(name.clone());
    let form = Form::new().part("file", part);

    let client = create_ipfs_client();
    let response = client
        .post(&endpoint)
        .header("User-Agent", "Alou-Desktop/1.0")
        .query(&[("pin", "true")])
        .multipart(form)
        .send()
        .await
        .map_err(|e| {
            format!(
                "IPFS add request failed: {}. 请确保 IPFS 节点正在运行 (API: {})",
                e, api_url
            )
        })?;

    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|e| format!("IPFS add response error: {}", e))?;

    if !status.is_success() {
        let status_code = status.as_u16();
        let error_msg = if text.is_empty() {
            "响应为空".to_string()
        } else {
            text.trim().to_string()
        };
        
        // 502 错误通常表示 IPFS 守护进程启动了但 API 服务还未就绪
        if status_code == 502 {
            return Err(format!(
                "IPFS API 服务尚未就绪 (502 Bad Gateway). 这通常意味着 IPFS 节点正在启动中，请等待几秒钟后重试。如果问题持续，请检查：\n1. IPFS 节点是否正常运行\n2. API 端口 {} 是否可访问\n3. 是否有多个 IPFS 节点实例在运行 (API: {})",
                api_url.replace("/api/v0/add", ""),
                api_url
            ));
        }
        
        return Err(format!(
            "IPFS API 返回错误状态码 {}: {}. 请检查 IPFS 节点是否正常运行 (API: {})",
            status_code,
            error_msg,
            api_url
        ));
    }

    if text.trim().is_empty() {
        return Err(format!(
            "IPFS add 响应为空. 请检查 IPFS 节点是否正常运行 (API: {})",
            api_url
        ));
    }

    let last_line = text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .last()
        .ok_or_else(|| {
            format!(
                "IPFS add 响应中没有有效内容. 原始响应: {}. 请检查 IPFS 节点 (API: {})",
                if text.len() > 200 {
                    format!("{}...", &text[..200])
                } else {
                    text.clone()
                },
                api_url
            )
        })?;

    let parsed: IpfsAddApiResponse = serde_json::from_str(last_line)
        .map_err(|e| {
            format!(
                "无法解析 IPFS add 响应: {}. 响应内容: {}. 请检查 IPFS 节点 (API: {})",
                e,
                if last_line.len() > 200 {
                    format!("{}...", &last_line[..200])
                } else {
                    last_line.to_string()
                },
                api_url
            )
        })?;

    Ok(IpfsAddResult {
        cid: parsed.hash,
        size: Some(parsed.size),
        name: Some(parsed.name),
    })
}

#[allow(dead_code)] // Exposed for future IPNS key management commands
pub async fn generate_ipns_key(
    api_url: &str,
    key_name: &str,
) -> Result<String, String> {
    let endpoint = format!("{}/api/v0/key/gen", normalize_base_url(api_url));
    let client = create_ipfs_client();
    
    let request = client
        .post(endpoint)
        .header("User-Agent", "Alou-Desktop/1.0")
        .query(&[("arg", key_name), ("type", "rsa"), ("size", "2048")]);

    let response = request
        .send()
        .await
        .map_err(|e| format!("IPNS key generation request failed: {}", e))?;

    let body = response
        .text()
        .await
        .map_err(|e| format!("IPNS key generation response error: {}", e))?;

    // Handle case where key already exists
    if body.contains("already exists") {
        return Ok(key_name.to_string());
    }

    #[derive(Deserialize)]
    struct KeyGenResponse {
        #[serde(rename = "Name")]
        name: String,
        #[serde(rename = "Id")]
        #[allow(dead_code)]
        id: String,
    }

    let key_gen: KeyGenResponse = serde_json::from_str(&body)
        .map_err(|e| format!("Failed to parse IPNS key generation response: {}. Response: {}", e, 
            if body.len() > 200 {
                format!("{}...", &body[..200])
            } else {
                body.clone()
            }))?;

    Ok(key_gen.name)
}

#[allow(dead_code)] // Not wired into the UI yet; keep available for future IPNS features
pub async fn publish_ipns_record(
    api_url: &str,
    cid: &str,
    ipns_key: Option<String>,
) -> Result<String, String> {
    let endpoint = format!("{}/api/v0/name/publish", normalize_base_url(api_url));
    let client = create_ipfs_client();
    let mut request = client
        .post(endpoint)
        .header("User-Agent", "Alou-Desktop/1.0")
        .query(&[("arg", format!("/ipfs/{}", cid))]);

    if let Some(key) = ipns_key {
        request = request.query(&[("key", key)]);
    }

    let response = request
        .send()
        .await
        .map_err(|e| format!("IPNS publish request failed: {}", e))?;

    let body = response
        .text()
        .await
        .map_err(|e| format!("IPNS publish response error: {}", e))?;

    let publish: IpnsPublishResponse = serde_json::from_str(&body)
        .map_err(|e| format!("Failed to parse IPNS publish response: {}", e))?;

    Ok(publish.name)
}

#[tauri::command]
pub async fn ipfs_add_base64(
    data_base64: String,
    file_name: Option<String>,
    ipfs_api_url: Option<String>,
) -> Result<IpfsAddResult, String> {
    let bytes = BASE64
        .decode(data_base64.trim())
        .map_err(|e| format!("Invalid base64 payload: {}", e))?;
    let api_url = ipfs_api_url.unwrap_or_else(default_ipfs_api_url);
    add_bytes_to_ipfs(&api_url, bytes, file_name).await
}



// ==================== PubSub Commands ====================

/// PubSub 消息结构体（预留，用于未来更完整的消息处理）
#[allow(dead_code)]
#[derive(Serialize, Deserialize, Debug)]
pub struct PubSubMessage {
    pub data: String,
    pub from: Option<String>,
    pub seqno: Option<String>,
    pub topic_ids: Option<Vec<String>>,
}

/// 发布消息到 IPFS PubSub 主题
#[tauri::command]
pub async fn ipfs_pubsub_publish(
    topic: String,
    message: String,
    ipfs_api_url: Option<String>,
) -> Result<bool, String> {
    let api_url = ipfs_api_url.unwrap_or_else(default_ipfs_api_url);
    let endpoint = format!("{}/api/v0/pubsub/pub", normalize_base_url(&api_url));
    
    let client = create_ipfs_client();
    
    // URL encode the message
    let encoded_message = urlencoding::encode(&message).into_owned();
    
    let response = client
        .post(&endpoint)
        .header("User-Agent", "Alou-Desktop/1.0")
        .query(&[("arg", topic.as_str()), ("arg", encoded_message.as_str())])
        .send()
        .await
        .map_err(|e| format!("PubSub publish failed: {}", e))?;
    
    if response.status().is_success() {
        Ok(true)
    } else {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        Err(format!("PubSub publish failed with status {}: {}", status, text))
    }
}

/// 订阅 IPFS PubSub 主题并获取一次消息（带超时）
#[tauri::command]
pub async fn ipfs_pubsub_subscribe_once(
    topic: String,
    timeout_ms: Option<u64>,
    ipfs_api_url: Option<String>,
) -> Result<Vec<String>, String> {
    let api_url = ipfs_api_url.unwrap_or_else(default_ipfs_api_url);
    let endpoint = format!("{}/api/v0/pubsub/sub", normalize_base_url(&api_url));
    let timeout = timeout_ms.unwrap_or(1000);
    
    let client = Client::builder()
        .http1_only()
        .timeout(std::time::Duration::from_millis(timeout))
        .connect_timeout(std::time::Duration::from_millis(timeout))
        .no_proxy()
        .build()
        .unwrap_or_else(|_| Client::new());
    
    let response = client
        .post(&endpoint)
        .header("User-Agent", "Alou-Desktop/1.0")
        .query(&[("arg", &topic)])
        .send()
        .await;
    
    match response {
        Ok(resp) => {
            if resp.status().is_success() {
                let text = resp.text().await.unwrap_or_default();
                // PubSub 返回的是 NDJSON 格式
                let messages: Vec<String> = text
                    .lines()
                    .filter(|line| !line.trim().is_empty())
                    .filter_map(|line| {
                        // 尝试解析 JSON 并提取 data 字段
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                            if let Some(data) = json.get("data").and_then(|d| d.as_str()) {
                                // data 是 base64 编码的
                                if let Ok(decoded) = BASE64.decode(data) {
                                    if let Ok(s) = String::from_utf8(decoded) {
                                        return Some(s);
                                    }
                                }
                            }
                        }
                        None
                    })
                    .collect();
                Ok(messages)
            } else {
                Ok(vec![])
            }
        }
        Err(_) => {
            // 超时或连接错误，返回空列表（不是错误）
            Ok(vec![])
        }
    }
}

/// 获取 PubSub 主题的订阅者列表
#[tauri::command]
pub async fn ipfs_pubsub_peers(
    topic: Option<String>,
    ipfs_api_url: Option<String>,
) -> Result<Vec<String>, String> {
    let api_url = ipfs_api_url.unwrap_or_else(default_ipfs_api_url);
    let endpoint = format!("{}/api/v0/pubsub/peers", normalize_base_url(&api_url));
    
    let client = create_ipfs_client();
    let mut request = client
        .post(&endpoint)
        .header("User-Agent", "Alou-Desktop/1.0");
    
    if let Some(t) = topic {
        request = request.query(&[("arg", t)]);
    }
    
    let response = request
        .send()
        .await
        .map_err(|e| format!("PubSub peers request failed: {}", e))?;
    
    if response.status().is_success() {
        let text = response.text().await.unwrap_or_default();
        
        #[derive(Deserialize)]
        struct PeersResponse {
            #[serde(rename = "Strings")]
            strings: Option<Vec<String>>,
        }
        
        let parsed: PeersResponse = serde_json::from_str(&text)
            .unwrap_or(PeersResponse { strings: None });
        
        Ok(parsed.strings.unwrap_or_default())
    } else {
        Ok(vec![])
    }
}

/// 获取当前订阅的主题列表
#[tauri::command]
pub async fn ipfs_pubsub_ls(
    ipfs_api_url: Option<String>,
) -> Result<Vec<String>, String> {
    let api_url = ipfs_api_url.unwrap_or_else(default_ipfs_api_url);
    let endpoint = format!("{}/api/v0/pubsub/ls", normalize_base_url(&api_url));
    
    let client = create_ipfs_client();
    let response = client
        .post(&endpoint)
        .header("User-Agent", "Alou-Desktop/1.0")
        .send()
        .await
        .map_err(|e| format!("PubSub ls request failed: {}", e))?;
    
    if response.status().is_success() {
        let text = response.text().await.unwrap_or_default();
        
        #[derive(Deserialize)]
        struct LsResponse {
            #[serde(rename = "Strings")]
            strings: Option<Vec<String>>,
        }
        
        let parsed: LsResponse = serde_json::from_str(&text)
            .unwrap_or(LsResponse { strings: None });
        
        Ok(parsed.strings.unwrap_or_default())
    } else {
        Ok(vec![])
    }
}
