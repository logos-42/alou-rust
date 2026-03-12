# Session 隔离消息队列集成指南

## 问题诊断

### 当前阻塞原因

1. **`loadingByAgent` 状态阻塞** (`useAgentMessages.ts:851`)
   ```typescript
   if (isAgentLoading(agentId)) { return } // 跳过消息
   ```

2. **同步等待 AI 响应** (`useAgentMessages.ts:860`)
   ```typescript
   await sendMessageToAgentRef.current(...) // 等待 5-30 秒
   ```

3. **多页面共享 loading 状态**
   - 页面 A 的 Agent A 执行中 → loading = true
   - 页面 B 的 Agent A 收到消息 → loading = true → 跳过

## 解决方案

### 新增文件
`src/services/sessionMessageQueue.ts` - Session 隔离的消息队列

### 核心修改

#### useAgentMessages.ts

**修改前**:
```typescript
if (isAgentLoading(agentId)) return
await sendMessageToAgentRef.current(...) // 阻塞
```

**修改后**:
```typescript
// 立即入队，不阻塞
enqueueMessage(agentId, messageContent, sessionId, {
  isGroupChat: true,
  priority: message.isMentioned ? 1 : 10,
})
```

## 架构

```
Session A → Queue A → Worker A → Agent A1, A2, A3... (并发)
Session B → Queue B → Worker B → Agent B1, B2, B3... (并发)
```

## 预期效果

- ✅ 无阻塞 - 消息入队立即返回
- ✅ 多页面独立 - 不同 session 互不影响
- ✅ 并发处理 - 同 session 最多 5 个 agent 同时执行
- ✅ 优先级 - 被@的消息优先处理

## 待修改内容

### 1. useAgentMessages.ts Line 791-879

替换整个 `handleAgentGroupMessage` useEffect 为：

```typescript
useEffect(() => {
  const handleAgentGroupMessage = (event: Event) => {
    const { agentId, message, sessionId: eventSessionId } = event.detail
    if (eventSessionId && eventSessionId !== sessionId) return
    
    const content = message.content || message.text || ''
    if (!content.trim()) return
    
    enqueueMessage(agentId, content, sessionId, {
      isGroupChat: true,
      groupId: message.groupId,
      originalMessage: content,
      priority: message.isMentioned ? 1 : 10,
    })
  }
  
  window.addEventListener('agent-group-message', handleAgentGroupMessage)
  return () => window.removeEventListener('agent-group-message', handleAgentGroupMessage)
}, [sessionId])
```

### 2. 添加队列处理器 useEffect

在 `sendMessageToAgentRef` 定义后添加：

```typescript
useEffect(() => {
  setSessionHandler(sessionId, async (queueMessage: QueueMessage) => {
    if (sendMessageToAgentRef.current) {
      await sendMessageToAgentRef.current(
        queueMessage.agentId,
        queueMessage.content,
        null,
        {
          isGroupChat: queueMessage.isGroupChat,
          groupId: queueMessage.groupId,
          originalMessage: queueMessage.originalMessage,
        }
      )
    }
  })
}, [sessionId])
```

## 决策

请确认是否继续修改以下文件：
1. `useAgentMessages.ts` - 修改事件监听逻辑
2. `sessionMessageQueue.ts` - 已创建

修改后效果：
- 每个 session 独立的工作流
- 多页面不再互相阻塞
- 支持并发处理和优先级
