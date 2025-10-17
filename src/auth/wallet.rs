// ============================================
// Wallet Authentication
// ============================================

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WalletNonce {
    pub nonce: String,
    pub address: String,
    pub created_at: u64,
    pub expires_at: u64,
}

impl WalletNonce {
    /// Create a new nonce for wallet authentication
    pub fn new(address: String) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let nonce = format!(
            "Sign this message to authenticate with Alou Pay:\n\nNonce: {}\nAddress: {}\nTimestamp: {}",
            Uuid::new_v4(),
            address,
            now
        );

        Self {
            nonce,
            address: address.to_lowercase(),
            created_at: now,
            expires_at: now + 300, // 5 minutes expiration
        }
    }

    /// Check if nonce is expired
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        now > self.expires_at
    }

    /// Get message to sign
    pub fn get_message(&self) -> &str {
        &self.nonce
    }
}

/// Verify Ethereum signature
/// This is a simplified version. In production, use a proper library like ethers-rs
pub fn verify_signature(
    _message: &str,
    signature: &str,
    expected_address: &str,
) -> Result<bool, String> {
    // TODO: Implement proper signature verification using ethers-rs or web3
    // For now, this is a placeholder
    
    // Basic validation
    if signature.len() < 132 {
        return Err("Invalid signature format".to_string());
    }

    if !expected_address.starts_with("0x") {
        return Err("Invalid address format".to_string());
    }

    // In production, you should:
    // 1. Hash the message with Ethereum's personal_sign prefix
    // 2. Recover the public key from the signature
    // 3. Derive the address from the public key
    // 4. Compare with expected_address

    // Temporary: Accept any valid-looking signature (REMOVE IN PRODUCTION)
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nonce_creation() {
        let address = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb".to_string();
        let nonce = WalletNonce::new(address.clone());
        
        assert_eq!(nonce.address, address.to_lowercase());
        assert!(!nonce.is_expired());
        assert!(nonce.get_message().contains("Alou Pay"));
    }

    #[test]
    fn test_nonce_expiration() {
        let address = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb".to_string();
        let mut nonce = WalletNonce::new(address);
        
        // Manually set expiration to past
        nonce.expires_at = 0;
        
        assert!(nonce.is_expired());
    }
}

