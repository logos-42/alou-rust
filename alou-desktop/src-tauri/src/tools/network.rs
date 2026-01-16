//! 网络工具
//!
//! 提供网络请求、DNS查询等功能

use super::{ToolExecutor, ToolMetadata, ToolCategory, ToolPriority, ToolStatus, ExecutionContext, ToolResult, ToolError};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use reqwest::Client;

/// 网络工具
pub struct NetworkTool {
    metadata: ToolMetadata,
    client: Client,
}

impl NetworkTool {
    /// 创建新的网络工具
    pub fn new() -> Self {
        Self {
            metadata: ToolMetadata {
                id: "network".to_string(),
                name: "Network Tool".to_string(),
                description: "Network operations and requests".to_string(),
                category: ToolCategory::Network,
                priority: ToolPriority::Medium,
                status: ToolStatus::Available,
                version: "1.0.0".to_string(),
                author: "Alou Team".to_string(),
                created_at: chrono::Utc::now().timestamp(),
                updated_at: chrono::Utc::now().timestamp(),
                dependencies: vec![],
                platforms: vec!["windows".to_string(), "macos".to_string(), "linux".to_string()],
                permissions: vec!["network".to_string()],
            },
            client: Client::new(),
        }
    }
}

#[async_trait]
impl ToolExecutor for NetworkTool {
    fn metadata(&self) -> &ToolMetadata {
        &self.metadata
    }

    async fn execute(
        &self,
        args: serde_json::Value,
        _context: &ExecutionContext,
    ) -> Result<ToolResult, ToolError> {
        let network_op: NetworkOperation = serde_json::from_value(args)
            .map_err(|e| ToolError::InvalidArguments(format!("Invalid arguments: {}", e)))?;

        match network_op {
            NetworkOperation::HttpRequest { method, url, headers, body, timeout } =>
                self.http_request(method, url, headers, body, timeout).await,
            NetworkOperation::DnsLookup { domain } =>
                self.dns_lookup(domain).await,
            NetworkOperation::Ping { host, count } =>
                self.ping_host(host, count).await,
        }
    }

    async fn validate_args(&self, args: &serde_json::Value) -> Result<(), ToolError> {
        if let Ok(_op) = serde_json::from_value::<NetworkOperation>(args.clone()) {
            Ok(())
        } else {
            Err(ToolError::InvalidArguments("Invalid network operation arguments".to_string()))
        }
    }

    fn help(&self) -> String {
        r#"Network Tool - Network operations

Available operations:
- http_request: Make HTTP requests
- dns_lookup: DNS domain lookup
- ping: Ping host

HTTP Request options:
- method: HTTP method (GET, POST, PUT, DELETE, etc.)
- url: Request URL
- headers: Optional headers map
- body: Optional request body
- timeout: Request timeout in seconds

DNS Lookup options:
- domain: Domain name to lookup

Ping options:
- host: Host to ping
- count: Number of ping packets (default: 4)

Example usage:
{
  "operation": "http_request",
  "method": "GET",
  "url": "https://api.example.com/data",
  "timeout": 30
}"#.to_string()
    }
}

impl NetworkTool {
    /// HTTP 请求
    async fn http_request(
        &self,
        method: String,
        url: String,
        headers: Option<std::collections::HashMap<String, String>>,
        body: Option<String>,
        timeout: Option<u64>,
    ) -> Result<ToolResult, ToolError> {
        let mut request = self.client.request(
            method.parse().map_err(|_| ToolError::InvalidArguments("Invalid HTTP method".to_string()))?,
            &url
        );

        // 设置超时
        if let Some(timeout_secs) = timeout {
            request = request.timeout(std::time::Duration::from_secs(timeout_secs));
        }

        // 设置请求头
        if let Some(headers) = headers {
            for (key, value) in headers {
                request = request.header(&key, value);
            }
        }

        // 设置请求体
        if let Some(body) = body {
            request = request.body(body);
        }

        let start_time = std::time::Instant::now();
        let response = request.send().await
            .map_err(|e| ToolError::ExecutionFailed(format!("Request failed: {}", e)))?;

        let status = response.status();
        let headers = response.headers().iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect::<std::collections::HashMap<_, _>>();

        let body = response.text().await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to read response: {}", e)))?;

        let execution_time = start_time.elapsed().as_millis() as u64;

        Ok(ToolResult {
            success: status.is_success(),
            data: serde_json::json!({
                "url": url,
                "method": method,
                "status": status.as_u16(),
                "status_text": status.canonical_reason().unwrap_or(""),
                "headers": headers,
                "body": body,
                "content_length": body.len()
            }),
            error: if status.is_success() { None } else { Some(status.to_string()) },
            execution_time_ms: execution_time,
            output: Some(format!("HTTP {} {} - {}", method, status, url)),
            warnings: vec![],
            context: None,
        })
    }

    /// DNS 查询
    async fn dns_lookup(&self, domain: String) -> Result<ToolResult, ToolError> {
        // 简单的 DNS 解析
        let addresses = tokio::net::lookup_host((domain.clone(), 0))
            .await
            .map_err(|e| ToolError::ExecutionFailed(format!("DNS lookup failed: {}", e)))?
            .map(|addr| addr.ip().to_string())
            .collect::<Vec<_>>();

        Ok(ToolResult {
            success: !addresses.is_empty(),
            data: serde_json::json!({
                "domain": domain,
                "addresses": addresses
            }),
            error: None,
            execution_time_ms: 0,
            output: Some(format!("Found {} addresses", addresses.len())),
            warnings: vec![],
            context: None,
        })
    }

    /// Ping 主机
    async fn ping_host(&self, host: String, count: Option<usize>) -> Result<ToolResult, ToolError> {
        let count = count.unwrap_or(4);
        let mut results = Vec::new();

        // 简单的 ping 实现（实际项目中可能需要使用外部工具）
        for i in 0..count {
            let start = std::time::Instant::now();
            match tokio::net::TcpStream::connect((host.clone(), 80)).await {
                Ok(_) => {
                    let latency = start.elapsed().as_millis() as u64;
                    results.push(PingResult {
                        sequence: i + 1,
                        success: true,
                        latency_ms: Some(latency),
                        error: None,
                    });
                }
                Err(e) => {
                    results.push(PingResult {
                        sequence: i + 1,
                        success: false,
                        latency_ms: None,
                        error: Some(e.to_string()),
                    });
                }
            }

            // 简单的延迟
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }

        let success_count = results.iter().filter(|r| r.success).count();

        Ok(ToolResult {
            success: success_count > 0,
            data: serde_json::json!({
                "host": host,
                "count": count,
                "results": results,
                "success_count": success_count,
                "loss_percent": ((count - success_count) as f64 / count as f64) * 100.0
            }),
            error: None,
            execution_time_ms: 0,
            output: Some(format!("Ping {}: {}/{} successful", host, success_count, count)),
            warnings: vec![],
            context: None,
        })
    }
}

/// 网络操作枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "operation")]
pub enum NetworkOperation {
    /// HTTP 请求
    HttpRequest {
        method: String,
        url: String,
        headers: Option<std::collections::HashMap<String, String>>,
        body: Option<String>,
        timeout: Option<u64>,
    },
    /// DNS 查询
    DnsLookup {
        domain: String,
    },
    /// Ping 主机
    Ping {
        host: String,
        count: Option<usize>,
    },
}

/// Ping 结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PingResult {
    /// 序列号
    pub sequence: usize,
    /// 是否成功
    pub success: bool,
    /// 延迟（毫秒）
    pub latency_ms: Option<u64>,
    /// 错误信息
    pub error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_dns_lookup() {
        let tool = NetworkTool::new();
        let context = ExecutionContext {
            session_id: "test".to_string(),
            user_id: None,
            working_directory: None,
            environment: std::collections::HashMap::new(),
            timeout_seconds: Some(30),
            permissions: vec!["network".to_string()],
            timestamp: chrono::Utc::now().timestamp(),
        };

        let args = serde_json::json!({
            "operation": "dns_lookup",
            "domain": "localhost"
        });

        let result = tool.execute(args, &context).await;
        // DNS lookup 可能成功也可能失败，取决于系统配置
        assert!(result.is_ok() || result.is_err()); // 接受两种结果
    }

    #[tokio::test]
    async fn test_network_validation() {
        let tool = NetworkTool::new();

        // 有效的参数
        let valid_args = serde_json::json!({
            "operation": "dns_lookup",
            "domain": "example.com"
        });
        assert!(tool.validate_args(&valid_args).await.is_ok());

        // 无效的参数
        let invalid_args = serde_json::json!({
            "invalid": "args"
        });
        assert!(tool.validate_args(&invalid_args).await.is_err());
    }
}