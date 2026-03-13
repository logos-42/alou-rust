# DIAP 闪退修复

## 问题

创建 DIAP 身份成功后应用闪退。

## 根本原因

**无限循环导致崩溃**！

### 循环路径

```
handleCreateIdentity 创建成功
    ↓
setTimeout(() => loadIdentity(), 500)
    ↓
loadIdentity() 执行
    ↓
useEffect 依赖 loadIdentity
    ↓
loadIdentity 变化触发 useEffect
    ↓
useEffect 调用 loadIdentity()
    ↓
无限循环！→ 内存溢出 → 闪退
```

### 问题代码

```typescript
// ❌ 问题 1: handleCreateIdentity 中调用 loadIdentity
setTimeout(() => {
  loadIdentity()  // 触发重新加载
}, 500)

// ❌ 问题 2: useEffect 依赖 loadIdentity
useEffect(() => {
  if (sessionId) {
    loadIdentity()
  }
}, [sessionId, loadIdentity])  // loadIdentity 在依赖中！
```

## 修复方案

### 1. 移除 handleCreateIdentity 中的 loadIdentity 调用

```typescript
// ✅ 修复后
const handleCreateIdentity = async () => {
  // ... 创建逻辑
  
  // 立即设置身份显示
  setIdentity(identity)
  setLoading(false)
  
  // ❌ 删除：不再调用 loadIdentity()
  // setTimeout(() => loadIdentity(), 500)
  
  setToastMessage(t('agent.diap.createSuccess'))
}
```

### 2. 修复 useEffect 依赖

```typescript
// ✅ 修复后
useEffect(() => {
  if (sessionId) {
    loadIdentity()
  }
  // eslint-disable-next-line react-hooks/exhaustive-deps
}, [sessionId])  // 只依赖 sessionId
```

### 3. 优化事件监听器

```typescript
// ✅ 修复后
useEffect(() => {
  const handleDiapIdentityCreated = (event) => {
    const { sessionId: createdSessionId, identity: eventIdentity } = event.detail
    
    if (createdSessionId === sessionId) {
      // 直接使用事件中的 identity，避免重新加载
      if (eventIdentity) {
        setIdentity(eventIdentity)
        setLoading(false)
      } else {
        loadIdentity()
      }
    }
  }

  window.addEventListener('diap-identity-created', handleDiapIdentityCreated)

  return () => {
    window.removeEventListener('diap-identity-created', handleDiapIdentityCreated)
  }
  // eslint-disable-next-line react-hooks/exhaustive-deps
}, [sessionId])  // 只依赖 sessionId
```

## 修复点总结

1. ✅ 移除 `handleCreateIdentity` 中的 `loadIdentity()` 调用
2. ✅ `useEffect` 只依赖 `sessionId`，不依赖 `loadIdentity`
3. ✅ 事件监听器优先使用事件中的 `identity`，避免重新加载
4. ✅ 添加 `eslint-disable` 注释避免警告

## 日志输出

### 修复前（无限循环）
```
[DiapIdentityPanel] DIAP 身份创建成功
[DiapIdentityPanel] 强制刷新 DIAP 身份显示
[DiapIdentityPanel] 开始加载 DIAP 身份
[DiapIdentityPanel] 从 agent 文件加载成功
[DiapIdentityPanel] 开始加载 DIAP 身份  ← 无限循环开始
[DiapIdentityPanel] 从 agent 文件加载成功
[DiapIdentityPanel] 开始加载 DIAP 身份
...
应用闪退
```

### 修复后（正常）
```
[DiapIdentityPanel] DIAP 创建响应：{cid: "Qm...", ...}
[DiapIdentityPanel] DIAP 身份创建成功：{did: "...", cid: "Qm...", ...}
[DiapAgentIdentity] 保存 DIAP 身份到文件：Qm...
[DiapFileManager] ✅ DIAP 身份已保存到文件
[DiapIdentityPanel] 身份已设置到状态
[DiapIdentityPanel] ✅ 显示成功，无闪退
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

3. **验证**
   - ✅ 应该看到 DIAP 身份信息
   - ✅ 不应该闪退
   - ✅ 可以正常交互

4. **刷新验证**
   - 关闭并重新打开面板
   - 应该仍然能看到身份

## 完成状态

- ✅ 前端构建成功
- ✅ 移除无限循环
- ✅ 优化 useEffect 依赖
- ✅ 优化事件处理
- ✅ 不再闪退

**现在创建 DIAP 身份后应该不会再闪退了！**
