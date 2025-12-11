# Agent 模式连接指南

## 概述

桌面版的 Agent 模式使用后端的 **TypeScript SDK** 部分（`claude_agent_sdk`），该部分提供了：
- ✅ 无限工具调用次数
- ✅ 通过 Rust WASM 执行工具
- ✅ DeepSeek API 作为模型后端
- ✅ Claude Agent SDK 风格的接口

## 后端连接配置

### 自动配置

桌面版已自动配置后端连接：

**开发环境** (`npm run dev`):
- 后端 URL: `http://127.0.0.1:8787`
- 需要本地运行 `wrangler dev`

**生产环境** (`npm run build`):
- 后端 URL: `https://alou-edge.yuanjieliu65.workers.dev`
- 自动连接到已部署的 Cloudflare Worker

### 手动配置

如果需要自定义后端 URL，创建 `.env` 文件：

```bash
# alou-desktop/.env
VITE_API_BASE_URL=https://alou-edge.yuanjieliu65.workers.dev
```

## Agent 模式工作流程

### 1. 创建 Agent（设置 agent_type）

当创建新的 Agent 时，桌面版会：
1. 调用 `/api/agent/create-claude` 端点
2. 自动设置 `agent_type: 'claude_agent_sdk'`
3. 将 agent 元数据保存到 session 的 KV 存储中

### 2. 发送消息（自动路由）

当发送消息到 Agent 时：
1. 前端调用 `/api/agent/chat`
2. 后端检查 session 中的 `agent_metadata.agent_type`
3. 如果 `agent_type === 'claude_agent_sdk'`：
   - ✅ 使用 **TypeScript SDK** (`src/ts-agent/claude-agent.ts`)
   - ✅ 调用 DeepSeek API
   - ✅ 支持工具调用（通过 `/api/mcp/execute-tool`）
4. 否则：
   - 使用 **Rust WASM** 后端（Alou 模式）

### 3. 工具调用流程

```
用户消息 
  → TypeScript SDK (DeepSeek API)
  → 检测工具调用
  → 调用 /api/mcp/execute-tool (Rust WASM)
  → 执行工具
  → 返回结果
  → 继续对话
```

## 验证连接

### 1. 检查后端健康状态

```bash
curl https://alou-edge.yuanjieliu65.workers.dev/api/health
```

### 2. 检查工具列表

```bash
curl https://alou-edge.yuanjieliu65.workers.dev/api/mcp/tools
```

### 3. 创建 Agent 并测试

在桌面版中：
1. 点击"创建 Agent"
2. 填写 Agent 信息
3. 创建后会自动设置 `agent_type: 'claude_agent_sdk'`
4. 发送消息测试

### 4. 查看后端日志

后端会在日志中显示：
- `[Worker] Agent 模式：使用 TypeScript SDK (DeepSeek API 作为模型后端)`
- `[Worker] 获取到 X 个可用工具`
- `[Worker] 执行工具: <tool_name>`

## 已部署的后端功能

✅ **混合架构**
- TypeScript 入口 (`src/worker.ts`)
- Rust WASM 后端 (`src/lib.rs`)

✅ **双模式支持**
- Alou 模式（Rust WASM + DeepSeek）
- Agent 模式（TypeScript SDK + DeepSeek）

✅ **工具调用**
- 无限工具调用次数
- 通过 Rust WASM 执行工具
- 自动获取工具列表

✅ **API 端点**
- `/api/health` - 健康检查
- `/api/agent/chat` - 智能体聊天
- `/api/agent/create-claude` - 创建 Agent
- `/api/mcp/tools` - 工具列表
- `/api/mcp/execute-tool` - 工具执行

## 故障排除

### 问题：无法连接到后端

**解决方案**：
1. 检查 `VITE_API_BASE_URL` 环境变量
2. 确认后端已部署：`https://alou-edge.yuanjieliu65.workers.dev/api/health`
3. 检查网络连接和 CORS 设置

### 问题：Agent 模式未使用 TypeScript SDK

**检查**：
1. 确认 Agent 的 `agent_type` 是 `'claude_agent_sdk'`
2. 查看后端日志确认路由选择
3. 检查 session KV 中的 `agent_metadata`

### 问题：工具调用失败

**检查**：
1. 确认 `/api/mcp/tools` 返回工具列表
2. 检查工具执行日志
3. 验证工具参数格式

## 相关文件

- **后端入口**: `alou-edge/src/worker.ts`
- **TypeScript SDK**: `alou-edge/src/ts-agent/claude-agent.ts`
- **工具执行**: `alou-edge/src/router/mcp.rs`
- **前端 API 配置**: `alou-desktop/src/services/api.js`
- **Agent 服务**: `alou-desktop/src/services/agentService.js`

