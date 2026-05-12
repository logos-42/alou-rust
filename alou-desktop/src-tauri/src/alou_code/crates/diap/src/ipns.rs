//! IPNS Module for alou_code DIAP

use super::IpnsPublishResult;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpnsConfig {
    pub api_url: String,
    pub gateway_url: String,
}

pub async fn publish_ipns(
    cid: &str,
    key_name: &str,
    config: &IpnsConfig,
) -> Result<IpnsPublishResult, String> {
    Err("publish_ipns not yet implemented in alou_code_diap".to_string())
}

pub async fn resolve_ipns(
    ipns_path: &str,
    config: &IpnsConfig,
) -> Result<String, String> {
    Err("resolve_ipns not yet implemented in alou_code_diap".to_string())
}

pub async fn generate_ipns_key(
    key_name: &str,
    config: &IpnsConfig,
) -> Result<String, String> {
    Err("generate_ipns_key not yet implemented in alou_code_diap".to_string())
}
