# Alou 边缘部署 - 设计文档

## Overview

基于 Claude Agent SDK 在 Cloudflare 边缘网络构建 Web3 AI Agent，通过 MCP 协议实现钱包认证和链上操作。Agent 运行在全球 300+ 边缘节点，提供低延迟的智能交互体验。

## Architecture

### 高层架构

```
┌─────────────────────────────────────────────────────────────┐
│                    用户层（全球）                              │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐                   │
│  │ 浏览器    │  │ MetaMask │  │ 移动端    │                   │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘                   │
└───────┼─────────────┼─────────────┼─────────────────────────┘
        │             │             │
        └─────────────┴─────────────┘
                      │
        ┌─────────────▼──────────────┐
        │  Cloudflare Edge (最近节点) │
        └─────────────┬──────────────┘
                      │
┌─────────────────────▼─────────────────────────────────────┐
│              Rust WASM Worker 层                           │
│  ┌──────────────────────────────────────────────────────┐ │
│  │           Claude Agent SDK Core                      │ │
│  │  - 对话管理                                           │ │
│  │  - 工具调用                                           │ │
│  │  - 上下文维护                                         │ │
│  └────────────────┬─────────────────────────────────────┘ │
│                   │                                        │
│  ┌────────────────▼─────────────────────────────────────┐ │
│  │              MCP 工具层                               │ │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐           │ │
│  │  │ 钱包认证  │  │ 链上查询  │  │ 交易构造  │           │ │
│  │  └──────────┘  └──────────┘  └──────────┘           │ │
│  │  ┌──────────┐  ┌──────────┐                         │ │
│  │  │ 交易广播  │  │ 合约交互  │                         │ │
│  │  └──────────┘  └──────────┘                         │ │
│  └──────────────────────────────────────────────────────┘ │
└────────────────────────────────────────────────────────────┘
                      │
┌───────────────────────▼────────────────────────────────────┐
│                   数据和存储层                              │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐                 │
│  │ D1 数据库 │  │ KV 存储   │  │ 区块链    │                 │
│  │ (用户)    │  │ (会话)    │  │ RPC       │                 │
│  └──────────┘  └──────────┘  └──────────┘                 │
└────────────────────────────────────────────────────────────┘
                      │
        ┌─────────────┴─────────────┐
        │                           │
┌───────▼────────┐         ┌────────▼────────┐
│  Claude API    │         │  区块链网络      │
│  (AI 推理)     │         │  (链上数据)      │
└────────────────┘         └─────────────────┘
```

### 模块架构

```rust
alou-edge/
├── src/
│   ├── lib.rs              // 主入口，Worker 处理
│   ├── router.rs           // API 路由
│   ├── agent/
│   │   ├── mod.rs
│   │   ├── core.rs         // Claude Agent SDK 集成
│   │   ├── session.rs      // 会话管理
│   │   └── context.rs      // 上下文维护
│   ├── mcp/
│   │   ├── mod.rs
│   │   ├── registry.rs     // 工具注册
│   │   ├── executor.rs     // 工具执行
│   │   └── tools/
│   │       ├── mod.rs
│   │       ├── wallet_auth.rs    // 钱包认证工具
│   │       ├── query.rs          // 链上查询工具
│   │       ├── transaction.rs    // 交易构造工具
│   │       ├── broadcast.rs      // 交易广播工具
│   │       └── contract.rs       // 合约交互工具
│   ├── web3/
│   │   ├── mod.rs
│   │   ├── auth.rs         // 钱包认证逻辑
│   │   ├── rpc.rs          // RPC 客户端
│   │   ├── signer.rs       // 签名验证
│   │   └── chains/
│   │       ├── ethereum.rs
│   │       └── solana.rs
│   ├── storage/
│   │   ├── mod.rs
│   │   ├── d1.rs           // D1 数据库
│   │   └── kv.rs           // KV 存储
│   └── utils/
│       ├── mod.rs
│       ├── crypto.rs       // 加密工具
│       └── error.rs        // 错误处理
└── Cargo.toml
```

## Components and Interfaces

### 1. Claude Agent 核心

**职责**: 集成 Claude Agent SDK，处理对话和工具调用

**设计决策**:
- 使用 Claude Agent SDK 的 Rust 绑定（如果可用）或通过 HTTP API 调用
- 会话上下文存储在 KV，TTL 24 小时
- 工具调用通过 MCP 协议路由到对应的工具实现
- 支持流式响应（SSE）提升用户体验

**接口**:
```rust
pub struct AgentCore {
    claude_api_key: String,
    mcp_registry: McpRegistry,
    session_manager: SessionManager,
}

impl AgentCore {
    // 处理用户消息
    pub async fn handle_message(
        &self,
        session_id: &str,
        message: &str,
        wallet_address: Option<String>,
    ) -> Result<AgentResponse>;
    
    // 执行工具调用
    async fn execute_tool(
        &self,
        tool_name: &str,
        args: Value,
        context: &AgentContext,
    ) -> Result<Value>;
    
    // 流式响应
    pub async fn stream_response(
        &self,
        session_id: &str,
        message: &str,
    ) -> Result<impl Stream<Item = String>>;
}

pub struct AgentResponse {
    pub content: String,
    pub tool_calls: Vec<ToolCall>,
    pub session_id: String,
}

pub struct AgentContext {
    pub session_id: String,
    pub wallet_address: Option<String>,
    pub chain: Option<ChainType>,
}
```

### 2. 会话管理

**职责**: 管理用户会话和对话历史

**设计决策**:
- 会话 ID 使用 UUID v4
- 对话历史存储在 KV，key 格式：`session:{session_id}`
- 每个会话最多保留 50 条消息
- 会话过期时间 24 小时，自动清理

**接口**:
```rust
pub struct SessionManager {
    kv: KvStore,
}

impl SessionManager {
    // 创建新会话
    pub async fn create_session(&self, wallet_address: Option<String>) -> Result<String>;
    
    // 获取会话
    pub async fn get_session(&self, session_id: &str) -> Result<Session>;
    
    // 添加消息
    pub async fn add_message(
        &self,
        session_id: &str,
        role: &str,
        content: &str,
    ) -> Result<()>;
    
    // 获取对话历史
    pub async fn get_history(&self, session_id: &str) -> Result<Vec<Message>>;
    
    // 清除会话
    pub async fn clear_session(&self, session_id: &str) -> Result<()>;
}

pub struct Session {
    pub session_id: String,
    pub wallet_address: Option<String>,
    pub messages: Vec<Message>,
    pub created_at: i64,
    pub updated_at: i64,
}

pub struct Message {
    pub role: String,      // user / assistant / tool
    pub content: String,
    pub timestamp: i64,
    pub tool_call_id: Option<String>,
}
```

### 3. MCP 工具注册和执行

**职责**: 注册 MCP 工具，处理工具调用

**设计决策**:
- 所有工具实现 `McpTool` trait
- 工具在启动时注册到 `McpRegistry`
- 工具调用通过 `McpExecutor` 执行
- 支持异步工具执行

**接口**:
```rust
pub struct McpRegistry {
    tools: HashMap<String, Box<dyn McpTool>>,
}

impl McpRegistry {
    // 注册工具
    pub fn register(&mut self, tool: Box<dyn McpTool>);
    
    // 获取工具列表
    pub fn list_tools(&self) -> Vec<ToolInfo>;
    
    // 获取工具
    pub fn get_tool(&self, name: &str) -> Option<&dyn McpTool>;
}

pub struct McpExecutor {
    registry: McpRegistry,
}

impl McpExecutor {
    // 执行工具
    pub async fn execute(
        &self,
        tool_name: &str,
        args: Value,
        context: &AgentContext,
    ) -> Result<Value>;
}

// MCP 工具 trait
pub trait McpTool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn input_schema(&self) -> Value;
    async fn execute(&self, args: Value, context: &AgentContext) -> Result<Value>;
}

pub struct ToolInfo {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}
```

### 4. MCP 工具 - 钱包认证

**职责**: 处理钱包签名验证和会话创建

**接口**:
```rust
pub struct WalletAuthTool {
    kv: KvStore,
    d1: D1Database,
}

impl McpTool for WalletAuthTool {
    fn name(&self) -> &str { "wallet_auth" }
    
    fn description(&self) -> &str {
        "验证钱包签名并创建认证会话"
    }
    
    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "address": { "type": "string" },
                "signature": { "type": "string" },
                "message": { "type": "string" },
                "chain": { "type": "string", "enum": ["ethereum", "solana"] }
            },
            "required": ["address", "signature", "message", "chain"]
        })
    }
    
    async fn execute(&self, args: Value, context: &AgentContext) -> Result<Value>;
}

// 辅助方法
impl WalletAuthTool {
    // 生成 nonce
    pub async fn generate_nonce(&self, address: &str) -> Result<String>;
    
    // 验证签名
    async fn verify_signature(
        &self,
        address: &str,
        signature: &str,
        message: &str,
        chain: ChainType,
    ) -> Result<bool>;
    
    // 创建会话
    async fn create_session(&self, address: &str, chain: ChainType) -> Result<String>;
}
```

### 5. MCP 工具 - 链上查询

**职责**: 查询余额、NFT、交易历史

**接口**:
```rust
pub struct QueryTool {
    rpc_clients: HashMap<ChainType, RpcClient>,
    kv_cache: KvStore,
}

impl McpTool for QueryTool {
    fn name(&self) -> &str { "query_chain" }
    
    fn description(&self) -> &str {
        "查询链上数据：余额、NFT、交易历史"
    }
    
    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "query_type": { 
                    "type": "string", 
                    "enum": ["balance", "nfts", "transactions"] 
                },
                "address": { "type": "string" },
                "chain": { "type": "string" }
            },
            "required": ["query_type", "address", "chain"]
        })
    }
    
    async fn execute(&self, args: Value, context: &AgentContext) -> Result<Value>;
}

// 辅助方法
impl QueryTool {
    async fn query_balance(&self, address: &str, chain: ChainType) -> Result<Vec<TokenBalance>>;
    async fn query_nfts(&self, address: &str, chain: ChainType) -> Result<Vec<NftInfo>>;
    async fn query_transactions(&self, address: &str, chain: ChainType) -> Result<Vec<Transaction>>;
}
```

### 6. MCP 工具 - 交易构造

**职责**: 构造未签名的交易

**接口**:
```rust
pub struct TransactionTool {
    rpc_clients: HashMap<ChainType, RpcClient>,
}

impl McpTool for TransactionTool {
    fn name(&self) -> &str { "build_transaction" }
    
    fn description(&self) -> &str {
        "构造区块链交易（转账、合约调用等）"
    }
    
    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "tx_type": { 
                    "type": "string", 
                    "enum": ["transfer", "contract_call"] 
                },
                "from": { "type": "string" },
                "to": { "type": "string" },
                "value": { "type": "string" },
                "data": { "type": "string" },
                "chain": { "type": "string" }
            },
            "required": ["tx_type", "from", "to", "chain"]
        })
    }
    
    async fn execute(&self, args: Value, context: &AgentContext) -> Result<Value>;
}

// 辅助方法
impl TransactionTool {
    async fn build_transfer(&self, from: &str, to: &str, value: &str, chain: ChainType) -> Result<UnsignedTransaction>;
    async fn build_contract_call(&self, from: &str, to: &str, data: &str, chain: ChainType) -> Result<UnsignedTransaction>;
    async fn estimate_gas(&self, tx: &UnsignedTransaction, chain: ChainType) -> Result<GasEstimate>;
}
```

### 7. MCP 工具 - 交易广播

**职责**: 广播已签名的交易

**接口**:
```rust
pub struct BroadcastTool {
    rpc_clients: HashMap<ChainType, RpcClient>,
    d1: D1Database,
}

impl McpTool for BroadcastTool {
    fn name(&self) -> &str { "broadcast_transaction" }
    
    fn description(&self) -> &str {
        "广播已签名的交易到区块链网络"
    }
    
    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "signed_tx": { "type": "string" },
                "chain": { "type": "string" }
            },
            "required": ["signed_tx", "chain"]
        })
    }
    
    async fn execute(&self, args: Value, context: &AgentContext) -> Result<Value>;
}

// 辅助方法
impl BroadcastTool {
    async fn broadcast(&self, signed_tx: &str, chain: ChainType) -> Result<String>;
    async fn verify_signature(&self, signed_tx: &str, chain: ChainType) -> Result<bool>;
    async fn save_transaction(&self, tx_hash: &str, user_id: &str, chain: ChainType) -> Result<()>;
}
```

### 8. MCP 工具 - 合约交互

**职责**: 解析 ABI，编码合约调用

**接口**:
```rust
pub struct ContractTool {
    rpc_clients: HashMap<ChainType, RpcClient>,
    kv_cache: KvStore,
}

impl McpTool for ContractTool {
    fn name(&self) -> &str { "interact_contract" }
    
    fn description(&self) -> &str {
        "与智能合约交互：读取状态、编码调用"
    }
    
    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "action": { 
                    "type": "string", 
                    "enum": ["read", "encode_call", "simulate"] 
                },
                "contract": { "type": "string" },
                "function": { "type": "string" },
                "args": { "type": "array" },
                "chain": { "type": "string" }
            },
            "required": ["action", "contract", "function", "chain"]
        })
    }
    
    async fn execute(&self, args: Value, context: &AgentContext) -> Result<Value>;
}

// 辅助方法
impl ContractTool {
    async fn get_abi(&self, contract: &str, chain: ChainType) -> Result<ContractAbi>;
    async fn read_contract(&self, contract: &str, function: &str, args: Vec<Value>, chain: ChainType) -> Result<Value>;
    async fn encode_call(&self, abi: &ContractAbi, function: &str, args: Vec<Value>) -> Result<Vec<u8>>;
    async fn simulate_call(&self, contract: &str, data: Vec<u8>, from: &str, chain: ChainType) -> Result<SimulationResult>;
}
```

## Data Models

### 用户和会话

```rust
// 用户表 (D1)
pub struct User {
    pub id: String,              // UUID
    pub wallet_address: String,  // 钱包地址
    pub chain: String,           // 链类型
    pub created_at: i64,
    pub last_login: i64,
}

// 会话 (KV)
pub struct Session {
    pub session_id: String,
    pub wallet_address: Option<String>,
    pub messages: Vec<Message>,
    pub created_at: i64,
    pub updated_at: i64,
}

// Nonce (KV)
pub struct AuthNonce {
    pub address: String,
    pub nonce: String,
    pub created_at: i64,
    pub expires_at: i64,
}
```

### Web3 数据

```rust
pub struct TokenBalance {
    pub token_address: String,
    pub symbol: String,
    pub balance: String,
    pub decimals: u8,
}

pub struct NftInfo {
    pub contract_address: String,
    pub token_id: String,
    pub name: String,
    pub image_url: String,
}

pub struct Transaction {
    pub hash: String,
    pub from: String,
    pub to: String,
    pub value: String,
    pub timestamp: i64,
}

pub struct UnsignedTransaction {
    pub from: String,
    pub to: String,
    pub value: String,
    pub data: Vec<u8>,
    pub gas_limit: u64,
    pub gas_price: String,
    pub nonce: u64,
    pub chain_id: u64,
}
```

## Error Handling

```rust
#[derive(Debug)]
pub enum AloudError {
    // Agent 错误
    AgentError(String),
    ClaudeApiError(String),
    
    // MCP 错误
    ToolNotFound(String),
    ToolExecutionError(String),
    InvalidToolArgs(String),
    
    // 认证错误
    AuthError(String),
    InvalidSignature,
    NonceExpired,
    
    // Web3 错误
    RpcError(String),
    TransactionFailed(String),
    InsufficientBalance,
    
    // 存储错误
    DatabaseError(String),
    CacheError(String),
    
    // 其他
    InvalidInput(String),
    InternalError(String),
}

impl From<AloudError> for Response {
    fn from(err: AloudError) -> Response {
        let (status, message) = match err {
            AloudError::AuthError(_) => (401, "Authentication failed"),
            AloudError::ToolNotFound(_) => (404, "Tool not found"),
            AloudError::InvalidSignature => (401, "Invalid signature"),
            _ => (500, "Internal server error"),
        };
        
        Response::error(message, status).unwrap()
    }
}
```

## Testing Strategy

### 单元测试

```rust
#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_wallet_auth_tool() {
        let tool = WalletAuthTool::new(kv, d1);
        let args = json!({
            "address": "0x...",
            "signature": "0x...",
            "message": "Sign this message",
            "chain": "ethereum"
        });
        
        let result = tool.execute(args, &context).await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_query_tool() {
        let tool = QueryTool::new(rpc_clients, kv);
        let args = json!({
            "query_type": "balance",
            "address": "0x...",
            "chain": "ethereum"
        });
        
        let result = tool.execute(args, &context).await;
        assert!(result.is_ok());
    }
}
```

### 集成测试

```rust
#[tokio::test]
async fn test_agent_workflow() {
    // 1. 创建会话
    let session_id = agent.create_session(None).await.unwrap();
    
    // 2. 发送消息
    let response = agent.handle_message(
        &session_id,
        "查询我的 ETH 余额",
        Some("0x...".to_string()),
    ).await.unwrap();
    
    // 3. 验证工具调用
    assert!(response.tool_calls.len() > 0);
    assert_eq!(response.tool_calls[0].name, "query_chain");
}
```

## Deployment

### 环境配置

```toml
# wrangler.toml
name = "alou-edge"
main = "build/worker/shim.mjs"
compatibility_date = "2024-01-01"

[vars]
ENVIRONMENT = "production"

[[d1_databases]]
binding = "DB"
database_name = "alou-production"
database_id = "xxx"

[[kv_namespaces]]
binding = "SESSIONS"
id = "xxx"

[[kv_namespaces]]
binding = "CACHE"
id = "xxx"

# Secrets
# CLAUDE_API_KEY
# JWT_SECRET
# ETH_RPC_URL
# SOLANA_RPC_URL
```

### 部署流程

```bash
# 1. 构建 WASM
cargo build --target wasm32-unknown-unknown --release

# 2. 运行 worker-build
worker-build --release

# 3. 部署
wrangler deploy
```

---

**设计版本**: v2.0 (MVP)
**最后更新**: 2025-10-23
**状态**: 待审查
