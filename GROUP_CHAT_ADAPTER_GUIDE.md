# 统一群聊架构使用指南

## 概述

新的统一群聊架构基于适配器模式，提供了一致的 API 来使用不同的群聊后端：

- **Memory** - 纯内存模式，离线可用
- **PubSub** - 基于 IPFS PubSub 的去中心化群聊
- **Iroh** - 基于 Iroh 协议的 P2P 群聊

## 架构设计

```
┌─────────────────────────────────────────────────────────────┐
│                      统一群聊 API                            │
│  createGroup() / joinGroup() / sendMessage() / listGroups() │
└─────────────────────────┬───────────────────────────────────┘
                          │
        ┌─────────────────┼─────────────────┐
        ▼                 ▼                 ▼
┌───────────────┐  ┌───────────────┐  ┌───────────────┐
│ MemoryAdapter │  │ PubSubAdapter │  │  IrohAdapter  │
│ (本地群聊)     │  │ (DIAP PubSub) │  │ (Iroh P2P)    │
└───────────────┘  └───────────────┘  └───────────────┘
```

## 快速开始

### 1. 使用统一 Hook（推荐）

```typescript
import { useUnifiedGroupChat } from '@/hooks/useUnifiedGroupChat'

function MyComponent() {
  const {
    // 状态
    isInitialized,
    currentMode,
    activeGroupId,
    activeGroup,
    messages,
    groups,
    isLoading,
    error,
    
    // 操作方法
    createGroup,
    joinGroup,
    leaveGroup,
    switchGroup,
    sendMessage,
    refreshGroups,
    loadHistory,
  } = useUnifiedGroupChat({
    activeChannelId: 'channel-1',
    localIdentity: { did: 'user123', name: '用户' },
    onGroupCreated: (group, agents) => {
      console.log('群聊创建成功:', group)
    },
    onMessage: (groupId, message) => {
      console.log('收到消息:', message)
    },
  })

  const handleCreate = async () => {
    const group = await createGroup({
      name: '测试群聊',
      description: '这是一个测试群聊',
      mode: 'auto', // 自动选择最佳模式
      members: [
        { id: 'agent1', name: '智能体 1', mode: 'agent' }
      ]
    })
  }

  const handleSend = async () => {
    await sendMessage(activeGroupId, 'Hello, World!')
  }

  return (
    <div>
      {/* UI 代码 */}
    </div>
  )
}
```

### 2. 直接使用适配器工厂

```typescript
import {
  groupChatAdapterFactory,
  getBestAvailableAdapter,
  createUnifiedGroup,
} from '@/adapters'

// 获取最佳可用适配器
const adapter = await getBestAvailableAdapter()
console.log('当前模式:', adapter.mode)

// 或者手动选择模式
const memoryAdapter = groupChatAdapterFactory.getAdapter('memory')
const pubsubAdapter = groupChatAdapterFactory.getAdapter('pubsub')

// 创建群聊
const group = await createUnifiedGroup({
  name: '我的群聊',
  description: '测试群聊',
  mode: 'auto'
})

// 发送消息
const message = await adapter.sendMessage(group.id, 'Hello!')
```

### 3. 使用智能体协调器

```typescript
import { agentCoordinator } from '@/services/unifiedAgentCoordinator'

// 注册智能体
await agentCoordinator.registerAgent(
  { id: 'agent1', name: '助手', mode: 'agent' },
  (message, agentId) => {
    console.log(`智能体 ${agentId} 收到消息:`, message)
    // 处理消息并生成回复
  }
)

// 注册到群聊
await agentCoordinator.registerToGroup(groupId, agent)

// 启用自动回复
agentCoordinator.setAutoReply(true)
agentCoordinator.setReplyDelay(1000) // 1 秒延迟
```

## API 参考

### 适配器接口

所有适配器都实现以下接口：

```typescript
interface GroupChatAdapter {
  readonly mode: GroupChatMode
  readonly available: boolean
  
  // 群聊管理
  createGroup(config: GroupChatConfig): Promise<UnifiedGroup>
  joinGroup(groupId: string): Promise<UnifiedGroup>
  leaveGroup(groupId: string): Promise<void>
  listGroups(): Promise<UnifiedGroup[]>
  getGroupInfo(groupId: string): Promise<UnifiedGroup | null>
  
  // 消息通信
  sendMessage(groupId: string, content: string, type?: MessageType): Promise<UnifiedMessage>
  subscribe(groupId: string, handler: MessageHandler): UnsubscribeFunction
  getHistory(groupId: string, limit?: number): Promise<UnifiedMessage[]>
}
```

### 统一群聊接口

```typescript
interface UnifiedGroup {
  id: string
  name: string
  description?: string
  mode: GroupChatMode
  members: AgentInfo[]
  createdAt: number
  createdBy?: string
  topic?: string        // PubSub 专用
  ticket?: string       // Iroh 专用
  rawData?: object      // 适配器特定数据
}
```

### 统一消息接口

```typescript
interface UnifiedMessage {
  id: string
  groupId: string
  type: MessageType
  sender: AgentInfo
  content: string
  timestamp: number
  metadata?: object
}
```

## 模式选择

### Memory 模式
- **适用场景**: 离线开发、测试、演示
- **优点**: 无需后端、快速、可靠
- **缺点**: 数据不持久化（会话级）

### PubSub 模式
- **适用场景**: 去中心化应用、多用户协作
- **优点**: 真正的去中心化、实时通信
- **缺点**: 需要 IPFS 节点

### Iroh 模式
- **适用场景**: P2P 应用、边缘计算
- **优点**: 原生 P2P、文档同步
- **缺点**: 当前为 Mock 实现

## 迁移指南

### 从旧代码迁移

**旧代码:**
```typescript
import localIpfsGroupChatService from '@/services/localIpfsGroupChatService'

const group = await localIpfsGroupChatService.createGroup({...})
await localIpfsGroupChatService.sendMessage(message)
```

**新代码:**
```typescript
import { createUnifiedGroup, sendUnifiedMessage } from '@/adapters'

const group = await createUnifiedGroup({...})
await sendUnifiedMessage(group, 'Hello!')
```

### 从 useDiapGroupChat 迁移

**旧代码:**
```typescript
import { useDiapGroupChat } from '@/hooks/useDiapGroupChat'

const diapGroupChat = useDiapGroupChat({...})
```

**新代码:**
```typescript
import { useUnifiedGroupChat } from '@/hooks/useUnifiedGroupChat'

const unifiedGroupChat = useUnifiedGroupChat({...})
```

## 错误处理

```typescript
try {
  const group = await createGroup({ name: '测试' })
} catch (error) {
  if (error.message.includes('IPFS')) {
    // IPFS 不可用，切换到 Memory 模式
  } else if (error.message.includes('不存在')) {
    // 群聊不存在
  }
}
```

## 最佳实践

1. **使用统一 Hook**: 优先使用 `useUnifiedGroupChat` 而非直接使用适配器
2. **错误处理**: 始终包装在 try-catch 中
3. **清理资源**: 组件卸载时自动清理订阅
4. **智能体注册**: 在应用启动时注册智能体
5. **模式检测**: 使用 `auto` 模式让系统自动选择

## 调试

```typescript
// 启用调试模式
import { agentCoordinator } from '@/services/unifiedAgentCoordinator'

agentCoordinator.setDebugMode(true)

// 查看适配器状态
import { groupChatAdapterFactory } from '@/adapters'

const status = groupChatAdapterFactory.getAllAdapterStatus()
console.log('适配器状态:', status)
```

## 相关文件

- `src/adapters/groupChatAdapter.ts` - 适配器工厂
- `src/adapters/memoryAdapter.ts` - Memory 适配器
- `src/adapters/pubsubAdapter.ts` - PubSub 适配器
- `src/adapters/irohAdapter.ts` - Iroh 适配器
- `src/hooks/useUnifiedGroupChat.ts` - 统一 Hook
- `src/services/unifiedAgentCoordinator.ts` - 智能体协调器
- `src/types/groupchat.ts` - 类型定义
