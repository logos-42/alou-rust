use crate::utils::crypto::{ChainType, generate_nonce};
use crate::utils::error::{AloudError, Result};
use crate::storage::kv::KvStore;
use crate::web3::signer::verify_signature;
use jwt_simple::prelude::*;
use serde::{Deserialize, Serialize};

/// JWT Claims for wallet authentication
#[derive(Debug, Serialize, Deserialize)]
pub struct WalletClaims {
    pub wallet_address: String,
    pub chain: String,
    pub exp: i64,  // Expiration time (Unix timestamp)
    pub iat: i64,  // Issued at (Unix timestamp)
}

/// Authentication nonce stored in KV
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthNonce {
    pub address: String,
    pub nonce: String,
    pub created_at: i64,
    pub expires_at: i64,
}

/// Wallet authentication manager
pub struct WalletAuth {
    kv: KvStore,
    jwt_key: HS256Key,
}

impl WalletAuth {
    pub fn new(kv: KvStore, jwt_secret: String) -> Self {
        let jwt_key = HS256Key::from_bytes(jwt_secret.as_bytes());
        Self { kv, jwt_key }
    }
    
    /// Generate a nonce for wallet authentication
    /// Nonce is stored in KV with 5 minute TTL
    pub async fn generate_nonce_for_address(&self, address: &str) -> Result<String> {
        let nonce = generate_nonce();
        let now = crate::utils::time::now_timestamp();
        let expires_at = now + 300; // 5 minutes
        
        let auth_nonce = AuthNonce {
            address: address.to_string(),
            nonce: nonce.clone(),
            created_at: now,
            expires_at,
        };
        
        // Store nonce in KV with 5 minute TTL
        let key = format!("nonce:{}", address);
        self.kv.put(&key, &auth_nonce, Some(300)).await?;
        
        Ok(nonce)
    }
    
    /// Verify wallet signature and create JWT token
    pub async fn verify_and_create_token(
        &self,
        address: &str,
        signature: &str,
        message: &str,
        chain: ChainType,
    ) -> Result<String> {
        // Get nonce from KV
        let key = format!("nonce:{}", address);
        let auth_nonce: Option<AuthNonce> = self.kv.get(&key).await?;
        
        let auth_nonce = auth_nonce
            .ok_or_else(|| AloudError::AuthError("Nonce not found".to_string()))?;
        
        // Check if nonce is expired
        let now = crate::utils::time::now_timestamp();
        if now > auth_nonce.expires_at {
            return Err(AloudError::NonceExpired);
        }
        
        // Verify that the message contains the nonce
        if !message.contains(&auth_nonce.nonce) {
            return Err(AloudError::AuthError("Message does not contain nonce".to_string()));
        }
        
        // Verify signature
        let is_valid = verify_signature(address, message, signature, chain)?;
        
        if !is_valid {
            return Err(AloudError::InvalidSignature);
        }
        
        // Delete used nonce
        self.kv.delete(&key).await?;
        
        // Create JWT token
        let token = self.create_token(address, chain)?;
        
        Ok(token)
    }
    
    /// Create a JWT token for authenticated wallet
    /// Token is valid for 24 hours
    pub fn create_token(&self, wallet_address: &str, chain: ChainType) -> Result<String> {
        let now_ts = crate::utils::time::now_timestamp();
        
        let claims = WalletClaims {
            wallet_address: wallet_address.to_string(),
            chain: chain.as_str().to_string(),
            exp: now_ts + 86400, // 24 hours
            iat: now_ts,
        };
        
        // Manually create JWT to avoid jwt_simple's time functions
        use serde_json::json;
        use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
        
        let header = json!({
            "alg": "HS256",
            "typ": "JWT"
        });
        
        let header_b64 = URL_SAFE_NO_PAD.encode(serde_json::to_string(&header).unwrap());
        let payload_b64 = URL_SAFE_NO_PAD.encode(serde_json::to_string(&claims).unwrap());
        
        let message = format!("{}.{}", header_b64, payload_b64);
        
        // Sign with HMAC-SHA256
        use hmac::{Hmac, Mac};
        use sha2::Sha256;
        type HmacSha256 = Hmac<Sha256>;
        
        let key_bytes = self.jwt_key.to_bytes();
        let mut mac = HmacSha256::new_from_slice(&key_bytes)
            .map_err(|e| AloudError::AuthError(format!("HMAC error: {}", e)))?;
        mac.update(message.as_bytes());
        let signature = mac.finalize().into_bytes();
        let signature_b64 = URL_SAFE_NO_PAD.encode(&signature[..]);
        
        let token = format!("{}.{}", message, signature_b64);
        Ok(token)
    }
    
    /// Verify and decode a JWT token
    pub fn verify_token(&self, token: &str) -> Result<WalletClaims> {
        let claims = self.jwt_key.verify_token::<WalletClaims>(token, None)
            .map_err(|e| AloudError::AuthError(format!("Invalid token: {}", e)))?;
        
        Ok(claims.custom)
    }
    
    /// Extract wallet address and chain from token
    pub fn parse_token(&self, token: &str) -> Result<(String, ChainType)> {
        let claims = self.verify_token(token)?;
        let chain = ChainType::from_str(&claims.chain)?;
        
        Ok((claims.wallet_address, chain))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Note: Tests are commented out because they require a real KV store
    // which is not available in unit tests. These should be integration tests.
    
    // #[test]
    // fn test_create_and_verify_token() {
    //     let jwt_secret = "test_secret_key_for_testing_only".to_string();
    //     let kv = KvStore::new(worker::kv::KvStore::default());
    //     let auth = WalletAuth::new(kv, jwt_secret);
    //     
    //     let token = auth.create_token("0x1234567890abcdef", ChainType::Ethereum).unwrap();
    //     let claims = auth.verify_token(&token).unwrap();
    //     
    //     assert_eq!(claims.wallet_address, "0x1234567890abcdef");
    //     assert_eq!(claims.chain, "ethereum");
    // }
}
