# 增量更新策略说明

## 核心设计

### 主安装包 vs 资源文件

```
┌─────────────────────────────────┐
│    主安装包（每次更新）          │
│  - 应用代码（React/Vite）       │
│  - Rust 后端                    │
│  - 配置文件                     │
│  大小：~10-50MB                 │
└─────────────────────────────────┘
           ↓ 更新时下载
┌─────────────────────────────────┐
│    Kubo 二进制（按需下载）       │
│  - 存储在应用数据目录            │
│  - 首次使用时下载               │
│  - 更新时不重新下载             │
│  大小：~50-100MB                │
└─────────────────────────────────┘
```

## 更新流程

### 场景 1：应用更新（不包含 Kubo）

1. 用户打开应用
2. Tauri 检测到新版本
3. 下载**仅包含应用代码**的更新包（小）
4. 安装更新
5. Kubo 二进制保持不变（在应用数据目录）

### 场景 2：首次使用 IPFS 功能

1. 用户点击"启动 IPFS 节点"
2. 检测到 Kubo 未安装
3. 提示用户下载 Kubo（约 50-100MB）
4. 下载并存储到应用数据目录
5. 启动 IPFS 节点

### 场景 3：Kubo 独立更新

如果需要更新 Kubo 版本：
1. 应用检测到新 Kubo 版本
2. 提示用户更新 Kubo
3. 下载新版本到临时目录
4. 替换旧版本
5. 重启 IPFS 节点

## 存储位置

### Windows
```
%APPDATA%\com.alou.desktop\
├── kubo\
│   └── ipfs.exe          # Kubo 二进制（不随应用更新）
└── ipfs\                  # IPFS 数据目录
    ├── config
    └── datastore
```

### macOS
```
~/Library/Application Support/com.alou.desktop/
├── kubo/
│   └── ipfs              # Kubo 二进制
└── ipfs/                  # IPFS 数据目录
```

### Linux
```
~/.local/share/com.alou.desktop/
├── kubo/
│   └── ipfs              # Kubo 二进制
└── ipfs/                  # IPFS 数据目录
```

## 优势

1. **更新包小**：主应用更新通常只有几 MB
2. **更新快速**：用户无需等待下载大文件
3. **按需下载**：不使用 IPFS 的用户不下载 Kubo
4. **独立管理**：Kubo 可以独立更新，不影响应用

## 实现细节

### Rust 后端

- `download_kubo_binary`：从 IPFS 官方源下载 Kubo
- `start_ipfs_node`：从应用数据目录读取 Kubo
- 自动检测 Kubo 是否存在

### 前端

- `ipfsService.downloadKubo()`：触发下载
- `ipfsService.startNode(autoDownload)`：自动下载（可选）
- `IpfsStatus` 组件：显示下载状态和选项

## 配置

### tauri.conf.json

```json
{
  "bundle": {
    "resources": [],      // 不包含 Kubo
    "externalBin": []     // 不包含 Kubo
  }
}
```

Kubo 存储在应用数据目录，不在 bundle 中。

## 更新服务器配置

更新服务器只需要提供应用更新包，不需要包含 Kubo：

```json
{
  "version": "0.2.2",
  "platforms": {
    "windows-x86_64": {
      "url": "https://releases.alou.app/app/alou-desktop_0.2.2_x64-setup.exe",
      "signature": "..."
    }
  }
}
```

更新包只包含应用代码，体积小，下载快。

