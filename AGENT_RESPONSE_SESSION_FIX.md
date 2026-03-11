# Agent 回复 Session 隔离修复总结

## 问题诊断

### 问题现象
多个页面的不同 agent 回复会**串行/混乱**，具体表现为：
- 页面 A 的 agent 收到消息，页面 B 的 agent 也响应
- 多个 agent 同时响应同一条消息
- 消息分发没有 session 隔离

### 根本原因

虽然之前修复了 `coordinator` 的 session 隔离，但**agent 实际处理消息是通过全局事件 `agent-group-message`**：

```typescript
// ❌ 问题 1: dispatchToAgent 触发全局事件时不带 sessionId
window.dispatchEvent(new CustomEvent('agent-group-message', {
  detail: { agentId, message }  // 没有 sessionId
}))

// ❌ 问题 2: useAgentMessages 监听全局事件时不过滤 session
window.addEventListener('agent-group-message', handleAgentGroupMessage)
// 所有页面的监听器都会收到消息
```

## 修复方案

### 1. 在 `dispatchToAgent` 中添加 sessionId 到事件

**文件**: `services/unifiedAgentCoordinator.ts`

```typescript
dispatchToAgent(agentId: string, message: UnifiedMessage): void {
  // 触发全局事件（带 session ID）
  window.dispatchEvent(new CustomEvent('agent-group-message', {
    detail: {
      agentId,
      message,
      sessionId: this.sessionId, // 添加 session ID
    }
  }))
}
```

### 2. 在 `useAgentMessages` 中添加 session 过滤

**文件**: `components/AgentChat/useAgentMessages.ts`

```typescript
useEffect(() => {
  const handleAgentGroupMessage = async (event: Event) => {
    const { agentId, message, sessionId: eventSessionId } = event.detail

    // Session 过滤：只处理当前 session 的消息
    if (eventSessionId && eventSessionId !== sessionId) {
      console.log('[useAgentMessages] Session 不匹配，跳过消息')
      return
    }

    // ... 处理消息
  }
}, [sessionId])
```

## 修复后的数据流

```
页面 A: coordinatorA.dispatchToAgent(agentA, message)
  → window.dispatchEvent('agent-group-message', { sessionId: 'session_A' })
  → 页面 A 的监听器收到 (sessionId === 'session_A') ✅
  → 页面 B 的监听器收到 (sessionId !== 'session_B') ❌ 过滤掉

页面 B: coordinatorB.dispatchToAgent(agentB, message)
  → window.dispatchEvent('agent-group-message', { sessionId: 'session_B' })
  → 页面 A 的监听器收到 (sessionId !== 'session_A') ❌ 过滤掉
  → 页面 B 的监听器收到 (sessionId === 'session_B') ✅
```

## 并发特性

- ✅ **多 agent 并行** - 不同 agent 可以同时处理消息
- ✅ **并发限制** - 默认最多 5 个并发任务
- ✅ **消息队列** - 超出限制的消息自动排队
- ✅ **防重处理** - 同一条消息不会被同一 agent 重复处理

## 验证方法

### 1. 检查控制台日志
```
[AgentCoordinator] 消息已分发到智能体：{ sessionId: "session-abc123" }
[useAgentMessages] Session 不匹配，跳过消息：{ eventSessionId: "session-xyz789" }
```

### 2. 测试 agent 响应隔离
1. 页面 A 注册 agent-A
2. 页面 B 注册 agent-B
3. 页面 A 发送消息 → 只有 agent-A 响应 ✅
4. 页面 B 发送消息 → 只有 agent-B 响应 ✅

## 总结

### 修复前
- ❌ 全局事件不带 sessionId
- ❌ 所有页面共享同一个监听器
- ❌ 消息会分发到所有页面的所有 agent
- ❌ agent 响应混乱

### 修复后
- ✅ 全局事件带 sessionId
- ✅ 每个页面过滤不同 session 的消息
- ✅ 消息只分发到当前 session 的 agent
- ✅ agent 响应完全隔离
- ✅ 并发控制正常工作

**现在不同 agent 的回复不会串行，不会混乱，每个 session 的 agent 独立工作。**
