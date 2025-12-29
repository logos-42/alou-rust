# CORS问题修复说明

## 问题描述
前端应用运行在 `http://localhost:1420`，但尝试直接访问后端API `https://alou-edge.yuanjieliu65.workers.dev` 时，由于CORS（跨域资源共享）策略，浏览器阻止了请求。

## 解决方案
配置Vite开发服务器作为代理，将所有 `/api` 请求转发到生产环境API，并自动添加CORS头。

## 配置更改

### 1. Vite配置 (`vite.config.js`)
添加了代理配置：
```javascript
proxy: {
  '/api': {
    target: 'https://alou-edge.yuanjieliu65.workers.dev',
    changeOrigin: true,
    rewrite: (path) => path.replace(/^\/api/, '/api'),
    configure: (proxy, options) => {
      // 添加CORS头
      proxy.on('proxyRes', (proxyRes, req, res) => {
        res.setHeader('Access-Control-Allow-Origin', 'http://localhost:1420')
        res.setHeader('Access-Control-Allow-Methods', 'GET, POST, PUT, DELETE, OPTIONS')
        res.setHeader('Access-Control-Allow-Headers', 'Content-Type, Authorization, X-Requested-With')
        res.setHeader('Access-Control-Allow-Credentials', 'true')
        
        // 处理预检请求
        if (req.method === 'OPTIONS') {
          res.writeHead(200)
          res.end()
          return
        }
      })
    },
  },
}
```

### 2. API基础URL更新
将所有开发环境下的API基础URL从 `http://127.0.0.1:8787` 或 `http://localhost:8787` 更新为 `http://localhost:1420`，这样请求会先发送到Vite开发服务器，然后由代理转发。

更新的文件：
- `src/services/api.js`
- `src/services/subscriptionService.js`
- `src/components/agent/AgentDetailPanel.jsx`
- `src/hooks/useAgentChat.js`
- `src/services/blockchainService.js`
- `src/services/pubsubService.js`

## 工作原理

1. **前端请求流程**：
   ```
   浏览器 (localhost:1420) 
   → 发送请求到 /api/agent/chat
   → Vite开发服务器 (localhost:1420)
   → 代理转发到 https://alou-edge.yuanjieliu65.workers.dev/api/agent/chat
   → 添加CORS头返回响应
   → 浏览器接收响应
   ```

2. **CORS头添加**：
   - `Access-Control-Allow-Origin: http://localhost:1420`
   - `Access-Control-Allow-Methods: GET, POST, PUT, DELETE, OPTIONS`
   - `Access-Control-Allow-Headers: Content-Type, Authorization, X-Requested-With`
   - `Access-Control-Allow-Credentials: true`

## 测试方法

### 方法1：使用测试脚本
```bash
cd alou-desktop
node test-proxy.js
```

### 方法2：手动测试
1. 启动开发服务器：
   ```bash
   npm run dev
   ```

2. 在浏览器中访问：
   ```
   http://localhost:1420/api/health
   ```

3. 检查响应头中是否包含正确的CORS头。

### 方法3：在应用中测试
1. 启动应用：
   ```bash
   npm run dev
   ```

2. 打开浏览器访问 `http://localhost:1420`
3. 尝试发送消息给智能体
4. 检查浏览器控制台是否还有CORS错误

## 注意事项

1. **开发环境专用**：此配置仅用于开发环境。生产环境会直接访问 `https://alou-edge.yuanjieliu65.workers.dev`。

2. **本地后端服务器**：如果将来需要启动本地后端服务器（端口8787），需要：
   - 将API基础URL改回 `http://localhost:8787`
   - 确保后端服务器正确配置CORS头
   - 或者继续使用Vite代理，但修改target为 `http://localhost:8787`

3. **网络要求**：需要网络连接才能访问生产环境API。

## 故障排除

### 问题1：代理不工作
- 检查Vite服务器是否正在运行
- 检查 `vite.config.js` 中的代理配置是否正确
- 查看Vite服务器日志是否有错误

### 问题2：仍然有CORS错误
- 检查浏览器开发者工具中的网络请求
- 确认请求是否发送到 `localhost:1420` 而不是直接到生产环境
- 检查响应头中是否包含CORS头

### 问题3：API返回500错误
- 检查生产环境API是否正常运行
- 查看Vite服务器控制台中的代理错误信息
- 测试直接访问生产环境API：`https://alou-edge.yuanjieliu65.workers.dev/api/health`

## 恢复原始配置
如果需要恢复使用本地后端服务器（端口8787），将所有 `http://localhost:1420` 改回 `http://localhost:8787` 或 `http://127.0.0.1:8787`，并移除Vite配置中的代理设置。
