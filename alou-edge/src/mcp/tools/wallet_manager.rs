use async_trait::async_trait;
use serde_json::{json, Value};
use crate::agent::context::AgentContext;
use crate::mcp::registry::McpTool;
use crate::utils::error::{AloudError, Result};

/// Wallet Manager MCP Tool
/// Allows the agent to manage wallet operations including network switching
#[derive(Clone)]
pub struct WalletManagerTool;

impl WalletManagerTool {
    pub fn new() -> Self {
        Self
    }
    
    /// Get supported networks configuration
    fn get_supported_networks() -> Value {
        json!([
            {
                "chainId": "0xaa36a7",
                "name": "Ethereum Sepolia",
                "type": "Testnet",
                "icon": "🔷",
                "rpcUrl": "https://sepolia.infura.io/v3/",
                "nativeCurrency": {
                    "name": "Sepolia ETH",
                    "symbol": "ETH",
                    "decimals": 18
                }
            },
            {
                "chainId": "0x14a34",
                "name": "Base Sepolia",
                "type": "Testnet",
                "icon": "🔵",
                "rpcUrl": "https://sepolia.base.org",
                "nativeCurrency": {
                    "name": "Base Sepolia ETH",
                    "symbol": "ETH",
                    "decimals": 18
                }
            },
            {
                "chainId": "0x13882",
                "name": "Polygon Amoy",
                "type": "Testnet",
                "icon": "🟣",
                "rpcUrl": "https://rpc-amoy.polygon.technology",
                "nativeCurrency": {
                    "name": "MATIC",
                    "symbol": "MATIC",
                    "decimals": 18
                }
            },
            {
                "chainId": "0x1",
                "name": "Ethereum Mainnet",
                "type": "Mainnet",
                "icon": "💎",
                "rpcUrl": "https://mainnet.infura.io/v3/",
                "nativeCurrency": {
                    "name": "Ether",
                    "symbol": "ETH",
                    "decimals": 18
                }
            },
            {
                "chainId": "0x2105",
                "name": "Base Mainnet",
                "type": "Mainnet",
                "icon": "🔷",
                "rpcUrl": "https://mainnet.base.org",
                "nativeCurrency": {
                    "name": "Ether",
                    "symbol": "ETH",
                    "decimals": 18
                }
            },
            {
                "chainId": "0x89",
                "name": "Polygon Mainnet",
                "type": "Mainnet",
                "icon": "🟣",
                "rpcUrl": "https://polygon-rpc.com",
                "nativeCurrency": {
                    "name": "MATIC",
                    "symbol": "MATIC",
                    "decimals": 18
                }
            }
        ])
    }
}

#[async_trait(?Send)]
impl McpTool for WalletManagerTool {
    fn name(&self) -> &str {
        "wallet_manager"
    }
    
    fn description(&self) -> &str {
        "Manage wallet operations including network switching, balance checking, and wallet information retrieval. The agent can use this tool to switch between different blockchain networks (Ethereum, Base, Polygon) on both mainnet and testnet."
    }
    
    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": [
                        "list_networks",
                        "switch_network",
                        "get_current_network",
                        "get_wallet_info",
                        "check_balance"
                    ],
                    "description": "Action to perform: list_networks (get all supported networks), switch_network (switch to a specific network), get_current_network (get current network info), get_wallet_info (get wallet address and type), check_balance (check wallet balance)"
                },
                "chainId": {
                    "type": "string",
                    "description": "Target chain ID for switch_network action (e.g., '0x1' for Ethereum Mainnet, '0xaa36a7' for Sepolia)"
                },
                "walletAddress": {
                    "type": "string",
                    "description": "Wallet address for balance checking (optional, uses current wallet if not provided)"
                }
            },
            "required": ["action"]
        })
    }
    
    async fn execute(&self, args: Value, context: &AgentContext) -> Result<Value> {
        let action = args.get("action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AloudError::InvalidToolArgs("Missing 'action' field".to_string()))?;
        
        match action {
            "list_networks" => {
                let networks = Self::get_supported_networks();
                Ok(json!({
                    "success": true,
                    "networks": networks,
                    "message": "Retrieved list of supported networks"
                }))
            }
            
            "switch_network" => {
                let chain_id = args.get("chainId")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| AloudError::InvalidToolArgs("Missing 'chainId' field for switch_network action".to_string()))?;
                
                let networks = Self::get_supported_networks();
                let network = networks.as_array()
                    .and_then(|arr| arr.iter().find(|n| n["chainId"].as_str() == Some(chain_id)))
                    .ok_or_else(|| AloudError::InvalidToolArgs(format!("Unsupported chainId: {}", chain_id)))?;
                
                // Return instruction for frontend to execute the network switch
                Ok(json!({
                    "success": true,
                    "action": "switch_network",
                    "network": network,
                    "instruction": {
                        "type": "wallet_operation",
                        "method": "wallet_switchEthereumChain",
                        "params": {
                            "chainId": chain_id
                        },
                        "fallback": {
                            "method": "wallet_addEthereumChain",
                            "params": {
                                "chainId": chain_id,
                                "chainName": network["name"],
                                "rpcUrls": [network["rpcUrl"]],
                                "nativeCurrency": network["nativeCurrency"]
                            }
                        }
                    },
                    "message": format!("Switching to {} ({})", network["name"], network["type"])
                }))
            }
            
            "get_current_network" => {
                // Get current network from context or session
                let session_id = &context.session_id;
                
                Ok(json!({
                    "success": true,
                    "sessionId": session_id,
                    "message": "To get current network, check wallet_chain_id in localStorage or query eth_chainId",
                    "instruction": {
                        "type": "query",
                        "method": "eth_chainId"
                    }
                }))
            }
            
            "get_wallet_info" => {
                let session_id = &context.session_id;
                
                Ok(json!({
                    "success": true,
                    "sessionId": session_id,
                    "message": "Wallet info should be retrieved from localStorage (wallet_address, wallet_type, wallet_chain_id)",
                    "instruction": {
                        "type": "query",
                        "keys": ["wallet_address", "wallet_type", "wallet_chain_id"]
                    }
                }))
            }
            
            "check_balance" => {
                let wallet_address = args.get("walletAddress")
                    .and_then(|v| v.as_str());
                
                Ok(json!({
                    "success": true,
                    "message": "To check balance, use eth_getBalance RPC method",
                    "instruction": {
                        "type": "query",
                        "method": "eth_getBalance",
                        "params": [
                            wallet_address.unwrap_or("current_wallet"),
                            "latest"
                        ]
                    }
                }))
            }
            
            _ => Err(AloudError::InvalidToolArgs(format!("Unknown action: {}", action)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_wallet_manager_tool_schema() {
        let tool = WalletManagerTool::new();
        
        assert_eq!(tool.name(), "wallet_manager");
        assert!(!tool.description().is_empty());
        
        let schema = tool.input_schema();
        assert!(schema.get("properties").is_some());
    }
    
    #[test]
    fn test_supported_networks() {
        let networks = WalletManagerTool::get_supported_networks();
        let networks_array = networks.as_array().unwrap();
        
        assert!(networks_array.len() >= 5);
        
        // Check Ethereum Mainnet
        let eth_mainnet = networks_array.iter()
            .find(|n| n["chainId"] == "0x1")
            .unwrap();
        assert_eq!(eth_mainnet["name"], "Ethereum Mainnet");
        assert_eq!(eth_mainnet["type"], "Mainnet");
    }
}
