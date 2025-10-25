# Alou Pay - Web3 AI Agent 边缘计算平台

基于 Cloudflare Workers 的 Web3 AI 智能体平台，支持区块链钱包认证和 MCP 工具集成。

## 🚀 核心特性

- **边缘计算**: 基于 Cloudflare Workers 的全球分布式部署
- **Web3 集成**: 支持以太坊和 Solana 钱包认证
- **AI 智能体**: 集成 DeepSeek API 的智能对话系统
- **MCP 工具**: 完整的 Model Context Protocol 工具生态
- **WASM 优化**: Rust 编译的 1.17MB 优化二进制文件

## 🏗️ 项目架构

```
aloupay/
├── alou-edge/          # Rust WASM 边缘计算核心
├── frontend/           # Vue.js 前端界面
├── alou-edge-ts/       # TypeScript 版本（可选）
└── migrations/         # 数据库迁移文件
```

## 🛠️ 快速开始

### 1. 环境要求

- Rust 1.70+
- Node.js 18+
- Cloudflare 账户
- DeepSeek API 密钥

### 2. 安装部署

```bash
# 克隆项目
git clone https://github.com/logos-42/alou-rust.git
cd alou-rust

# 构建 WASM 模块
cd alou-edge
cargo build --release --target wasm32-unknown-unknown

# 部署到 Cloudflare Workers
wrangler deploy

# 启动前端开发服务器
cd ../frontend
npm install
npm run dev
```

### 3. 配置环境变量

```bash
# DeepSeek API
export DEEPSEEK_API_KEY=your_api_key

# Cloudflare Workers
export CLOUDFLARE_API_TOKEN=your_token
export CLOUDFLARE_ACCOUNT_ID=your_account_id
```

## 🎯 主要功能

### AI 智能体
- 多轮对话上下文管理
- 流式响应支持
- 会话持久化存储

### Web3 钱包认证
- MetaMask / Phantom 连接
- 签名验证和身份认证
- 多链支持（ETH/SOL）

### MCP 工具集成
- 文件系统操作
- 区块链查询
- 交易广播
- 智能合约交互

## 🔧 配置说明

### MCP 配置 (mcp.json)
```json
{
  "mcpServers": {
    "filesystem": {
      "command": "npx",
      "args": ["@modelcontextprotocol/server-filesystem", "/Users"],
      "timeout": 30000
    }
  }
}
```

### Cloudflare Workers 配置
```toml
# wrangler.toml
name = "alou-edge"
main = "src/lib.rs"
compatibility_date = "2024-01-01"

[[d1_databases]]
binding = "DB"
database_name = "alou-edge-dev"
```

## 🚀 部署指南

### 开发环境
```bash
# 本地开发
wrangler dev

# 前端开发
cd frontend && npm run dev
```

### 生产环境
```bash
# 部署 Workers
wrangler deploy --env production

# 构建前端
cd frontend && npm run build
```

## 📊 性能指标

- **启动时间**: < 50ms
- **响应延迟**: < 100ms (全球边缘节点)
- **二进制大小**: 1.17MB (WASM)
- **并发支持**: 1000+ 请求/秒

## 🧪 测试

```bash
# 运行 Rust 测试
cd alou-edge && cargo test

# 运行前端测试
cd frontend && npm test

# 端到端测试
npm run test:e2e
```

## 📄 许可证

MIT License - 详见 [LICENSE](LICENSE) 文件

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

---

**注意**: 这是 Alou Pay 的边缘计算版本，专注于 Web3 和 AI 智能体的全球分布式部署。
