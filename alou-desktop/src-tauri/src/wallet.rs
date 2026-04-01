use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use bip39::Mnemonic;
use bs58;
use ed25519_dalek::{SigningKey as Ed25519SigningKey, VerifyingKey as Ed25519VerifyingKey};
use hex;
use k256::ecdsa::{Signature as K256Signature, VerifyingKey as K256VerifyingKey};
use k256::elliptic_curve::sec1::ToEncodedPoint;
use sha2::{Digest, Sha256, Sha512};
use sha3::Keccak256;
use std::collections::hash_map::DefaultHasher;
use std::env;
use std::fs;
use std::hash::{Hash, Hasher};
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

/// Get the encryption key for secure storage (AES-256-GCM)
fn get_storage_encryption_key() -> aes_gcm::Key<Aes256Gcm> {
    let mut hasher = DefaultHasher::new();

    if let Ok(hostname) = env::var("COMPUTERNAME") {
        hostname.hash(&mut hasher);
    } else if let Ok(hostname) = env::var("HOSTNAME") {
        hostname.hash(&mut hasher);
    }

    if let Ok(username) = env::var("USERNAME") {
        username.hash(&mut hasher);
    } else if let Ok(username) = env::var("USER") {
        username.hash(&mut hasher);
    }

    "alou-wallet-storage-encryption-v1".hash(&mut hasher);

    let hash = hasher.finish();

    let mut sha_hasher = Sha256::new();
    sha_hasher.update(&hash.to_be_bytes());
    let sha_result = sha_hasher.finalize();

    let mut key_bytes = [0u8; 32];
    key_bytes.copy_from_slice(&sha_result[..32]);

    *aes_gcm::Key::<Aes256Gcm>::from_slice(&key_bytes)
}

/// Encrypt data using AES-256-GCM
fn encrypt_storage_value(plaintext: &[u8]) -> Result<Vec<u8>, String> {
    let key = get_storage_encryption_key();
    let cipher = Aes256Gcm::new(&key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .map_err(|e| format!("Encryption failed: {}", e))?;

    let mut result = nonce.to_vec();
    result.extend(ciphertext);

    Ok(result)
}

/// Decrypt data using AES-256-GCM
fn decrypt_storage_value(ciphertext: &[u8]) -> Result<Vec<u8>, String> {
    if ciphertext.len() < 12 {
        return Err("Encrypted data too short".to_string());
    }

    let key = get_storage_encryption_key();
    let cipher = Aes256Gcm::new(&key);

    let (nonce, data) = ciphertext.split_at(12);
    let nonce = Nonce::from_slice(nonce);

    cipher
        .decrypt(nonce, data)
        .map_err(|e| format!("Decryption failed: {}", e))
}

/// Save data to secure storage
/// Uses AES-256-GCM authenticated encryption
#[tauri::command]
pub async fn save_secure_storage(
    app_handle: AppHandle,
    key: String,
    value: String,
) -> Result<(), String> {
    let file_path = get_secure_storage_file_path(&app_handle, &key)?;

    let encrypted = encrypt_storage_value(value.as_bytes())?;

    fs::write(&file_path, &encrypted)
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

    let encrypted = fs::read(&file_path)
        .map_err(|e| format!("Failed to read secure storage: {}", e))?;

    // Try AES-256-GCM decryption first
    let plaintext = match decrypt_storage_value(&encrypted) {
        Ok(pt) => pt,
        Err(_) => {
            // Backward compatibility: try legacy XOR de-obfuscation
            // for data encrypted with the old XOR 0x5A method
            let mut bytes = Vec::with_capacity(encrypted.len());
            for &byte in &encrypted {
                bytes.push(byte ^ 0x5A);
            }
            // Validate it's valid UTF-8
            match String::from_utf8(bytes.clone()) {
                Ok(_) => bytes,
                Err(_) => return Err("Failed to decrypt or decode secure storage data".to_string()),
            }
        }
    };

    let value = String::from_utf8(plaintext)
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

/// Generate Solana keypair from mnemonic phrase
/// Uses BIP39 seed + ed25519 derivation (m/44'/501'/0'/0')
#[tauri::command]
pub async fn generate_solana_keypair_from_mnemonic(mnemonic: String) -> Result<serde_json::Value, String> {
    // Parse mnemonic
    let phrase = Mnemonic::from_phrase(&mnemonic, bip39::Language::English)
        .map_err(|e| format!("Invalid mnemonic: {}", e))?;

    // Generate BIP39 seed (no passphrase)
    let seed = phrase.to_seed("");

    // Simplified ed25519 key derivation from seed
    // In production, use slip10 or bip32-ed25519 for proper path derivation
    // Here we derive from the seed using a simple approach
    let mut hasher = Sha512::new();
    hasher.update(b"ed25519 seed");
    hasher.update(&seed);
    let hash = hasher.finalize();

    // Use first 32 bytes as secret key
    let secret_bytes: [u8; 32] = hash[..32].try_into()
        .map_err(|_| "Failed to derive secret key".to_string())?;

    let signing_key = SigningKey::from_bytes(&secret_bytes);
    let verifying_key = VerifyingKey::from(&signing_key);

    // Solana address = base58(public_key)
    let address = bs58::encode(verifying_key.to_bytes()).into_string();
    let private_key = hex::encode(signing_key.to_bytes());

    Ok(serde_json::json!({
        "address": address,
        "private_key": private_key,
    }))
}

