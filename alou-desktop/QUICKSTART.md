# 快速开始 - 本地开发

## 前置要求

1. **Node.js** 20.19.0+ 或 22.12.0+
2. **Rust** (最新稳定版)
3. **Tauri CLI**: `npm install -g @tauri-apps/cli`

## 启动步骤

### 方式一：使用生产环境后端（推荐，最简单）

如果后端已经部署到 Cloudflare Workers，可以直接使用：

1. **安装依赖**
```bash
cd alou-desktop
npm install
```

2. **配置后端地址**（可选）

创建 `.env` 文件：
```env
VITE_API_BASE_URL=https://alou-edge.yuanjieliu65.workers.dev
```

如果不创建 `.env`，默认会使用生产环境地址。

3. **启动桌面应用**
```bash
npm run tauri:dev
```

这会：
- 启动 Vite 开发服务器（端口 1420）
- 编译 Rust 后端
- 打开 Tauri 桌面窗口
- 前端会自动连接到配置的后端

### 方式二：本地开发（需要运行本地后端）

如果你想在本地开发和调试后端：

1. **启动本地 Workers 后端**

在另一个终端：
```bash
cd alou-edge
npm run dev
# 或
wrangler dev
```

后端会运行在 `http://127.0.0.1:8787`

2. **配置前端连接本地后端**

在 `alou-desktop` 目录创建 `.env` 文件：
```env
VITE_API_BASE_URL=http://127.0.0.1:8787
```

3. **启动桌面应用**
```bash
cd alou-desktop
npm install  # 如果还没安装
npm run tauri:dev
```

## 开发流程

### 前端开发

1. 修改 `src/` 目录下的文件
2. Vite 会自动热重载
3. Tauri 窗口会自动刷新

### 后端开发

1. 修改 `alou-edge/src/` 目录下的 Rust 文件
2. Workers 会自动重新编译
3. 前端会自动连接到新的后端

### Rust 后端开发（Tauri）

1. 修改 `src-tauri/src/` 目录下的文件
2. 需要重新编译 Rust 代码
3. Tauri 会自动重启应用

## 常见问题

### 1. 端口冲突

如果 1420 端口被占用，修改 `vite.config.js`：
```javascript
server: {
  port: 1421,  // 改为其他端口
}
```

同时修改 `tauri.conf.json`：
```json
"devPath": "http://localhost:1421"
```

### 2. 后端连接失败

检查：
- 后端是否正在运行（如果使用本地后端）
- `.env` 文件中的 `VITE_API_BASE_URL` 是否正确
- 网络连接是否正常

### 3. Rust 编译错误

确保：
- Rust 已正确安装：`rustc --version`
- Tauri CLI 已安装：`tauri --version`
- 依赖已安装：`cd src-tauri && cargo build`

### 4. IPFS/Kubo 相关

首次使用 IPFS 功能时：
- 不需要预先下载 Kubo
- 应用会在需要时自动下载
- 下载的 Kubo 存储在应用数据目录

## 调试

### 查看前端日志

在 Tauri 窗口中：
- 右键 → "检查元素" 或按 `F12`
- 打开开发者工具查看控制台

### 查看 Rust 日志

在运行 `npm run tauri:dev` 的终端中查看 Rust 输出。

### 查看后端日志

如果使用本地 Workers：
- 在运行 `wrangler dev` 的终端查看日志

## 项目结构

```
alou-desktop/          # 桌面应用
├── src/              # 前端代码
├── src-tauri/        # Rust 后端
└── package.json

alou-edge/            # Workers 后端（可选，如果本地开发）
├── src/              # Rust 代码
└── wrangler.toml
```

## 推荐工作流

**最简单的方式**（推荐新手）：
1. 只运行桌面应用：`npm run tauri:dev`
2. 使用已部署的生产环境后端
3. 专注于前端开发

**完整开发环境**（需要修改后端）：
1. 终端 1：`cd alou-edge && npm run dev`（后端）
2. 终端 2：`cd alou-desktop && npm run tauri:dev`（前端）
3. 同时开发前后端

