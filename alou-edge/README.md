# Alou Edge - Web3 AI Agent on Cloudflare Workers

Edge-deployed AI agent with Web3 wallet authentication and MCP tool integration.

## Features

- 🚀 **Edge Deployment**: Runs on Cloudflare Workers for global low-latency
- 🔐 **Web3 Authentication**: Ethereum and Solana wallet signature verification
- 🤖 **AI Agent**: Claude-compatible API with DeepSeek backend
- 🔧 **MCP Integration**: Model Context Protocol for extensible tool system
- 💾 **Persistent Storage**: D1 database and KV storage for sessions and cache
- ⚡ **Optimized**: 1.17 MB WASM binary with aggressive optimization

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Cloudflare Worker                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │   Router     │  │ SessionMgr   │  │  AgentCore   │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │ WalletAuth   │  │  MCP Bridge  │  │ MCP Executor │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
└─────────────────────────────────────────────────────────────┘
         │                  │                  │
         ▼                  ▼                  ▼
    ┌────────┐        ┌────────┐        ┌────────┐
    │   KV   │        │   D1   │        │  MCP   │
    │ Store  │        │   DB   │        │ Server │
    └────────┘        └────────┘        └────────┘
```

## Quick Start

### Prerequisites

- Rust 1.70+ with `wasm32-unknown-unknown` target
- Node.js 18+ and npm
- Wrangler CLI: `npm install -g wrangler`
- Cloudflare account

### Installation

1. **Install Rust WASM target**:
```bash
rustup target add wasm32-unknown-unknown
```

2. **Clone and build**:
```bash
cd alou-edge
cargo build --target wasm32-unknown-unknown --lib --release
```

3. **Configure Cloudflare resources**:
```bash
# Create D1 database
wrangler d1 create alou-edge-dev

# Create KV namespaces
wrangler kv:namespace create "SESSIONS"
wrangler kv:namespace create "CACHE"
wrangler kv:namespace create "NONCES"

# Set secrets
wrangler secret put CLAUDE_API_KEY
wrangler secret put JWT_SECRET
```

4. **Update wrangler.toml** with your resource IDs

5. **Deploy**:
```bash
wrangler deploy
```

## Local Development

### Build and Test Locally

```powershell
# Build the project
.\build.ps1 -Release

# Start local dev server
.\test-local.ps1 -Build

# In another terminal, test endpoints
.\test-endpoints.ps1 -Verbose
```

### Manual Testing

```bash
# Health check
curl http://localhost:8787/api/health

# Create session
curl -X POST http://localhost:8787/api/session

# Chat with agent
curl -X POST http://localhost:8787/api/agent/chat \
  -H "Content-Type: application/json" \
  -d '{"session_id":"xxx","message":"Hello!"}'
```

## API Endpoints

### Health & Status
- `GET /api/health` - Health check
- `GET /api/status` - Service status and metrics

### Session Management
- `POST /api/session` - Create new session
- `GET /api/session/:id` - Get session history
- `DELETE /api/session/:id` - Delete session

### Wallet Authentication
- `GET /api/wallet/nonce/:address` - Get authentication nonce
- `POST /api/wallet/verify` - Verify signature and get JWT token
- `GET /api/wallet/me` - Get current user info (requires auth)

### Agent Chat
- `POST /api/agent/chat` - Send message to agent
- `POST /api/agent/stream` - Stream chat response (SSE)

## Configuration

### Environment Variables

Set via `wrangler secret put <NAME>`:

- `CLAUDE_API_KEY` or `DEEPSEEK_API_KEY` - AI API key
- `JWT_SECRET` - Secret for JWT token signing
- `ETH_RPC_URL` - Ethereum RPC endpoint (optional)
- `SOLANA_RPC_URL` - Solana RPC endpoint (optional)
- `MCP_SERVER_URL` - External MCP server URL (optional)

## Testnet 合约交互

1. 复制项目根目录的 `env.example` 为 `.env`，并填入：
   - `TESTNET_RPC_URL`：以太坊测试网 RPC（如 Sepolia）
   - `TESTNET_PRIVATE_KEY`：用于交互的测试网钱包私钥
   - `TOKEN_ADDRESS`：已部署的代币合约地址
   - 可选：`TRANSFER_RECIPIENT`、`TRANSFER_AMOUNT` 用于演示转账
2. 进入 Hardhat 项目目录：
   ```bash
   cd alou-edge/src/web3
   npm install
   ```
   （确保已安装 Hardhat 及 `@nomicfoundation/hardhat-toolbox`。）
3. 运行交互脚本：
   ```bash
   npx hardhat run scripts/interact.ts --network sepolia
   ```
   脚本会先输出当前网络与账户余额；若设置了 `TRANSFER_RECIPIENT` 与 `TRANSFER_AMOUNT`，会自动发起 ERC20 `transfer` 并等待交易确认。

> 如果只需读取余额，可忽略可选变量，脚本将以只读方式运行。

## 智能体注册（ERC-4337）

1. `.env` 中补充以下变量（可按需替换为自己的 DID、公钥、salt）：
   ```
   DIAP_AGENT_NETWORK_ADDRESS=0x9eF71FD5be68ebab2ABE20c5Fab826b14BfBc089
   DIAP_TOKEN_ADDRESS=0x2a5b6A672e9028962Ab4DaF20d256C0978604Cb3
   DIAP_ACCOUNT_FACTORY_ADDRESS=0xeaf2cb64685695497bf20f70c6F74bA86851edfD
   AGENT_DID=ipns/k51qzi5uqu5dik127hx8dsbqdosj4gehgtdahx5ynmi5wjdjqqb6uzj14o2127
   AGENT_PUBLIC_KEY=ipns/k51qzi5uqu5dik127hx8dsbqdosj4gehgtdahx5ynmi5wjdjqqb6uzj14o2127
   AGENT_AA_SALT=42
   ```
   *脚本会自动去掉前缀 `ipns/`，请确保 DID 实际格式符合 `k51...` 或 IPFS CID。*
2. 运行注册脚本：
   ```bash
   cd alou-edge/src/web3
   npx hardhat run scripts/registerAgent.ts --network sepolia
   ```
3. 输出内容包括：
   - 最小质押、注册费、授权金额
   - `approve` 与 `registerAgentWithAA` 的交易哈希与区块号
   - `getAgent` 返回的状态（`isActive`、`isAAAccount`、AA 地址等）
   - AA 钱包的 owner、DIAP 余额

注册完成后，可基于生成的 AA 钱包继续配置 Session Key、白名单或 Paymaster 交互。

ETHERSCAN_API_KEY=

# Core Contracts
DIAP_TOKEN_ADDRESS=0x2a5b6A672e9028962Ab4DaF20d256C0978604Cb3
DIAP_NETWORK_ADDRESS=0x9eF71FD5be68ebab2ABE20c5Fab826b14BfBc089
DIAP_VERIFICATION_ADDRESS=0x8F513135a6865173b6fC08e7A1138211ba174109
DIAP_PAYMENT_CORE_ADDRESS=0x498CbdD8d509058FfDe7335391B8a053Bb4Ab0e7
DIAP_PAYMENT_CHANNEL_ADDRESS=0x471cB216e5bF64d9E33b92E12d6AE3327c7a7a80
DIAP_PAYMENT_PRIVACY_ADDRESS=0x69bd0c763F86B80C043eA7CF1af58186E23E21cc
DIAP_GOVERNANCE_ADDRESS=0xFBD843F3ECDd5398639d849763088BF9Cd36f2Be
TIMELOCK_CONTROLLER_ADDRESS=0x4CFDC3D8aAabDB6E9f78a0CEe5d32Fb062eCD17A

# ERC-4337 Contracts
DIAP_ACCOUNT_FACTORY_ADDRESS=0xeaf2cb64685695497bf20f70c6F74bA86851edfD
DIAP_PAYMASTER_ADDRESS=0xA960cf9053FA76278e16f9D4BA35225f7634DC54
ENTRY_POINT_ADDRESS=0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789

### wrangler.toml

```toml
name = "alou-edge"
main = "build/worker/shim.mjs"
compatibility_date = "2024-01-01"

[[d1_databases]]
binding = "DB"
database_id = "your-database-id"

[[kv_namespaces]]
binding = "SESSIONS"
id = "your-kv-id"
```

## Project Structure

```
alou-edge/
├── abi/                  # Compiled contract ABIs (embedded via include_str!)
│   ├── diap/
│   ├── aa/
│   └── openzeppelin/
├── src/
│   ├── lib.rs              # Worker entry point
│   ├── router.rs           # API routing
│   ├── agent/              # AI agent core
│   │   ├── core.rs
│   │   ├── session.rs
│   │   └── claude_client.rs
│   ├── mcp/                # MCP client & tools
│   │   ├── client.rs
│   │   ├── executor.rs
│   │   ├── registry.rs
│   │   └── tools/
│   ├── web3/               # Web3 authentication
│   │   ├── auth.rs
│   │   └── signer.rs
│   ├── storage/            # Storage layer
│   │   ├── kv.rs
│   │   └── d1.rs
│   └── utils/              # Utilities
│       ├── error.rs
│       ├── crypto.rs
│       └── metrics.rs
├── Cargo.toml
├── wrangler.toml
├── build.ps1               # Build script
├── deploy.ps1              # Deployment script
└── test-local.ps1          # Local testing script
```

### ABI Assets

- 所有链上交互使用的 ABI 文件已迁移到 `alou-edge/abi/`，构建时通过 `include_str!` 打包到 WASM。
- `Smart ETH` 目录仅作为原始 Hardhat 产物备份，可在确认无其他依赖后删除或裁剪，只需保留必要的 ABI 文件即可。

## Performance

- **WASM Size**: 1.17 MB (optimized)
- **Cold Start**: < 50ms
- **Response Time**: < 100ms (excluding AI API)
- **Optimization Level**: `z` (size-optimized)
- **LTO**: Enabled
- **Codegen Units**: 1

## Development

### Adding New MCP Tools

1. Create tool in `src/mcp/tools/`:
```rust
pub struct MyTool;

#[async_trait(?Send)]
impl McpTool for MyTool {
    fn name(&self) -> &str { "my_tool" }
    fn description(&self) -> &str { "..." }
    fn input_schema(&self) -> Value { ... }
    async fn execute(&self, args: Value, ctx: &AgentContext) -> Result<Value> { ... }
}
```

2. Register in `src/lib.rs`:
```rust
registry.register(Arc::new(MyTool));
```

### Running Tests

```bash
cargo test --lib
cargo test --target wasm32-unknown-unknown --lib
```

## Troubleshooting

### Build Issues

- **Missing WASM target**: `rustup target add wasm32-unknown-unknown`
- **Compilation errors**: Check Rust version (1.70+)
- **Large binary**: Ensure release mode with optimizations

### Deployment Issues

- **Resource not found**: Update wrangler.toml with correct IDs
- **Secret errors**: Set all required secrets with `wrangler secret put`
- **KV/D1 errors**: Verify bindings in wrangler.toml

### Runtime Issues

- **500 errors**: Check worker logs with `wrangler tail`
- **Timeout**: Increase timeout in wrangler.toml
- **Memory**: Monitor usage in Cloudflare dashboard

## License

MIT OR Apache-2.0

## Contributing

Contributions welcome! Please open an issue or PR.

