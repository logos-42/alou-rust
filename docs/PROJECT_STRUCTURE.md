# 项目结构说明

## 项目关系概览

本项目包含两个前端应用：

### 1. `frontend/` - 网页版前端
- **用途**: 部署到 Cloudflare Pages，供用户在浏览器中访问
- **特点**: 
  - 纯 React + Vite 应用
  - 依赖浏览器环境（window.ethereum, 浏览器插件等）
  - 部署为静态网站
- **部署方式**: 
  - 构建命令: `npm run build`
  - 部署命令: `npm run deploy` 或使用 `deploy-frontend.ps1`
  - 部署目标: Cloudflare Pages (`https://alou-frontend.pages.dev`)

### 2. `alou-desktop/` - 桌面版应用
- **用途**: Tauri 桌面应用，打包成 exe/dmg/AppImage 等桌面程序
- **特点**:
  - React + Vite + Tauri
  - 支持桌面特定功能：
    - WalletConnect (移动钱包扫码)
    - 本地钱包管理（私钥/助记词）
    - IPFS 本地节点
    - 系统级功能（打开浏览器等）
  - 可以离线运行（部分功能）
- **部署方式**:
  - 构建命令: `npm run build:tauri:release`
  - 输出: 桌面应用程序文件（exe/dmg/AppImage）
  - 可以配置自动更新服务

## 代码关系

### 共享代码
两个项目共享大部分前端代码：
- **组件** (`components/`): 大部分组件相同
  - `AgentChat.jsx`, `ChatHeader.jsx`, `AgentCanvas.jsx` 等
- **服务** (`services/`): API 服务相同
  - `api.js`, `agentService.js`, `authService.js` 等
- **状态管理** (`stores/`): 相同的状态管理逻辑
- **Hooks** (`hooks/`): 共享的业务逻辑 Hooks
- **路由** (`routes/`): 相同的路由配置

### 差异代码
- **桌面版独有**:
  - `services/desktopWalletService.js` - 桌面钱包服务
  - `services/ipfsService.js` - IPFS 服务
  - `components/wallet/WalletConnectQR.jsx` - WalletConnect 二维码组件
  - `components/wallet/LocalWalletForm.jsx` - 本地钱包表单
  - `components/ErrorBoundary.jsx` - 错误边界
  - `components/IpfsStatus.jsx` - IPFS 状态组件
  - `src-tauri/` - Rust 后端代码

- **网页版独有**:
  - 目前没有独有代码，但将来可能有网页特定优化

## 冲突分析

### ❌ 没有冲突
两个项目是**独立的应用**，各自有自己的：
- `package.json` 和依赖
- 构建配置 (`vite.config.js`)
- 输出目录 (`dist/`)
- 部署流程

### ⚠️ 潜在问题

1. **代码重复**:
   - 相同的组件需要维护两份
   - 修改时需要同步更新两个项目

2. **功能差异**:
   - 桌面版有额外的钱包连接方式
   - `LoginView.jsx` 在桌面版有更多功能

3. **依赖差异**:
   - 桌面版有额外的依赖（`@tauri-apps/api`, `ethers`, `qrcode.react` 等）
   - 网页版依赖更轻量

## 部署说明

### 网页版部署 (`frontend/`)
```bash
cd frontend
npm run build
npm run deploy  # 或使用 deploy-frontend.ps1
```

部署到: **Cloudflare Pages**
- URL: `https://alou-frontend.pages.dev`（示例）
- 用户通过浏览器访问
- 自动与 MetaMask 等浏览器插件交互

### 桌面版部署 (`alou-desktop/`)
```bash
cd alou-desktop
npm run build:tauri:release
```

输出: 桌面应用程序文件
- Windows: `.exe` 安装包
- macOS: `.dmg` 镜像
- Linux: `.AppImage` 或 `.deb` 包

可以配置自动更新服务器，用户可以在应用内更新。

## 代码同步建议

### 选项 1: 保持独立（当前方案）
- ✅ 简单直接
- ✅ 各项目可以独立优化
- ❌ 代码重复，需要手动同步

### 选项 2: 提取共享代码包
创建一个共享包，包含通用组件和服务：
```
shared/
├── components/     # 共享组件
├── services/       # 共享服务
├── hooks/          # 共享 Hooks
└── stores/         # 共享状态管理
```

然后两个项目都依赖这个共享包：
- `frontend/package.json`: `"@alou/shared": "workspace:*"`
- `alou-desktop/package.json`: `"@alou/shared": "workspace:*"`

### 选项 3: Monorepo 结构（推荐）
使用 pnpm/npm workspaces 或 Turborepo：
```
aloupay/
├── packages/
│   ├── shared/        # 共享代码
│   ├── frontend/      # 网页版
│   └── desktop/       # 桌面版
├── pnpm-workspace.yaml
└── package.json
```

## 当前最佳实践

1. **保持两个项目独立部署**
   - 网页版继续部署到 Cloudflare Pages
   - 桌面版打包成应用程序

2. **手动同步重要更改**
   - 当修改共享组件时，需要同步到两个项目
   - 优先在桌面版开发（功能更全），然后复制到网页版

3. **环境检测**
   - 使用 `desktopWalletService.isDesktop()` 检测环境
   - 根据环境显示不同的 UI 和功能

4. **API 一致性**
   - 两个项目使用相同的 API 后端
   - 确保 API 兼容性

## 总结

- ✅ **没有冲突**: 两个项目可以并行开发和部署
- ✅ **独立部署**: 网页版和桌面版各自独立部署，互不影响
- ⚠️ **代码重复**: 需要手动保持代码同步
- 💡 **建议**: 如果项目规模扩大，考虑使用 Monorepo 结构共享代码

