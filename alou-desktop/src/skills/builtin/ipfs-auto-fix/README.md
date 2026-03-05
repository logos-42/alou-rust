# IPFS Auto-Fix Skill 使用指南

## 概述
IPFS Auto-Fix Skill 是一个自动检测和修复 IPFS 节点问题的技能。当检测到 IPFS 不可用时，AI 可以自动调用这个 skill 来修复问题。

## 功能特性
- ✅ 自动检测 IPFS 节点状态
- ✅ 自动下载 Kubo 二进制文件（如果缺失）
- ✅ 自动启动 IPFS 节点
- ✅ 处理端口冲突
- ✅ 提供详细的诊断信息
- ✅ 智能重试机制

## 使用方法

### 1. 在 Skills Manager 中使用
打开 Skills Manager，找到 "IPFS 自动修复" skill，选择操作：
- **检查状态**: 查看 IPFS 节点当前状态
- **自动修复**: 自动检测并修复所有问题
- **启动节点**: 手动启动 IPFS 节点
- **停止节点**: 停止 IPFS 节点
- **重启节点**: 重启 IPFS 节点
- **诊断问题**: 获取详细的诊断报告

### 2. 让 AI 自动调用
当群聊或其他功能需要 IPFS 时，AI 会自动检测并调用这个 skill：

```
用户: 群聊无法使用，提示 IPFS 不可用
AI: 我来帮你修复 IPFS...
    [调用 ipfs-auto-fix.fix]
    ✅ IPFS 节点已成功启动
    现在可以使用群聊功能了！
```

### 3. 通过代码调用
```typescript
import ipfsAutoFixSkill from '@/skills/builtin/ipfs-auto-fix/skill';

// 检查状态
const status = await ipfsAutoFixSkill.execute({ action: 'check' });
console.log('IPFS 状态:', status);

// 自动修复
const fixResult = await ipfsAutoFixSkill.execute({ 
  action: 'fix',
  autoDownload: true,
  maxRetries: 3
});
console.log('修复结果:', fixResult);

// 诊断问题
const diagnosis = await ipfsAutoFixSkill.execute({ action: 'diagnose' });
console.log('诊断报告:', diagnosis);
```

## API 参考

### check - 检查状态
检查 IPFS 节点当前状态。

**参数**: 无

**返回**:
```typescript
{
  success: true,
  status: 'running',
  nodeInfo: {
    id: 'QmXXX...',
    version: '0.20.0',
    addresses: [...],
    peerId: 'QmXXX...'
  },
  message: 'IPFS 节点运行正常'
}
```

### fix - 自动修复
自动检测并修复 IPFS 问题。

**参数**:
- `autoDownload` (boolean, 默认: true): 是否自动下载 Kubo
- `maxRetries` (number, 默认: 3): 最大重试次数

**返回**:
```typescript
{
  success: true,
  status: 'running',
  nodeInfo: {...},
  actions: ['downloaded_kubo', 'started'],
  message: 'IPFS 节点已成功启动'
}
```

### start - 启动节点
启动 IPFS 节点。

**参数**:
- `autoDownload` (boolean, 默认: true): 是否自动下载 Kubo

**返回**:
```typescript
{
  success: true,
  status: 'running',
  nodeInfo: {...},
  actions: ['started'],
  message: 'IPFS 节点已启动'
}
```

### stop - 停止节点
停止 IPFS 节点。

**参数**: 无

**返回**:
```typescript
{
  success: true,
  status: 'stopped',
  actions: ['stopped'],
  message: 'IPFS 节点已停止'
}
```

### restart - 重启节点
重启 IPFS 节点。

**参数**:
- `autoDownload` (boolean, 默认: true): 是否自动下载 Kubo

**返回**:
```typescript
{
  success: true,
  status: 'running',
  nodeInfo: {...},
  actions: ['stopped', 'started'],
  message: 'IPFS 节点已重启'
}
```

### diagnose - 诊断问题
诊断 IPFS 问题并提供修复建议。

**参数**: 无

**返回**:
```typescript
{
  success: true,
  diagnosis: {
    kuboInstalled: true,
    nodeRunning: false,
    apiAccessible: false,
    portConflict: false,
    recommendations: [
      'IPFS 节点未运行，尝试启动: 运行自动修复或手动启动'
    ]
  },
  message: '诊断完成'
}
```

## 常见问题

### Q: Kubo 下载失败怎么办？
A: 可以手动安装 Kubo：
1. 访问 https://docs.ipfs.tech/install/command-line/
2. 下载适合你系统的版本
3. 安装后重启应用

### Q: 端口 5001 被占用怎么办？
A: 有两种解决方案：
1. 关闭其他占用 5001 端口的 IPFS 实例
2. 使用现有的 IPFS 实例（skill 会自动检测）

### Q: 自动修复失败怎么办？
A: 运行诊断操作获取详细信息：
```typescript
const diagnosis = await ipfsAutoFixSkill.execute({ action: 'diagnose' });
console.log(diagnosis.diagnosis.recommendations);
```

### Q: 如何让 AI 自动调用这个 skill？
A: AI 会在以下情况自动调用：
1. 用户提到 IPFS 不可用
2. 群聊功能报错
3. 文件共享失败
4. 任何需要 IPFS 的功能出错

## 集成示例

### 在应用启动时自动修复
```typescript
// App.tsx
import ipfsAutoFixSkill from '@/skills/builtin/ipfs-auto-fix/skill';

useEffect(() => {
  const ensureIpfs = async () => {
    const result = await ipfsAutoFixSkill.execute({ 
      action: 'fix',
      autoDownload: true,
      maxRetries: 3
    });
    
    if (result.success) {
      console.log('✅ IPFS 已就绪');
    } else {
      console.warn('⚠️ IPFS 启动失败，将使用降级模式');
    }
  };
  
  ensureIpfs();
}, []);
```

### 在群聊初始化时检查
```typescript
// useLocalIpfsGroupChat.ts
import ipfsAutoFixSkill from '@/skills/builtin/ipfs-auto-fix/skill';

const initialize = async () => {
  // 检查 IPFS 状态
  const status = await ipfsAutoFixSkill.execute({ action: 'check' });
  
  if (!status.success) {
    // 尝试自动修复
    const fixResult = await ipfsAutoFixSkill.execute({ action: 'fix' });
    
    if (fixResult.success) {
      console.log('✅ IPFS 已自动修复');
    } else {
      console.log('⚠️ 降级到内存模式');
    }
  }
};
```

## 版本历史
- v1.0.0 (2026-03-05): 初始版本
  - 支持自动检测和修复
  - 支持 Kubo 自动下载
  - 支持端口冲突处理
  - 支持详细诊断

## 许可证
MIT

## 作者
Alou Team
