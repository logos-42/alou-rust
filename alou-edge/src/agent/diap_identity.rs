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

        // Generate DID document using SDK
        let did_document = identity_manager
            .generate_did_document()
            .await
            .map_err(|e| AloudError::AgentError(format!("Failed to generate DID document: {}", e)))?;

        // Publish DID document to IPFS
        let cid = identity_manager
            .publish_to_ipfs(&did_document)
            .await
            .map_err(|e| AloudError::AgentError(format!("Failed to publish to IPFS: {}", e)))?;

        // Publish to IPNS
        let ipns_name = if let Some(ref key) = self.config.ipns_key {
            identity_manager
                .publish_to_ipns_with_key(&cid, key)
                .await
                .map_err(|e| AloudError::AgentError(format!("Failed to publish to IPNS: {}", e)))?
        } else {
            identity_manager
                .publish_to_ipns(&cid)
                .await
                .map_err(|e| AloudError::AgentError(format!("Failed to publish to IPNS: {}", e)))?
        };

        let ipns = if ipns_name.starts_with("/ipns/") {
            ipns_name
        } else {
            format!("/ipns/{}", ipns_name)
        };

        // Extract public key from IPNS name (or generate from IPNS key)
        let public_key = format!("pubkey_{}", ipns.trim_start_matches("/ipns/"));

        Ok(DiapIdentity::new(
            did_document.id,
            ipns,
            cid,
            public_key,
            None, // Encrypted peer ID would be generated during actual identity creation
        ))
    }

    /// Add data to IPFS and return CID
    async fn add_to_ipfs(&self, data: &[u8], filename: Option<String>) -> Result<String> {
        use worker::{Request, RequestInit, Method, Fetch, Headers};
        use js_sys::{Object, Reflect, Uint8Array};
        use wasm_bindgen::{JsValue, JsCast};

        let api_url = self.config.ipfs_api_url.trim_end_matches('/');
        let endpoint = format!("{}/api/v0/add?pin=true", api_url);

        // Create FormData using js_sys
        let form_data = js_sys::Object::new();
        let file_name = filename.unwrap_or_else(|| "did.json".to_string());
        
        // Create a Blob from the data
        let uint8_array = Uint8Array::new_with_length(data.len() as u32);
        uint8_array.copy_from(data);
        
        let blob_options = js_sys::Object::new();
        Reflect::set(&blob_options, &JsValue::from_str("type"), &JsValue::from_str("application/json"))
            .map_err(|e| AloudError::AgentError(format!("Failed to set blob type: {:?}", e)))?;
        
        let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(
            &js_sys::Array::of1(&uint8_array.into()),
            blob_options.as_ref(),
        )
        .map_err(|e| AloudError::AgentError(format!("Failed to create Blob: {:?}", e)))?;

        // Create FormData and append the blob
        let form_data_js = js_sys::Reflect::construct(
            &js_sys::Function::new_no_args("FormData").into(),
            &js_sys::Array::new(),
        )
        .map_err(|e| AloudError::AgentError(format!("Failed to create FormData: {:?}", e)))?;
        
        let form_data_obj: &web_sys::FormData = form_data_js.dyn_ref()
            .ok_or_else(|| AloudError::AgentError("FormData is not available".to_string()))?;
        
        form_data_obj.append_with_blob("file", &blob)
            .map_err(|e| AloudError::AgentError(format!("Failed to append file: {:?}", e)))?;

        let mut init = RequestInit::new();
        init.with_method(Method::Post);
        init.with_body(Some(form_data_js.into()));

        let request = Request::new_with_init(&endpoint, &init)
            .map_err(|e| AloudError::AgentError(format!("Failed to create request: {}", e)))?;

        let mut response = Fetch::Request(request)
            .send()
            .await
            .map_err(|e| AloudError::AgentError(format!("IPFS add request failed: {}", e)))?;

        let response_text = response
            .text()
            .await
            .map_err(|e| AloudError::AgentError(format!("IPFS add response error: {}", e)))?;

        // Parse response - IPFS returns newline-delimited JSON
        let lines: Vec<&str> = response_text.trim().lines().collect();
        let last_line = lines.last().ok_or_else(|| {
            AloudError::AgentError("Empty response from IPFS add".to_string())
        })?;

        #[derive(Deserialize)]
        struct IpfsAddResponse {
            #[serde(rename = "Hash")]
            hash: String,
        }

        let parsed: IpfsAddResponse = serde_json::from_str(last_line)
            .map_err(|e| AloudError::AgentError(format!("Failed to parse IPFS add response: {}", e)))?;

        Ok(parsed.hash)
    }

    /// Publish CID to IPNS and return IPNS name
    async fn publish_ipns(&self, cid: &str) -> Result<String> {
        use worker::{Request, RequestInit, Method, Fetch};
        use wasm_bindgen::JsValue;

        let api_url = self.config.ipfs_api_url.trim_end_matches('/');
        let mut endpoint = format!("{}/api/v0/name/publish?arg=/ipfs/{}", api_url, cid);

        if let Some(ref key) = self.config.ipns_key {
            endpoint.push_str(&format!("&key={}", key));
        }

        let mut init = RequestInit::new();
        init.with_method(Method::Post);

        let request = Request::new_with_init(&endpoint, &init)
            .map_err(|e| AloudError::AgentError(format!("Failed to create IPNS publish request: {}", e)))?;

        let mut response = Fetch::Request(request)
            .send()
            .await
            .map_err(|e| AloudError::AgentError(format!("IPNS publish request failed: {}", e)))?;

        let response_text = response
            .text()
            .await
            .map_err(|e| AloudError::AgentError(format!("IPNS publish response error: {}", e)))?;

        #[derive(Deserialize)]
        struct IpnsPublishResponse {
            name: String,
        }

        let parsed: IpnsPublishResponse = serde_json::from_str(&response_text)
            .map_err(|e| AloudError::AgentError(format!("Failed to parse IPNS publish response: {}", e)))?;

        Ok(parsed.name)
    }

    fn simple_hash_fragment(input: &str) -> String {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        input.hash(&mut hasher);
        format!("{:x}", hasher.finish())
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
