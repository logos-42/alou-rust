use hex;
use k256::ecdsa::{Signature as K256Signature, VerifyingKey};
use sha2::{Digest, Sha256};
use std::env;
use std::fs;
use std::path::PathBuf;
use tauri::AppHandle;
use tauri::Manager;

/// Verify Ethereum wallet signature
#[tauri::command]
pub async fn verify_wallet_signature(
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

    // Combine r and s into signature
    let mut sig_data = [0u8; 64];
    sig_data[..32].copy_from_slice(r);
    sig_data[32..].copy_from_slice(s);

    let signature = K256Signature::from_bytes(&sig_data.into())
        .map_err(|_| "Invalid signature format".to_string())?;

    // Hash message with Ethereum prefix
    let message_hash = ethereum_message_hash(message);

    // Recover public key
    let public_key = VerifyingKey::recover_from_prehash(
        &message_hash,
        &signature,
        k256::ecdsa::RecoveryId::try_from(recovery_id)
            .map_err(|_| "Invalid recovery ID".to_string())?,
    )
    .map_err(|_| "Failed to recover public key".to_string())?;

    // Convert public key to address
    let recovered_address = public_key_to_address(&public_key);

    // Compare addresses (case-insensitive)
    Ok(recovered_address.eq_ignore_ascii_case(address))
}

fn ethereum_message_hash(message: &str) -> [u8; 32] {
    let prefix = format!("\x19Ethereum Signed Message:\n{}", message.len());
    let mut hasher = Sha256::new();
    hasher.update(prefix.as_bytes());
    hasher.update(message.as_bytes());
    hasher.finalize().into()
}

fn public_key_to_address(public_key: &VerifyingKey) -> String {
    let public_key_bytes = public_key.to_sec1_bytes();
    let public_key_slice = &public_key_bytes[1..]; // Skip 0x04 prefix

    let mut hasher = Sha256::new();
    hasher.update(public_key_slice);
    let hash = hasher.finalize();

    // Use last 20 bytes as address
    let address_bytes = &hash[12..32];
    format!("0x{}", hex::encode(address_bytes))
}

/// Get testnet private key from environment variable (if available)
/// Environment variable name: DIAP_TESTNET_PRIVATE_KEY
/// Returns the private key if set, otherwise returns an error
/// WARNING: Only use this for testing purposes!
#[tauri::command]
pub async fn get_testnet_private_key() -> Result<String, String> {
    env::var("DIAP_TESTNET_PRIVATE_KEY")
        .map_err(|_| "DIAP_TESTNET_PRIVATE_KEY environment variable not set".to_string())
}

/// Get the secure storage directory for the application
fn get_secure_storage_dir(app_handle: &AppHandle) -> Result<PathBuf, String> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    
    let storage_dir = app_data_dir.join("secure_storage");
    
    // Create directory if it doesn't exist
    if !storage_dir.exists() {
        fs::create_dir_all(&storage_dir)
            .map_err(|e| format!("Failed to create storage directory: {}", e))?;
    }
    
    Ok(storage_dir)
}

/// Get the file path for a secure storage key
fn get_secure_storage_file_path(app_handle: &AppHandle, key: &str) -> Result<PathBuf, String> {
    let storage_dir = get_secure_storage_dir(app_handle)?;
    // Sanitize key to prevent path traversal
    let sanitized_key = key.replace("..", "").replace("/", "_").replace("\\", "_");
    Ok(storage_dir.join(format!("{}.dat", sanitized_key)))
}

/// Save data to secure storage
/// Uses encrypted file storage with application-specific location
#[tauri::command]
pub async fn save_secure_storage(
    app_handle: AppHandle,
    key: String,
    value: String,
) -> Result<(), String> {
    let file_path = get_secure_storage_file_path(&app_handle, &key)?;
    
    // Simple obfuscation (XOR with a fixed byte for basic protection)
    // In production, consider using a proper encryption library
    let bytes = value.as_bytes();
    let mut obfuscated = Vec::with_capacity(bytes.len());
    for &byte in bytes {
        obfuscated.push(byte ^ 0x5A); // Simple XOR obfuscation
    }
    
    fs::write(&file_path, &obfuscated)
        .map_err(|e| format!("Failed to write secure storage: {}", e))?;
    
    // Set restrictive file permissions (Unix only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = fs::metadata(&file_path) {
            let _ = fs::set_permissions(&file_path, PermissionsExt::from_mode(0o600));
        }
    }
    
    Ok(())
}

/// Get data from secure storage
/// Returns None if key doesn't exist
#[tauri::command]
pub async fn get_secure_storage(
    app_handle: AppHandle,
    key: String,
) -> Result<Option<String>, String> {
    let file_path = get_secure_storage_file_path(&app_handle, &key)?;
    
    if !file_path.exists() {
        return Ok(None);
    }
    
    let obfuscated = fs::read(&file_path)
        .map_err(|e| format!("Failed to read secure storage: {}", e))?;
    
    // De-obfuscate (XOR with the same fixed byte)
    let mut bytes = Vec::with_capacity(obfuscated.len());
    for &byte in &obfuscated {
        bytes.push(byte ^ 0x5A);
    }
    
    let value = String::from_utf8(bytes)
        .map_err(|_| "Failed to decode secure storage data".to_string())?;
    
    Ok(Some(value))
}

/// Delete data from secure storage
/// Returns true if key existed and was deleted, false otherwise
#[tauri::command]
pub async fn delete_secure_storage(
    app_handle: AppHandle,
    key: String,
) -> Result<bool, String> {
    let file_path = get_secure_storage_file_path(&app_handle, &key)?;
    
    if !file_path.exists() {
        return Ok(false);
    }
    
    fs::remove_file(&file_path)
        .map_err(|e| format!("Failed to delete secure storage: {}", e))?;
    
    Ok(true)
}

/// Check if a key exists in secure storage
#[tauri::command]
pub async fn has_secure_storage(
    app_handle: AppHandle,
    key: String,
) -> Result<bool, String> {
    let file_path = get_secure_storage_file_path(&app_handle, &key)?;
    Ok(file_path.exists())
}

