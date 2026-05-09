// Browser utilities module
use reqwest::Client;
use serde_json;

/// Open URL in default browser
#[tauri::command]
pub async fn open_browser(url: String) -> Result<(), String> {
    #[cfg(desktop)]
    {
        if let Err(e) = open::that(&url) {
            return Err(format!("Failed to open browser: {}", e));
        }
        Ok(())
    }

    #[cfg(not(desktop))]
    {
        Err("Open browser is only available on desktop".to_string())
    }
}

/// 创建配置好的 IPFS HTTP 客户端
fn create_ipfs_client() -> Client {
    Client::builder()
        .http1_only() // 强制使用 HTTP/1.1，因为 IPFS 可能不支持 HTTP/2
        .timeout(std::time::Duration::from_secs(10))
        .connect_timeout(std::time::Duration::from_secs(5))
        .tcp_keepalive(std::time::Duration::from_secs(60))
        .no_proxy() // 避免使用系统代理访问本地主机
        .build()
        .unwrap_or_else(|_| Client::new())
}

/// 测试连接到指定的 IPFS 节点
#[tauri::command]
pub async fn test_ipfs_node_connection(api_url: Option<String>) -> Result<serde_json::Value, String> {
    // 默认使用配置中的 API 地址
    let api_url = api_url.unwrap_or_else(|| "http://127.0.0.1:5001".to_string());
    
    let client = create_ipfs_client();
    
    // 测试 1: 检查版本信息
    let version_endpoint = format!("{}/api/v0/version", api_url.trim_end_matches('/'));
    let version_result = client
        .post(&version_endpoint) // IPFS HTTP API 通常要求 POST
        .header("User-Agent", "Alou-Desktop/1.0")
        .send()
        .await;
    
    // 测试 2: 检查节点 ID
    let id_endpoint = format!("{}/api/v0/id", api_url.trim_end_matches('/'));
    let id_result = client
        .post(&id_endpoint)
        .header("User-Agent", "Alou-Desktop/1.0")
        .send()
        .await;
    
    // 测试 3: 检查节点信息
    let swarm_peers_endpoint = format!("{}/api/v0/swarm/peers", api_url.trim_end_matches('/'));
    let peers_result = client
        .post(&swarm_peers_endpoint)
        .header("User-Agent", "Alou-Desktop/1.0")
        .send()
        .await;
    
    let mut result = serde_json::json!({
        "api_url": api_url,
        "version_test": {
            "success": false,
            "error": null
        },
        "id_test": {
            "success": false,
            "error": null,
            "peer_id": null
        },
        "peers_test": {
            "success": false,
            "error": null,
            "peer_count": 0
        }
    });
    
    // 处理版本测试结果
    match version_result {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<serde_json::Value>().await {
                    Ok(version_info) => {
                        result["version_test"]["success"] = serde_json::json!(true);
                        result["version_test"]["data"] = version_info;
                    }
                    Err(e) => {
                        result["version_test"]["error"] = serde_json::json!(format!("解析响应失败: {}", e));
                    }
                }
            } else {
                result["version_test"]["error"] = serde_json::json!(format!("HTTP 状态码: {}", response.status()));
            }
        }
        Err(e) => {
            let error_msg = e.to_string();
            if error_msg.contains("Connection refused") {
                result["version_test"]["error"] = serde_json::json!("连接被拒绝 - IPFS 节点可能未运行");
            } else if error_msg.contains("timeout") {
                result["version_test"]["error"] = serde_json::json!("连接超时");
            } else {
                result["version_test"]["error"] = serde_json::json!(error_msg);
            }
        }
    }
    
    // 处理 ID 测试结果
    match id_result {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<serde_json::Value>().await {
                    Ok(id_info) => {
                        result["id_test"]["success"] = serde_json::json!(true);
                        if let Some(peer_id) = id_info.get("ID").and_then(|v| v.as_str()) {
                            result["id_test"]["peer_id"] = serde_json::json!(peer_id);
                        }
                        result["id_test"]["data"] = id_info;
                    }
                    Err(e) => {
                        result["id_test"]["error"] = serde_json::json!(format!("解析响应失败: {}", e));
                    }
                }
            } else {
                result["id_test"]["error"] = serde_json::json!(format!("HTTP 状态码: {}", response.status()));
            }
        }
        Err(e) => {
            result["id_test"]["error"] = serde_json::json!(e.to_string());
        }
    }
    
    // 处理对等节点测试结果
    match peers_result {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<serde_json::Value>().await {
                    Ok(peers_info) => {
                        result["peers_test"]["success"] = serde_json::json!(true);
                        if let Some(peers) = peers_info.get("Peers").and_then(|v| v.as_array()) {
                            result["peers_test"]["peer_count"] = serde_json::json!(peers.len());
                        }
                        result["peers_test"]["data"] = peers_info;
                    }
                    Err(e) => {
                        result["peers_test"]["error"] = serde_json::json!(format!("解析响应失败: {}", e));
                    }
                }
            } else {
                result["peers_test"]["error"] = serde_json::json!(format!("HTTP 状态码: {}", response.status()));
            }
        }
        Err(e) => {
            result["peers_test"]["error"] = serde_json::json!(e.to_string());
        }
    }
    
    // 计算总体连接状态
    let all_tests_passed = result["version_test"]["success"].as_bool().unwrap_or(false)
        && result["id_test"]["success"].as_bool().unwrap_or(false);
    
    result["connection_successful"] = serde_json::json!(all_tests_passed);
    
    Ok(result)
}

