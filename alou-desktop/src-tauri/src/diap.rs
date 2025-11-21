// DIAP module
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::ipfs_commands::{add_bytes_to_ipfs, publish_ipns_record};
use crate::utils::{default_ipfs_api_url, default_ipfs_gateway_url, normalize_base_url};

#[derive(Default, Deserialize)]
pub struct LocalDiapIdentityRequest {
    pub agent_name: Option<String>,
    pub agent_description: Option<String>,
    pub ipfs_api_url: Option<String>,
    pub ipfs_gateway_url: Option<String>,
    pub ipns_key: Option<String>,
}

#[derive(Serialize)]
pub struct LocalDiapIdentityResponse {
    pub did: String,
    pub cid: String,
    pub ipns: String,
    pub public_key: String,
    pub gateway_url: String,
}

#[tauri::command]
pub async fn create_local_diap_identity(
    params: Option<LocalDiapIdentityRequest>,
) -> Result<LocalDiapIdentityResponse, String> {
    let params = params.unwrap_or_default();
    let ipfs_api = params
        .ipfs_api_url
        .clone()
        .unwrap_or_else(default_ipfs_api_url);
    let gateway = params
        .ipfs_gateway_url
        .clone()
        .unwrap_or_else(default_ipfs_gateway_url);
    let agent_name = params
        .agent_name
        .clone()
        .unwrap_or_else(|| "Claude Agent SDK".to_string());
    let agent_description = params.agent_description.clone().unwrap_or_default();

    let did = format!("did:alou:{}", Uuid::new_v4());
    let created = Utc::now().to_rfc3339();

    let did_document = json!({
        "@context": ["https://www.w3.org/ns/did/v1"],
        "id": did,
        "created": created,
        "service": [{
            "id": format!("{}#agent", did),
            "type": "AgentEndpoint",
            "serviceEndpoint": {
                "type": "AgentProfile",
                "name": agent_name,
                "description": agent_description,
            }
        }]
    });

    let doc_bytes = serde_json::to_vec(&did_document)
        .map_err(|e| format!("Failed to serialize DID document: {}", e))?;
    let add_result = add_bytes_to_ipfs(&ipfs_api, doc_bytes, Some("did.json".to_string())).await?;

    let ipns_name =
        publish_ipns_record(&ipfs_api, &add_result.cid, params.ipns_key.clone()).await?;
    let ipns_path = format!("/ipns/{}", ipns_name);
    let gateway_url = format!("{}/ipfs/{}", normalize_base_url(&gateway), add_result.cid);
    let public_key = format!("pubkey_{}", ipns_name);

    Ok(LocalDiapIdentityResponse {
        did,
        cid: add_result.cid,
        ipns: ipns_path,
        public_key,
        gateway_url,
    })
}

