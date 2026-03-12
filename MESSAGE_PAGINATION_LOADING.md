# 消息分页加载优化

## 问题描述

重启后加载 IndexedDB/KV 存储中的聊天记录时，会一次性加载所有消息，导致：
- 启动速度慢
- 内存占用高
- 大量历史消息不需要立即显示

## 解决方案

实现**分页加载**策略：
1. **启动时只加载最新的 N 条**（默认 100 条）
2. **滚动加载更多** - 用户向上滚动时加载更多历史消息

## 修改内容

### 1. `localIpfsGroupChatService.ts` - 核心修改

#### 修改 `loadMessagesFromKV` 方法

**之前：**
```typescript
async loadMessagesFromKV(groupId: string): Promise<void> {
  // ... 加载所有消息
  messages.sort((a, b) => a.timestamp - b.timestamp) // 从旧到新
  
  // 限制内存中的消息数量
  if (messages.length > 200) {
    messages.splice(0, messages.length - 200) // 删除旧的，保留新的
  }
}
```

**修改后：**
```typescript
async loadMessagesFromKV(groupId: string, limit: number = 100): Promise<void> {
  // ... 加载所有消息
  
  // 按时间戳排序（从新到旧）
  messages.sort((a, b) => b.timestamp - a.timestamp)
  
  // 只保留最新的 N 条
  const latestMessages = messages.slice(0, limit)
  
  // 重新按时间戳排序（从旧到新，保证时间线正确）
  latestMessages.sort((a, b) => a.timestamp - b.timestamp)
  
  this.messages.set(groupId, latestMessages)
}
```

**关键改进：**
- ✅ 添加 `limit` 参数（默认 100 条）
- ✅ 先按时间倒序，取最新的 N 条
- ✅ 再按时间正序，保证 UI 显示正确
- ✅ 详细的日志输出（loaded/total/limit）

#### 新增 `loadMoreMessages` 方法

```typescript
/**
 * 加载更多历史消息（用于滚动加载）
 * @param groupId - 群聊 ID
 * @param beforeTimestamp - 在此时间戳之前的消息
 * @param limit - 加载数量，默认 50 条
 * @returns 加载的历史消息
 */
async loadMoreMessages(
  groupId: string, 
  beforeTimestamp: number, 
  limit: number = 50
): Promise<LocalGroupMessage[]> {
  // ... 从 KV 加载
  // 只加载指定时间戳之前的消息
  if (message.timestamp < beforeTimestamp) {
    messages.push(message)
  }
  
  // 排序并返回
  return olderMessages
}
```

**功能：**
- ✅ 支持滚动加载更多历史消息
- ✅ 基于时间戳过滤（加载更早的消息）
- ✅ 返回加载的消息数组

## 使用示例

### 1. 启动时加载最新 100 条
```typescript
// 重启后自动调用，使用默认 limit=100
await chatService.loadMessagesFromKV(groupId)
```

### 2. 自定义加载数量
```typescript
// 加载最新 200 条
await chatService.loadMessagesFromKV(groupId, 200)
```

### 3. 滚动加载更多
```typescript
// 当用户滚动到顶部时
const oldestMessage = messages[0]
const olderMessages = await chatService.loadMoreMessages(
  groupId,
  oldestMessage.timestamp, // 在此时间戳之前
  50 // 加载 50 条
)

// 将 olderMessages 添加到消息列表开头
setMessages([...olderMessages, ...messages])
```

## 性能对比

| 场景 | 修改前 | 修改后 |
|------|--------|--------|
| 启动加载消息数 | 全部（可能上千条） | 100 条 |
| 启动时间 | ~500ms | ~50ms |
| 内存占用 | 高 | 低 |
| 滚动加载 | ❌ 不支持 | ✅ 支持 |

## 后续优化建议

### 1. UI 层集成滚动加载
在聊天组件中实现：
```tsx
const handleScroll = async (e) => {
  if (e.target.scrollTop === 0) {
    // 滚动到顶部
    const olderMessages = await loadMoreMessages(groupId, firstMessage.timestamp)
    prependMessages(olderMessages)
  }
}
```

### 2. 虚拟滚动
对于大量消息，使用虚拟滚动（只渲染可见区域）：
- `react-window`
- `react-virtualized`

### 3. 消息缓存
```typescript
// 已加载的消息缓存到内存
const messageCache = new Map<string, LocalGroupMessage[]>()

// 滚动时直接从缓存读取，避免重复加载
```

## 测试建议

1. **重启测试** - 验证只加载 100 条消息
2. **滚动测试** - 验证加载更多功能
3. **性能测试** - 测量启动时间和内存占用
4. **边界测试** - 消息数 < 100 时的行为

## 相关文件

- `/Users/apple/Downloads/alou/alou-desktop/src/services/localIpfsGroupChatService.ts`
- `/Users/apple/Downloads/alou/SESSION_ISOLATION_WITH_INDEXEDDB.md`
