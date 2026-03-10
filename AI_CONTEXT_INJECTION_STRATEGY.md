# AI 上下文注入优化策略

## 问题背景

在群聊协作场景中，AI 智能体需要知道其他智能体的名称和信息才能有效协作。但持续注入所有智能体信息会导致：

1. **Token 浪费** - 每次请求都携带所有智能体信息
2. **上下文污染** - 无关信息干扰 AI 判断
3. **性能下降** - 过大的上下文影响响应速度

## 解决方案：智能上下文注入

### 策略 1：按需注入（Lazy Injection）

只在以下场景注入其他智能体信息：

```typescript
// 需要注入的场景
const NEED_CONTEXT_INJECTION = [
  'group_chat_creation',      // 创建群聊时
  'agent_invitation',          // 邀请智能体时
  'task_assignment',           // 任务分配时
  'multi_agent_coordination',  // 多智能体协同时
  'agent_introduction',        // 新智能体加入时
]

// 不需要注入的场景
const NO_CONTEXT_INJECTION = [
  'regular_chat_message',      // 普通聊天消息
  'status_query',              // 状态查询
  'simple_acknowledgment',     // 简单确认
]
```

### 策略 2：增量注入（Incremental Injection）

只注入变化的智能体信息，而非全量注入：

```typescript
interface ContextInjection {
  // 基础上下文（始终注入）
  baseContext: {
    currentAgent: AgentInfo
    groupId: string
    mode: GroupChatMode
  }
  
  // 增量上下文（按需注入）
  deltaContext?: {
    newAgents: AgentInfo[]      // 新加入的智能体
    departedAgents: string[]    // 离开的智能体 ID
    updatedAgents: AgentInfo[]  // 信息更新的智能体
  }
  
  // 任务上下文（任务相关时注入）
  taskContext?: {
    taskId: string
    assignedAgents: string[]
    taskDescription: string
  }
}
```

### 策略 3：摘要注入（Summary Injection）

使用摘要而非完整信息：

```typescript
// 完整信息（Token 消耗大）
const FULL_AGENT_INFO = {
  id: 'agent-123',
  name: '数据分析助手',
  display_name: '数据分析助手',
  avatar: 'ipfs://...',
  did: 'did:...',
  ipns: 'ipns://...',
  mode: 'agent',
  capabilities: ['data_analysis', 'chart_generation'],
  // ... 更多字段
}

// 摘要信息（Token 消耗小）
const AGENT_SUMMARY = {
  id: 'agent-123',
  name: '数据分析助手',
  role: '数据分析',  // 从 capabilities 提取的主要角色
}
```

## 实现方案

### 1. 上下文注入管理器

```typescript
// src/services/contextInjectionManager.ts

import { getAgentName } from '@/utils/agentNameUtils'

export interface InjectionConfig {
  // 是否启用注入
  enabled: boolean
  
  // 注入模式
  mode: 'full' | 'summary' | 'minimal'
  
  // 注入场景白名单
  whitelistScenes: string[]
  
  // 注入场景黑名单
  blacklistScenes: string[]
  
  // 最大注入智能体数量（超过则使用摘要）
  maxAgentsForFullInjection: number
}

export class ContextInjectionManager {
  private config: InjectionConfig
  private injectedAgents: Map<string, AgentInfo> = new Map()
  
  constructor(config: Partial<InjectionConfig> = {}) {
    this.config = {
      enabled: true,
      mode: 'summary',
      whitelistScenes: ['group_chat_creation', 'agent_invitation'],
      blacklistScenes: ['regular_chat_message'],
      maxAgentsForFullInjection: 5,
      ...config,
    }
  }
  
  /**
   * 判断是否需要注入上下文
   */
  shouldInject(scene: string): boolean {
    if (!this.config.enabled) return false
    if (this.config.blacklistScenes.includes(scene)) return false
    if (this.config.whitelistScenes.includes(scene)) return true
    
    // 默认不注入
    return false
  }
  
  /**
   * 构建注入的上下文
   */
  buildContext(
    scene: string,
    currentAgent: AgentInfo,
    groupAgents: AgentInfo[]
  ): ContextInjection {
    const baseContext = {
      currentAgent: this.compressAgentInfo(currentAgent),
      groupId: (currentAgent as any).groupId,
      mode: currentAgent.mode,
    }
    
    // 如果不应该注入，返回基础上下文
    if (!this.shouldInject(scene)) {
      return { baseContext }
    }
    
    // 计算增量
    const deltaContext = this.calculateDelta(groupAgents)
    
    // 根据数量决定使用完整信息还是摘要
    const injectionMode = groupAgents.length <= this.config.maxAgentsForFullInjection
      ? 'full'
      : 'summary'
    
    return {
      baseContext,
      deltaContext,
      ...(injectionMode === 'summary' && {
        agentSummaries: groupAgents.map(a => this.compressAgentInfo(a)),
      }),
      ...(injectionMode === 'full' && {
        fullAgents: groupAgents.map(a => this.normalizeAgentInfo(a)),
      }),
    }
  }
  
  /**
   * 压缩智能体信息（用于摘要模式）
   */
  private compressAgentInfo(agent: AgentInfo): CompressedAgentInfo {
    return {
      id: agent.id,
      name: getAgentName(agent),  // 使用统一名称
      role: this.extractAgentRole(agent),
    }
  }
  
  /**
   * 标准化智能体信息（用于完整模式）
   */
  private normalizeAgentInfo(agent: AgentInfo): NormalizedAgent {
    return normalizeAgent(agent)
  }
  
  /**
   * 计算增量变化
   */
  private calculateDelta(currentAgents: AgentInfo[]): DeltaContext | undefined {
    const newAgents: AgentInfo[] = []
    const departedAgents: string[] = []
    const updatedAgents: AgentInfo[] = []
    
    // 检测新智能体
    for (const agent of currentAgents) {
      if (!this.injectedAgents.has(agent.id)) {
        newAgents.push(agent)
      } else {
        // 检测更新的智能体
        const existing = this.injectedAgents.get(agent.id)!
        if (this.hasAgentChanged(existing, agent)) {
          updatedAgents.push(agent)
        }
      }
    }
    
    // 检测离开的智能体
    for (const [id] of this.injectedAgents) {
      if (!currentAgents.find(a => a.id === id)) {
        departedAgents.push(id)
      }
    }
    
    // 更新缓存
    currentAgents.forEach(a => this.injectedAgents.set(a.id, a))
    
    if (newAgents.length === 0 && departedAgents.length === 0 && updatedAgents.length === 0) {
      return undefined
    }
    
    return { newAgents, departedAgents, updatedAgents }
  }
  
  /**
   * 提取智能体角色
   */
  private extractAgentRole(agent: AgentInfo): string {
    // 从名称提取
    const name = getAgentName(agent)
    if (name.includes('数据')) return '数据分析'
    if (name.includes('翻译')) return '翻译'
    if (name.includes('写作')) return '内容创作'
    
    // 默认
    return '通用助手'
  }
  
  /**
   * 检查智能体信息是否变化
   */
  private hasAgentChanged(old: AgentInfo, new: AgentInfo): boolean {
    return old.name !== new.name || 
           old.avatar !== new.avatar || 
           old.mode !== new.mode
  }
  
  /**
   * 清除注入缓存
   */
  clearCache(): void {
    this.injectedAgents.clear()
  }
}

// 导出单例
export const contextInjectionManager = new ContextInjectionManager()
```

### 2. 在群聊中使用

```typescript
// src/hooks/useGroupChat.ts

import { contextInjectionManager } from '@/services/contextInjectionManager'

export const useGroupChat = ({ actionId }) => {
  // ...
  
  const sendMessage = async (content: string) => {
    // 判断是否需要注入上下文
    const scene = 'regular_chat_message'
    const context = contextInjectionManager.buildContext(
      scene,
      localIdentity,
      groupAgents
    )
    
    // 只在需要时注入
    if (context.deltaContext || context.fullAgents) {
      message.metadata.context = context
    }
    
    // 发送消息
    await adapter.sendMessage(groupId, content)
  }
  
  const createGroup = async (config: GroupChatConfig) => {
    // 创建群聊时总是注入完整上下文
    const scene = 'group_chat_creation'
    const context = contextInjectionManager.buildContext(
      scene,
      localIdentity,
      config.members || []
    )
    
    // 注入上下文
    config.metadata = {
      ...config.metadata,
      context,
    }
    
    return await adapter.createGroup(config)
  }
  
  // ...
}
```

### 3. AI 服务端处理

```typescript
// src/services/agentService.ts

export const processAgentRequest = async (request: AgentRequest) => {
  const { messages, context } = request
  
  // 提取上下文
  const systemPrompt = buildSystemPrompt(context)
  
  // 构建消息
  const apiMessages = [
    { role: 'system', content: systemPrompt },
    ...messages,
  ]
  
  // 调用 AI API
  const response = await callAI(apiMessages)
  
  return response
}

const buildSystemPrompt = (context: ContextInjection): string => {
  const parts: string[] = []
  
  // 基础上下文
  parts.push(`你是一个智能体助手，当前模式：${context.baseContext.mode}`)
  parts.push(`你的身份：${context.baseContext.currentAgent.name}`)
  
  // 增量上下文（如果有）
  if (context.deltaContext) {
    const { newAgents, departedAgents, updatedAgents } = context.deltaContext
    
    if (newAgents.length > 0) {
      parts.push(`新加入的智能体：${newAgents.map(a => a.name).join(', ')}`)
    }
    
    if (departedAgents.length > 0) {
      parts.push(`离开的智能体：${departedAgents.join(', ')}`)
    }
    
    if (updatedAgents.length > 0) {
      parts.push(`信息更新的智能体：${updatedAgents.map(a => a.name).join(', ')}`)
    }
  }
  
  // 完整智能体列表（如果存在）
  if (context.fullAgents) {
    parts.push(`群聊中的所有智能体：`)
    context.fullAgents.forEach(agent => {
      parts.push(`- ${agent.name} (${agent.mode})`)
    })
  }
  
  // 摘要列表（如果存在）
  if (context.agentSummaries) {
    parts.push(`协作智能体：`)
    context.agentSummaries.forEach(agent => {
      parts.push(`- ${agent.name} - ${agent.role}`)
    })
  }
  
  return parts.join('\n')
}
```

## Token 优化效果

### 优化前（每次请求）

```
系统提示词：~500 tokens
智能体列表（5 个智能体）：~1000 tokens
消息历史：~2000 tokens
总计：~3500 tokens/请求
```

### 优化后

#### 场景 1：普通聊天（无注入）
```
系统提示词：~200 tokens（只有基础信息）
消息历史：~2000 tokens
总计：~2200 tokens/请求（节省 37%）
```

#### 场景 2：创建群聊（完整注入）
```
系统提示词：~500 tokens
智能体列表：~1000 tokens
消息历史：~500 tokens
总计：~2000 tokens（仅首次）
```

#### 场景 3：新智能体加入（增量注入）
```
系统提示词：~300 tokens（基础 + 增量）
智能体摘要：~200 tokens（仅新智能体）
消息历史：~2000 tokens
总计：~2500 tokens（节省 29%）
```

## 最佳实践

1. **默认不注入** - 普通聊天不注入智能体信息
2. **事件触发** - 只在特定事件时注入（创建、加入、离开）
3. **使用摘要** - 智能体数量多时使用摘要而非完整信息
4. **增量更新** - 只注入变化的部分
5. **缓存感知** - 缓存已注入的信息，避免重复

## 相关文件

- `src/utils/agentNameUtils.ts` - 统一名称工具
- `src/services/contextInjectionManager.ts` - 上下文注入管理器（待创建）
- `src/stores/agentStore.ts` - 智能体存储
- `src/components/AgentChat/useAgentInvite.ts` - 智能体邀请逻辑
