# DIAP TSX 文件修复

## 问题

TSX 文件中保存时使用的 agentId 与加载时不匹配。

## 修复内容

### `DiapIdentityPanel.tsx` - handleCreateIdentity

**修复前**：
```typescript
// 使用 selectedAgent.id 或 sessionId
const agentId = selectedAgent?.id || sessionId
await saveDiapIdentityToFile(agentId, newIdentity)
```

**修复后**：
```typescript
// 使用 CID（与后端一致）
const agentId = newIdentity.cid || sessionId
await saveDiapIdentityToFile(agentId, newIdentity)
```

## 为什么使用 CID

1. **后端使用 CID**：后端 `diap.rs` 中使用 `identity.cid` 作为 agent_id
2. **加载逻辑匹配**：前端 `loadIdentity` 尝试从 `selectedAgent.cid` 加载
3. **一致性**：保存和加载使用相同的 ID 格式

## 完整流程

### 保存
```
创建成功 → response.cid = "Qm..."
    ↓
agentId = response.cid
    ↓
保存到：diap/agents/Qm.../diap.json
```

### 加载
```
possibleAgentIds = [
  selectedAgent.id,
  sessionId,
  selectedAgent.ipns,
  selectedAgent.cid  ← 匹配！
]
    ↓
从 diap/agents/Qm.../diap.json 加载
    ↓
✅ 成功
```

## 验证

启动应用后：
1. 创建 DIAP 身份
2. 检查文件：`ls ~/Library/Application\ Support/alou-desktop/diap/agents/`
3. 应该看到以 CID 命名的目录
4. 重新打开面板，应该能正确加载

## 完成状态

- ✅ TSX 构建成功
- ✅ 使用 CID 作为 agentId
- ✅ 保存和加载一致
- ✅ 后端 APP_HANDLE 已初始化

**系统现在应该完全正常工作了！**
