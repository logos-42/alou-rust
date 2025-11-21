// IPFS commands module
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use reqwest::{multipart::Form, multipart::Part, Client};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::utils::{default_ipfs_api_url, normalize_base_url};

/// 创建配置好的 IPFS HTTP 客户端
fn create_ipfs_client() -> Client {
    Client::builder()
        .http1_only() // 强制使用 HTTP/1.1，因为 IPFS 可能不支持 HTTP/2
        .timeout(std::time::Duration::from_secs(30)) // 增加超时时间，因为文件上传可能需要更长时间
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
    #[allow(dead_code)]
    value: String, // IPNS 响应中的 Value 字段，保留以备将来使用
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

