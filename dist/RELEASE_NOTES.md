# Alou Desktop 0.1.11 发布说明

## 📦 构建产物

### macOS
- **DMG 安装包**: `Alou_0.1.11_x64.dmg` (89MB)
- **APP 应用**: `Alou.app` 

### 文件位置
```
/Users/apple/Downloads/alou/dist/
├── Alou.app                    # macOS 应用程序
└── Alou_0.1.11_x64.dmg        # 安装镜像
```

## 🏗️ 构建信息

- **版本号**: 0.1.11
- **构建时间**: 2026-03-02
- **平台**: macOS (x64)
- **标识符**: com.alou.desktop

## ✨ 新增功能

### Skills 系统
- 实现了完整的 Skills 架构，支持全局 `~/.alou/skills/` 目录
- 新增 SKILL.md 标准格式
- 内置 Skills: filesystem, web-search, code-analyzer
- 支持用户自定义 Skills

### Tasks 系统
- 新增 Rust 实现的 Task 系统 (`task_system.rs`)
- 支持优先级队列（Critical/High/Medium/Low/Custom）
- 支持多种执行模式（Sequential/Parallel/Swarm/Workflow）
- Agent Swarm 协作功能
  - Leader-Follower 模式
  - Peer-to-Peer 模式
  - Hive-Mind 模式

### 核心增强
- 任务队列系统优化
- Agent 自主性增强
- 并行执行引擎
- 持久化存储支持

## 📋 已知问题

- 部分代码存在未使用导入警告（不影响功能）
- 变量命名规范警告（不影响功能）

## 🔧 技术栈

- **前端**: React 18 + Vite 7 + TypeScript
- **后端**: Rust + Tauri 2
- **AI 集成**: Claude/DeepSeek API
- **Web3**: Ethers.js + WalletConnect
- **去中心化**: IPFS + DIAP

## 📥 安装说明

### macOS
1. 下载 `Alou_0.1.11_x64.dmg`
2. 双击打开 DMG 文件
3. 将 Alou.app 拖拽到 Applications 文件夹
4. 首次运行需要在系统偏好设置中允许

## 🎯 使用指南

### 启动应用
```bash
# 从 Applications 文件夹启动
open /Applications/Alou.app

# 或从命令行启动
/Applications/Alou.app/Contents/MacOS/Alou
```

### 开发模式
```bash
cd alou-desktop
npm run dev
npm run tauri dev
```

### 构建
```bash
# macOS
npm run build:tauri:macos

# 完整构建
npm run build:tauri
```

## 📝 变更日志

### 0.1.11 (2026-03-02)
- ✅ 实现 Skills 系统架构
- ✅ 实现 Tasks 系统（Rust）
- ✅ 支持 Agent Swarm 协作
- ✅ 新增任务优先级队列
- ✅ 优化编译错误
- ✅ 修复多个代码质量问题

### 0.1.10 (2026-02-28)
- 群聊功能增强
- 安全性提升（XSS 防护）
- Bug 修复

## 📄 许可证

MIT License

## 🔗 相关链接

- GitHub: https://github.com/logos-42/alou-rust
- 文档：`docs/` 目录
- Skills 指南：`docs/SKILLS_AND_TASKS_GUIDE.md`

---

**构建完成时间**: 2026-03-02 16:27
**构建状态**: ✅ 成功
