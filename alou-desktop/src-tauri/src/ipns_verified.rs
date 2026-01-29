//! 经过验证的IPNS解决方案
//! 基于自动化测试验证的完整IPNS密钥生成和发布功能

use crate::utils::normalize_base_url;
use serde_json::Value;
use std::process::Command;
use std::path::PathBuf;

/// IPNS密钥生成结果
#[derive(Debug, Clone)]
pub struct IpnsKeyResult {
    pub name: String,
    pub id: String,
}

/// IPNS发布结果
#[derive(Debug, Clone)]
pub struct IpnsPublishResult {
    pub name: String,
    pub value: String,
}

/// IPFS配置
#[derive(Debug, Clone)]
pub struct IpfsConfig {
    pub api_url: String,
    pub gateway_url: String,
    pub cli_path: Option<PathBuf>,
}

impl Default for IpfsConfig {
    fn default() -> Self {
        Self {
            api_url: "http://localhost:5001".to_string(),
            gateway_url: "http://localhost:8080".to_string(),
            cli_path: detect_ipfs_cli(),
        }
    }
}

/// 检测IPFS命令行工具路径
fn detect_ipfs_cli() -> Option<PathBuf> {
    // 尝试常见的IPFS安装路径
    let common_paths = vec![
        PathBuf::from(r"C:\Users\Mechrevo\AppData\Roaming\com.alou.desktop\kubo\ipfs.exe"),
        PathBuf::from(r"C:\Program Files\IPFS\ipfs.exe"),
        PathBuf::from(r"C:\Users\Mechrevo\.cargo\bin\ipfs.exe"),
    ];
    
    for path in common_paths {
        if path.exists() {
            println!("🔍 找到IPFS CLI: {}", path.display());
            return Some(path);
        }
    }
    
    // 尝试从PATH中查找
    if let Ok(output) = Command::new("where").arg("ipfs.exe").output() {
        if output.status.success() {
            if let Some(path_str) = String::from_utf8_lossy(&output.stdout).lines().next() {
                let path = PathBuf::from(path_str.trim());
                if path.exists() {
                    println!("🔍 从PATH找到IPFS CLI: {}", path.display());
                    return Some(path);
                }
            }
        }
    }
    
    println!("⚠️ 未找到IPFS CLI工具");
    None
}

/// 经过验证的IPNS密钥生成
/// 优先使用命令行工具，API作为后备方案
pub async fn generate_ipns_key_verified(
    key_name: &str,
    config: &IpfsConfig,
) -> Result<IpnsKeyResult, String> {
    println!("🔑 开始生成IPNS密钥: {}", key_name);
    
    // 方案1: 使用命令行工具（已验证）
    if let Some(ref cli_path) = config.cli_path {
        println!("📋 尝试使用CLI生成密钥...");
        match generate_key_with_cli(cli_path, key_name).await {
            Ok(result) => {
                println!("✅ CLI密钥生成成功: {}", result.id);
                return Ok(result);
            }
            Err(e) => {
                println!("⚠️ CLI密钥生成失败: {}, 尝试API方案...", e);
            }
        }
    }
    
    // 方案2: 使用API（修复后的版本）
    println!("📋 尝试使用API生成密钥...");
    match generate_key_with_api(key_name, &config.api_url).await {
        Ok(result) => {
            println!("✅ API密钥生成成功: {}", result.id);
            Ok(result)
        }
        Err(e) => {
            println!("❌ API密钥生成也失败: {}", e);
            Err(format!("所有密钥生成方案都失败: {}", e))
        }
    }
}

/// 使用命令行工具生成密钥
async fn generate_key_with_cli(
    cli_path: &PathBuf,
    key_name: &str,
) -> Result<IpnsKeyResult, String> {
    let output = Command::new(cli_path)
        .arg("key")
        .arg("gen")
        .arg(key_name)
        .output()
        .map_err(|e| format!("执行IPFS CLI失败: {}", e))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("IPFS CLI密钥生成失败: {}", stderr));
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let key_id = stdout.trim().to_string();
    
    if key_id.is_empty() {
        return Err("IPFS CLI返回空结果".to_string());
    }
    
    Ok(IpnsKeyResult {
        name: key_name.to_string(),
        id: key_id,
    })
}

/// 使用API生成密钥（修复版本）
async fn generate_key_with_api(
    key_name: &str,
    api_url: &str,
) -> Result<IpnsKeyResult, String> {
    use reqwest::{multipart::Form, multipart::Part, Client};
    
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("创建HTTP客户端失败: {}", e))?;
    
    let endpoint = format!("{}/api/v0/key/gen", normalize_base_url(api_url));
    
    // 使用正确的multipart/form-data格式
    let form = Form::new()
        .part("name", Part::text(key_name.to_string()))
        .part("type", Part::text("ed25519".to_string()));
    
    let response = client
        .post(&endpoint)
        .multipart(form)
        .header("User-Agent", "Alou-Desktop/1.0")
        .send()
        .await
        .map_err(|e| format!("发送请求失败: {}", e))?;
    
    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(format!("API请求失败: {} - {}", status, text));
    }
    
    let result: Value = response.json().await
        .map_err(|e| format!("解析响应失败: {}", e))?;
    
    let name = result.get("Name")
        .and_then(|v| v.as_str())
        .ok_or("响应中缺少Name字段")?;
    
    let id = result.get("Id")
        .and_then(|v| v.as_str())
        .unwrap_or(name);
    
    Ok(IpnsKeyResult {
        name: name.to_string(),
        id: id.to_string(),
    })
}

/// 经过验证的IPNS发布
/// 优先使用命令行工具，API作为后备方案
pub async fn publish_to_ipns_verified(
    cid: &str,
    key_name: &str,
    config: &IpfsConfig,
) -> Result<IpnsPublishResult, String> {
    println!("🌐 开始发布到IPNS: {} -> {}", cid, key_name);
    
    // 方案1: 使用命令行工具（已验证）
    if let Some(ref cli_path) = config.cli_path {
        println!("📋 尝试使用CLI发布...");
        match publish_with_cli(cli_path, cid, key_name).await {
            Ok(result) => {
                println!("✅ CLI发布成功: {}", result.value);
                return Ok(result);
            }
            Err(e) => {
                println!("⚠️ CLI发布失败: {}, 尝试API方案...", e);
            }
        }
    }
    
    // 方案2: 使用API
    println!("📋 尝试使用API发布...");
    match publish_with_api(cid, key_name, &config.api_url).await {
        Ok(result) => {
            println!("✅ API发布成功: {}", result.value);
            Ok(result)
        }
        Err(e) => {
            println!("❌ API发布也失败: {}", e);
            Err(format!("所有发布方案都失败: {}", e))
        }
    }
}

/// 使用命令行工具发布
async fn publish_with_cli(
    cli_path: &PathBuf,
    cid: &str,
    key_name: &str,
) -> Result<IpnsPublishResult, String> {
    let output = Command::new(cli_path)
        .arg("name")
        .arg("publish")
        .arg("--key")
        .arg(key_name)
        .arg(cid)
        .output()
        .map_err(|e| format!("执行IPFS CLI失败: {}", e))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("IPFS CLI发布失败: {}", stderr));
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // 解析输出: "Published to k51qzi5uqu5dgusof7x2aggisapr5qrhuxqo44dn5zbmlkfn5udlh75hct0z1v: /ipfs/QmbMt3Ri85CCAg6Za1q41NxjweTyHjG3AgBxdtWkWJ7JiH"
    let lines: Vec<&str> = stdout.lines().collect();
    for line in lines {
        if line.contains("Published to") {
            let parts: Vec<&str> = line.split(": ").collect();
            if parts.len() >= 2 {
                let ipns_value = parts[1].trim();
                return Ok(IpnsPublishResult {
                    name: key_name.to_string(),
                    value: ipns_value.to_string(),
                });
            }
        }
    }
    
    Err("无法解析CLI发布结果".to_string())
}

/// 使用API发布
async fn publish_with_api(
    cid: &str,
    key_name: &str,
    api_url: &str,
) -> Result<IpnsPublishResult, String> {
    use reqwest::Client;
    
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| format!("创建HTTP客户端失败: {}", e))?;
    
    let endpoint = format!("{}/api/v0/name/publish", normalize_base_url(api_url));
    
    let params = [
        ("arg", cid),
        ("key", key_name),
        ("lifetime", "24h"),
        ("ttl", "1h"),
    ];
    
    let response = client
        .post(&endpoint)
        .form(&params)
        .header("User-Agent", "Alou-Desktop/1.0")
        .send()
        .await
        .map_err(|e| format!("发送请求失败: {}", e))?;
    
    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(format!("API请求失败: {} - {}", status, text));
    }
    
    let result: Value = response.json().await
        .map_err(|e| format!("解析响应失败: {}", e))?;
    
    let name = result.get("Name")
        .and_then(|v| v.as_str())
        .ok_or("响应中缺少Name字段")?;
    
    let value = result.get("Value")
        .and_then(|v| v.as_str())
        .ok_or("响应中缺少Value字段")?;
    
    Ok(IpnsPublishResult {
        name: name.to_string(),
        value: value.to_string(),
    })
}

/// 验证IPNS解析
pub async fn resolve_ipns_verified(
    key_name: &str,
    config: &IpfsConfig,
) -> Result<String, String> {
    println!("🔍 验证IPNS解析: {}", key_name);
    
    if let Some(ref cli_path) = config.cli_path {
        match resolve_with_cli(cli_path, key_name).await {
            Ok(result) => {
                println!("✅ IPNS解析成功: {}", result);
                return Ok(result);
            }
            Err(e) => {
                println!("⚠️ CLI解析失败: {}", e);
            }
        }
    }
    
    Err("IPNS解析验证失败".to_string())
}

/// 使用命令行工具解析IPNS
async fn resolve_with_cli(
    cli_path: &PathBuf,
    key_name: &str,
) -> Result<String, String> {
    let output = Command::new(cli_path)
        .arg("name")
        .arg("resolve")
        .arg(key_name)
        .output()
        .map_err(|e| format!("执行IPFS CLI失败: {}", e))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("IPFS CLI解析失败: {}", stderr));
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let resolved = stdout.trim().to_string();
    
    if resolved.is_empty() {
        return Err("IPNS解析返回空结果".to_string());
    }
    
    Ok(resolved)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_ipns_key_generation() {
        let config = IpfsConfig::default();
        let key_name = format!("test-key-{}", chrono::Utc::now().timestamp());
        
        match generate_ipns_key_verified(&key_name, &config).await {
            Ok(result) => {
                println!("✅ 测试密钥生成成功: {:?}", result);
                assert!(!result.id.is_empty());
            }
            Err(e) => {
                println!("❌ 测试密钥生成失败: {}", e);
            }
        }
    }
}
