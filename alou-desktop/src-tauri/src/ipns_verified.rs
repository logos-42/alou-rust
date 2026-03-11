//! 经过验证的IPNS解决方案
//! 基于自动化测试验证的完整IPNS密钥生成和发布功能

use crate::utils::normalize_base_url;
use serde_json::Value;
use std::process::Command;
use std::path::PathBuf;
use std::collections::HashMap;

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
    pub repo_path: Option<PathBuf>,
}

impl Default for IpfsConfig {
    fn default() -> Self {
        let cli_path = detect_ipfs_cli();
        let repo_path = detect_ipfs_repo();
        Self {
            api_url: "http://localhost:5001".to_string(),
            gateway_url: "http://localhost:8080".to_string(),
            cli_path,
            repo_path,
        }
    }
}

/// 检测IPFS命令行工具路径
pub(crate) fn detect_ipfs_cli() -> Option<PathBuf> {
    // 尝试常见的IPFS安装路径
    let common_paths = vec![
        PathBuf::from(r"C:\Users\Mechrevo\AppData\Roaming\com.alou.desktop\kubo\ipfs.exe"),
        PathBuf::from(r"C:\Program Files\IPFS\ipfs.exe"),
        PathBuf::from(r"C:\Users\Mechrevo\.cargo\bin\ipfs.exe"),
    ];

    // macOS: 检查应用数据目录 (可执行文件)
    if let Some(home) = dirs::home_dir() {
        let app_ipfs = home.join("Library/Application Support/com.alou.desktop/kubo/ipfs");
        if app_ipfs.exists() && app_ipfs.is_file() {
            println!("🔍 找到 Alou 应用 IPFS 可执行文件: {}", app_ipfs.display());
            return Some(app_ipfs);
        }
    }
    
    for path in common_paths {
        if path.exists() {
            println!("🔍 找到IPFS CLI: {}", path.display());
            return Some(path);
        }
    }
    
    // 尝试从PATH中查找
    // 尝试从 PATH 中查找
    #[cfg(target_os = "windows")]
    let where_cmd = Command::new("where").arg("ipfs.exe").output();
    #[cfg(not(target_os = "windows"))]
    let where_cmd = Command::new("which").arg("ipfs").output();

    if let Ok(output) = where_cmd {
        if output.status.success() {
            if let Some(path_str) = String::from_utf8_lossy(&output.stdout).lines().next() {
                let path = PathBuf::from(path_str.trim());
                if path.exists() {
                    println!("🔍 从 PATH 找到 IPFS CLI: {}", path.display());
                    return Some(path);
                }
            }
        }
    }
    println!("⚠️ 未找到IPFS CLI工具");
    None
}

/// 检测IPFS repo路径
pub(crate) fn detect_ipfs_repo() -> Option<PathBuf> {
    // macOS: 检查 Alou 应用数据目录
    if let Some(home) = dirs::home_dir() {
        let app_ipfs_repo = home.join("Library/Application Support/com.alou.desktop/ipfs");
        if app_ipfs_repo.exists() && app_ipfs_repo.is_dir() {
            println!("🔍 找到 Alou 应用 IPFS repo: {}", app_ipfs_repo.display());
            return Some(app_ipfs_repo);
        }
        
        // 也检查 kubo 目录下的 ipfs 目录
        let kubo_ipfs_repo = home.join("Library/Application Support/com.alou.desktop/kubo/ipfs");
        if kubo_ipfs_repo.exists() && kubo_ipfs_repo.is_dir() {
            println!("🔍 找到 Alou 应用 IPFS repo (kubo): {}", kubo_ipfs_repo.display());
            return Some(kubo_ipfs_repo);
        }
    }
    
    // Windows: 检查应用数据目录
    #[cfg(target_os = "windows")]
    if let Some(app_data) = std::env::var_os("APPDATA") {
        let app_ipfs_repo = PathBuf::from(app_data).join("com.alou.desktop").join("ipfs");
        if app_ipfs_repo.exists() && app_ipfs_repo.is_dir() {
            println!("🔍 找到 Windows IPFS repo: {}", app_ipfs_repo.display());
            return Some(app_ipfs_repo);
        }
    }
    
    // 默认使用 ~/.ipfs
    if let Some(home) = dirs::home_dir() {
        let default_ipfs = home.join(".ipfs");
        if default_ipfs.exists() && default_ipfs.is_dir() {
            println!("🔍 找到默认 IPFS repo: {}", default_ipfs.display());
            return Some(default_ipfs);
        }
    }
    
    println!("⚠️ 未找到 IPFS repo，使用默认路径");
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
        let repo_path = config.repo_path.as_ref();
        match generate_key_with_cli(cli_path, key_name, repo_path).await {
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
    repo_path: Option<&PathBuf>,
) -> Result<IpnsKeyResult, String> {
    // 构建环境变量，包括 IPFS_PATH
    let mut env_vars: HashMap<String, String> = HashMap::new();
    if let Some(repo) = repo_path {
        env_vars.insert("IPFS_PATH".to_string(), repo.to_string_lossy().to_string());
    }
    
    // 获取当前环境变量并添加 IPFS_PATH
    let mut cmd = Command::new(cli_path);
    for (key, value) in env_vars {
        cmd.env(&key, &value);
    }
    
    let output = cmd
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
    use reqwest::Client;
    
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("创建HTTP客户端失败: {}", e))?;
    
    // IPFS key gen API 使用查询参数而不是 multipart/form-data
    let endpoint = format!("{}/api/v0/key/gen?arg={}&type=ed25519", 
        normalize_base_url(api_url), 
        urlencoding::encode(key_name));
    let response = client
        .post(&endpoint)
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
        let repo_path = config.repo_path.as_ref();
        match publish_with_cli(cli_path, cid, key_name, repo_path).await {
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
    repo_path: Option<&PathBuf>,
) -> Result<IpnsPublishResult, String> {
    // 构建环境变量，包括 IPFS_PATH
    let mut env_vars: HashMap<String, String> = HashMap::new();
    if let Some(repo) = repo_path {
        env_vars.insert("IPFS_PATH".to_string(), repo.to_string_lossy().to_string());
    }
    
    let mut cmd = Command::new(cli_path);
    for (key, value) in env_vars {
        cmd.env(&key, &value);
    }
    
    let output = cmd
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

    // 使用与 ipfs_commands 相同的配置
    let client = Client::builder()
        .http1_only() // 强制使用 HTTP/1.1
        .timeout(std::time::Duration::from_secs(60))
        .connect_timeout(std::time::Duration::from_secs(10))
        .tcp_keepalive(std::time::Duration::from_secs(60))
        .no_proxy() // 避免通过系统代理访问本地 API
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败：{}", e))?;

    let endpoint = format!("{}/api/v0/name/publish", normalize_base_url(api_url));

    // 使用查询参数而不是表单数据
    let url = format!("{}?arg={}&key={}&lifetime=24h&ttl=1h",
        endpoint,
        urlencoding::encode(cid),
        urlencoding::encode(key_name));

    let response = client
        .post(&url)
        .header("User-Agent", "Alou-Desktop/1.0")
        .send()
        .await
        .map_err(|e| format!("发送请求失败：{}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(format!("API 请求失败：{} - {}", status, text));
    }

    let result: Value = response.json().await
        .map_err(|e| format!("解析响应失败：{}", e))?;

    let name = result.get("Name")
        .and_then(|v| v.as_str())
        .ok_or("响应中缺少 Name 字段")?;

    let value = result.get("Value")
        .and_then(|v| v.as_str())
        .ok_or("响应中缺少 Value 字段")?;

    Ok(IpnsPublishResult {
        name: name.to_string(),
        value: value.to_string(),
    })
}
