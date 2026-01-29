use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;

/// Configuration for agent discovery through DIAP/IPFS.
#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
#[derive(Debug, Clone)]
pub struct AgentDiscoveryConfig {
    pub ipfs_api_url: String,
    pub ipfs_gateway_url: String,
    pub ipns_key: Option<String>,
    pub request_timeout: Duration,
    pub cache_ttl: Duration,
}

impl AgentDiscoveryConfig {
    pub fn new(api_url: String, gateway_url: String) -> Self {
        Self {
            ipfs_api_url: api_url,
            ipfs_gateway_url: gateway_url,
            ipns_key: None,
            request_timeout: Duration::from_secs(10),
            cache_ttl: Duration::from_secs(300),
        }
    }

    pub fn with_ipns_key(mut self, ipns_key: Option<String>) -> Self {
        self.ipns_key = ipns_key;
        self
    }

    pub fn with_request_timeout(mut self, timeout: Duration) -> Self {
        self.request_timeout = timeout;
        self
    }

    pub fn with_cache_ttl(mut self, ttl: Duration) -> Self {
        self.cache_ttl = ttl;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EncryptedPeerPayload {
    pub ciphertext_b64: String,
    pub nonce_b64: String,
    pub signature_b64: String,
    pub method: String,
}

impl EncryptedPeerPayload {
    #[cfg(not(target_arch = "wasm32"))]
    #[allow(dead_code)]
    pub(crate) fn from_encrypted(_source: &str) -> Self {
        Self {
            ciphertext_b64: String::new(),
            nonce_b64: String::new(),
            signature_b64: String::new(),
            method: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedAgent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipns: Option<String>,
    pub cid: String,
    pub did: String,
    pub did_document: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encrypted_peer_id: Option<EncryptedPeerPayload>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved_path: Option<String>,
}

// 简化的 AgentDiscovery 实现，不依赖 DIAP SDK
#[derive(Clone)]
pub struct AgentDiscovery;

impl AgentDiscovery {
    pub fn new(_config: AgentDiscoveryConfig) -> crate::utils::error::Result<Self> {
        // 简化实现，不依赖外部 SDK
        Ok(Self)
    }

    pub async fn resolve_agent(&self, _target: &str) -> crate::utils::error::Result<ResolvedAgent> {
        // 简化实现，返回错误直到功能完全实现
        Err(crate::utils::error::AloudError::AgentError(
            "Agent discovery is not fully implemented yet".to_string(),
        ))
    }

    pub async fn search_agents(&self, _query: &str) -> crate::utils::error::Result<Vec<ResolvedAgent>> {
        // 简化实现，返回空结果
        Ok(Vec::new())
    }
}
