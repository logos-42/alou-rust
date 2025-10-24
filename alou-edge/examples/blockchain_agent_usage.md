# Blockchain Agent 使用示例

## 概述

Blockchain Agent 集成了多模型 AI 和链上操作功能，支持：

- **多模型支持**：DeepSeek、Qwen（通义千问）、OpenAI、Claude
- **链上查询**：ETH/SOL 余额、ERC20/SPL Token 余额、交易历史
- **交易构建**：构建转账交易、估算 Gas、生成待签名数据
- **交易广播**：广播已签名交易、查询交易状态

## 配置

### 1. 环境变量配置（wrangler.toml）

```toml
[vars]
ENVIRONMENT = "development"
AI_PROVIDER = "deepseek"  # 或 "qwen", "openai", "claude"
AI_MODEL = "deepseek-chat"  # 或 "qwen-max", "gpt-4", "claude-3-5-sonnet-20241022"
```

### 2. 密钥配置

```bash
# 设置 AI API Key
wrangler secret put AI_API_KEY

# 设置 RPC 端点
wrangler secret put ETH_RPC_URL
wrangler secret put SOL_RPC_URL

# 设置 JWT Secret
wrangler secret put JWT_SECRET
```

## 代码示例

### 基础使用

```rust
use alou_edge::agent::BlockchainAgent;

// 创建 Blockchain Agent
let agent = BlockchainAgent::new(
    "deepseek",  // 或 "qwen", "openai", "claude"
    env.secret("AI_API_KEY")?.to_string(),
    Some("deepseek-chat".to_string()),
    env.secret("ETH_RPC_URL")?.to_string(),
    env.secret("SOL_RPC_URL")?.to_string(),
)?;

// 处理用户消息
let response = agent.process_message(
    "查询地址 0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb 的 ETH 余额"
).await?;

console_log!("Response: {}", response);
```

### 查询余额

```rust
use alou_edge::agent::tools::QueryTool;

let query_tool = QueryTool::new(
    eth_rpc_url,
    sol_rpc_url,
);

// 查询 ETH 余额
let eth_balance = query_tool.get_eth_balance(
    "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb"
).await?;
console_log!("ETH Balance: {}", eth_balance);

// 查询 SOL 余额
let sol_balance = query_tool.get_sol_balance(
    "DYw8jCTfwHNRJhhmFcbXvVDTqWMEVFBX6ZKUmG5CNSKK"
).await?;
console_log!("SOL Balance: {}", sol_balance);

// 查询 ERC20 Token 余额
let token_balance = query_tool.get_erc20_balance(
    "0xdAC17F958D2ee523a2206206994597C13D831ec7", // USDT
    "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb"
).await?;
console_log!("Token Balance: {}", token_balance);
```

### 构建交易

```rust
use alou_edge::agent::tools::TransactionTool;

let tx_tool = TransactionTool::new(
    eth_rpc_url,
    sol_rpc_url,
);

// 构建 ETH 转账交易
let tx_data = tx_tool.build_eth_transaction(
    "0xSenderAddress",
    "0xRecipientAddress",
    0.1  // 0.1 ETH
).await?;

console_log!("Transaction data: {:?}", tx_data);
// 输出包含：from, to, value, gas, gas_price, nonce

// 构建 ERC20 转账交易
let erc20_tx = tx_tool.build_erc20_transaction(
    "0xSenderAddress",
    "0xTokenContractAddress",
    "0xRecipientAddress",
    1000000  // Token amount (考虑 decimals)
).await?;

// 估算 Gas
let estimated_gas = tx_tool.estimate_gas(&tx_data).await?;
console_log!("Estimated gas: {}", estimated_gas);
```

### 广播交易

```rust
use alou_edge::agent::tools::BroadcastTool;

let broadcast_tool = BroadcastTool::new(
    eth_rpc_url,
    sol_rpc_url,
);

// 广播已签名的 ETH 交易
let tx_hash = broadcast_tool.broadcast_eth_transaction(
    "0xSignedTransactionData"
).await?;
console_log!("Transaction hash: {}", tx_hash);

// 查询交易状态
let receipt = broadcast_tool.get_eth_transaction_receipt(&tx_hash).await?;
console_log!("Transaction status: {}", receipt.status);

// 检查交易是否确认
let confirmed = broadcast_tool.is_transaction_confirmed(
    &tx_hash,
    "eth"
).await?;
console_log!("Confirmed: {}", confirmed);
```

## AI 模型切换

### DeepSeek（推荐用于中国）

```rust
let agent = BlockchainAgent::new(
    "deepseek",
    api_key,
    Some("deepseek-chat".to_string()),
    eth_rpc_url,
    sol_rpc_url,
)?;
```

### Qwen（通义千问）

```rust
let agent = BlockchainAgent::new(
    "qwen",
    api_key,
    Some("qwen-max".to_string()),  // 或 "qwen-plus", "qwen-turbo"
    eth_rpc_url,
    sol_rpc_url,
)?;
```

### OpenAI

```rust
let agent = BlockchainAgent::new(
    "openai",
    api_key,
    Some("gpt-4".to_string()),  // 或 "gpt-3.5-turbo"
    eth_rpc_url,
    sol_rpc_url,
)?;
```

### Claude

```rust
let agent = BlockchainAgent::new(
    "claude",
    api_key,
    Some("claude-3-5-sonnet-20241022".to_string()),
    eth_rpc_url,
    sol_rpc_url,
)?;
```

## RPC 端点推荐

### Ethereum

- **Infura**: `https://mainnet.infura.io/v3/YOUR_PROJECT_ID`
- **Alchemy**: `https://eth-mainnet.g.alchemy.com/v2/YOUR_API_KEY`
- **QuickNode**: `https://YOUR_ENDPOINT.quiknode.pro/YOUR_API_KEY/`
- **公共节点**: `https://eth.llamarpc.com` (不推荐生产环境)

### Solana

- **QuickNode**: `https://YOUR_ENDPOINT.solana-mainnet.quiknode.pro/YOUR_API_KEY/`
- **Alchemy**: `https://solana-mainnet.g.alchemy.com/v2/YOUR_API_KEY`
- **公共节点**: `https://api.mainnet-beta.solana.com` (有速率限制)

## 对话示例

用户可以用自然语言与 Agent 交互：

```
用户: "查询我的 ETH 余额，地址是 0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb"
Agent: [调用 get_eth_balance 工具]
      "您的 ETH 余额是 1.234567 ETH"

用户: "帮我构建一笔转账交易，从 0xAAA 转 0.1 ETH 到 0xBBB"
Agent: [调用 build_eth_transaction 工具]
      "交易已构建，详情如下：
       From: 0xAAA
       To: 0xBBB
       Value: 0.1 ETH
       Gas: 21000
       Gas Price: 20 Gwei
       Nonce: 5"

用户: "查询交易 0x123abc 的状态"
Agent: [调用 get_transaction_status 工具]
      "交易已确认，状态：成功"
```

## 注意事项

1. **安全性**：
   - 永远不要在代码中硬编码私钥
   - 使用 Cloudflare Workers Secrets 存储敏感信息
   - 交易签名应该在客户端完成

2. **Gas 估算**：
   - 实际 Gas 消耗可能高于估算值
   - 建议添加 10-20% 的 Gas 缓冲

3. **RPC 限制**：
   - 公共 RPC 节点有速率限制
   - 生产环境建议使用付费 RPC 服务

4. **错误处理**：
   - 所有工具调用都返回 `Result<T>`
   - 需要适当处理网络错误和 RPC 错误

5. **模型选择**：
   - DeepSeek 和 Qwen 在中国访问更快
   - OpenAI 和 Claude 功能更强但可能需要代理
