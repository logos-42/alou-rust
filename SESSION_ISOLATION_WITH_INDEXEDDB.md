# Session 隔离与 IndexedDB 存储方案总结

## 问题描述

多页面并行的智能体运行过程中，存在两个主要问题：
1. **Session 分离不完整** - 多个 session 的数据互相干扰
2. **localStorage 容量有限** - 只有 5-10MB，无法存储大量 session 数据

## 解决方案

采用 **Session 实例隔离 + IndexedDB 持久化** 方案：

1. **Session 实例隔离** - 为每个 session 创建独立的协调器实例
2. **IndexedDB 存储** - 使用 IndexedDB 替代 localStorage，提供大容量存储（50MB+）

## 核心组件

### 1. IndexedDB 封装层 (`utils/sessionStorage.ts`)

```typescript
import * as sessionStorage from '@/utils/sessionStorage'

// 设置数据（带过期时间）
await sessionStorage.set('key', data, {
  sessionId: 'session_123',
  expiresAt: Date.now() + 30 * 60 * 1000
})

// 获取数据
const data = await sessionStorage.get('key')

// 按 session 查询
const items = await sessionStorage.getBySessionId('session_123')

// 获取存储统计
const stats = await sessionStorage.getStats()
```

### 2. SessionManager (`services/sessionManager.ts`)

```typescript
import { sessionManager } from '@/services/sessionManager'

const session = sessionManager.getOrCreateSession({
  sessionId: 'page1_session',
  ttl: 30 * 60 * 1000,
})

await session.coordinator.registerAgent(agent)
await sessionManager.destroySession('page1_session')
```

### 3. AgentStore (`stores/agentStore.ts`)

```typescript
import useAgentStore, { setCurrentSessionId } from '@/stores/agentStore'

setCurrentSessionId('session_123')
// zustand persist 自动使用 IndexedDB 存储
```

## 存储结构

### IndexedDB Database: `alou_session_db`

| Key 前缀 | 用途 | 示例 |
|---------|------|------|
| `alou_agents_` | AgentStore 数据 | `alou_agents_session_123` |
| `session_meta_` | Session 元数据 | `session_meta_session_123` |

## 容量对比

| 存储方式 | 容量限制 | 性能 |
|---------|---------|------|
| localStorage | 5-10MB | 同步，阻塞 |
| IndexedDB | 50MB+ | 异步，非阻塞 |

## 总结

- ✅ Session 完全隔离
- ✅ 大容量存储（50MB+）
- ✅ 异步非阻塞
- ✅ 自动过期清理
- ✅ 持久化恢复
