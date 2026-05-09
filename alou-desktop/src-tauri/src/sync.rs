// Wallet sync server module
use tauri::AppHandle;

#[tauri::command]
pub async fn write_wallet_sync_data(data: String) -> Result<(), String> {
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

#[tauri::command]
pub async fn read_wallet_sync_data() -> Result<Option<String>, String> {
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

#[tauri::command]
pub async fn start_wallet_sync_server(_app: AppHandle) -> Result<u16, String> {
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

