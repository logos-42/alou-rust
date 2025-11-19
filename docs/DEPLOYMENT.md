# 部署和更新指南

## 增量更新策略

### Kubo 二进制分离

为了减小更新包大小，Kubo (IPFS) 二进制**不包含在主安装包中**：

- **主安装包**：只包含应用代码，体积小，更新快速
- **Kubo 二进制**：首次使用时按需下载，存储在应用数据目录
- **更新时**：只下载应用更新，不重新下载 Kubo

### 优势

1. **更新包小**：主应用更新包通常只有几 MB
2. **按需下载**：只有使用 IPFS 功能的用户才下载 Kubo
3. **独立更新**：Kubo 可以独立更新，不影响应用更新流程

## 自动更新配置

### 1. 生成更新密钥对

```bash
cd alou-desktop
tauri signer generate -w ~/.tauri/myapp.key
```

这将生成：
- 私钥：`~/.tauri/myapp.key`（保密）
- 公钥：需要添加到 `tauri.conf.json` 的 `plugins.updater.pubkey`

### 2. 配置更新服务器

更新服务器需要提供以下端点：
```
https://releases.alou.app/{{target}}/{{current_version}}
```

响应格式：
```json
{
  "version": "0.2.2",
  "notes": "更新说明",
  "pub_date": "2024-01-01T00:00:00Z",
  "platforms": {
    "windows-x86_64": {
      "signature": "签名",
      "url": "https://releases.alou.app/.../alou-desktop_0.2.2_x64-setup.exe"
    }
  }
}
```

### 3. 构建和发布

```bash
# 构建发布版本
npm run build:tauri:release

# 签名应用（Windows）
# 使用 signtool 或其他签名工具

# 上传到更新服务器
# 将构建产物上传到你的 CDN/服务器
```

## 压缩优化

### Cargo 优化配置

已在 `Cargo.toml` 中配置：
- `opt-level = "z"` - 优化大小
- `lto = true` - 链接时优化
- `strip = true` - 移除符号

### 构建压缩版本

```bash
# 使用压缩构建
TAURI_COMPRESSION=1 npm run build:tauri
```

## Kubo (IPFS) 集成

### 设置 Kubo 二进制

**Windows:**
```powershell
npm run setup:kubo:win
```

**macOS/Linux:**
```bash
npm run setup:kubo:unix
```

**跨平台 (Node.js):**
```bash
npm run setup:kubo
```

### 目录结构

```
src-tauri/
├── kubo/
│   ├── ipfs.exe (Windows)
│   └── ipfs (macOS/Linux)
└── ...
```

### 在应用中使用

```javascript
import ipfsService from '@/services/ipfsService'

// 启动 IPFS 节点
await ipfsService.startNode()

// 获取节点信息
const info = await ipfsService.getNodeInfo()

// 停止节点
await ipfsService.stopNode()
```

## CI/CD 自动发布

使用 GitHub Actions 自动构建和发布：

1. 创建 Git tag: `git tag v0.2.2`
2. 推送 tag: `git push origin v0.2.2`
3. GitHub Actions 会自动：
   - 构建所有平台的应用
   - 创建 GitHub Release
   - 上传构建产物
   - 生成更新清单

## 更新流程

1. **开发** → 提交代码
2. **测试** → 本地测试
3. **打标签** → `git tag v0.2.2`
4. **推送** → `git push origin v0.2.2`
5. **自动构建** → GitHub Actions
6. **用户更新** → 应用内自动检测并提示更新

## 注意事项

1. **Kubo 二进制大小**：约 50-100MB，会增加应用体积
2. **首次启动**：IPFS 初始化需要时间
3. **存储空间**：IPFS 数据目录会占用空间
4. **网络要求**：IPFS 节点需要网络连接

## 故障排除

### IPFS 节点无法启动

1. 检查二进制文件是否存在
2. 检查文件权限（Unix）
3. 查看应用日志

### 更新失败

1. 检查网络连接
2. 验证更新服务器配置
3. 检查签名密钥

