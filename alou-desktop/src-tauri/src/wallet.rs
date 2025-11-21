use hex;
use k256::ecdsa::{Signature as K256Signature, VerifyingKey};
use sha2::{Digest, Sha256};

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

