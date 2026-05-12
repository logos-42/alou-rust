//! Identity Handler
//!
//! Handles DIAP identity management by integrating desktop's DIAP implementation
//! with alou_code's session identity system.

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use alou_code_diap::DiapIdentity;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityInfo {
    pub did: String,
    pub ipns: String,
    pub cid: String,
    pub session_id: String,
    pub public_key: Option<String>,
    pub gateway_url: Option<String>,
}

impl From<DiapIdentity> for IdentityInfo {
    fn from(diap: DiapIdentity) -> Self {
        Self {
            did: diap.did,
            ipns: diap.ipns,
            cid: diap.cid,
            session_id: String::new(),
            public_key: Some(diap.public_key),
            gateway_url: Some(diap.gateway_url),
        }
    }
}

pub struct IdentityHandler {
    session_id: Option<String>,
    identity: Option<IdentityInfo>,
}

impl IdentityHandler {
    pub fn new() -> Self {
        Self {
            session_id: None,
            identity: None,
        }
    }

    pub fn set_session(&mut self, session_id: String) {
        self.session_id = Some(session_id);
    }

    pub fn set_identity(&mut self, identity: IdentityInfo) {
        self.identity = Some(identity);
    }

    pub fn get_identity(&self) -> Option<&IdentityInfo> {
        self.identity.as_ref()
    }

    pub fn get_session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }

    pub fn clear(&mut self) {
        self.session_id = None;
        self.identity = None;
    }

    pub fn to_session_identity(&self) -> Option<alou_code_runtime::SessionIdentity> {
        self.identity.as_ref().map(|id| {
            alou_code_runtime::SessionIdentity::new(
                format!("Agent-{}", &id.did[..8]),
                id.session_id.clone(),
                format!("DID: {}, IPNS: {}", id.did, id.ipns),
            )
        })
    }

    pub fn get_diap_identity(&self) -> Option<DiapIdentity> {
        self.identity.as_ref().map(|id| DiapIdentity {
            did: id.did.clone(),
            ipns: id.ipns.clone(),
            cid: id.cid.clone(),
            public_key: id.public_key.clone().unwrap_or_default(),
            gateway_url: id.gateway_url.clone().unwrap_or_default(),
            ipns_key: None,
            encrypted_node_id: None,
            pubsub_topics: None,
        })
    }
}

impl Default for IdentityHandler {
    fn default() -> Self {
        Self::new()
    }
}
