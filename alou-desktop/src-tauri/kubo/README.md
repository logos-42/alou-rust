# Kubo (IPFS) 二进制文件

此目录应包含对应平台的 Kubo (IPFS) 二进制文件。

**当前版本：v0.39.0**

## 下载 Kubo

### Windows
```bash
# 下载 Windows 版本
# https://dist.ipfs.tech/kubo/v0.39.0/kubo_v0.39.0_windows-amd64.zip
# 解压后将 ipfs.exe 放到此目录
```

### macOS
```bash
# 下载 macOS 版本 (amd64)
# https://dist.ipfs.tech/kubo/v0.39.0/kubo_v0.39.0_darwin-amd64.tar.gz
# 解压后将 ipfs 二进制放到此目录

# macOS (arm64/Apple Silicon)
# https://dist.ipfs.tech/kubo/v0.39.0/kubo_v0.39.0_darwin-arm64.tar.gz
```

### Linux
```bash
# 下载 Linux 版本
# https://dist.ipfs.tech/kubo/v0.39.0/kubo_v0.39.0_linux-amd64.tar.gz
# 解压后将 ipfs 二进制放到此目录
```

## 快速下载

使用项目提供的脚本自动下载：

```bash
# 跨平台脚本（Node.js）
npm run setup:kubo

# Windows PowerShell
.\scripts\setup-kubo.ps1

# macOS/Linux
./scripts/setup-kubo.sh
```

## 目录结构

```
kubo/
├── ipfs.exe     (Windows v0.39.0)
├── ipfs.darwin  (macOS v0.39.0)
├── ipfs.linux   (Linux v0.39.0)
├── ipfs         (通用文件，打包时由脚本自动准备)
└── README.md
```

## 多平台打包

项目支持为 Windows、macOS 和 Linux 打包，每个平台的二进制文件独立存放：

### Windows
```bash
npm run build:tauri:windows
```
打包前会自动：
1. 下载 Windows 版本的 Kubo (`ipfs.exe`)
2. 准备二进制文件用于打包
3. 构建 NSIS 和 MSI 安装包

### macOS
```bash
npm run build:tauri:macos
```
打包前会自动：
1. 下载 macOS 版本的 Kubo (`ipfs.darwin`)
2. 复制为 `ipfs` 供 Rust 代码使用
3. 构建 DMG 安装包

### Linux
```bash
npm run build:tauri:linux
```
打包前会自动：
1. 下载 Linux 版本的 Kubo (`ipfs.linux`)
2. 复制为 `ipfs` 供 Rust 代码使用
3. 构建 AppImage 安装包

### 下载所有平台版本
```bash
npm run setup:kubo:all
```
这会下载所有平台的二进制文件（Windows、macOS、Linux），方便在多平台机器上打包。

## 注意事项

1. **版本管理**：所有平台使用统一的 Kubo 版本（当前：v0.39.0）
2. **打包流程**：打包脚本会自动调用 `prepare-kubo-for-build.js`，为当前平台准备正确的二进制文件
3. **文件权限**：macOS/Linux 二进制文件会自动设置执行权限
4. **资源打包**：Tauri 会将 `kubo` 目录作为资源打包，每个平台的安装包只包含对应平台的二进制文件
5. **运行时**：首次运行时，IPFS 会在应用数据目录初始化

