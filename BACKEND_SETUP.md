# 后端启动指南

## 🚨 当前问题

后端启动失败，错误信息：
```
Error: Postgres protocol error - DATABASE_URL must be set
```

## ✅ 快速解决方案

### 方案 1：不使用数据库运行（推荐用于开发）

1. **创建 `.env` 文件**（已自动创建）
```bash
# 在项目根目录
DATABASE_URL=sqlite::memory:
SKIP_DATABASE=true
JWT_SECRET=dev-secret-key
```

2. **启动简化版后端**
```bash
cargo run --bin agent_http_server
```

### 方案 2：使用完整数据库功能

#### 安装 PostgreSQL

**Windows:**
1. 下载: https://www.postgresql.org/download/windows/
2. 安装并记住密码
3. 启动 PostgreSQL 服务

**使用 Docker（推荐）:**
```bash
docker run --name aloupay-db -e POSTGRES_PASSWORD=password -e POSTGRES_DB=aloupay -p 5432:5432 -d postgres:15
```

#### 配置数据库

1. **修改 `.env` 文件**
```env
DATABASE_URL=postgresql://postgres:password@localhost/aloupay
```

2. **初始化数据库**
```bash
# 运行迁移脚本
cd migrations
powershell -ExecutionPolicy Bypass -File setup_local_db.ps1
```

或手动执行：
```bash
psql -U postgres -d aloupay -f migrations/001_init_en.sql
```

3. **启动后端**
```bash
cargo run --bin agent_http_server
```

## 🎯 API 端点

启动成功后，可以访问：

### 核心功能（无需数据库）
- `GET http://localhost:3001/api/health` - 健康检查
- `POST http://localhost:3001/api/chat` - AI 对话

### 认证功能（需要数据库）
- `POST /api/auth/wallet/nonce` - 获取钱包签名 nonce
- `POST /api/auth/wallet/verify` - 验证钱包签名并登录
- `GET /api/auth/google/login` - Google OAuth 登录
- `POST /api/auth/verify` - 验证 JWT token
- `POST /api/auth/logout` - 登出

## 🧪 测试后端

### 1. 健康检查
```bash
curl http://localhost:3001/api/health
```

预期响应：
```json
{
  "status": "ok",
  "agent_ready": true,
  "timestamp": 1234567890
}
```

### 2. AI 对话测试
```bash
curl -X POST http://localhost:3001/api/chat \
  -H "Content-Type: application/json" \
  -d '{
    "message": "你好",
    "session_id": "test123"
  }'
```

### 3. 钱包认证测试（需要数据库）
```bash
# 获取 nonce
curl -X POST http://localhost:3001/api/auth/wallet/nonce \
  -H "Content-Type: application/json" \
  -d '{"address": "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb"}'
```

## 🐛 常见问题

### 问题 1: DATABASE_URL 错误
**解决**: 确保 `.env` 文件存在于项目根目录

### 问题 2: PostgreSQL 连接失败
**解决**: 
- 检查 PostgreSQL 是否运行
- 检查端口 5432 是否开放
- 验证用户名密码是否正确

### 问题 3: UTF-8 错误
**解决**: 
```sql
-- 连接到 PostgreSQL
psql -U postgres

-- 设置数据库编码
ALTER DATABASE aloupay SET lc_messages TO 'C';
```

### 问题 4: 端口被占用
**解决**: 
```bash
# 查找占用端口的进程
netstat -ano | findstr :3001

# 结束进程
taskkill /PID <进程ID> /F

# 或更改端口
# 在 .env 中设置
PORT=3002
```

## 📦 依赖检查

确保已安装：
- [x] Rust (cargo)
- [ ] PostgreSQL 或 Docker
- [x] .env 文件配置

## 🚀 启动顺序

1. **环境配置**
   ```bash
   # 确认 .env 存在
   ls .env
   ```

2. **（可选）启动数据库**
   ```bash
   docker start aloupay-db
   # 或
   # 启动 PostgreSQL 服务
   ```

3. **编译并运行**
   ```bash
   cargo run --bin agent_http_server
   ```

4. **测试连接**
   ```bash
   curl http://localhost:3001/api/health
   ```

## 🔧 开发模式建议

如果只是测试前端，可以：

1. **临时跳过认证**：前端暂时使用 localStorage 模拟登录
2. **只使用 /api/chat 接口**：专注于 AI 对话功能
3. **稍后添加数据库**：等功能稳定后再集成完整认证

## 📚 下一步

- [ ] 配置 DeepSeek API Key
- [ ] 设置 Google OAuth (可选)
- [ ] 配置数据库 (可选)
- [ ] 测试钱包连接功能

祝您开发顺利！🎉

