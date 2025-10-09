# 🔐 Alou智能助手 - 认证系统使用指南

## ✅ 完成的功能

### 后端 (Rust + Warp)
- ✅ PostgreSQL数据库集成
- ✅ JWT Token认证
- ✅ Google OAuth 2.0登录
- ✅ 用户管理API
- ✅ 会话管理
- ✅ 认证中间件

### 前端 (Vue 3 + TypeScript)
- ✅ 登录页面
- ✅ OAuth回调处理
- ✅ 状态管理 (Pinia)
- ✅ 路由守卫
- ✅ API服务层
- ✅ UI组件

---

## 🚀 快速启动

### 1. 确认配置

检查 `.env` 文件是否正确配置：

```bash
# 数据库
DATABASE_URL=postgresql://alou_admin:local_dev_password@localhost:5432/alou_pay

# Google OAuth
GOOGLE_CLIENT_ID=your-client-id
GOOGLE_CLIENT_SECRET=your-client-secret
GOOGLE_REDIRECT_URI=http://localhost:5173/auth/callback

# JWT
JWT_SECRET=dev-secret-key-replace-in-production-min-32-chars-long
JWT_EXPIRATION_HOURS=24
```

### 2. 启动后端

```powershell
# 方法1：使用cargo run
cargo run --bin agent_http_server

# 方法2：使用release构建（更快）
cargo build --release
.\target\release\agent_http_server.exe
```

后端将启动在：`http://localhost:3001`

### 3. 启动前端

```powershell
cd frontend
npm run dev
```

前端将启动在：`http://localhost:5173`

---

## 🧪 测试流程

### 1. 验证后端API

```powershell
# 健康检查
curl http://localhost:3001/api/health

# 查看所有端点
curl http://localhost:3001/
```

### 2. 测试Google登录

1. 访问 `http://localhost:5173`
2. 应该会自动跳转到登录页面
3. 点击"使用 Google 账号登录"
4. 完成Google授权
5. 自动跳转回首页

### 3. 验证认证状态

打开浏览器开发者工具：
- **Application → Cookies** 查看：
  - `access_token`
  - `refresh_token`
- **Network → XHR** 查看API请求

---

## 📡 API端点列表

### 认证API

| 方法 | 路径 | 说明 | 需要认证 |
|------|------|------|---------|
| GET  | `/api/auth/google/login` | 获取Google登录URL | ❌ |
| GET  | `/api/auth/google/callback` | 处理OAuth回调 | ❌ |
| POST | `/api/auth/verify` | 验证Token | ✅ |
| POST | `/api/auth/refresh` | 刷新Token | ❌ |
| POST | `/api/auth/logout` | 登出 | ✅ |

### 用户API

| 方法 | 路径 | 说明 | 需要认证 |
|------|------|------|---------|
| GET  | `/api/user/me` | 获取当前用户 | ✅ |
| PUT  | `/api/user/profile` | 更新用户资料 | ✅ |

### 智能体API

| 方法 | 路径 | 说明 | 需要认证 |
|------|------|------|---------|
| GET  | `/api/health` | 健康检查 | ❌ |
| POST | `/api/chat` | 智能体聊天 | ❌ (将来需要) |

---

## 🐛 常见问题

### 1. 后端启动失败

**错误**: `DATABASE_URL must be set`

**解决**:
```powershell
# 检查.env文件是否存在
Get-Content .env

# 如果不存在，复制模板
Copy-Item .env.example .env
# 然后编辑.env填入真实值
```

---

### 2. 数据库连接失败

**错误**: `error connecting to database`

**解决**:
```powershell
# 检查PostgreSQL是否运行
Get-Service -Name "postgresql*"

# 测试数据库连接
psql -U alou_admin -d alou_pay -c "SELECT 1;"
```

---

### 3. Google登录失败

**错误**: `Invalid redirect URI`

**解决**:
1. 检查Google Cloud Console中的重定向URI
2. 确认包含：`http://localhost:5173/auth/callback`
3. 确认`.env`中的`GOOGLE_REDIRECT_URI`一致

---

### 4. Token刷新失败

**问题**: 一段时间后自动登出

**原因**: 
- Token过期
- Refresh token失效

**解决**:
- 检查`.env`中的过期时间设置
- 清除浏览器Cookies重新登录

---

## 📂 项目结构

```
.
├── src/                     # 后端Rust代码
│   ├── models/             # 数据模型
│   ├── db/                 # 数据库
│   ├── auth/               # 认证模块
│   ├── api/                # API端点
│   └── bin/                # 可执行文件
│       └── agent_http_server.rs
│
├── frontend/               # 前端Vue代码
│   └── src/
│       ├── views/          # 页面
│       │   ├── LoginView.vue
│       │   └── AuthCallbackView.vue
│       ├── components/     # 组件
│       │   ├── GoogleLoginButton.vue
│       │   └── UserAvatar.vue
│       ├── stores/         # 状态管理
│       │   └── auth.ts
│       ├── services/       # API服务
│       │   ├── api.ts
│       │   ├── auth.service.ts
│       │   └── user.service.ts
│       ├── types/          # 类型定义
│       │   └── auth.ts
│       └── router/         # 路由配置
│           └── index.ts
│
├── migrations/             # 数据库迁移
│   └── 001_init_en.sql
│
├── .env                    # 环境配置（不提交）
├── .env.example            # 配置模板
└── AUTHENTICATION_GUIDE.md # 本文档
```

---

## 🔒 安全最佳实践

### 开发环境
- ✅ 使用`.env`文件管理密钥
- ✅ `.env`已加入`.gitignore`
- ✅ JWT Secret至少32字符
- ✅ Token有过期时间

### 生产环境（未来）
- ⚠️ 使用HTTPS
- ⚠️ 更改所有默认密钥
- ⚠️ 启用CORS白名单
- ⚠️ 配置Rate Limiting
- ⚠️ 启用SSL证书

---

## 📊 性能监控

### 查看后端日志

```powershell
# 开发模式（详细日志）
$env:RUST_LOG="debug"
cargo run --bin agent_http_server

# 生产模式（简洁日志）
$env:RUST_LOG="info"
cargo run --bin agent_http_server --release
```

### 数据库查询

```sql
-- 查看用户列表
SELECT id, email, name, created_at FROM users;

-- 查看活跃会话
SELECT user_id, expires_at FROM sessions WHERE expires_at > NOW();

-- 清理过期会话
DELETE FROM sessions WHERE expires_at < NOW();
```

---

## 🎯 下一步开发

### 短期（1-2周）
- [ ] 添加用户个人中心页面
- [ ] 实现头像上传
- [ ] 添加更多用户设置

### 中期（1个月）
- [ ] DID去中心化身份集成
- [ ] IPFS文档存储
- [ ] 钱包连接（MetaMask）

### 长期（3个月+）
- [ ] 多因素认证（2FA）
- [ ] OAuth支持更多提供商（GitHub, 微信等）
- [ ] SSO单点登录

---

## 📞 技术支持

如有问题，请查看：
- [项目README](README.md)
- [GitHub Issues](https://github.com/your-repo/issues)
- [API文档](http://localhost:3001/)

---

**祝开发愉快！** 🚀

