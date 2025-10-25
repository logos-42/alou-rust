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

