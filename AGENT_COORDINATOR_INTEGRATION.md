# 群聊适配器与智能体协调器集成总结

## 概述

本次集成将 `unifiedAgentCoordinator`（统一智能体协调器）与新迁移的群聊适配器架构深度集成，实现了智能体在群聊中的自动消息分发和协调。

## 集成架构

```
┌─────────────────────────────────────────────────────────┐
│                    React UI Layer                        │
│  (群聊界面、智能体管理界面)                                │
└────────────────────┬────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────┐
│           unifiedAgentCoordinator.ts                     │ ← 上层：智能体管理
│  - 智能体注册/注销                                        │
│  - 群聊订阅管理 (新增)                                    │
│  - 消息分发到智能体                                        │
│  - 自动回复                                               │
└────────────────────┬────────────────────────────────────┘
                     │ 使用
┌────────────────────▼────────────────────────────────────┐
│         adapters/groupChatAdapter.ts                     │ ← 中层：群聊适配层
│  - BaseAdapter (Memory/PubSub/Iroh)                      │
│  - 调用 Rust 后端                                         │
│  - 本地缓存                                               │
│  - 订阅管理                                               │
└────────────────────┬────────────────────────────────────┘
                     │ invoke
┌────────────────────▼────────────────────────────────────┐
│      Rust tools/adapters/group_adapter.rs                │ ← 底层：核心逻辑
│  - 三种模式实现                                           │
│  - 群聊操作                                               │
│  - 消息存储                                               │
└─────────────────────────────────────────────────────────┘
```

## 完成的工作

### 1. unifiedAgentCoordinator.ts 更新

#### 新增功能
- ✅ **群聊订阅管理**
  - `subscribeToGroup(groupId, mode?)` - 订阅群聊消息
  - `unsubscribeFromGroup(groupId)` - 取消订阅
  - `getSubscribedGroups()` - 获取已订阅群聊列表
  - `getGroupSubscriptionStatus(groupId)` - 获取订阅状态

- ✅ **自动消息监听**
  - 通过群聊适配器订阅消息
  - 自动分发到群聊中的智能体
  - 支持自定义消息处理器

- ✅ **配置选项增强**
  ```typescript
  interface AgentCoordinatorConfig {
    autoReply?: boolean      // 是否启用自动回复
    replyDelay?: number      // 回复延迟（毫秒）
    debug?: boolean          // 调试模式
    autoSubscribe?: boolean  // 是否自动订阅群聊消息（新增）
    onMessage?: (message) => void  // 消息处理器（新增）
  }
  ```

#### 核心流程
```typescript
// 1. 订阅群聊
await agentCoordinator.subscribeToGroup(groupId, GroupChatMode.PUBSUB)

// 2. 注册智能体到群聊
await agentCoordinator.registerToGroup(groupId, agentInfo)

// 3. 自动接收消息并分发
// 当群聊收到消息时：
//   a. 调用自定义 onMessage 处理器
//   b. 分发到群聊中的所有智能体
//   c. 触发智能体的 handler 函数
//   d. 如果启用自动回复，触发自动回复逻辑
```

### 2. 群聊适配器更新

#### groupChatAdapter.ts
- ✅ 导出 `BaseAdapter` 类（从 `private` 改为 `public`）
- ✅ 导出 `DefaultGroupChatAdapterFactory` 类
- ✅ 修复 `available` 属性可见性（从 `protected` 改为 `public`）
- ✅ 添加 `topic` 字段到 `GroupChatConfig`
- ✅ 修复类型转换问题

#### types/groupchat.ts
- ✅ 更新 `MessageHandler` 类型，接受 `UnifiedMessage | GroupChatMessage`
- ✅ 添加 `topic` 字段到 `GroupChatConfig`
- ✅ 更新 `UnifiedGroup` 和 `UnifiedMessage` 接口，与 Rust 后端对齐

### 3. 便捷函数导出

```typescript
// 智能体管理
export function registerAgent(agent, handler?)
export function registerToGroup(groupId, agent)
export function unregisterAgent(agentId)

// 群聊订阅（新增）
export function subscribeToGroup(groupId, mode?)
export function unsubscribeFromGroup(groupId)

// 消息分发
export function dispatchToAgent(agentId, message)
export function getActiveAgents(groupId?)

// 配置
export function setAutoReply(enabled)
export function setReplyDelay(ms)

// 状态查询
export function getCoordinatorStatus()
```

## 使用示例

### 示例 1：基本使用

```typescript
import {
  agentCoordinator,
  registerAgent,
  registerToGroup,
  subscribeToGroup,
} from '@/services/unifiedAgentCoordinator'

import { getAdapterForGroup } from '@/adapters'

// 1. 初始化协调器
const coordinator = agentCoordinator

// 2. 订阅群聊消息
await subscribeToGroup('group-123', GroupChatMode.PUBSUB)

// 3. 注册智能体
await registerAgent({
  id: 'agent-1',
  name: '助手',
  mode: 'agent'
}, (message, agentId) => {
  console.log(`${agentId} 收到消息:`, message.content)
  // 处理消息逻辑
})

// 4. 将智能体注册到群聊
await registerToGroup('group-123', {
  id: 'agent-1',
  name: '助手',
})

// 5. 启用自动回复
setAutoReply(true)
setReplyDelay(1000) // 1 秒延迟
```

### 示例 2：高级使用 - 自定义消息处理器

```typescript
import { UnifiedAgentCoordinator } from '@/services/unifiedAgentCoordinator'

// 创建自定义配置的协调器
const coordinator = UnifiedAgentCoordinator.getInstance({
  autoReply: false,
  replyDelay: 500,
  debug: true,
  autoSubscribe: true,
  onMessage: (message) => {
    // 全局消息处理器
    console.log('收到消息:', message)
    
    // 可以在这里进行消息过滤、日志记录等
    if (message.type === 'system') {
      // 处理系统消息
    }
  }
})

// 订阅群聊后，所有消息都会通过 onMessage 处理器
await coordinator.subscribeToGroup('group-123')
```

### 示例 3：与现有群聊组件集成

```typescript
import { useGroupChat } from '@/hooks/useGroupChat'
import { agentCoordinator } from '@/services/unifiedAgentCoordinator'

function GroupChatComponent({ groupId }) {
  const { messages, sendMessage } = useGroupChat(groupId)
  
  useEffect(() => {
    // 组件挂载时订阅群聊
    agentCoordinator.subscribeToGroup(groupId)
    
    // 注册群聊助手智能体
    const assistantAgent = {
      id: 'assistant',
      name: '群聊助手',
      mode: 'agent'
    }
    
    agentCoordinator.registerToGroup(groupId, assistantAgent)
    
    return () => {
      // 组件卸载时取消订阅
      agentCoordinator.unsubscribeFromGroup(groupId)
    }
  }, [groupId])
  
  return (
    <div>
      {/* 群聊 UI */}
    </div>
  )
}
```

## 消息流转

```
┌──────────────┐
│  群聊成员     │
│  发送消息     │
└──────┬───────┘
       │
       ▼
┌─────────────────────────────────┐
│  Rust GroupAdapter              │
│  - 接收消息                     │
│  - 存储到对应模式存储            │
└──────────────┬──────────────────┘
               │
               ▼
┌─────────────────────────────────┐
│  BaseAdapter.subscribe()        │
│  - 监听消息事件                 │
└──────────────┬──────────────────┘
               │
               ▼
┌─────────────────────────────────┐
│  AgentCoordinator.              │
│  handleGroupMessage()           │
│  - 调用 onMessage 处理器         │
│  - 分发到群聊中的智能体          │
└──────────────┬──────────────────┘
               │
       ┌───────┴────────┐
       │                │
       ▼                ▼
┌─────────────┐  ┌─────────────┐
│ Agent 1     │  │ Agent 2     │
│ handler()   │  │ handler()   │
└─────────────┘  └─────────────┘
```

## 配置选项

### AgentCoordinatorConfig

| 选项 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `autoReply` | `boolean` | `false` | 是否启用智能体自动回复 |
| `replyDelay` | `number` | `1000` | 自动回复延迟（毫秒） |
| `debug` | `boolean` | `false` | 调试模式，输出详细日志 |
| `autoSubscribe` | `boolean` | `true` | 注册智能体时自动订阅所有群聊 |
| `onMessage` | `function` | `undefined` | 全局消息处理器 |

## 状态查询

```typescript
// 获取协调器状态
const status = getCoordinatorStatus()
console.log(status)
// {
//   registeredAgents: 3,
//   subscribedGroups: 2,
//   autoReply: true,
//   debug: false
// }

// 获取所有智能体
const agents = agentCoordinator.getAllAgents()

// 获取智能体所在的群聊
const groups = agentCoordinator.getAgentGroups('agent-1')

// 获取群聊中的活跃智能体
const activeAgents = agentCoordinator.getActiveAgents('group-123')
```

## 注意事项

### 1. 类型兼容
- `MessageHandler` 现在接受 `UnifiedMessage | GroupChatMessage`
- 确保智能体处理器能处理两种类型的消息

### 2. 资源清理
- 组件卸载时调用 `unsubscribeFromGroup()` 取消订阅
- 使用 `destroy()` 方法销毁协调器实例（测试场景）

### 3. 性能考虑
- 大量群聊时，注意订阅数量
- 自动回复延迟不宜过短，避免频繁请求

### 4. 错误处理
- 智能体处理器的错误会被捕获并记录
- Promise 错误会自动处理，不会导致未捕获异常

## 后续优化

1. **持久化订阅** - 将群聊订阅持久化，重启后自动恢复
2. **智能体优先级** - 支持智能体优先级，高优先级智能体优先处理
3. **消息过滤** - 支持智能体设置消息过滤规则
4. **分布式协调** - 支持多实例间的智能体协调
5. **性能监控** - 添加性能指标监控，如消息处理延迟

## 总结

通过本次集成，`unifiedAgentCoordinator` 现在完全基于新的群聊适配器架构，实现了：

- ✅ **统一的消息订阅** - 自动监听所有群聊模式的消息
- ✅ **智能体自动分发** - 消息自动分发到群聊中的智能体
- ✅ **灵活的配置** - 支持自定义消息处理器和自动回复
- ✅ **类型安全** - 与 Rust 后端类型对齐
- ✅ **向后兼容** - 保持现有 API 不变

智能体协调器现在可以作为所有群聊相关智能体功能的统一入口。
