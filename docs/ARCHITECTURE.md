# Alou Desktop 架构说明

## 前后端分离架构

### 架构概览

```
┌─────────────────────────────────────┐
│      Tauri Desktop App (前端)       │
│  ┌───────────────────────────────┐ │
│  │   React + Vite (UI Layer)     │ │
│  │   - Components                │ │
│  │   - Hooks                     │ │
│  │   - Services                  │ │
│  └───────────────────────────────┘ │
│           │                         │
│           │ HTTP/HTTPS              │
│           ▼                         │
└─────────────────────────────────────┘
           │
           │ REST API
           │
┌─────────────────────────────────────┐
│  Cloudflare Workers (后端)          │
│  ┌───────────────────────────────┐ │
│  │   alou-edge (Rust/WASM)      │ │
│  │   - API Routes               │ │
│  │   - Agent Core               │ │
│  │   - DIAP Integration         │ │
│  │   - Blockchain Tools         │ │
│  └───────────────────────────────┘ │
└─────────────────────────────────────┘
```

## 前后端交互逻辑

### 1. API 通信层

**统一使用 `apiClient` (axios 实例)**：
- 位置：`src/services/api.js`
- 功能：
  - 统一配置 baseURL（从环境变量读取）
  - 自动添加认证 token
  - 统一错误处理
  - Token 自动刷新机制

**配置逻辑**：
```javascript
// 开发环境：http://127.0.0.1:8787
// 生产环境：https://alou-edge.yuanjieliu65.workers.dev
// 可通过 VITE_API_BASE_URL 环境变量覆盖
```

### 2. 服务层

**AgentService**：
- 位置：`src/services/agentService.js`
- 职责：封装所有与后端 API 的交互
- 特点：
  - 使用 `apiClient`，不重复配置 URL
  - 所有 API 调用都通过相对路径（如 `/session`，`/agent/chat`）
  - `apiClient` 自动处理 baseURL 拼接

### 3. 组件层

**AgentChat 组件**：
- 使用 `agentService` 或 `apiClient` 进行 API 调用
- 不再直接使用 `fetch`，确保：
  - 统一的错误处理
  - 自动的 token 管理
  - 一致的 URL 配置

## 数据流

### 请求流程

```
Component
  ↓
agentService / apiClient
  ↓
api.js (axios instance)
  ↓
HTTP Request (with auth headers)
  ↓
Cloudflare Workers
  ↓
Response (JSON)
  ↓
api.js (error handling, token refresh)
  ↓
agentService (data extraction)
  ↓
Component (state update)
```

### 错误处理流程

```
API Error
  ↓
api.js interceptor
  ↓
401 Unauthorized?
  ↓
Yes → Try refresh token
  ↓
Refresh success → Retry request
  ↓
Refresh failed → Clear tokens → Redirect to login
  ↓
Other errors → Propagate to component
```

## 环境配置

### 开发环境

```bash
# .env
VITE_API_BASE_URL=http://127.0.0.1:8787
```

需要本地运行 Workers：
```bash
cd alou-edge
npm run dev
# 或
wrangler dev
```

### 生产环境

```bash
# .env
VITE_API_BASE_URL=https://alou-edge.yuanjieliu65.workers.dev
```

后端已部署到 Cloudflare Workers。

## 关键设计决策

### 1. 为什么使用 axios 而不是 fetch？

- **统一配置**：baseURL、timeout、headers 集中管理
- **拦截器**：自动 token 刷新、错误处理
- **请求/响应转换**：自动 JSON 序列化/反序列化
- **错误处理**：统一的错误格式

### 2. 为什么服务层使用相对路径？

- **解耦**：服务层不关心具体的后端地址
- **灵活性**：通过环境变量切换开发/生产环境
- **一致性**：所有 API 调用都通过同一个 baseURL

### 3. 为什么不在组件中直接使用 fetch？

- **重复代码**：每个组件都要处理 URL、headers、错误
- **不一致**：不同组件可能有不同的错误处理逻辑
- **维护困难**：URL 变更需要修改多个地方

## 最佳实践

1. **所有 API 调用**都通过 `agentService` 或 `apiClient`
2. **不要**在组件中直接使用 `fetch` 或硬编码 URL
3. **使用环境变量**配置后端地址
4. **统一错误处理**，让 `apiClient` 拦截器处理常见错误
5. **Token 管理**由 `apiClient` 自动处理，组件无需关心

## 调试

### 检查 API 连接

```javascript
// 在浏览器控制台
import apiClient from '@/services/api'
console.log(apiClient.defaults.baseURL)
```

### 查看请求日志

axios 会自动记录请求，可在浏览器 DevTools Network 标签查看。

### 常见问题

1. **CORS 错误**：确保后端配置了正确的 CORS 头
2. **401 错误**：检查 token 是否有效，查看 token 刷新逻辑
3. **连接失败**：检查 `VITE_API_BASE_URL` 配置和后端是否运行

