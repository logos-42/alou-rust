# 📦 Alou Desktop 打包指南

## 🚀 当前打包状态

**打包命令**: `npm run build:tauri:macos`

**目标平台**: macOS (DMG + APP)

**预计时间**: 5-15 分钟（首次打包可能需要更长时间）

---

## 📋 打包流程

### 完整打包流程（推荐）

```bash
cd alou-desktop
npm run build:tauri:macos
```

这个命令会自动执行以下步骤：

1. **设置 Kubo (IPFS)** - 下载和配置 IPFS Kubo 二进制文件
2. **准备构建** - 复制 Kubo 到构建目录
3. **构建前端** - Vite 构建 React 应用
4. **构建 Tauri** - Rust 编译和打包
5. **创建 DMG** - 生成 macOS 安装包

### 分步打包流程

#### 1. 仅构建前端
```bash
npm run build
```

#### 2. 仅构建 Tauri
```bash
npm run tauri build
```

#### 3. 清理后重新打包
```bash
rm -rf dist src-tauri/target/release/bundle
npm run build:tauri:macos
```

---

## 📦 打包输出位置

### macOS 打包产物

```
alou-desktop/src-tauri/target/release/bundle/
├── dmg/
│   └── Alou_0.1.11_x64.dmg          # macOS 安装包
└── macos/
    └── Alou.app/                     # macOS 应用程序
        └── Contents/
            ├── Info.plist
            ├── MacOS/
            │   └── Alou              # 可执行文件
            ├── Resources/
            └── Frameworks/
```

### 分发建议

- **DMG 文件**: 分发给最终用户（推荐）
- **APP 文件**: 直接拖拽到 Applications 文件夹

---

## 🔧 其他平台打包

### Windows
```bash
npm run build:tauri:windows
```
输出：`src-tauri/target/release/bundle/nsis/Alou_0.1.11_x64-setup.exe`

### Linux
```bash
npm run build:tauri:linux
```
输出：`src-tauri/target/release/bundle/appimage/Alou_0.1.11_x86_64.AppImage`

### 所有平台
```bash
npm run build:tauri:release:all
```

---

## ⚠️ 常见问题

### 1. Rust 编译失败

**错误**: `error: could not compile `tauri`

**解决**:
```bash
# 更新 Rust
rustup update

# 清理构建缓存
cd src-tauri
cargo clean

# 重新构建
cd ..
npm run build:tauri:macos
```

### 2. 前端构建失败

**错误**: `Error: Cannot find module ...`

**解决**:
```bash
# 重新安装依赖
rm -rf node_modules package-lock.json
npm install

# 重新构建
npm run build:tauri:macos
```

### 3. Kubo 下载失败

**错误**: `Failed to download Kubo`

**解决**:
```bash
# 手动设置 Kubo
npm run setup:kubo:unix

# 或者跳过 Kubo 直接构建
npm run build
npm run tauri build
```

### 4. 签名问题 (macOS)

**错误**: `The application cannot be opened because the developer cannot be verified`

**解决**:
```bash
# 方法 1: 系统设置
# 系统偏好设置 → 安全性与隐私 → 仍要打开

# 方法 2: 命令行移除隔离
xattr -rd com.apple.quarantine /Applications/Alou.app

# 方法 3: 使用开发证书签名
codesign --sign - --force --deep /Applications/Alou.app
```

### 5. 打包体积过大

**查看包大小**:
```bash
du -sh src-tauri/target/release/bundle/dmg/*
du -sh src-tauri/target/release/bundle/macos/*
```

**优化建议**:
- 移除不必要的资源文件
- 启用代码压缩
- 优化 Kubo 二进制文件

---

## 🎯 快速打包命令参考

| 命令 | 说明 | 输出 |
|------|------|------|
| `npm run build` | 仅构建前端 | `dist/` |
| `npm run tauri build` | 构建 Tauri 应用 | `macos/Alou.app` |
| `npm run build:tauri:macos` | 完整打包 (DMG+APP) | `dmg/*.dmg` |
| `npm run build:tauri:windows` | Windows 打包 | `nsis/*.exe` |
| `npm run build:tauri:linux` | Linux 打包 | `appimage/*.AppImage` |

---

## 📊 构建时间参考

| 步骤 | 首次构建 | 后续构建 |
|------|----------|----------|
| Kubo 设置 | 2-5 分钟 | 30 秒 |
| 前端构建 | 1-2 分钟 | 30 秒 |
| Rust 编译 | 5-10 分钟 | 1-3 分钟 |
| 打包 DMG | 1 分钟 | 1 分钟 |
| **总计** | **10-18 分钟** | **3-5 分钟** |

---

## 🔍 构建日志查看

### 实时查看构建日志
```bash
# 前端构建日志
npm run build -- --debug

# Tauri 构建日志
npm run tauri build -- --verbose
```

### 查看构建产物信息
```bash
# 查看生成的文件
ls -lh src-tauri/target/release/bundle/dmg/
ls -lh src-tauri/target/release/bundle/macos/

# 查看 APP 信息
mdls src-tauri/target/release/bundle/macos/Alou.app
```

---

## 📝 发布前检查清单

- [ ] 前端构建成功（无错误）
- [ ] Tauri 编译成功（无警告）
- [ ] DMG 文件生成
- [ ] APP 可以正常打开
- [ ] 基本功能测试通过
- [ ] 版本号正确
- [ ] 应用图标显示正常
- [ ] 无控制台错误

---

## 🎨 图标和资源

### 图标生成
```bash
# 重新生成图标
npm run icons:regenerate
```

### 图标要求
- PNG: 1024x1024 (推荐)
- ICNS: macOS 专用
- ICO: Windows 专用

---

## 📞 需要帮助？

如果打包过程中遇到问题：

1. 查看完整的错误日志
2. 检查系统要求（Rust, Node.js 版本）
3. 清理缓存后重试
4. 查看 Tauri 官方文档：https://tauri.app/

---

**最后更新**: 2026 年 3 月 6 日  
**版本**: 0.1.11  
**平台**: macOS
