# IPFS Auto-Fix Skill

## 描述
自动检测和修复 IPFS 节点问题的技能。当检测到 IPFS 不可用时，自动尝试启动或修复 IPFS 节点。

## 功能
- 检测 IPFS 节点状态
- 自动下载 Kubo 二进制文件（如果缺失）
- 自动启动 IPFS 节点
- 处理端口冲突
- 修复配置问题
- 提供详细的诊断信息

## 使用场景
1. **应用启动时**: 自动检查并启动 IPFS
2. **群聊功能**: 当群聊需要 IPFS 时自动修复
3. **文件共享**: 当需要 IPFS 存储时自动启动
4. **后台监控**: 定期检查 IPFS 健康状态

## 参数
- `action`: 操作类型
  - `check`: 检查 IPFS 状态
  - `fix`: 自动修复 IPFS
  - `start`: 启动 IPFS 节点
  - `stop`: 停止 IPFS 节点
  - `restart`: 重启 IPFS 节点
  - `diagnose`: 诊断 IPFS 问题

## 返回值
```json
{
  "success": true,
  "status": "running",
  "nodeInfo": {
    "id": "QmXXX...",
    "version": "0.20.0",
    "addresses": [...]
  },
  "actions": ["started", "downloaded_kubo"],
  "message": "IPFS 节点已成功启动"
}
```

## 示例

### 检查状态
```typescript
await ipfsAutoFix.execute({ action: 'check' })
```

### 自动修复
```typescript
await ipfsAutoFix.execute({ action: 'fix' })
```

### 诊断问题
```typescript
await ipfsAutoFix.execute({ action: 'diagnose' })
```

## 错误处理
- 自动重试失败的操作
- 提供详细的错误信息和建议
- 降级到内存模式（如果无法修复）

## 依赖
- Tauri IPFS 命令
- Kubo 二进制文件
- 系统权限（启动进程）

## 版本
1.0.0

## 作者
Alou Team

## 许可证
MIT
