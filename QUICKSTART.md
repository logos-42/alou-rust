# 🚀 快速启动指南

## 问题：后端无法启动

错误信息：`DATABASE_URL must be set`

## ✅ 解决方案（选择一种）

### 方案 1：使用 Docker 数据库（最快）

```powershell
# 1. 安装 Docker Desktop (如果还没有)
# 下载: https://www.docker.com/products/docker-desktop/

# 2. 启动 PostgreSQL
docker run --name aloupay-postgres `
  -e POSTGRES_PASSWORD=mypassword `
  -e POSTGRES_DB=aloupay `
  -p 5432:5432 `
  -d postgres:15

# 3. 确认 .env 文件内容
DATABASE_URL=postgresql://postgres:mypassword@localhost:5432/aloupay
JWT_SECRET=dev-secret-key-please-change
JWT_EXPIRATION_HOURS=24
REFRESH_TOKEN_EXPIRATION_DAYS=30
GOOGLE_CLIENT_ID=
GOOGLE_CLIENT_SECRET=
GOOGLE_REDIRECT_URI=http://localhost:3001/api/auth/google/callback
PORT=3001

# 4. 运行迁移
# （第一次运行时）
# cd migrations
# psql -U postgres -h localhost -d aloupay -f 001_init_en.sql

# 5. 启动后端
cargo run --bin agent_http_server
```

###方案 2：跳过数据库（临时开发）

如果只想测试 AI 对话功能，不需要认证：

```powershell
# 1. 修改代码让数据库可选
# 编辑 src/bin/agent_http_server.rs
# 注释掉数据库相关代码

# 或者使用已有的简单版本
cargo run --bin quick_test
```

### 方案 3：安装本地 PostgreSQL

**1. 下载并安装 PostgreSQL**
- 下载: https://www.postgresql.org/download/windows/
- 选择版本 15.x
- 记住设置的密码（例如：`mypassword`）

**2. 创建数据库**
```bash
# 打开 PostgreSQL SQL Shell (psql)
# 输入密码后执行：
CREATE DATABASE aloupay;
\q
```

**3. 运行迁移**
```powershell
cd D:\AI\alou-pay\aloupay\migrations
psql -U postgres -d aloupay -f 001_init_en.sql
```

**4. 配置 .env**
```env
DATABASE_URL=postgresql://postgres:mypassword@localhost:5432/aloupay
# ... 其他配置
```

**5. 启动后端**
```powershell
cd D:\AI\alou-pay\aloupay
cargo run --bin agent_http_server
```

## 🧪 验证后端启动

成功启动后应该看到：
```
🚀 智能体 HTTP 服务器启动成功
🌐 服务地址: http://localhost:3001
```

测试：
```powershell
# 测试健康检查
curl http://localhost:3001/api/health

# 或在浏览器打开
start http://localhost:3001
```

## 📝 .env 文件完整示例

创建 `D:\AI\alou-pay\aloupay\.env` 文件：

```env
# 数据库配置
DATABASE_URL=postgresql://postgres:mypassword@localhost:5432/aloupay

# JWT配置
JWT_SECRET=your-secret-key-change-this-in-production-1234567890
JWT_EXPIRATION_HOURS=24
REFRESH_TOKEN_EXPIRATION_DAYS=30

# Google OAuth配置（可选，暂时可以留空）
GOOGLE_CLIENT_ID=
GOOGLE_CLIENT_SECRET=  
GOOGLE_REDIRECT_URI=http://localhost:3001/api/auth/google/callback

# DeepSeek API配置（可选）
DEEPSEEK_API_KEY=
DEEPSEEK_BASE_URL=https://api.deepseek.com

# 服务器配置
PORT=3001
```

## 🎯 启动完整应用

### 1. 启动数据库
```powershell
# 如果使用 Docker
docker start aloupay-postgres

# 如果使用本地 PostgreSQL
# 确保 PostgreSQL 服务正在运行
```

### 2. 启动后端
```powershell
cd D:\AI\alou-pay\aloupay
cargo run --bin agent_http_server
```

### 3. 启动前端
```powershell
cd D:\AI\alou-pay\aloupay\frontend
npm run dev
```

### 4. 打开浏览器
```
http://localhost:5173
```

## 🐛 常见错误解决

### 错误 1: DATABASE_URL must be set
**原因**: 缺少 .env 文件  
**解决**: 在项目根目录创建 `.env` 文件

### 错误 2: Connection refused (port 5432)
**原因**: PostgreSQL 未运行  
**解决**: 
```powershell
# Docker
docker start aloupay-postgres

# 或启动 PostgreSQL 服务
services.msc
# 找到 postgresql-x64-15 并启动
```

### 错误 3: UTF-8 encoding error
**原因**: PostgreSQL 语言设置  
**解决**:
```sql
-- 连接到数据库
psql -U postgres -d aloupay

-- 执行
ALTER DATABASE aloupay SET lc_messages TO 'en_US.UTF-8';
```

### 错误 4: Port 3001 already in use
**原因**: 端口被占用  
**解决**:
```powershell
# 查找并结束占用进程
netstat -ano | findstr :3001
taskkill /PID <进程ID> /F
```

## ✨ 成功标志

当看到这些信息时，表示一切正常：

**后端**:
```
🚀 智能体 HTTP 服务器启动成功
🌐 服务地址: http://localhost:3001
🔐 认证接口已启用
👤 用户接口已启用
```

**前端**:
```
VITE v7.x ready in xxx ms
➜  Local:   http://localhost:5173/
```

现在打开浏览器访问 `http://localhost:5173`，应该能看到全屏的 AI 对话界面！🎉

## 需要帮助？

如果遇到其他问题，请检查：
1. PostgreSQL 是否正在运行
2. .env 文件是否存在且配置正确
3. 端口 3001 和 5432 是否可用
4. 防火墙设置是否阻止了连接

