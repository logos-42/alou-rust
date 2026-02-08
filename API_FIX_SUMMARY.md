# API 功能修复总结

## 🔧 已修复的问题

### 1. Vite 代理配置错误
**问题**: `vite.config.js` 将所有 API 请求代理到了生产环境，而不是本地开发服务器

**修复**: 修改 `alou-desktop/vite.config.js`
```javascript
proxy: {
  '/api': {
    target: 'http://127.0.0.1:8787', // 改为本地后端
    changeOrigin: true,
    secure: false,
    // ...
  },
}
```

### 2. 工具执行 API 请求格式不匹配
**问题**: 前端发送的工具执行请求格式与后端期望的格式不匹配

**前端发送**:
```json
{
  "tool_id": "wallet_balance",
  "args": {...},
  "session_id": "...",
  "timeout_seconds": 30
}
```

**后端期望**:
```json
{
  "tool_name": "wallet_balance",
  "args": {...},
  "wallet_address": "...",
  "chain": "..."
}
```

**修复**: 修改 `useAsyncTaskPolling.ts`
- 更新了请求格式以匹配后端期望
- 添加了 walletAddress 和 chain 参数到 Hook

### 3. Hook 参数缺失
**问题**: `useAsyncTaskPolling` Hook 缺少 wallet 和 chain 信息

**修复**: 
- 修改 Hook 参数类型，添加 `walletAddress` 和 `chain`
- 在 `useAgentMessages.ts` 中传递这些参数
- 将 walletAddress 提取到组件级别，避免重复获取

## ✅ 修复的文件清单

1. `alou-desktop/vite.config.js` - 代理配置
2. `alou-desktop/src/components/AgentChat/hooks/useAsyncTaskPolling.ts` - 工具执行格式
3. `alou-desktop/src/components/AgentChat/useAgentMessages.ts` - Hook 参数传递

## 🚀 如何使用

### 1. 确保后端服务运行
```bash
cd alou-edge
npm run dev
# 服务将运行在 http://127.0.0.1:8787
```

### 2. 启动桌面应用
```bash
cd alou-desktop
npm run tauri:dev
```

### 3. 测试 API
使用提供的测试脚本验证功能：
```bash
node test-api-flow.js
```

## 📋 API 端点列表

### 核心端点
- `POST /api/ai-task/init-and-start` - 发送消息并启动 AI 任务
- `GET /api/ai-task/{task_id}/status` - 查询任务状态
- `GET /api/ai-task/{task_id}/pending-tools` - 获取待处理工具调用
- `POST /api/ai-task/{task_id}/tool-result` - 提交工具执行结果
- `POST /api/mcp/execute-tool` - 执行单个工具

### 其他端点
- `GET /api/health` - 健康检查
- `POST /api/agent/chat` - 直接 Agent 对话
- `POST /api/agent/create` - 创建智能体
- `GET /api/agent/list` - 获取智能体列表

## 🔄 完整消息流程

1. **用户发送消息**
   - 前端调用 `apiClient.post('/ai-task/init-and-start', request)`
   - 请求包含：prompt, systemPrompt, tools, agentInfo, model 等

2. **后端创建任务**
   - 创建异步任务并返回 task_id
   - 使用 Durable Object 管理任务状态

3. **前端轮询状态**
   - 使用 `pollAsyncTask` 轮询 `/ai-task/{task_id}/status`
   - 显示加载状态和进度

4. **工具调用处理**
   - 如果任务需要工具调用，后端返回 `processing` 状态
   - 前端调用 `/ai-task/{task_id}/pending-tools` 获取待处理工具
   - 前端执行工具（调用 `/mcp/execute-tool`）
   - 提交工具结果到 `/ai-task/{task_id}/tool-result`

5. **任务完成**
   - 后端返回 `completed` 状态和 AI 响应
   - 前端显示 AI 回复消息

## 🔍 调试技巧

### 1. 检查后端日志
后端服务会输出详细的日志，包括：
- 任务创建和初始化
- AI 调用过程
- 工具执行结果
- 状态变更

### 2. 浏览器开发者工具
- Network 标签查看 API 请求
- Console 查看前端日志
- 检查请求 URL 是否正确

### 3. 测试脚本
使用 `test-api-flow.js` 验证后端功能：
```bash
node test-api-flow.js
```

## ⚠️ 注意事项

1. **确保后端服务先启动**，否则前端请求会失败
2. **检查环境变量**，确保 `VITE_API_BASE_URL` 配置正确
3. **API Key 配置**，后端需要配置 AI API Key（DEEPSEEK_API_KEY 或 AI_API_KEY）
4. **端口冲突**，确保端口 8787 和 1420 未被占用

## 🎉 预期结果

修复后，你应该能够：
- ✅ 发送消息并收到 AI 回复
- ✅ 看到任务执行进度
- ✅ 使用工具（如查询钱包余额）
- ✅ 看到工具调用结果
- ✅ 正常完成对话流程
