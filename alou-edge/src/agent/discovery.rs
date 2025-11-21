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
    pub(crate) fn from_encrypted(source: &diap_rs_sdk::EncryptedPeerID) -> Self {
        use base64::engine::general_purpose::STANDARD;
        use base64::Engine;

        Self {
            ciphertext_b64: STANDARD.encode(&source.ciphertext),
            nonce_b64: STANDARD.encode(&source.nonce),
            signature_b64: STANDARD.encode(&source.signature),
            method: source.method.clone(),
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

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use super::{AgentDiscoveryConfig, EncryptedPeerPayload, ResolvedAgent};
    use crate::utils::error::{AloudError, Result};
    use diap_rs_sdk::{get_did_document_from_cid, identity_manager::IdentityManager, IpfsClient};
    use reqwest::StatusCode;
    use serde::Deserialize;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use std::time::Instant;

    const IPNS_PREFIX: &str = "/ipns/";

    #[derive(Debug)]
    struct CachedAgent {
        agent: ResolvedAgent,
        expires_at: Instant,
    }

    #[derive(Debug, Deserialize)]
    struct IpnsResolveResponse {
        #[serde(rename = "Path")]
        path: String,
    }

    pub struct AgentDiscovery {
        config: AgentDiscoveryConfig,
        ipfs_client: IpfsClient,
        identity_manager: IdentityManager,
        http_client: reqwest::Client,
        cache: Arc<Mutex<HashMap<String, CachedAgent>>>,
    }

    impl AgentDiscovery {
        pub fn new(config: AgentDiscoveryConfig) -> Result<Self> {
            let api_url = config.ipfs_api_url.trim_end_matches('/').to_string();
            let gateway_url = config.ipfs_gateway_url.trim_end_matches('/').to_string();

            let ipfs_client = IpfsClient::new_with_remote_node(
                api_url.clone(),
                gateway_url,
                config.request_timeout.as_secs(),
            );
            let identity_manager = IdentityManager::new(ipfs_client.clone());

            let http_client = reqwest::Client::builder()
                .timeout(config.request_timeout)
                .build()
                .map_err(|e| {
                    AloudError::InternalError(format!("Failed to build HTTP client: {}", e))
                })?;

            Ok(Self {
                config: AgentDiscoveryConfig {
                    ipfs_api_url: api_url,
                    ..config
                },
                ipfs_client,
                identity_manager,
                http_client,
                cache: Arc::new(Mutex::new(HashMap::new())),
            })
        }

        pub async fn resolve_agent(&self, target: &str) -> Result<ResolvedAgent> {
            let trimmed = target.trim();
            if trimmed.is_empty() {
                return Err(AloudError::InvalidInput(
                    "resolve_agent requires a non-empty identifier".to_string(),
                ));
            }

            let target_kind = TargetKind::detect(trimmed);
            match target_kind {
                TargetKind::Ipns(name) => self.resolve_from_ipns(&name).await,
                TargetKind::Cid(cid) => self.resolve_from_cid(&cid, None).await,
                TargetKind::Did(did) => self.resolve_from_did(&did).await,
            }
        }

        pub async fn search_agents(&self, query: &str) -> Result<Vec<ResolvedAgent>> {
            let trimmed = query.trim();
            if trimmed.is_empty() {
                return Ok(Vec::new());
            }

            match self.resolve_agent(trimmed).await {
                Ok(agent) => Ok(vec![agent]),
                Err(_) => Ok(Vec::new()),
            }
        }

        async fn resolve_from_ipns(&self, raw_ipns: &str) -> Result<ResolvedAgent> {
            let normalized = Self::normalize_ipns(raw_ipns);

            if let Some(agent) = self.get_cached(&normalized) {
                return Ok(agent);
            }

            let path = self.resolve_ipns_path(&normalized).await?;

            let cid = path.trim_start_matches("/ipfs/").to_string();

            let mut agent = self.resolve_from_cid(&cid, Some(path.clone())).await?;
            agent.ipns = Some(normalized.clone());
            agent.resolved_path = Some(path);

            self.insert_cache(normalized.clone(), agent.clone());
            self.insert_cache(cid.clone(), agent.clone());

            Ok(agent)
        }

        async fn resolve_from_cid(
            &self,
            cid: &str,
            resolved_path: Option<String>,
        ) -> Result<ResolvedAgent> {
            if let Some(agent) = self.get_cached(cid) {
                return Ok(agent);
            }

            let did_document = get_did_document_from_cid(&self.ipfs_client, cid)
                .await
                .map_err(|e| {
                    AloudError::AgentError(format!("Failed to fetch DID document: {}", e))
                })?;

            let encrypted_peer = self
                .identity_manager
                .extract_encrypted_peer_id(&did_document)
                .ok();

            let did = did_document.id.clone();
            let did_json = serde_json::to_value(&did_document).map_err(|e| {
                AloudError::InternalError(format!("Failed to serialize DID document: {}", e))
            })?;

            let encrypted_peer_id =
                encrypted_peer.map(|inner| EncryptedPeerPayload::from_encrypted(&inner));

            let mut agent = ResolvedAgent {
                ipns: None,
                cid: cid.to_string(),
                did,
                did_document: did_json,
                encrypted_peer_id,
                resolved_path: None,
            };
            agent.resolved_path = resolved_path;

            self.insert_cache(cid.to_string(), agent.clone());

            Ok(agent)
        }

        async fn resolve_from_did(&self, did: &str) -> Result<ResolvedAgent> {
            if let Some(agent) = self.get_cached(did) {
                return Ok(agent);
            }

            Err(AloudError::InvalidInput(format!(
                "Resolving by DID is not supported yet: {}",
                did
            )))
        }

        fn get_cached(&self, key: &str) -> Option<ResolvedAgent> {
            let cache = self.cache.lock().ok()?;
            cache.get(key).and_then(|entry| {
                if entry.expires_at > Instant::now() {
                    Some(entry.agent.clone())
                } else {
                    None
                }
            })
        }

        fn insert_cache(&self, key: String, agent: ResolvedAgent) {
            if let Ok(mut cache) = self.cache.lock() {
                cache.insert(
                    key,
                    CachedAgent {
                        agent: agent.clone(),
                        expires_at: Instant::now() + self.config.cache_ttl,
                    },
                );
            }
        }

        async fn resolve_ipns_path(&self, ipns: &str) -> Result<String> {
            let url = format!("{}/api/v0/name/resolve", self.config.ipfs_api_url);
            let response = self
                .http_client
                .post(url)
                .query(&[("arg", ipns), ("recursive", "true"), ("dht-timeout", "10s")])
                .send()
                .await
                .map_err(|e| {
                    AloudError::AgentError(format!("IPNS resolve request failed: {}", e))
                })?;

            if response.status() == StatusCode::NOT_FOUND {
                return Err(AloudError::InvalidInput(format!(
                    "IPNS name not found: {}",
                    ipns
                )));
            }

            if !response.status().is_success() {
                return Err(AloudError::AgentError(format!(
                    "IPNS resolve returned HTTP {}",
                    response.status()
                )));
            }

            let payload: IpnsResolveResponse = response.json().await.map_err(|e| {
                AloudError::AgentError(format!("Failed to parse IPNS resolve response: {}", e))
            })?;

            Ok(payload.path)
        }

        fn normalize_ipns(value: &str) -> String {
            if value.starts_with(IPNS_PREFIX) {
                value.to_string()
            } else if value.starts_with("ipns://") {
                format!("{}{}", IPNS_PREFIX, &value[7..])
            } else {
                format!("{}{}", IPNS_PREFIX, value.trim_start_matches(IPNS_PREFIX))
            }
        }
    }

    enum TargetKind {
        Ipns(String),
        Cid(String),
        Did(String),
    }

    impl TargetKind {
        fn detect(raw: &str) -> Self {
            let value = raw.trim();
            if value.starts_with(IPNS_PREFIX) || value.contains(".ipns") || value.starts_with("k51")
            {
                TargetKind::Ipns(value.to_string())
            } else if value.starts_with("did:") {
                TargetKind::Did(value.to_string())
            } else {
                TargetKind::Cid(value.to_string())
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use native::AgentDiscovery;

#[cfg(target_arch = "wasm32")]
#[derive(Clone)]
pub struct AgentDiscovery;

#[cfg(target_arch = "wasm32")]
impl AgentDiscovery {
    pub fn new(_config: AgentDiscoveryConfig) -> crate::utils::error::Result<Self> {
        Err(crate::utils::error::AloudError::AgentError(
            "Agent discovery is not available in the WASM build".to_string(),
        ))
    }

    pub async fn resolve_agent(&self, _target: &str) -> crate::utils::error::Result<ResolvedAgent> {
        Err(crate::utils::error::AloudError::AgentError(
            "Agent discovery is not available in the WASM build".to_string(),
        ))
    }

    pub async fn search_agents(
        &self,
        _query: &str,
    ) -> crate::utils::error::Result<Vec<ResolvedAgent>> {
        Ok(Vec::new())
    }
}
