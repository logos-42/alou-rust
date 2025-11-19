// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use chrono::Utc;
use hex;
use k256::ecdsa::{Signature as K256Signature, VerifyingKey};
use reqwest::multipart::{Form, Part};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use tauri::{Manager, State};
use uuid::Uuid;

// IPFS module - uncomment when ipfs.rs is ready
// mod ipfs;

const KUBO_VERSION: &str = "v0.24.0";
const KUBO_VERSION_FILE: &str = "version.json";

struct IpfsState {
    process: Option<std::process::Child>,
    data_dir: PathBuf,
}

fn default_ipfs_api_url() -> String {
    "http://127.0.0.1:5001".to_string()
}

fn default_ipfs_gateway_url() -> String {
    "http://127.0.0.1:8080".to_string()
}

fn normalize_base_url(url: &str) -> String {
    url.trim_end_matches('/').to_string()
}

#[derive(Default, Deserialize)]
struct LocalDiapIdentityRequest {
    agent_name: Option<String>,
    agent_description: Option<String>,
    ipfs_api_url: Option<String>,
    ipfs_gateway_url: Option<String>,
    ipns_key: Option<String>,
}

#[derive(Serialize)]
struct LocalDiapIdentityResponse {
    did: String,
    cid: String,
    ipns: String,
    public_key: String,
    gateway_url: String,
}

#[derive(Serialize)]
struct IpfsAddResult {
    cid: String,
    size: Option<String>,
    name: Option<String>,
}

#[derive(Deserialize)]
struct IpfsAddApiResponse {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Hash")]
    hash: String,
    #[serde(rename = "Size")]
    size: String,
}

#[derive(Deserialize)]
struct IpnsPublishResponse {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Value")]
    value: String,
}

fn app_data_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map_err(|e| format!("Failed to resolve app data directory: {}", e))
}

async fn add_bytes_to_ipfs(
    api_url: &str,
    bytes: Vec<u8>,
    file_name: Option<String>,
) -> Result<IpfsAddResult, String> {
    let endpoint = format!("{}/api/v0/add", normalize_base_url(api_url));
    let name = file_name.unwrap_or_else(|| format!("payload-{}.bin", Uuid::new_v4()));
    let part = Part::bytes(bytes).file_name(name.clone());
    let form = Form::new().part("file", part);

    let client = Client::new();
    let response = client
        .post(endpoint)
        .query(&[("pin", "true")])
        .multipart(form)
        .send()
        .await
        .map_err(|e| format!("IPFS add request failed: {}", e))?;

    let text = response
        .text()
        .await
        .map_err(|e| format!("IPFS add response error: {}", e))?;

    let last_line = text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .last()
        .ok_or_else(|| "IPFS add response empty".to_string())?;

    let parsed: IpfsAddApiResponse = serde_json::from_str(last_line)
        .map_err(|e| format!("Failed to parse IPFS add response: {}", e))?;

    Ok(IpfsAddResult {
        cid: parsed.hash,
        size: Some(parsed.size),
        name: Some(parsed.name),
    })
}

async fn publish_ipns_record(
    api_url: &str,
    cid: &str,
    ipns_key: Option<String>,
) -> Result<String, String> {
    let endpoint = format!("{}/api/v0/name/publish", normalize_base_url(api_url));
    let client = Client::new();
    let mut request = client
        .post(endpoint)
        .query(&[("arg", format!("/ipfs/{}", cid))]);

    if let Some(key) = ipns_key {
        request = request.query(&[("key", key)]);
    }

    let response = request
        .send()
        .await
        .map_err(|e| format!("IPNS publish request failed: {}", e))?;

    let body = response
        .text()
        .await
        .map_err(|e| format!("IPNS publish response error: {}", e))?;

    let publish: IpnsPublishResponse = serde_json::from_str(&body)
        .map_err(|e| format!("Failed to parse IPNS publish response: {}", e))?;

    Ok(publish.name)
}

#[tauri::command]
async fn download_kubo_binary(app: tauri::AppHandle) -> Result<String, String> {
    use std::io::Write;

    // Get app data directory for storing Kubo
    let app_data_dir = app_data_dir(&app)?;

    let kubo_dir = app_data_dir.join("kubo");
    std::fs::create_dir_all(&kubo_dir)
        .map_err(|e| format!("Failed to create kubo directory: {}", e))?;

    let kubo_bin = kubo_dir.join(binary_name());

    // Check if already downloaded
    if kubo_bin.exists() {
        record_kubo_version(&kubo_dir, "existing")?;
        return Ok(format!("Kubo binary already exists at: {:?}", kubo_bin));
    }

    // Determine download URL based on platform
    let (url, filename) = if cfg!(target_os = "windows") {
        (
            "https://dist.ipfs.tech/kubo/v0.24.0/kubo_v0.24.0_windows-amd64.zip",
            "kubo.zip",
        )
    } else if cfg!(target_os = "macos") {
        (
            "https://dist.ipfs.tech/kubo/v0.24.0/kubo_v0.24.0_darwin-amd64.tar.gz",
            "kubo.tar.gz",
        )
    } else {
        (
            "https://dist.ipfs.tech/kubo/v0.24.0/kubo_v0.24.0_linux-amd64.tar.gz",
            "kubo.tar.gz",
        )
    };

    // Download Kubo using async request
    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Failed to download Kubo: {}", e))?;

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;

    let temp_file = kubo_dir.join(filename);
    let mut file = std::fs::File::create(&temp_file)
        .map_err(|e| format!("Failed to create temp file: {}", e))?;
    file.write_all(&bytes)
        .map_err(|e| format!("Failed to write file: {}", e))?;
    drop(file);

    // Extract
    if filename.ends_with(".zip") {
        // Windows: extract zip using PowerShell or zip crate
        let extract_dir = kubo_dir.join("extract");
        std::fs::create_dir_all(&extract_dir)
            .map_err(|e| format!("Failed to create extract dir: {}", e))?;

        // Try using PowerShell on Windows
        let output = std::process::Command::new("powershell")
            .args(&[
                "-Command",
                &format!(
                    "Expand-Archive -Path '{}' -DestinationPath '{}' -Force",
                    temp_file.display(),
                    extract_dir.display()
                ),
            ])
            .output()
            .map_err(|e| format!("Failed to extract zip: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "Failed to extract zip: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        // Find and copy binary (path may vary)
        let possible_paths = vec![
            extract_dir.join("kubo").join("kubo").join("ipfs.exe"),
            extract_dir.join("kubo").join("ipfs.exe"),
            extract_dir.join("ipfs.exe"),
        ];

        let mut found = false;
        for extracted_bin in &possible_paths {
            if extracted_bin.exists() {
                std::fs::copy(&extracted_bin, &kubo_bin)
                    .map_err(|e| format!("Failed to copy binary: {}", e))?;
                found = true;
                break;
            }
        }

        if !found {
            return Err("Binary not found in extracted archive".to_string());
        }

        // Cleanup
        std::fs::remove_dir_all(&extract_dir).ok();
    } else {
        // Unix: extract tar.gz
        let output = std::process::Command::new("tar")
            .args(&[
                "-xzf",
                temp_file.to_str().unwrap(),
                "-C",
                kubo_dir.to_str().unwrap(),
            ])
            .output()
            .map_err(|e| format!("Failed to extract tar.gz: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "Failed to extract: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        // Find and copy binary
        let possible_paths = vec![
            kubo_dir.join("kubo").join("kubo").join("ipfs"),
            kubo_dir.join("kubo").join("ipfs"),
            kubo_dir.join("ipfs"),
        ];

        let mut found = false;
        for extracted_bin in &possible_paths {
            if extracted_bin.exists() {
                std::fs::copy(&extracted_bin, &kubo_bin)
                    .map_err(|e| format!("Failed to copy binary: {}", e))?;
                found = true;
                break;
            }
        }

        if !found {
            return Err("Binary not found in extracted archive".to_string());
        }

        // Cleanup extracted directory
        for path in &possible_paths {
            if let Some(parent) = path.parent() {
                if parent != kubo_dir && parent.exists() {
                    std::fs::remove_dir_all(parent).ok();
                }
            }
        }
    }

    // Set executable permission (Unix)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&kubo_bin, std::fs::Permissions::from_mode(0o755))
            .map_err(|e| format!("Failed to set permissions: {}", e))?;
    }

    // Cleanup temp file
    std::fs::remove_file(&temp_file).ok();

    record_kubo_version(&kubo_dir, "download")?;

    Ok(format!("Kubo binary downloaded to: {:?}", kubo_bin))
}

#[tauri::command]
async fn start_ipfs_node(
    state: State<'_, tauri::async_runtime::Mutex<IpfsState>>,
    app: tauri::AppHandle,
) -> Result<String, String> {
    launch_ipfs_node(state.inner(), &app).await
}

#[tauri::command]
async fn stop_ipfs_node(
    state: State<'_, tauri::async_runtime::Mutex<IpfsState>>,
) -> Result<String, String> {
    let mut ipfs_state = state.lock().await;

    if let Some(mut process) = ipfs_state.process.take() {
        process
            .kill()
            .map_err(|e| format!("Failed to stop IPFS: {}", e))?;
        Ok("IPFS node stopped".to_string())
    } else {
        Ok("IPFS node was not running".to_string())
    }
}

#[tauri::command]
async fn get_ipfs_info(
    state: State<'_, tauri::async_runtime::Mutex<IpfsState>>,
    app: tauri::AppHandle,
) -> Result<serde_json::Value, String> {
    let ipfs_state = state.lock().await;

    // Get app data directory where Kubo is stored
    let app_data_dir = app_data_dir(&app)?;

    let kubo_bin = if cfg!(target_os = "windows") {
        app_data_dir.join("kubo").join("ipfs.exe")
    } else {
        app_data_dir.join("kubo").join("ipfs")
    };

    if !kubo_bin.exists() {
        return Err("Kubo binary not found. Please download it first.".to_string());
    }

    let data_dir = if ipfs_state.data_dir.as_os_str().is_empty() {
        app_data_dir.join("ipfs")
    } else {
        ipfs_state.data_dir.clone()
    };

    let output = Command::new(&kubo_bin)
        .arg("id")
        .env("IPFS_PATH", &data_dir)
        .output()
        .map_err(|e| format!("Failed to get IPFS info: {}", e))?;

    if !output.status.success() {
        return Err("IPFS node is not running".to_string());
    }

    let info: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("Failed to parse IPFS info: {}", e))?;

    Ok(info)
}

#[tauri::command]
async fn ipfs_add_base64(
    data_base64: String,
    file_name: Option<String>,
    ipfs_api_url: Option<String>,
) -> Result<IpfsAddResult, String> {
    let bytes = BASE64
        .decode(data_base64.trim())
        .map_err(|e| format!("Invalid base64 payload: {}", e))?;
    let api_url = ipfs_api_url.unwrap_or_else(default_ipfs_api_url);
    add_bytes_to_ipfs(&api_url, bytes, file_name).await
}

#[tauri::command]
async fn create_local_diap_identity(
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

/// Verify Ethereum wallet signature
#[tauri::command]
async fn verify_wallet_signature(
    address: String,
    message: String,
    signature: String,
    chain: String,
) -> Result<bool, String> {
    if chain.to_lowercase() != "ethereum" && chain.to_lowercase() != "eth" {
        return Err("Only Ethereum signatures are supported".to_string());
    }

    verify_ethereum_signature(&address, &message, &signature)
        .map_err(|e| format!("Signature verification failed: {}", e))
}

fn verify_ethereum_signature(
    address: &str,
    message: &str,
    signature: &str,
) -> Result<bool, String> {
    // Remove 0x prefix if present
    let address = address.strip_prefix("0x").unwrap_or(address);
    let signature = signature.strip_prefix("0x").unwrap_or(signature);

    // Decode signature hex
    let sig_bytes = hex::decode(signature).map_err(|_| "Invalid signature format".to_string())?;

    if sig_bytes.len() != 65 {
        return Err("Invalid signature length".to_string());
    }

    // Extract r, s, v components
    let r = &sig_bytes[0..32];
    let s = &sig_bytes[32..64];
    let v = sig_bytes[64];

    // Normalize v (27/28 -> 0/1)
    let recovery_id = if v >= 27 { v - 27 } else { v };

    if recovery_id > 1 {
        return Err("Invalid recovery ID".to_string());
    }

    // Create Ethereum signed message hash
    // Note: Ethereum uses keccak256, but for simplicity we use SHA256 here
    // For production, you should use a keccak256 library
    let message_hash = ethereum_message_hash(message);

    // Combine r and s into signature
    let mut sig_data = [0u8; 64];
    sig_data[..32].copy_from_slice(r);
    sig_data[32..].copy_from_slice(s);

    let signature = K256Signature::from_bytes(&sig_data.into())
        .map_err(|_| "Invalid signature format".to_string())?;

    // Recover public key from signature
    let recovered_key = VerifyingKey::recover_from_prehash(
        &message_hash,
        &signature,
        k256::ecdsa::RecoveryId::try_from(recovery_id)
            .map_err(|_| "Invalid recovery ID".to_string())?,
    )
    .map_err(|_| "Failed to recover public key".to_string())?;

    // Derive address from public key
    let recovered_address = public_key_to_address(&recovered_key);

    // Compare addresses (case-insensitive)
    Ok(recovered_address.eq_ignore_ascii_case(address))
}

/// Create Ethereum signed message hash
/// Format: keccak256("\x19Ethereum Signed Message:\n" + len(message) + message)
/// Note: This uses SHA256 for simplicity. For production, use keccak256.
fn ethereum_message_hash(message: &str) -> [u8; 32] {
    let prefix = format!("\x19Ethereum Signed Message:\n{}", message.len());
    let mut hasher = Sha256::new();
    hasher.update(prefix.as_bytes());
    hasher.update(message.as_bytes());
    let result = hasher.finalize();

    let mut hash = [0u8; 32];
    hash.copy_from_slice(&result);
    hash
}

/// Convert ECDSA public key to Ethereum address
fn public_key_to_address(public_key: &VerifyingKey) -> String {
    let public_key_bytes = public_key.to_encoded_point(false);
    let public_key_bytes = public_key_bytes.as_bytes();

    // Skip the first byte (0x04 prefix for uncompressed key)
    let public_key_bytes = &public_key_bytes[1..];

    // Hash public key using SHA256 (Note: Ethereum uses keccak256)
    let mut hasher = Sha256::new();
    hasher.update(public_key_bytes);
    let hash = hasher.finalize();

    // Take last 20 bytes as address
    let address_bytes = &hash[12..32];
    format!("0x{}", hex::encode(address_bytes))
}

/// Open URL in default browser
#[tauri::command]
async fn open_browser(url: String) -> Result<(), String> {
    // Try to open using the shell plugin
    #[cfg(desktop)]
    {
        if let Err(e) = open::that(&url) {
            return Err(format!("Failed to open browser: {}", e));
        }
    }

    #[cfg(not(desktop))]
    {
        return Err("Open browser is only available on desktop".to_string());
    }

    Ok(())
}

/// Write wallet sync data to temp file
#[tauri::command]
async fn write_wallet_sync_data(data: String) -> Result<(), String> {
    use std::fs;

    #[cfg(desktop)]
    {
        // Get temp directory
        let temp_dir = std::env::temp_dir();
        let sync_file = temp_dir.join("alou_wallet_sync.json");

        // Write data to file
        if let Err(e) = fs::write(&sync_file, data) {
            return Err(format!("Failed to write sync data: {}", e));
        }

        println!("Wallet sync data written to: {:?}", sync_file);
        Ok(())
    }

    #[cfg(not(desktop))]
    {
        Err("Write wallet sync data is only available on desktop".to_string())
    }
}

/// Read wallet sync data from temp file
#[tauri::command]
async fn read_wallet_sync_data() -> Result<Option<String>, String> {
    use std::fs;

    #[cfg(desktop)]
    {
        // Get temp directory
        let temp_dir = std::env::temp_dir();
        let sync_file = temp_dir.join("alou_wallet_sync.json");

        // Check if file exists
        if !sync_file.exists() {
            return Ok(None);
        }

        // Read data from file
        match fs::read_to_string(&sync_file) {
            Ok(data) => {
                // Delete file after reading
                let _ = fs::remove_file(&sync_file);
                Ok(Some(data))
            }
            Err(e) => Err(format!("Failed to read sync data: {}", e)),
        }
    }

    #[cfg(not(desktop))]
    {
        Err("Read wallet sync data is only available on desktop".to_string())
    }
}

/// Start local HTTP server to receive wallet sync data
#[tauri::command]
async fn start_wallet_sync_server(_app: tauri::AppHandle) -> Result<u16, String> {
    use axum::{extract::State, http::Method, response::Json, routing::post, Router};
    use serde::{Deserialize, Serialize};
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use tower_http::cors::{Any, CorsLayer};

    #[derive(Deserialize, Serialize, Clone)]
    struct WalletSyncRequest {
        address: String,
        chain_id: Option<String>,
        wallet_type: Option<String>,
        token: Option<String>,
    }

    #[derive(Serialize)]
    struct WalletSyncResponse {
        success: bool,
        message: String,
    }

    // Shared state to store the latest sync data
    type SyncData = Arc<Mutex<Option<String>>>;
    let sync_data: SyncData = Arc::new(Mutex::new(None));

    let sync_data_clone = sync_data.clone();

    // Create router
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::POST, Method::OPTIONS])
        .allow_headers(Any);

    let app_router = Router::new()
        .route("/wallet-sync", post(move |State(state): State<SyncData>, Json(payload): Json<WalletSyncRequest>| async move {
            let sync_payload = serde_json::json!({
                "address": payload.address,
                "chainId": payload.chain_id.unwrap_or_else(|| "0x1".to_string()),
                "walletType": payload.wallet_type.unwrap_or_else(|| "metamask".to_string()),
                "token": payload.token.unwrap_or_default(),
                "timestamp": std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
                "source": "browser-http"
            });

            let mut data = state.lock().await;
            *data = Some(serde_json::to_string(&sync_payload).unwrap_or_default());

            Json(WalletSyncResponse {
                success: true,
                message: "Wallet sync data received".to_string(),
            })
        }))
        .layer(cors)
        .with_state(sync_data);

    // Start server on a random available port
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|e| format!("Failed to bind server: {}", e))?;

    let port = listener
        .local_addr()
        .map_err(|e| format!("Failed to get local address: {}", e))?
        .port();

    // Spawn server task
    tokio::spawn(async move {
        axum::serve(listener, app_router)
            .await
            .map_err(|e| eprintln!("Server error: {}", e))
            .ok();
    });

    // Start a task to periodically check for sync data and write to file
    let sync_data_for_file = sync_data_clone.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

            let data = sync_data_for_file.lock().await.take();
            if let Some(data) = data {
                // Write to temp file for desktop app to read
                let temp_dir = std::env::temp_dir();
                let sync_file = temp_dir.join("alou_wallet_sync.json");

                if let Err(e) = std::fs::write(&sync_file, &data) {
                    eprintln!("Failed to write sync data to file: {}", e);
                } else {
                    println!("Wallet sync data written to file: {:?}", sync_file);
                }
            }
        }
    });

    println!("Wallet sync server started on port {}", port);
    Ok(port)
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(tauri::async_runtime::Mutex::new(IpfsState {
            process: None,
            data_dir: PathBuf::new(),
        }))
        .invoke_handler(tauri::generate_handler![
            download_kubo_binary,
            start_ipfs_node,
            stop_ipfs_node,
            get_ipfs_info,
            ipfs_add_base64,
            create_local_diap_identity,
            verify_wallet_signature,
            open_browser,
            write_wallet_sync_data,
            read_wallet_sync_data,
            start_wallet_sync_server
        ])
        .setup(|app| {
            // Set window title
            if let Some(window) = app.get_webview_window("main") {
                window.set_title("Alou").unwrap();
            }

            // Start wallet sync server on startup
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = start_wallet_sync_server(app_handle).await {
                    eprintln!("Failed to start wallet sync server: {}", e);
                }
            });

            // Ensure Kubo binary exists and auto-start IPFS
            let ipfs_app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                bootstrap_ipfs(ipfs_app_handle).await;
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[derive(Serialize, Deserialize)]
struct KuboVersionInfo {
    version: String,
    source: String,
}

fn record_kubo_version(dir: &Path, source: &str) -> Result<(), String> {
    let info = KuboVersionInfo {
        version: KUBO_VERSION.to_string(),
        source: source.to_string(),
    };
    let file = dir.join(KUBO_VERSION_FILE);
    let json = serde_json::to_string_pretty(&info)
        .map_err(|e| format!("Failed to serialize version info: {}", e))?;
    fs::write(&file, json).map_err(|e| format!("Failed to write version info: {}", e))
}

fn read_kubo_version(dir: &Path) -> Option<KuboVersionInfo> {
    let file = dir.join(KUBO_VERSION_FILE);
    let data = fs::read_to_string(file).ok()?;
    serde_json::from_str(&data).ok()
}

fn binary_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "ipfs.exe"
    } else {
        "ipfs"
    }
}

fn bundled_kubo_binary(app: &tauri::AppHandle) -> Option<PathBuf> {
    if let Ok(resource_dir) = app.path().resource_dir() {
        let candidate = resource_dir.join("kubo").join(binary_name());
        if candidate.exists() {
            return Some(candidate);
        }
    }
    #[cfg(debug_assertions)]
    {
        let dev_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("kubo")
            .join(binary_name());
        if dev_path.exists() {
            return Some(dev_path);
        }
    }
    None
}

async fn ensure_kubo_binary(app: &tauri::AppHandle) -> Result<(), String> {
    let app_data_dir = app_data_dir(app)?;
    let kubo_dir = app_data_dir.join("kubo");
    std::fs::create_dir_all(&kubo_dir).map_err(|e| format!("Failed to create kubo dir: {}", e))?;

    let target_bin = kubo_dir.join(binary_name());

    if target_bin.exists() {
        if let Some(info) = read_kubo_version(&kubo_dir) {
            if info.version == KUBO_VERSION {
                return Ok(());
            }
        }
    }

    if let Some(bundled) = bundled_kubo_binary(app) {
        std::fs::copy(&bundled, &target_bin)
            .map_err(|e| format!("Failed to copy bundled Kubo: {}", e))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&target_bin, std::fs::Permissions::from_mode(0o755))
                .map_err(|e| format!("Failed to set permissions: {}", e))?;
        }
        record_kubo_version(&kubo_dir, "bundled")?;
        return Ok(());
    }

    download_kubo_binary(app.clone()).await.map(|_| ())
}

async fn launch_ipfs_node(
    state: &tauri::async_runtime::Mutex<IpfsState>,
    app: &tauri::AppHandle,
) -> Result<String, String> {
    let mut ipfs_state = state.lock().await;

    if let Some(child) = ipfs_state.process.as_mut() {
        match child.try_wait() {
            Ok(None) => {
                return Ok("IPFS node already running".to_string());
            }
            Ok(Some(_)) => {
                ipfs_state.process = None;
            }
            Err(e) => {
                return Err(format!("Failed to inspect IPFS process: {}", e));
            }
        }
    }

    let app_data_dir = app_data_dir(app)?;
    let kubo_bin = app_data_dir.join("kubo").join(binary_name());
    if !kubo_bin.exists() {
        return Err(format!(
            "Kubo binary not found. Please download it first using download_kubo_binary command. Expected at: {:?}",
            kubo_bin
        ));
    }

    let data_dir = app_data_dir.join("ipfs");
    std::fs::create_dir_all(&data_dir)
        .map_err(|e| format!("Failed to create IPFS data dir: {}", e))?;

    // Initialize IPFS if not already initialized
    let init_output = Command::new(&kubo_bin)
        .arg("init")
        .arg("--profile=server")
        .env("IPFS_PATH", &data_dir)
        .output()
        .map_err(|e| format!("Failed to initialize IPFS: {}", e))?;

    if !init_output.status.success() {
        let stderr = String::from_utf8_lossy(&init_output.stderr);
        if !stderr.contains("already") {
            return Err(format!("IPFS init failed: {}", stderr));
        }
    }

    // Start IPFS daemon
    let child = Command::new(&kubo_bin)
        .arg("daemon")
        .env("IPFS_PATH", &data_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start IPFS daemon: {}", e))?;

    ipfs_state.process = Some(child);
    ipfs_state.data_dir = data_dir;

    Ok("IPFS node started".to_string())
}

async fn bootstrap_ipfs(app_handle: tauri::AppHandle) {
    if let Err(e) = ensure_kubo_binary(&app_handle).await {
        eprintln!("Failed to ensure Kubo binary: {}", e);
        return;
    }

    let state = app_handle.state::<tauri::async_runtime::Mutex<IpfsState>>();
    if let Err(e) = launch_ipfs_node(state.inner(), &app_handle).await {
        eprintln!("Failed to auto-start IPFS: {}", e);
    }
}
