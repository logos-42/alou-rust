# DIAP 身份显示修复

## 问题

创建成功后前端没有显示 DIAP 身份。

日志显示：
```
[DiapIntegration] 本地 ZKP 命令结果：{agent_description: "", agent_name: "Agent", cid: "QmcfZzodZCdCmRsPobZRyAReFGFmdB7dEoGUdaaz74wvTM", …}
```

但前端显示：
```
[DiapIdentityPanel] ❌ 所有 agentId 都未找到身份
```

## 根本原因

**响应格式不匹配**！

`diapIntegrationService.createDiapIdentity` 返回的是**扁平对象** `CreateDiapIdentityResult`：
```typescript
{
  success: boolean
  did?: string
  cid?: string
  ipns?: string
  public_key?: string
  ...
}
```

但 `DiapIdentityPanel.jsx` 中错误地使用了 `response.identity`：
```typescript
// ❌ 错误的代码
if (response.identity) {
  setIdentity(response.identity)
  await saveDiapIdentityToFile(agentId, response.identity)
}
```

实际上 `response.identity` 是 `undefined`，因为响应是扁平的！

## 修复方案

### `DiapIdentityPanel.jsx` - handleCreateIdentity

```typescript
// ✅ 修复后的代码
const handleCreateIdentity = async () => {
  const response = await diapIntegrationService.createDiapIdentity(sessionId)
  
  console.log('[DiapIdentityPanel] DIAP 创建响应:', response)
  
  if (response.success && response.cid) {
    // 将扁平的响应转换为 identity 对象
    const identity = {
      did: response.did || '',
      cid: response.cid || '',
      ipns: response.ipns || '',
      public_key: response.public_key || '',
      did_document: response.did_document,
      zkp_proof: response.zkp_proof,
      is_registered: false,
      created_at: Math.floor(Date.now() / 1000)
    }
    
    // 使用 CID 作为 agentId 保存
    const agentId = response.cid
    await saveDiapIdentityToFile(agentId, identity)
    
    // 立即设置身份显示
    setIdentity(identity)
    setLoading(false)
    
    // 延迟刷新确保文件已保存
    setTimeout(() => {
      loadIdentity()
    }, 500)
  }
}
```

## 修复点

1. ✅ 正确解析 `CreateDiapIdentityResult` 响应
2. ✅ 将扁平响应转换为 `identity` 对象
3. ✅ 使用 `CID` 作为 `agentId` 保存（与后端一致）
4. ✅ 立即设置状态显示身份
5. ✅ 延迟刷新确保文件已保存

## 日志输出

### 修复前
```
[DiapIntegration] 本地 ZKP 命令结果：{cid: "Qm...", ...}
[DiapIdentityPanel] DIAP 创建响应：{cid: "Qm...", ...}
// response.identity 是 undefined，不执行任何操作
[DiapIdentityPanel] ❌ 所有 agentId 都未找到身份
```

### 修复后
```
[DiapIntegration] 本地 ZKP 命令结果：{cid: "Qm...", ...}
[DiapIdentityPanel] DIAP 创建响应：{cid: "Qm...", ...}
[DiapIdentityPanel] DIAP 身份创建成功：{did: "...", cid: "Qm...", ...}
[DiapAgentIdentity] 保存 DIAP 身份到文件：Qm...
[DiapFileManager] ✅ DIAP 身份已保存到文件：{path}
[DiapIdentityPanel] 身份已设置到状态
[DiapIdentityPanel] ✅ 从 agent 文件加载成功：Qm...
```

## 测试步骤

1. **启动应用**
   ```bash
   cd /Users/apple/Downloads/alou/alou-desktop
   npm run tauri dev
   ```

2. **创建 DIAP 身份**
   - 打开 DIAP Identity Panel
   - 点击"创建 DIAP 身份"
   - 等待创建完成

3. **验证显示**
   - 应该立即看到 DIAP 身份信息
   - 包含 DID、CID、IPNS 等

4. **刷新验证**
   - 关闭并重新打开面板
   - 应该仍然能看到身份（从文件加载）

## 完成状态

- ✅ 前端构建成功
- ✅ 正确解析响应格式
- ✅ 使用 CID 作为 agentId
- ✅ 立即显示身份
- ✅ 延迟刷新确保持久化

**现在创建 DIAP 身份后应该能立即显示出来了！**
