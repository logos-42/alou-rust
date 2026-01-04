# Claude Agent SDK 兼容性设置指南

## 概述

本指南说明如何设置 Claude Agent SDK 兼容性，使桌面应用能够使用 Claude Agent SDK，而后端实际使用 DeepSeek API。

## 架构

```
桌面应用 (Tauri)
    ↓ 使用 Claude Agent SDK
claude-agent.js (Node.js 脚本)
    ↓ 发送 Claude SDK 格式请求
/api/claude-agent/query (后端端点)
    ↓ 转换 Claude → DeepSeek 格式
DeepSeek API
```

## 配置步骤

### 1. 后端配置

#### 环境变量
在 `alou-edge/.dev.vars` 中配置 DeepSeek API 密钥：
```
AI_API_KEY=sk-your-deepseek-api-key-here
```

#### 路由配置
已修改 `alou-edge/src/compatibility/claude_sdk.rs` 中的 `get_provider_from_model` 函数：
```rust
// Claude 模型路由到 deepseek（与 claude-agent.js 配置一致）
m if m.starts_with("claude-3-5-sonnet-") => "deepseek",
m if m.starts_with("claude-3-haiku-") => "deepseek",
m if m.starts_with("claude-3-opus-") => "deepseek",
```

### 2. 桌面端配置

#### claude-agent.js 配置
`alou-desktop/scripts/claude-agent.js` 已配置：
- `ANTHROPIC_BASE_URL`: `http://127.0.0.1:8787/api/claude-agent/query`
- 模型路由：Claude 模型 → deepseek

### 3. 启动步骤

#### 启动后端服务
```bash
cd alou-edge
wrangler dev --port 8787
```

#### 测试 Claude Agent SDK
```bash
cd alou-desktop/scripts
node test-claude-agent.js
```

#### 直接测试 API
```bash
cd alou-desktop/scripts
node test-direct-api.js
```

## 验证测试

### 测试 1：健康检查
```bash
curl -X POST http://127.0.0.1:8787/api/health
```

### 测试 2：Claude Agent SDK 端点
```bash
curl -X POST http://127.0.0.1:8787/api/claude-agent/query \
  -H "Content-Type: application/json" \
  -d '{
    "apiKey": "alou-backend-default-token",
    "prompt": "Hello, world!",
    "model": "claude-3-5-sonnet-20241022",
    "maxTokens": 100
  }'
```

## 故障排除

### 问题 1：后端服务无法启动
- 检查 `build` 目录是否存在
- 检查 `.dev.vars` 文件中的 API 密钥
- 查看 wrangler 日志

### 问题 2：API 调用返回 403 错误
- 检查模型路由配置
- 验证 `get_provider_from_model` 函数
- 检查 DeepSeek API 密钥是否有效

### 问题 3：响应为空
- 检查后端日志中的调试信息
- 验证 DeepSeek API 是否返回有效响应
- 检查格式转换逻辑

## 代码修改总结

### 已修改的文件

1. **`alou-edge/src/compatibility/claude_sdk.rs`**
   - 修改 `get_provider_from_model` 函数，将 Claude 模型路由到 deepseek

2. **`alou-edge/src/router/claude.rs`**
   - 添加调试日志到 `handle_claude_sdk_query` 函数

3. **`alou-desktop/scripts/`**
   - 创建 `test-claude-agent.js` - 完整测试脚本
   - 创建 `test-direct-api.js` - 直接 API 测试
   - 创建 `test-claude-request.json` - 测试请求模板

## 预期结果

成功配置后，您应该能够：
1. 使用 Claude Agent SDK 发送请求
2. 后端将 Claude 格式转换为 DeepSeek 格式
3. 调用 DeepSeek API 获取响应
4. 将响应转换回 Claude 格式并返回

## 注意事项

1. **API 密钥安全**：确保 API 密钥安全存储，不要提交到版本控制
2. **模型兼容性**：某些 Claude 模型特性可能不完全兼容 DeepSeek
3. **工具调用**：工具调用格式可能需要额外转换
4. **性能监控**：监控 API 调用延迟和错误率

## 扩展支持

要支持更多模型，需要：
1. 在 `claude-agent.js` 的 `routing` 配置中添加映射
2. 在 `get_provider_from_model` 函数中添加对应的路由逻辑
3. 确保对应的 AI 提供商已实现
