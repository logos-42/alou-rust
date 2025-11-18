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
        Self {
            did,
            ipns,
            cid,
            public_key,
            encrypted_peer_id,
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

    /// Create a new DIAP identity for an agent
    pub async fn create_identity(&self) -> Result<DiapIdentity> {
        use diap_rs_sdk::identity_manager::IdentityManager;
        use diap_rs_sdk::IpfsClient;

        // Create IPFS client
        let ipfs_client = IpfsClient::new_with_remote_node(
            self.config.ipfs_api_url.clone(),
            self.config.ipfs_gateway_url.clone(),
            self.config.timeout_secs,
        );

        // Create identity manager
        let identity_manager = IdentityManager::new(ipfs_client.clone());

        // Generate new identity and publish to IPFS
        // Note: The actual API may vary - this is a placeholder implementation
        // In practice, you would:
        // 1. Generate a new identity using IdentityManager
        // 2. Create a DID document
        // 3. Publish to IPFS to get CID
        // 4. Optionally publish to IPNS
        
        // For now, we'll create a simplified identity structure
        // The actual implementation should use diap-rs-sdk's identity creation methods
        use uuid::Uuid;
        let session_id = Uuid::new_v4().to_string();
        let did = format!("did:alou:{}", session_id);
        
        // Generate a placeholder CID (in production, this would be the actual IPFS CID)
        let cid = format!("bafy{}", simple_hash_fragment(&session_id));
        
        // Generate IPNS name
        let ipns = if let Some(ref ipns_key) = self.config.ipns_key {
            if ipns_key.starts_with("/ipns/") {
                ipns_key.clone()
            } else {
                format!("/ipns/{}", ipns_key)
            }
        } else {
            // Generate a placeholder IPNS (in production, this would be published to IPNS)
            format!("/ipns/k51{}", simple_hash_fragment(&session_id))
        };

        // Placeholder public key (in production, extract from identity)
        let public_key = format!("pubkey_{}", simple_hash_fragment(&session_id));

        Ok(DiapIdentity::new(
            did,
            ipns,
            cid,
            public_key,
            None, // Encrypted peer ID would be generated during actual identity creation
        ))
    }
    
    fn simple_hash_fragment(input: &str) -> String {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        input.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Get identity information from CID
    pub async fn get_identity(&self, cid: &str) -> Result<DiapIdentity> {
        use diap_rs_sdk::identity_manager::IdentityManager;
        use diap_rs_sdk::IpfsClient;
        use diap_rs_sdk::get_did_document_from_cid;

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
            .map(|ep| EncryptedPeerPayload::from_encrypted(&ep));

        // Get IPNS from DID document service endpoints
        let ipns = did_document
            .service
            .iter()
            .find_map(|s| {
                if s.service_endpoint.contains("ipns") {
                    s.service_endpoint
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
            .unwrap_or_else(|| format!("/ipns/{}", cid));

        // Extract public key from DID document (simplified)
        let public_key = "".to_string(); // Would need to extract from verificationMethod in DID document

        Ok(DiapIdentity::new(
            did_document.id,
            ipns,
            cid.to_string(),
            public_key,
            encrypted_peer_id,
        ))
    }
}

/// WASM-compatible stub for DIAP identity manager
#[cfg(target_arch = "wasm32")]
pub struct DiapIdentityManager;

#[cfg(target_arch = "wasm32")]
impl DiapIdentityManager {
    pub fn new(_config: DiapIdentityConfig) -> Result<Self> {
        Err(AloudError::AgentError(
            "DIAP identity management is not available in WASM build".to_string(),
        ))
    }

    pub async fn create_identity(&self) -> Result<DiapIdentity> {
        Err(AloudError::AgentError(
            "DIAP identity management is not available in WASM build".to_string(),
        ))
    }

    pub async fn get_identity(&self, _cid: &str) -> Result<DiapIdentity> {
        Err(AloudError::AgentError(
            "DIAP identity management is not available in WASM build".to_string(),
        ))
    }
}

// EncryptedPeerPayload::from_encrypted is already implemented in discovery.rs

