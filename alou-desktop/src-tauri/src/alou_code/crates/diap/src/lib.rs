//! DIAP Identity System for alou_code Kernel
//!
//! This crate provides DIAP (Distributed Identity and Access Protocol) identity
//! management for the alou_code kernel. It handles DID creation, IPNS publishing,
//! and identity storage.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub mod identity;
pub mod ipns;

pub use identity::{
    IdentityInfo,
    CreateIdentityRequest,
    CreateIdentityResponse,
    update_identity,
    get_identity,
    archive_identity,
};
pub use ipns::{publish_ipns, resolve_ipns};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiapIdentity {
    pub did: String,
    pub cid: String,
    pub ipns: String,
    pub public_key: String,
    pub gateway_url: String,
    pub ipns_key: Option<String>,
    pub encrypted_node_id: Option<EncryptedNodeId>,
    pub pubsub_topics: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedNodeId {
    pub ciphertext: String,
    pub nonce: String,
    pub signature: String,
    pub method: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpnsPublishResult {
    pub name: String,
    pub value: String,
    pub ipns_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpnsResolveResult {
    pub ipns_path: String,
    pub cid: String,
}

pub fn create_empty_identity() -> DiapIdentity {
    DiapIdentity {
        did: String::new(),
        cid: String::new(),
        ipns: String::new(),
        public_key: String::new(),
        gateway_url: String::new(),
        ipns_key: None,
        encrypted_node_id: None,
        pubsub_topics: None,
    }
}
