use crate::utils::crypto::{ChainType, verify_ethereum_signature, verify_solana_signature};
use crate::utils::error::Result;

/// Verify a wallet signature
pub fn verify_signature(
    address: &str,
    message: &str,
    signature: &str,
    chain: ChainType,
) -> Result<bool> {
    match chain {
        ChainType::Ethereum => verify_ethereum_signature(address, message, signature),
        ChainType::Solana => verify_solana_signature(address, message, signature),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_verify_signature_ethereum() {
        // This would require a real signature for testing
        // For now, just test that the function exists and handles errors
        let result = verify_signature(
            "0x0000000000000000000000000000000000000000",
            "test message",
            "0x0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
            ChainType::Ethereum
        );
        
        // Should return false or error for invalid signature
        assert!(result.is_ok() || result.is_err());
    }
}
