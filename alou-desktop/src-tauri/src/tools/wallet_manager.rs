use crate::tools::{ToolExecutor, ToolResult, ToolError, ToolMetadata, ExecutionContext, ToolCategory, ToolPriority, ToolStatus};
use async_trait::async_trait;
use serde_json::{json, Value};

#[derive(Clone)]
pub struct WalletManagerTool {
    metadata: ToolMetadata,
}

impl WalletManagerTool {
    pub fn new() -> Self {
        let metadata = ToolMetadata {
            id: "wallet_manager".to_string(),
            name: "wallet_manager".to_string(),
            description: "Manage wallet operations including network switching, balance checking, and wallet information retrieval.".to_string(),
            category: ToolCategory::Web3,
            priority: ToolPriority::High,
            status: ToolStatus::Active,
            version: "1.0.0".to_string(),
            author: "Alou".to_string(),
            created_at: chrono::Utc::now().timestamp(),
            updated_at: chrono::Utc::now().timestamp(),
            dependencies: vec![],
            platforms: vec!["windows".to_string(), "macos".to_string(), "linux".to_string()],
            permissions: vec![],
                tags: vec![],
            };

        Self { metadata }
    }

    fn get_supported_networks(&self) -> Value {
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

impl Default for WalletManagerTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ToolExecutor for WalletManagerTool {
    fn metadata(&self) -> &ToolMetadata {
        &self.metadata
    }

    async fn execute(
        &self,
        args: Value,
        context: &ExecutionContext,
    ) -> Result<ToolResult, ToolError> {
        let start = std::time::Instant::now();
        
        let action = args
            .get("action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArguments("Missing 'action' field".to_string()))?;

        let result = match action {
            "list_networks" => {
                let networks = self.get_supported_networks();
                Ok(json!({
                    "success": true,
                    "networks": networks,
                    "message": "Retrieved list of supported networks"
                }))
            }

            "switch_network" => {
                let chain_id = args.get("chainId").and_then(|v| v.as_str()).ok_or_else(|| {
                    ToolError::InvalidArguments("Missing 'chainId' field for switch_network action".to_string())
                })?;

                let networks = self.get_supported_networks();
                let network = networks.as_array()
                    .and_then(|arr| arr.iter().find(|n| n.get("chainId").and_then(|c| c.as_str()) == Some(chain_id)))
                    .ok_or_else(|| {
                        ToolError::InvalidArguments(format!("Unsupported chainId: {}", chain_id))
                    })?;

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
                                "chainName": network.get("name"),
                                "rpcUrls": [network.get("rpcUrl")],
                                "nativeCurrency": network.get("nativeCurrency")
                            }
                        }
                    },
                    "message": format!("Switching to {} ({})", network.get("name").and_then(|n| n.as_str()).unwrap_or("Unknown"), network.get("type").and_then(|t| t.as_str()).unwrap_or("Unknown"))
                }))
            }

            "get_current_network" => {
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
                let wallet_address = args.get("walletAddress").and_then(|v| v.as_str());

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

            _ => Err(ToolError::InvalidArguments(format!(
                "Unknown action: {}",
                action
            ))),
        };

        let execution_time_ms = start.elapsed().as_millis() as u64;

        match result {
            Ok(data) => Ok(ToolResult {
                success: true,
                data,
                error: None,
                execution_time_ms,
                output: None,
                warnings: vec![],
                context: None,
            }),
            Err(e) => Ok(ToolResult {
                success: false,
                data: json!({}),
                error: Some(e.to_string()),
                execution_time_ms,
                output: None,
                warnings: vec![],
                context: None,
            }),
        }
    }

    async fn validate_args(&self, args: &Value) -> Result<(), ToolError> {
        if !args.is_object() {
            return Err(ToolError::InvalidArguments("Args must be an object".to_string()));
        }

        if let Some(action) = args.get("action").and_then(|v| v.as_str()) {
            match action {
                "switch_network" => {
                    if !args.get("chainId").is_some() {
                        return Err(ToolError::InvalidArguments("Missing 'chainId' field".to_string()));
                    }
                }
                "check_balance" => {}
                _ => {}
            }
        } else {
            return Err(ToolError::InvalidArguments("Missing 'action' field".to_string()));
        }

        Ok(())
    }

    fn help(&self) -> String {
        r#"Wallet Manager Tool - Manage wallet operations

Actions:
- list_networks: Get all supported blockchain networks
- switch_network: Switch to a specific blockchain network
- get_current_network: Get current network information
- get_wallet_info: Get wallet address and type
- check_balance: Check wallet balance

Parameters:
- action: The action to perform (required)
- chainId: Target chain ID for switch_network (e.g., '0x1' for Ethereum Mainnet)
- walletAddress: Wallet address for balance checking (optional)

Example:
  {"action": "list_networks"}
  {"action": "switch_network", "chainId": "0x1"}"#.to_string()
    }
}
