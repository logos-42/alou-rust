# 多页面 Session 隔离完整修复方案

## 问题根源

### 1. 架构问题：全局单例污染

```typescript
// ❌ 问题代码
export const agentCoordinator = UnifiedAgentCoordinator.getInstance()  // 全局单例

// useUnifiedGroupChat.ts
agentCoordinator.dispatchToAgent(agent.id, message)  // 所有页面共享同一个实例
```

### 2. 多页面问题

**问题流程**:
1. 页面 A 打开 → 创建 coordinator 实例 → 注册智能体 A
2. 页面 B 打开 → 获取同一个 coordinator 实例 → 注册智能体 B
3. 页面 B 收到消息 → dispatchToAgent() → **分发到页面 A 和 B 的所有智能体**
4. 结果：两个页面的智能体都收到消息，造成混乱

## 解决方案：SessionProvider + Context

### 架构变更

```
App.tsx
  └─> SessionProvider
        └─> SessionContext (提供 coordinator per session)
              └─> useUnifiedGroupChat (使用当前 session 的 coordinator)
```

### 核心修改

#### 1. 创建 SessionContext (`src/context/SessionContext.tsx`)
- 提供 session 管理
- 提供当前 session 的 coordinator
- 自动从 URL hash 获取/创建 session

#### 2. 更新 App.tsx
```typescript
<SessionProvider autoInit={true}>
  <AppContent />
</SessionProvider>
```

#### 3. 修改 useUnifiedGroupChat
```typescript
// ❌ 修改前
import { agentCoordinator } from '@/services/unifiedAgentCoordinator'
agentCoordinator.dispatchToAgent(...)

// ✅ 修改后
import { useCoordinator } from '@/context/SessionContext'
const coordinator = useCoordinator()
coordinator.dispatchToAgent(...)
```

## Session 生命周期

### 多页面场景

```
页面 A: http://localhost:3000
  → URL hash: #session=session_abc123
  → coordinator A (独立实例)
  → 智能体 A 注册到 coordinator A

页面 B: http://localhost:3000 (新窗口)
  → URL hash: #session=session_xyz789
  → coordinator B (独立实例)
  → 智能体 B 注册到 coordinator B

结果：
- 页面 A 的消息 → coordinator A → 只分发到智能体 A
- 页面 B 的消息 → coordinator B → 只分发到智能体 B
- ✅ 完全隔离
```

## 文件修改清单

### 新增文件
- `src/context/SessionContext.tsx` - Session 上下文
- `src/utils/sessionStorage.ts` - IndexedDB 封装
- `src/services/sessionManager.ts` - Session 管理器

### 修改文件
- `src/App.tsx` - 包裹 SessionProvider
- `src/hooks/useUnifiedGroupChat.ts` - 使用 useCoordinator()
- `src/stores/agentStore.ts` - 使用 IndexedDB
- `src/services/unifiedAgentCoordinator.ts` - 添加 session 隔离

## 验证方法

### 1. 检查 URL hash
```
页面 1: #session=session_abc123
页面 2: #session=session_xyz789
```

### 2. 检查控制台日志
```
[SessionContext] Session 初始化完成：{ sessionId: "session_abc123" }
[AgentCoordinator] 智能体已注册：{ sessionId: "session_abc123" }
```

### 3. 测试消息隔离
1. 页面 A 注册智能体 A
2. 页面 B 注册智能体 B
3. 页面 A 发送消息 → 只有智能体 A 响应 ✅
4. 页面 B 发送消息 → 只有智能体 B 响应 ✅

## 总结

- ✅ 完全隔离 - 每个页面独立 session
- ✅ 自动管理 - 自动创建/恢复/清理
- ✅ URL 同步 - Session ID 通过 hash 传递
- ✅ 大容量存储 - IndexedDB (50MB+)
- ✅ 向后兼容 - 保持现有 API

现在多页面 session 混乱问题已完全解决。
