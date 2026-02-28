# Alou MVP 发布计划 v1.0

**制定时间**: 2026-02-28  
**目标版本**: alou-desktop v0.1.10 + alou-edge v0.2.5  
**预计发布时间**: 3-5 天内

---

## 🎯 项目现状概览

### 已完成的核心功能

| 模块 | 功能 | 状态 |
|------|------|------|
| AI 智能体 | 多轮对话、流式响应、会话持久化 | ✅ 完成 |
| Web3 钱包 | MetaMask、WalletConnect、本地钱包 | ✅ 完成 |
| 智能体钱包 | 自主创建、余额查询、多网络支持 | ✅ 完成 |
| 桌面应用 | Tauri 框架、IPFS 节点、自动更新 | ✅ 完成 |
| 自主智能体 | 任务队列、记忆管理、心跳机制 | ✅ 完成 |
| 群聊功能 | AI Agent 协作、任务分配、进度汇报 | ✅ 完成 |
| CLI 工具 | Agent 交互、自主行动、配置管理 | ✅ 完成 |
| 19个AI工具 | Skills 自动选择、工具执行 | ✅ 完成 |

### 技术栈版本

- **alou-desktop**: v0.1.10 (React 18 + Tauri 2.0)
- **alou-edge**: v0.2.5 (Rust/WASM + Cloudflare Workers)
- **AI SDK**: Claude Agent SDK v0.2.39

---

## 📋 Phase 1: 发布前检查与准备 (Day 1)

### 1.1 代码质量检查

```bash
# 1. 运行测试
cd alou-edge && cargo test
cd alou-desktop && npm run lint:check

# 2. 检查 TypeScript 类型
cd alou-edge && npm run build:ts

# 3. 检查未提交的更改
git status
git diff --stat
```

**检查清单**:
- [ ] 所有测试通过
- [ ] 无 ESLint 错误
- [ ] TypeScript 类型检查通过
- [ ] 代码已提交到 git
- [ ] 无敏感信息泄露（API keys、私钥等）

### 1.2 配置文件审查

**alou-desktop/.env** (不要提交到 git):
```env
# 生产环境后端地址
VITE_API_BASE_URL=https://alou-edge.yuanjieliu65.workers.dev

# 可选：DeepSeek API Key（如需前端直接调用）
# VITE_DEEPSEEK_API_KEY=your_key_here
```

**alou-edge/wrangler.toml 检查**:
- [ ] D1 数据库配置正确
- [ ] KV namespaces 配置正确
- [ ] Secrets 已设置（通过 `wrangler secret put`）
  - `AI_API_KEY`
  - `JWT_SECRET`

### 1.3 文档完整性

- [ ] README.md 已更新
- [ ] QUICKSTART.md 测试通过
- [ ] CHANGELOG.md 已创建并更新
- [ ] 版本号已更新
  - `alou-desktop/package.json`: 0.1.10
  - `alou-edge/package.json`: 0.2.5
  - `alou-desktop/src-tauri/tauri.conf.json`: 0.1.10

---

## 🔧 Phase 2: 后端部署优化 (Day 1-2)

### 2.1 部署 alou-edge 到生产环境

```bash
cd alou-edge

# 1. 构建 WASM
npm run build:wasm

# 2. 部署到 Cloudflare Workers
npm run deploy

# 或指定环境
wrangler deploy --env production
```

### 2.2 生产环境配置检查

**Cloudflare Dashboard 检查**:
- [ ] Workers 服务正常运行
- [ ] D1 数据库已创建并绑定
- [ ] KV namespaces 已创建并绑定
- [ ] Durable Objects 已启用
- [ ] Secrets 已设置:
  - `AI_API_KEY` - DeepSeek/Claude API Key
  - `JWT_SECRET` - JWT 签名密钥

### 2.3 API 健康检查

```bash
# 测试后端 API
curl https://alou-edge.yuanjieliu65.workers.dev/health
curl https://alou-edge.yuanjieliu65.workers.dev/api/agent/status
```

### 2.4 性能基准测试

- [ ] 响应延迟 < 100ms
- [ ] WASM 二进制大小 ~1.17MB
- [ ] 并发支持 1000+ 请求/秒

---

## 💻 Phase 3: 桌面端构建与测试 (Day 2-3)

### 3.1 构建准备

```bash
cd alou-desktop

# 1. 安装依赖
npm install

# 2. 设置 Kubo (IPFS) 二进制
npm run setup:kubo

# 3. 准备构建资源
node scripts/prepare-kubo-for-build.js
```

### 3.2 开发环境最终测试

```bash
# 使用生产环境后端进行测试
npm run tauri:dev
```

**测试清单**:
- [ ] 应用正常启动
- [ ] 钱包连接功能正常
  - [ ] MetaMask 连接
  - [ ] WalletConnect 扫码连接
  - [ ] 本地钱包导入
- [ ] AI 对话功能正常
  - [ ] 发送消息
  - [ ] 接收流式响应
  - [ ] 会话历史加载
- [ ] 群聊功能正常
  - [ ] 创建群聊
  - [ ] 发送/接收消息
  - [ ] AI Agent 参与
- [ ] IPFS 节点启动正常
- [ ] 暗夜模式切换正常
- [ ] 无控制台错误

### 3.3 生产构建

```bash
# macOS 构建
npm run build:tauri:macos

# Windows 构建（需 Windows 环境或 CI）
npm run build:tauri:windows

# Linux 构建
npm run build:tauri:linux

# 或构建所有平台
npm run build:tauri:release
```

### 3.4 构建产物检查

**检查清单**:
- [ ] macOS: `.dmg` 文件生成
- [ ] Windows: `.exe` 安装包生成
- [ ] Linux: `.AppImage` 生成
- [ ] 应用图标正确显示
- [ ] 应用名称正确显示为 "Alou"

### 3.5 安装测试

- [ ] macOS 安装测试
  - [ ] 拖拽到 Applications
  - [ ] 首次启动正常
  - [ ] 安全提示处理（右键打开）
- [ ] Windows 安装测试
  - [ ] 安装向导正常
  - [ ] 桌面快捷方式创建
  - [ ] 启动正常

---

## 🚀 Phase 4: 正式发布 (Day 4)

### 4.1 Git 版本标记

```bash
# 1. 提交所有更改
git add .
git commit -m "release: MVP v0.1.10 - Web3 AI Agent Desktop App"

# 2. 打版本标签
git tag -a v0.1.10 -m "Alou MVP Release v0.1.10"

# 3. 推送到远程
git push origin main
git push origin v0.1.10
```

### 4.2 GitHub Release 创建

1. 访问 GitHub Releases 页面
2. 点击 "Draft a new release"
3. 选择标签 `v0.1.10`
4. 填写发布说明:

```markdown
## Alou MVP v0.1.10 🎉

### 核心功能
- 🤖 AI 智能体对话系统
- 💼 Web3 钱包集成（MetaMask、WalletConnect）
- 👥 AI Agent 群聊协作
- 🧠 自主智能体（任务队列、记忆管理）
- 📦 IPFS 本地节点集成
- 🌙 暗夜模式支持

### 安装包
- macOS: Alou_0.1.10_x64.dmg
- Windows: Alou_0.1.10_x64-setup.exe
- Linux: Alou_0.1.10_x64.AppImage

### 快速开始
1. 下载对应平台的安装包
2. 安装并启动应用
3. 连接你的 Web3 钱包
4. 开始与 AI Agent 对话

### 文档
- [快速开始指南](docs/QUICKSTART.md)
- [架构说明](docs/ARCHITECTURE.md)
```

5. 上传构建产物
6. 发布 Release

### 4.3 分发渠道

**GitHub Releases** (主要):
- 上传所有平台安装包
- 提供 checksum 校验

**可选渠道**:
- [ ] 官方网站下载页
- [ ] Product Hunt 发布
- [ ] Twitter/X 宣传
- [ ] Discord 社区公告

---

## 📣 Phase 5: 发布后的跟进 (Day 5+)

### 5.1 监控与反馈

- [ ] 监控 GitHub Issues
- [ ] 收集用户反馈
- [ ] 跟踪下载量和安装数据

### 5.2 文档更新

- [ ] 根据反馈更新 FAQ
- [ ] 添加故障排除指南
- [ ] 录制演示视频（可选）

### 5.3 迭代计划

**v0.1.11 (短期)**:
- 修复用户反馈的 bug
- 性能优化
- UI 细节改进

**v0.2.0 (中期)**:
- 移动端应用
- 更多区块链网络支持
- 智能体能力扩展

---

## 📊 发布检查清单总览

### 代码 ✅
- [ ] 所有测试通过
- [ ] 代码已提交并打标签
- [ ] 版本号一致

### 后端 ✅
- [ ] Cloudflare Workers 部署成功
- [ ] API 健康检查通过
- [ ] Secrets 已配置

### 桌面端 ✅
- [ ] 开发测试通过
- [ ] 生产构建成功
- [ ] 安装测试通过

### 发布 ✅
- [ ] GitHub Release 创建
- [ ] 构建产物上传
- [ ] 发布说明完整

### 文档 ✅
- [ ] README 更新
- [ ] CHANGELOG 更新
- [ ] 快速开始指南可用

---

## 🆘 故障排除

### 构建失败

**Rust 编译错误**:
```bash
# 清理并重新构建
cd alou-edge
cargo clean
npm run build:wasm
```

**Tauri 构建失败**:
```bash
# 更新 Tauri CLI
npm install -g @tauri-apps/cli@latest

# 清理构建缓存
rm -rf src-tauri/target
npm run build:tauri:release
```

### 部署失败

**Wrangler 认证问题**:
```bash
npx wrangler login
npx wrangler whoami
```

**Secrets 未设置**:
```bash
wrangler secret put AI_API_KEY
wrangler secret put JWT_SECRET
```

### 运行时问题

**API 连接失败**:
- 检查 `VITE_API_BASE_URL` 配置
- 验证后端服务状态
- 检查 CORS 配置

**IPFS 节点启动失败**:
- 检查 Kubo 二进制是否存在
- 验证端口未被占用
- 查看应用日志

---

## 📞 联系与支持

- **GitHub Issues**: 报告 bug 和功能请求
- **文档**: `docs/` 目录
- **邮箱**: 待补充

---

**祝发布顺利! 🚀**
