use crate::utils::error::{AloudError, Result};
use serde_json::{json, Value};
use worker::{Fetch, Headers, Method, RequestInit};

/// 轻量级 JSON-RPC 客户端，基于 Cloudflare Workers 的 `fetch` 实现。
#[derive(Debug, Clone)]
pub struct JsonRpcClient {
    endpoint: String,
}

impl JsonRpcClient {
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
        }
    }

    /// 调用任意 JSON-RPC 方法，`params` 应该是数组或对象。
    pub async fn call_method(&self, method: &str, params: Value) -> Result<Value> {
        let payload = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": uuid::Uuid::new_v4().to_string(),
        });
        self.call_raw(payload).await
    }

    /// 发送已构建好的 JSON-RPC 请求体。
    pub async fn call_raw(&self, payload: Value) -> Result<Value> {
        let body = serde_json::to_string(&payload)?;

        let mut init = RequestInit::new();
        init.with_method(Method::Post);

        let mut headers = Headers::new();
        headers
            .set("Content-Type", "application/json")
            .map_err(|e| AloudError::WorkerError(e.to_string()))?;

        init.with_headers(headers);
        init.with_body(Some(body.into()));

        let request = worker::Request::new_with_init(&self.endpoint, &init)?;
        let mut response = Fetch::Request(request).send().await?;

        let text = response.text().await?;
        let value: Value = serde_json::from_str(&text)?;

        if let Some(err) = value.get("error") {
            return Err(AloudError::RpcError(err.to_string()));
        }

        value
            .get("result")
            .cloned()
            .ok_or_else(|| AloudError::RpcError("Missing result field in RPC response".into()))
    }
}
