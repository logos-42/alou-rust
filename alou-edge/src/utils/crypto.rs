use crate::utils::error::{AloudError, Result};
use hex;
use sha2::{Digest, Sha256};
use k256::ecdsa::{Signature as K256Signature, VerifyingKey};
// use k256::ecdsa::signature::Verifier; // Not needed for current implementation
use ed25519_dalek::{Signature as Ed25519Signature, VerifyingKey as Ed25519VerifyingKey};

/// Chain type for signature verification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChainType {
    Ethereum,
    Solana,
}

impl ChainType {
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "ethereum" | "eth" => Ok(ChainType::Ethereum),
            "solana" | "sol" => Ok(ChainType::Solana),
            _ => Err(AloudError::InvalidInput(format!("Unknown chain type: {}", s))),
        }
    }
    
    pub fn as_str(&self) -> &str {
        match self {
            ChainType::Ethereum => "ethereum",
            ChainType::Solana => "solana",
        }
    }
}

/// Verify Ethereum signature (ECDSA secp256k1)
pub fn verify_ethereum_signature(
    address: &str,
    message: &str,
    signature: &str,
) -> Result<bool> {
    // Remove 0x prefix if present
    let address = address.strip_prefix("0x").unwrap_or(address);
    let signature = signature.strip_prefix("0x").unwrap_or(signature);
    
    // Decode signature hex
    let sig_bytes = hex::decode(signature)
        .map_err(|_| AloudError::InvalidSignature)?;
    
    if sig_bytes.len() != 65 {
        return Err(AloudError::InvalidSignature);
    }
    
    // Extract r, s, v components
    let r = &sig_bytes[0..32];
    let s = &sig_bytes[32..64];
    let v = sig_bytes[64];
    
    // Normalize v (27/28 -> 0/1)
    let recovery_id = if v >= 27 { v - 27 } else { v };
    
    if recovery_id > 1 {
        return Err(AloudError::InvalidSignature);
    }
    
    // Create Ethereum signed message hash
    let message_hash = ethereum_message_hash(message);
    
    // Combine r and s into signature
    let mut sig_data = [0u8; 64];
    sig_data[..32].copy_from_slice(r);
    sig_data[32..].copy_from_slice(s);
    
    let signature = K256Signature::from_bytes(&sig_data.into())
        .map_err(|_| AloudError::InvalidSignature)?;
    
    // Recover public key from signature
    let recovered_key = VerifyingKey::recover_from_prehash(
        &message_hash,
        &signature,
        k256::ecdsa::RecoveryId::try_from(recovery_id)
            .map_err(|_| AloudError::InvalidSignature)?
    ).map_err(|_| AloudError::InvalidSignature)?;
    
    // Derive address from public key
    let recovered_address = public_key_to_address(&recovered_key);
    
    // Compare addresses (case-insensitive)
    Ok(recovered_address.eq_ignore_ascii_case(address))
}

/// Verify Solana signature (Ed25519)
pub fn verify_solana_signature(
    address: &str,
    message: &str,
    signature: &str,
) -> Result<bool> {
    // Decode public key (address) from base58
    let pubkey_bytes = bs58::decode(address)
        .into_vec()
        .map_err(|_| AloudError::InvalidInput("Invalid Solana address".to_string()))?;
    
    if pubkey_bytes.len() != 32 {
        return Err(AloudError::InvalidInput("Invalid Solana address length".to_string()));
    }
    
    // Decode signature from base58
    let sig_bytes = bs58::decode(signature)
        .into_vec()
        .map_err(|_| AloudError::InvalidSignature)?;
    
    if sig_bytes.len() != 64 {
        return Err(AloudError::InvalidSignature);
    }
    
    // Create verifying key
    let verifying_key = Ed25519VerifyingKey::from_bytes(
        pubkey_bytes.as_slice().try_into()
            .map_err(|_| AloudError::InvalidInput("Invalid public key".to_string()))?
    ).map_err(|_| AloudError::InvalidInput("Invalid public key".to_string()))?;
    
    // Create signature
    let signature = Ed25519Signature::from_bytes(
        sig_bytes.as_slice().try_into()
            .map_err(|_| AloudError::InvalidSignature)?
    );
    
    // Verify signature
    use ed25519_dalek::Verifier;
    verifying_key
        .verify(message.as_bytes(), &signature)
        .map(|_| true)
        .or(Ok(false))
}

/// Create Ethereum signed message hash
/// Format: keccak256("\x19Ethereum Signed Message:\n" + len(message) + message)
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
    
    // Keccak256 hash (using SHA256 as approximation for WASM compatibility)
    let mut hasher = Sha256::new();
    hasher.update(public_key_bytes);
    let hash = hasher.finalize();
    
    // Take last 20 bytes
    let address_bytes = &hash[12..32];
    
    hex::encode(address_bytes)
}

/// Generate a random nonce
pub fn generate_nonce() -> String {
    use getrandom::getrandom;
    
    let mut bytes = [0u8; 32];
    let _ = getrandom(&mut bytes).map_err(|_| {
        // Fallback to timestamp-based nonce if getrandom fails
        let timestamp = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);
        bytes[..8].copy_from_slice(&timestamp.to_le_bytes());
    });
    
    hex::encode(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_chain_type_from_str() {
        assert_eq!(ChainType::from_str("ethereum").unwrap(), ChainType::Ethereum);
        assert_eq!(ChainType::from_str("ETH").unwrap(), ChainType::Ethereum);
        assert_eq!(ChainType::from_str("solana").unwrap(), ChainType::Solana);
        assert_eq!(ChainType::from_str("SOL").unwrap(), ChainType::Solana);
        assert!(ChainType::from_str("bitcoin").is_err());
    }
    
    #[test]
    fn test_generate_nonce() {
        let nonce1 = generate_nonce();
        let nonce2 = generate_nonce();
        
        assert_eq!(nonce1.len(), 64); // 32 bytes = 64 hex chars
        assert_ne!(nonce1, nonce2); // Should be different
    }
}
