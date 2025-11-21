// IPFS API diagnostic and testing module
use reqwest::Client;
use serde_json;
use tauri::State;

use crate::ipfs_node::IpfsState;
use crate::utils::{app_data_dir, default_ipfs_api_url, normalize_base_url};

/// 从 IPFS 配置中读取 API 地址
pub async fn get_ipfs_api_address_from_config(
    state: &tauri::async_runtime::Mutex<IpfsState>,
    app: &tauri::AppHandle,
) -> Result<String, String> {
    let ipfs_state = state.lock().await;
    let app_data_dir = app_data_dir(app)?;
    
    let data_dir = if ipfs_state.data_dir.as_os_str().is_empty() {
        app_data_dir.join("ipfs")
    } else {
        ipfs_state.data_dir.clone()
    };
    
    let config_file = data_dir.join("config");
    if !config_file.exists() {
        return Err("IPFS 配置文件不存在".to_string());
    }
    
    let config_content = std::fs::read_to_string(&config_file)
        .map_err(|e| format!("无法读取 IPFS 配置: {}", e))?;
    
    let config: serde_json::Value = serde_json::from_str(&config_content)
        .map_err(|e| format!("无法解析 IPFS 配置: {}", e))?;
    
    // 读取 Addresses.API 配置
    if let Some(addresses) = config.get("Addresses").and_then(|a| a.get("API")) {
        // API 可能是字符串或数组
        if let Some(api_addr) = addresses.as_str() {
            // IPFS 配置中的地址格式通常是 "/ip4/127.0.0.1/tcp/5001"
            // 需要转换为 "http://127.0.0.1:5001"
            if api_addr.starts_with("/ip4/") {
                let parts: Vec<&str> = api_addr.split('/').collect();
                if parts.len() >= 5 {
                    let ip = parts[2];
                    let port = parts[4];
                    return Ok(format!("http://{}:{}", ip, port));
                }
            } else if api_addr.starts_with("http://") || api_addr.starts_with("https://") {
                return Ok(api_addr.to_string());
            }
        } else if let Some(api_addrs) = addresses.as_array() {
            // 如果是数组，取第一个非空地址
            for addr in api_addrs {
                if let Some(addr_str) = addr.as_str() {
                    if !addr_str.is_empty() && addr_str != "null" {
                        if addr_str.starts_with("/ip4/") {
                            let parts: Vec<&str> = addr_str.split('/').collect();
                            if parts.len() >= 5 {
                                let ip = parts[2];
                                let port = parts[4];
                                return Ok(format!("http://{}:{}", ip, port));
                            }
                        } else if addr_str.starts_with("http://") || addr_str.starts_with("https://") {
                            return Ok(addr_str.to_string());
                        }
                    }
                }
            }
        }
    }
    
    Err("无法从 IPFS 配置中读取 API 地址".to_string())
}

/// 诊断 IPFS API 配置问题
#[tauri::command]
pub async fn diagnose_ipfs_api(
    state: State<'_, tauri::async_runtime::Mutex<IpfsState>>,
    app: tauri::AppHandle,
) -> Result<serde_json::Value, String> {
    let ipfs_state = state.lock().await;
    let app_data_dir = app_data_dir(&app)?;
    
    let data_dir = if ipfs_state.data_dir.as_os_str().is_empty() {
        app_data_dir.join("ipfs")
    } else {
        ipfs_state.data_dir.clone()
    };
    
    let mut diagnosis = serde_json::json!({
        "config_exists": false,
        "api_address": null,
        "api_enabled": false,
        "recommendations": []
    });
    
    let config_file = data_dir.join("config");
    if !config_file.exists() {
        diagnosis["recommendations"] = serde_json::json!([
            "IPFS 配置文件不存在，请先初始化 IPFS 节点"
        ]);
        return Ok(diagnosis);
    }
    
    diagnosis["config_exists"] = serde_json::json!(true);
    
    let config_content = match std::fs::read_to_string(&config_file) {
        Ok(content) => content,
        Err(e) => {
            diagnosis["recommendations"] = serde_json::json!([
                format!("无法读取 IPFS 配置: {}", e)
            ]);
            return Ok(diagnosis);
        }
    };
    
    let config: serde_json::Value = match serde_json::from_str(&config_content) {
        Ok(config) => config,
        Err(e) => {
            diagnosis["recommendations"] = serde_json::json!([
                format!("无法解析 IPFS 配置: {}", e)
            ]);
            return Ok(diagnosis);
        }
    };
    
    // 检查 API 地址配置
    if let Some(addresses) = config.get("Addresses").and_then(|a| a.get("API")) {
        if addresses.is_null() {
            diagnosis["recommendations"] = serde_json::json!([
                "IPFS API 地址被设置为 null，API 已被禁用",
                "请运行: ipfs config Addresses.API /ip4/127.0.0.1/tcp/5001"
            ]);
        } else if let Some(api_addrs) = addresses.as_array() {
            if api_addrs.is_empty() {
                diagnosis["recommendations"] = serde_json::json!([
                    "IPFS API 地址数组为空，API 已被禁用",
                    "请运行: ipfs config Addresses.API /ip4/127.0.0.1/tcp/5001"
                ]);
            } else {
                diagnosis["api_enabled"] = serde_json::json!(true);
                if let Some(api_addr) = api_addrs.get(0).and_then(|a| a.as_str()) {
                    diagnosis["api_address"] = serde_json::json!(api_addr);
                }
            }
        } else if let Some(api_addr) = addresses.as_str() {
            if api_addr.is_empty() || api_addr == "null" {
                diagnosis["recommendations"] = serde_json::json!([
                    "IPFS API 地址为空，API 已被禁用",
                    "请运行: ipfs config Addresses.API /ip4/127.0.0.1/tcp/5001"
                ]);
            } else {
                diagnosis["api_enabled"] = serde_json::json!(true);
                diagnosis["api_address"] = serde_json::json!(api_addr);
            }
        }
    } else {
        diagnosis["recommendations"] = serde_json::json!([
            "IPFS 配置中缺少 Addresses.API 设置",
            "请运行: ipfs config Addresses.API /ip4/127.0.0.1/tcp/5001"
        ]);
    }
    
    // 尝试从配置读取 API 地址
    match get_ipfs_api_address_from_config(state.inner(), &app).await {
        Ok(url) => {
            diagnosis["api_address_http"] = serde_json::json!(url);
        }
        Err(_) => {}
    }
    
    // 测试 API 是否可访问
    let test_url = if let Some(url) = diagnosis.get("api_address_http").and_then(|u| u.as_str()) {
        url.to_string()
    } else {
        default_ipfs_api_url()
    };
    
    match test_ipfs_api_ready(&test_url).await {
        Ok(true) => {
            diagnosis["api_accessible"] = serde_json::json!(true);
        }
        Ok(false) => {
            diagnosis["api_accessible"] = serde_json::json!(false);
            diagnosis["recommendations"] = serde_json::json!([
                "API 配置存在但无法访问",
                "请检查 IPFS 节点是否正在运行",
                format!("尝试访问: {}", test_url)
            ]);
        }
        Err(e) => {
            diagnosis["api_accessible"] = serde_json::json!(false);
            let mut recs = vec![
                "API 无法访问".to_string(),
                format!("错误: {}", e),
                format!("尝试访问: {}", test_url)
            ];
            if let Some(existing) = diagnosis.get("recommendations").and_then(|r| r.as_array()) {
                recs.extend(existing.iter().filter_map(|v| v.as_str().map(|s| s.to_string())));
            }
            diagnosis["recommendations"] = serde_json::json!(recs);
        }
    }
    
    Ok(diagnosis)
}

/// 创建配置好的 IPFS HTTP 客户端
fn create_ipfs_client() -> Client {
    Client::builder()
        .http1_only() // 强制使用 HTTP/1.1，因为 IPFS 可能不支持 HTTP/2
        .timeout(std::time::Duration::from_secs(10)) // 增加超时时间
        .connect_timeout(std::time::Duration::from_secs(5))
        .tcp_keepalive(std::time::Duration::from_secs(60))
        .no_proxy() // 避免通过系统代理访问 127.0.0.1
        .build()
        .unwrap_or_else(|_| Client::new()) // 如果构建失败，回退到默认客户端
}

/// 测试 IPFS HTTP API 是否就绪，返回详细结果
pub async fn test_ipfs_api_ready_detailed(api_url: &str) -> Result<serde_json::Value, String> {
    let endpoint = format!("{}/api/v0/version", normalize_base_url(api_url));
    let client = create_ipfs_client();
    
    match client
        .post(&endpoint) // 大多数 IPFS API 期望 POST 请求
        .header("User-Agent", "Alou-Desktop/1.0")
        .send()
        .await
    {
        Ok(response) => {
            let status = response.status();
            let status_code = status.as_u16();
            
            // 尝试读取响应体
            let response_text = response.text().await.unwrap_or_else(|_| "无法读取响应体".to_string());
            
            if status.is_success() {
                Ok(serde_json::json!({
                    "success": true,
                    "status_code": status_code,
                    "response": response_text
                }))
            } else {
                // 502 错误通常表示 API 服务器正在启动中
                if status_code == 502 {
                    return Err(format!(
                        "IPFS API 服务器正在启动中 (502 Bad Gateway). 这通常发生在 IPFS 守护进程刚启动时。请等待几秒钟后重试。如果问题持续，请检查：\n1. IPFS 守护进程是否正常运行\n2. 是否有多个 IPFS 实例在运行\n3. API 端口 {} 是否被其他程序占用",
                        api_url
                    ));
                }
                
                Err(format!(
                    "IPFS API 返回错误状态码 {}: {}. 请检查 IPFS 配置中的 API.HTTPHeaders.Access-Control-Allow-Origin 设置（如果是 CORS 问题）。",
                    status_code,
                    if response_text.len() > 200 {
                        format!("{}...", &response_text[..200])
                    } else {
                        response_text
                    }
                ))
            }
        }
        Err(e) => {
            // 连接错误或超时，提供更详细的错误信息
            let error_msg = e.to_string();
            if error_msg.contains("Connection refused") {
                Err(format!("IPFS API 连接被拒绝。请检查：\n1. IPFS 节点是否正在运行\n2. API 地址是否正确: {}\n3. 防火墙是否阻止了连接", api_url))
            } else if error_msg.contains("timeout") {
                Err(format!("IPFS API 连接超时。请检查：\n1. IPFS 节点是否正在运行\n2. API 地址是否正确: {}", api_url))
            } else {
                Err(format!("IPFS API 连接失败: {} (地址: {})", error_msg, api_url))
            }
        }
    }
}

/// 测试 IPFS HTTP API 是否就绪（简化版本，用于向后兼容）
pub async fn test_ipfs_api_ready(api_url: &str) -> Result<bool, String> {
    match test_ipfs_api_ready_detailed(api_url).await {
        Ok(result) => Ok(result.get("success").and_then(|s| s.as_bool()).unwrap_or(false)),
        Err(e) => Err(e)
    }
}

#[tauri::command]
pub async fn get_ipfs_api_address(
    state: State<'_, tauri::async_runtime::Mutex<IpfsState>>,
    app: tauri::AppHandle,
) -> Result<String, String> {
    get_ipfs_api_address_from_config(state.inner(), &app).await
}

#[tauri::command]
pub async fn test_ipfs_api(
    ipfs_api_url: Option<String>,
) -> Result<serde_json::Value, String> {
    let api_url = ipfs_api_url.unwrap_or_else(default_ipfs_api_url);
    
    match test_ipfs_api_ready_detailed(&api_url).await {
        Ok(result) => {
            let mut response = serde_json::json!({
                "success": true,
                "api_url": api_url
            });
            if let Some(status_code) = result.get("status_code") {
                response["status_code"] = status_code.clone();
            }
            if let Some(resp_text) = result.get("response") {
                response["response"] = resp_text.clone();
            }
            Ok(response)
        }
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "api_url": api_url,
            "error": e
        }))
    }
}

#[tauri::command]
pub async fn test_ipfs_api_with_config(
    state: State<'_, tauri::async_runtime::Mutex<IpfsState>>,
    app: tauri::AppHandle,
    ipfs_api_url: Option<String>,
) -> Result<serde_json::Value, String> {
    let api_url = if let Some(url) = ipfs_api_url {
        url
    } else {
        // 尝试从 IPFS 配置中读取
        match get_ipfs_api_address_from_config(state.inner(), &app).await {
            Ok(url) => url,
            Err(_) => default_ipfs_api_url(),
        }
    };
    
    match test_ipfs_api_ready_detailed(&api_url).await {
        Ok(result) => {
            let mut response = serde_json::json!({
                "success": true,
                "api_url": api_url
            });
            if let Some(status_code) = result.get("status_code") {
                response["status_code"] = status_code.clone();
            }
            if let Some(resp_text) = result.get("response") {
                response["response"] = resp_text.clone();
            }
            Ok(response)
        }
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "api_url": api_url,
            "error": e
        }))
    }
}

