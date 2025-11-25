//! DIAP identity management for agents
//! Provides DID/IPNS generation and management using diap-rs-sdk

use crate::agent::discovery::EncryptedPeerPayload;
use crate::utils::error::{AloudError, Result};
use serde::{Deserialize, Serialize};

/// DIAP identity information for an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiapIdentity {
    pub did: String,
    pub ipns: String,
    pub cid: String,
    pub public_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encrypted_peer_id: Option<EncryptedPeerPayload>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipns_key: Option<String>, // IPNS key name used for this identity
    pub is_registered: bool, // Whether registered on-chain
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registered_address: Option<String>, // On-chain registered address
    pub created_at: i64,
}

impl DiapIdentity {
    pub fn new(
        did: String,
        ipns: String,
        cid: String,
        public_key: String,
        encrypted_peer_id: Option<EncryptedPeerPayload>,
    ) -> Self {
        Self::new_with_ipns_key(did, ipns, cid, public_key, encrypted_peer_id, None)
    }

    pub fn new_with_ipns_key(
        did: String,
        ipns: String,
        cid: String,
        public_key: String,
        encrypted_peer_id: Option<EncryptedPeerPayload>,
        ipns_key: Option<String>,
    ) -> Self {
        Self {
            did,
            ipns,
            cid,
            public_key,
            encrypted_peer_id,
            ipns_key,
            is_registered: false,
            registered_address: None,
            created_at: crate::utils::time::now_timestamp(),
        }
    }
}

/// Configuration for DIAP identity creation
#[cfg(not(target_arch = "wasm32"))]
pub struct DiapIdentityConfig {
    pub ipfs_api_url: String,
    pub ipfs_gateway_url: String,
    pub ipns_key: Option<String>,
    pub timeout_secs: u64,
}

#[cfg(not(target_arch = "wasm32"))]
impl DiapIdentityConfig {
    pub fn new(ipfs_api_url: String, ipfs_gateway_url: String) -> Self {
        Self {
            ipfs_api_url,
            ipfs_gateway_url,
            ipns_key: None,
            timeout_secs: 10,
        }
    }

    pub fn with_ipns_key(mut self, ipns_key: Option<String>) -> Self {
        self.ipns_key = ipns_key;
        self
    }

    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = timeout_secs;
        self
    }
}

/// DIAP identity manager
#[cfg(not(target_arch = "wasm32"))]
pub struct DiapIdentityManager {
    config: DiapIdentityConfig,
}

#[cfg(not(target_arch = "wasm32"))]
impl DiapIdentityManager {
    pub fn new(config: DiapIdentityConfig) -> Result<Self> {
        Ok(Self { config })
    }

    // Note: generate_ipns_key() method removed - IPNS key generation is now handled by Desktop

    // Note: create_identity() method removed - identity creation is now handled by Desktop
    // Workers only provide resolution and validation functionality

    /// Add data to IPFS and return CID (unused, kept for potential future use)
    #[allow(dead_code)]
    async fn add_to_ipfs(&self, data: &[u8], filename: Option<String>) -> Result<String> {
        use reqwest::Client;

        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(self.config.timeout_secs))
            .build()
            .map_err(|e| AloudError::AgentError(format!("Failed to create HTTP client: {}", e)))?;

        let file_name = filename.unwrap_or_else(|| "did.json".to_string());
        let form = reqwest::multipart::Form::new()
            .part("file", reqwest::multipart::Part::bytes(data.to_vec()).file_name(file_name));

        let api_url = self.config.ipfs_api_url.trim_end_matches('/');
        let add_url = format!("{}/api/v0/add?pin=true", api_url);

        #[derive(Deserialize)]
        struct IpfsAddResponse {
            #[serde(rename = "Hash")]
            hash: String,
        }

        let response = http_client
            .post(&add_url)
            .multipart(form)
            .send()
            .await
            .map_err(|e| AloudError::AgentError(format!("Failed to add to IPFS: {}", e)))?;

        let response_text = response
            .text()
            .await
            .map_err(|e| AloudError::AgentError(format!("Failed to read IPFS response: {}", e)))?;

        // Parse response - IPFS returns newline-delimited JSON
        let lines: Vec<&str> = response_text.trim().lines().collect();
        let last_line = lines.last().ok_or_else(|| {
            AloudError::AgentError("Empty response from IPFS add".to_string())
        })?;

        let parsed: IpfsAddResponse = serde_json::from_str(last_line)
            .map_err(|e| AloudError::AgentError(format!("Failed to parse IPFS add response: {}", e)))?;

        Ok(parsed.hash)
    }

    /// Publish CID to IPNS and return IPNS name (unused, kept for potential future use)
    #[allow(dead_code)]
    async fn publish_ipns(&self, cid: &str) -> Result<String> {
        use reqwest::Client;

        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(self.config.timeout_secs))
            .build()
            .map_err(|e| AloudError::AgentError(format!("Failed to create HTTP client: {}", e)))?;

        let api_url = self.config.ipfs_api_url.trim_end_matches('/');
        let mut publish_url = format!("{}/api/v0/name/publish?arg=/ipfs/{}", api_url, cid);

        if let Some(ref key) = self.config.ipns_key {
            publish_url.push_str(&format!("&key={}", key));
        }

        #[derive(Deserialize)]
        struct IpnsPublishResponse {
            name: String,
        }

        let response = http_client
            .post(&publish_url)
            .send()
            .await
            .map_err(|e| AloudError::AgentError(format!("Failed to publish to IPNS: {}", e)))?;

        let response_text = response
            .text()
            .await
            .map_err(|e| AloudError::AgentError(format!("Failed to read IPNS response: {}", e)))?;

        let parsed: IpnsPublishResponse = serde_json::from_str(&response_text)
            .map_err(|e| AloudError::AgentError(format!("Failed to parse IPNS publish response: {}", e)))?;

        Ok(parsed.name)
    }

    #[allow(dead_code)]
    fn simple_hash_fragment(input: &str) -> String {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        input.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Resolve identity from IPNS name (解析功能)
    pub async fn resolve_identity_from_ipns(&self, ipns_name: &str) -> Result<DiapIdentity> {
        use diap_rs_sdk::IpfsClient;

        // Create IPFS client
        let ipfs_client = IpfsClient::new_with_remote_node(
            self.config.ipfs_api_url.clone(),
            self.config.ipfs_gateway_url.clone(),
            self.config.timeout_secs,
        );

        // Resolve IPNS to CID
        let cid = ipfs_client
            .resolve_ipns(ipns_name)
            .await
            .map_err(|e| AloudError::AgentError(format!("Failed to resolve IPNS: {}", e)))?;

        // Get identity from CID
        self.get_identity(&cid).await
    }

    /// Validate identity information (验证功能)
    pub fn validate_identity(&self, identity: &DiapIdentity) -> Result<()> {
        // Validate IPNS format
        if !identity.ipns.starts_with("/ipns/") && !identity.ipns.starts_with("k51") {
            return Err(AloudError::AgentError("Invalid IPNS format".to_string()));
        }

        // Validate DID format
        if !identity.did.starts_with("did:") {
            return Err(AloudError::AgentError("Invalid DID format".to_string()));
        }

        // Validate CID format (basic check)
        if identity.cid.is_empty() {
            return Err(AloudError::AgentError("CID cannot be empty".to_string()));
        }

        // Validate public key format
        if identity.public_key.is_empty() {
            return Err(AloudError::AgentError("Public key cannot be empty".to_string()));
        }

        Ok(())
    }

    /// Get identity information from CID
    pub async fn get_identity(&self, cid: &str) -> Result<DiapIdentity> {
        use diap_rs_sdk::get_did_document_from_cid;
        use diap_rs_sdk::identity_manager::IdentityManager;
        use diap_rs_sdk::IpfsClient;

        // Create IPFS client
        let ipfs_client = IpfsClient::new_with_remote_node(
            self.config.ipfs_api_url.clone(),
            self.config.ipfs_gateway_url.clone(),
            self.config.timeout_secs,
        );

        // Get DID document from CID
        let did_document = get_did_document_from_cid(&ipfs_client, cid)
            .await
            .map_err(|e| AloudError::AgentError(format!("Failed to get DID document: {}", e)))?;

        // Create identity manager
        let identity_manager = IdentityManager::new(ipfs_client);

        // Extract encrypted peer ID
        let encrypted_peer_id = identity_manager
            .extract_encrypted_peer_id(&did_document)
            .map(|ep| EncryptedPeerPayload::from_encrypted(&ep))
            .ok();

        // Get IPNS from DID document service endpoints
        // Convert DID document to JSON to access service fields
        let did_json = serde_json::to_value(&did_document)
            .map_err(|e| AloudError::AgentError(format!("Failed to serialize DID document: {}", e)))?;

        let ipns = did_json
            .get("service")
            .and_then(|services| services.as_array())
            .and_then(|services| {
                services.iter().find_map(|s| {
                    // Access serviceEndpoint field (camelCase in DID document)
                    let endpoint = s.get("serviceEndpoint")
                        .and_then(|ep| ep.as_str())
                        .or_else(|| {
                            // Try as object with nested structure
                            s.get("serviceEndpoint")
                                .and_then(|ep| ep.as_object())
                                .and_then(|obj| obj.get("type"))
                                .and_then(|t| t.as_str())
                                .filter(|t| *t == "ipns")
                                .and_then(|_| s.get("id").and_then(|id| id.as_str()))
                        });
                    
                    endpoint.and_then(|endpoint| {
                        if endpoint.contains("ipns") {
                            endpoint
                                .split('/')
                                .find(|part| part.starts_with("k51") || part.starts_with("/ipns/"))
                                .map(|s| {
                                    if s.starts_with("/ipns/") {
                                        s.to_string()
                                    } else {
                                        format!("/ipns/{}", s)
                                    }
                                })
                        } else {
                            None
                        }
                    })
                })
            })
            .unwrap_or_else(|| format!("/ipns/{}", cid));

        // Extract public key from DID document (simplified)
        let public_key = "".to_string(); // Would need to extract from verificationMethod in DID document

        // Note: get_identity doesn't have access to the original IPNS key,
        // so we set it to None for existing identities
        Ok(DiapIdentity::new_with_ipns_key(
            did_document.id,
            ipns,
            cid.to_string(),
            public_key,
            encrypted_peer_id,
            None, // IPNS key not available when retrieving existing identity
        ))
    }
}

/// Configuration for DIAP identity creation (WASM stub)
#[cfg(target_arch = "wasm32")]
pub struct DiapIdentityConfig {
    pub ipfs_api_url: String,
    pub ipfs_gateway_url: String,
    pub ipns_key: Option<String>,
    pub timeout_secs: u64,
}

#[cfg(target_arch = "wasm32")]
impl DiapIdentityConfig {
    pub fn new(ipfs_api_url: String, ipfs_gateway_url: String) -> Self {
        Self {
            ipfs_api_url,
            ipfs_gateway_url,
            ipns_key: None,
            timeout_secs: 10,
        }
    }

    pub fn with_ipns_key(mut self, ipns_key: Option<String>) -> Self {
        self.ipns_key = ipns_key;
        self
    }

    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = timeout_secs;
        self
    }
}

/// WASM-compatible stub for DIAP identity manager
#[cfg(target_arch = "wasm32")]
pub struct DiapIdentityManager;

#[cfg(target_arch = "wasm32")]
impl DiapIdentityManager {
    pub fn new(_config: DiapIdentityConfig) -> Result<Self> {
        // In WASM, we can't use reqwest, but we can still create the manager
        // The actual IPFS operations will need to be done via external HTTP calls
        Ok(Self)
    }

    pub async fn create_identity(&self) -> Result<DiapIdentity> {
        Err(AloudError::AgentError(
            "DIAP identity creation is not available in WASM build. Use Desktop app instead.".to_string(),
        ))
    }

    pub async fn get_identity(&self, _cid: &str) -> Result<DiapIdentity> {
        Err(AloudError::AgentError(
            "DIAP identity retrieval is not available in WASM build".to_string(),
        ))
    }

    /// Resolve identity from IPNS name (WASM stub - returns error)
    pub async fn resolve_identity_from_ipns(&self, _ipns_name: &str) -> Result<DiapIdentity> {
        Err(AloudError::AgentError(
            "IPNS resolution is not available in WASM build. Use Desktop app or configure external IPFS gateway.".to_string(),
        ))
    }

    /// Validate identity information (WASM stub - basic validation only)
    pub fn validate_identity(&self, identity: &DiapIdentity) -> Result<()> {
        // Basic format validation only (no network calls)
        if !identity.ipns.starts_with("/ipns/") && !identity.ipns.starts_with("k51") {
            return Err(AloudError::AgentError("Invalid IPNS format".to_string()));
        }

        if !identity.did.starts_with("did:") {
            return Err(AloudError::AgentError("Invalid DID format".to_string()));
        }

        if identity.cid.is_empty() {
            return Err(AloudError::AgentError("CID cannot be empty".to_string()));
        }

        if identity.public_key.is_empty() {
            return Err(AloudError::AgentError("Public key cannot be empty".to_string()));
        }

        Ok(())
    }
}

// EncryptedPeerPayload::from_encrypted is already implemented in discovery.rs
