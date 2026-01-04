# Alou Desktop

桌面版 Alou Web3 AI Agent 应用，基于 Tauri 构建。

## 功能特性

- ✅ 支持两种智能体类型：
  - **支付智能体**：系统预设的支付助手
  - **Claude Agent SDK**：可创建的智能体，自动生成 DIAP 身份
- ✅ **Claude Agent SDK 内置工具集成**：
  - **核心工具**: Bash、Read、Write、Edit、Glob、Grep、NotebookEdit
  - **网络工具**: WebSearch、WebFetch
  - **流程控制**: Plan、ExitPlanMode、AskUserQuestion、Subagents
  - **Web3工具**: 区块链查询、交易构建、钱包管理等
- ✅ DIAP 去中心化通信集成
- ✅ 轻量化前端设计
- ✅ 后端运行在 Cloudflare Workers
- ✅ **自动更新**：支持应用内自动更新
- ✅ **IPFS 节点**：内置 Kubo (IPFS) 二进制，支持本地 IPFS 节点
- ✅ **压缩优化**：构建时自动压缩，减小应用体积

## 开发

### 前置要求

- Node.js 20.19.0+ 或 22.12.0+
- Rust (最新稳定版)
- Tauri CLI: `npm install -g @tauri-apps/cli`

### 安装依赖

```bash
cd alou-desktop
npm install
```

### 设置 Kubo (IPFS) 二进制

在构建应用前，需要下载 Kubo 二进制文件：

**Windows:**
```powershell
npm run setup:kubo:win
```

**macOS/Linux:**
```bash
npm run setup:kubo:unix
```

**跨平台 (Node.js):**
```bash
npm run setup:kubo
```

这会将 Kubo 二进制文件下载到 `src-tauri/kubo/` 目录。

### 开发模式

```bash
npm run tauri:dev
```

这将启动 Vite 开发服务器（端口 1420）并打开 Tauri 桌面窗口。

### 构建

仅构建前端：
```bash
npm run build
```

构建桌面应用（开发版）：
```bash
npm run build:tauri
```

构建发布版本（所有平台，压缩优化）：
```bash
npm run build:tauri:release
```

## 配置

### API 后端

在 `.env` 文件中配置 Workers 后端地址：

```env
VITE_API_BASE_URL=https://your-workers-endpoint.workers.dev
```

开发环境可以使用本地 Workers：
```env
VITE_API_BASE_URL=http://127.0.0.1:8787
```

### 自动更新

1. 生成更新密钥对：
```bash
tauri signer generate -w ~/.tauri/myapp.key
```

2. 将公钥添加到 `src-tauri/tauri.conf.json` 的 `plugins.updater.pubkey`

3. 配置更新服务器端点（在 `tauri.conf.json` 中）

## IPFS 节点功能

应用内置了 Kubo (IPFS) 二进制，可以在本地运行 IPFS 节点。

### 使用 IPFS 服务

```javascript
import ipfsService from '@/services/ipfsService'

// 启动 IPFS 节点
await ipfsService.startNode()

// 获取节点信息
const info = await ipfsService.getNodeInfo()
console.log('Peer ID:', info.ID)

// 停止节点
await ipfsService.stopNode()
```

### IPFS 组件

使用 `IpfsStatus` 组件显示和管理 IPFS 节点：

```jsx
import IpfsStatus from '@/components/IpfsStatus'

<IpfsStatus />
```

## 项目结构

```
alou-desktop/
├── src/              # 前端源代码
│   ├── components/   # React 组件
│   │   └── IpfsStatus.jsx  # IPFS 状态组件
│   ├── services/     # API 服务
│   │   ├── api.js           # API 客户端
│   │   ├── agentService.js  # Agent 服务
│   │   └── ipfsService.js   # IPFS 服务
│   ├── hooks/        # React Hooks
│   └── views/        # 页面视图
├── src-tauri/        # Tauri 后端
│   ├── src/
│   │   ├── main.rs   # Tauri 入口（包含 IPFS 命令）
│   │   └── ipfs.rs   # IPFS 工具模块
│   ├── kubo/         # Kubo 二进制文件目录
│   │   ├── ipfs.exe (Windows)
│   │   └── ipfs (macOS/Linux)
│   ├── Cargo.toml
│   └── tauri.conf.json
├── scripts/
│   ├── setup-kubo.js    # 跨平台 Kubo 设置脚本
│   ├── setup-kubo.ps1   # Windows PowerShell 脚本
│   └── setup-kubo.sh    # Unix shell 脚本
├── package.json
└── vite.config.js
```

## 压缩和优化

### Rust 构建优化

已在 `Cargo.toml` 中配置：
- `opt-level = "z"` - 优化大小
- `lto = true` - 链接时优化
- `strip = true` - 移除调试符号

### 构建压缩版本

```bash
# 使用环境变量启用压缩
TAURI_COMPRESSION=1 npm run build:tauri:release
```

## 自动更新

应用支持自动更新功能：

1. **配置更新服务器**：在 `tauri.conf.json` 中设置更新端点
2. **生成签名密钥**：使用 `tauri signer generate` 生成密钥对
3. **发布更新**：构建新版本并上传到更新服务器
4. **用户更新**：应用会自动检测并提示更新

详细说明请查看 [DEPLOYMENT.md](../docs/DEPLOYMENT.md)

## 注意事项

1. **Kubo 二进制大小**：约 50-100MB，会增加应用体积
2. **首次启动**：IPFS 初始化需要时间（几秒到几分钟）
3. **存储空间**：IPFS 数据目录会占用空间（默认在应用数据目录）
4. **网络要求**：IPFS 节点需要网络连接才能加入网络
5. **权限要求**：IPFS 节点需要网络和文件系统权限

## 故障排除

### IPFS 节点无法启动

1. 检查 `src-tauri/kubo/` 目录中是否有二进制文件
2. 检查文件权限（Unix 系统需要执行权限）
3. 查看应用日志或控制台错误信息

### 更新失败

1. 检查网络连接
2. 验证更新服务器配置
3. 检查签名密钥是否正确

### 构建失败

1. 确保已安装 Rust 和 Tauri CLI
2. 确保 Kubo 二进制文件已下载
3. 检查 `tauri.conf.json` 配置是否正确

## Claude Agent SDK 工具集成

Alou Desktop 已完整集成 Claude Agent SDK 的所有内置工具，让AI能够像程序员一样在本地执行任务。

### 可用工具

#### 核心内置工具
- **Bash**: 运行终端命令、脚本、Git操作（支持持久化会话）
- **Read**: 读取工作目录中的任何文件内容
- **Write**: 创建新文件并写入内容
- **Edit**: 差分编辑，精确修改已有文件
- **Glob**: 使用模式匹配查找文件
- **Grep**: 使用正则表达式搜索文件内容
- **NotebookEdit**: 专门针对Jupyter Notebook文件的单元格操作

#### 网络与多模态工具
- **WebSearch**: 调用搜索引擎获取实时互联网信息
- **WebFetch**: 获取并解析网页的Markdown内容

#### 辅助与流程控制工具
- **Plan**: 进入"规划模式"，列出步骤并寻求用户确认
- **ExitPlanMode**: 退出规划模式，开始执行任务
- **AskUserQuestion**: 当遇到模糊需求时，主动询问用户
- **Subagents**: 创建"子Agent"来并行处理特定任务

#### Web3专用工具
- **query_blockchain**: 查询区块链数据
- **build_transaction**: 构建区块链交易
- **broadcast_transaction**: 广播交易
- **wallet_manager**: 钱包管理
- **agent_wallet**: 智能体钱包操作

### 快速使用示例

```javascript
import { AgentService } from './src/services/agentService';

const agentService = new AgentService();

// 使用Claude Agent SDK工具
const response = await agentService.sendMessage(
  sessionId,
  "请帮我分析这个项目的代码结构",
  walletAddress,
  {
    mode: 'agent',
    categories: ['CORE', 'NETWORK'] // 使用核心和网络工具
  }
);

// 处理工具调用结果
if (response.tool_calls && response.tool_calls.length > 0) {
  console.log('Agent调用了以下工具:', response.tool_calls);
  
  response.tool_results?.forEach(result => {
    if (result.success) {
      console.log(`工具 ${result.tool} 执行成功:`, result.result);
    } else {
      console.error(`工具 ${result.tool} 执行失败:`, result.error);
    }
  });
}
```

### 预定义配置

项目提供了多个预定义配置，位于 `config/claude-agent-tools.example.js`：

```javascript
import { 
  DEVELOPER_AGENT_CONFIG,    // 开发者配置
  WEB3_AGENT_CONFIG,         // Web3专家配置  
  FULL_FEATURED_AGENT_CONFIG // 全功能配置
} from './config/claude-agent-tools.example';

// 使用预定义配置
const response = await agentService.sendMessage(
  sessionId,
  message,
  walletAddress,
  {
    ...DEVELOPER_AGENT_CONFIG,
    mode: 'agent'
  }
);
```

### 测试工具集成

```bash
# 运行测试脚本
cd alou-desktop
node scripts/test-claude-agent-tools.js
```

### 安全注意事项

1. **Bash工具**: 避免执行未知来源的命令，敏感操作需用户确认
2. **文件操作**: 重要文件操作前建议备份，限制操作范围
3. **网络操作**: 验证URL安全性，使用HTTPS连接
4. **权限控制**: 限制工具执行权限，记录操作日志

## 相关文档

- [架构说明](../docs/ARCHITECTURE.md) - 前后端分离架构
- [部署指南](../docs/DEPLOYMENT.md) - 自动更新和部署说明
- [Claude Agent SDK 集成指南](../docs/CLAUDE_AGENT_SDK_INTEGRATION.md) - 详细工具使用说明
