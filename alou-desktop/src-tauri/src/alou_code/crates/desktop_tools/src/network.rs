//! Network Tool Adapter for alou_code Kernel

use alou_code_runtime::PermissionMode;
use reqwest;
use serde_json::{json, Value};

pub fn tool_spec() -> (
    String,
    String,
    Value,
    PermissionMode,
    Box<dyn Fn(&Value) -> Result<String, String> + Send + Sync>,
) {
    let name = "desktop_network".to_string();
    let description = "Network operations: HTTP requests, DNS lookup, port scanning".to_string();
    let schema = json!({
        "type": "object",
        "properties": {
            "operation": {
                "type": "string",
                "enum": ["http_get", "http_post", "dns_lookup"]
            },
            "url": { "type": "string", "format": "uri" },
            "method": { "type": "string", "enum": ["GET", "POST", "PUT", "DELETE"] },
            "body": { "type": "string" },
            "headers": { "type": "object" },
            "host": { "type": "string" }
        },
        "required": ["operation"]
    });
    let permission = PermissionMode::DangerFullAccess;

    let executor: Box<dyn Fn(&Value) -> Result<String, String> + Send + Sync> = Box::new(|input: &Value| {
        let operation = input.get("operation")
            .and_then(|v| v.as_str())
            .unwrap_or("http_get");

        match operation {
            "http_get" | "http_post" | "http_put" | "http_delete" => {
                let url = input.get("url")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let method = input.get("method")
                    .and_then(|v| v.as_str())
                    .unwrap_or("GET");

                let client = reqwest::blocking::Client::new();
                let request_builder = match method {
                    "POST" => client.post(url),
                    "PUT" => client.put(url),
                    "DELETE" => client.delete(url),
                    _ => client.get(url),
                };

                let response = request_builder
                    .send()
                    .map_err(|e| format!("HTTP request failed: {}", e))?;

                let status = response.status().as_u16();
                let body = response.text().unwrap_or_default();

                Ok(serde_json::to_string(&serde_json::json!({
                    "success": true,
                    "status": status,
                    "body": body
                })
            }
            "dns_lookup" => {
                let host = input.get("host")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                let output = std::process::Command::new("nslookup")
                    .arg(host)
                    .output()
                    .map_err(|e| format!("DNS lookup failed: {}", e))?;

                let stdout = String::from_utf8_lossy(&output.stdout).to_string();

                Ok(serde_json::to_string(&serde_json::json!({
                    "success": true,
                    "host": host,
                    "result": stdout
                })
            }
            _ => Err(format!("Unknown operation: {}", operation))
        }
    });

    (name, description, schema, permission, executor)
}

pub fn tool_definition() -> ToolDefinition {
    let (name, description, schema, _, _) = tool_spec();
    ToolDefinition {
        name,
        description: Some(description),
        input_schema: schema,
    }
}
