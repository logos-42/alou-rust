# 群聊智能体名称同步修复方案

## 问题描述

智能体在左侧栏显示的名称与群聊中显示的名称不一致。左侧栏的智能体名称来自 `channels` 数组，但群聊中的智能体名称可能来自不同的数据源，导致名称不匹配。

## 根本原因

1. **左侧栏数据来源**: `AgentSidebarLeft` 组件从 `channels` 数组获取智能体信息，显示 `channel.name`
2. **群聊数据来源**: `GroupChatPanel` 从 `activeGroup.agents` 获取智能体信息，但名称字段可能是 `name`、`display_name` 或 `agent_name`
3. **数据不一致**: 邀请智能体时，传递的名称字段可能与左侧栏使用的字段不同

## 解决方案

### 方案 1: 统一名称获取逻辑（推荐）

创建一个统一的工具函数来获取智能体名称，确保所有组件使用相同的逻辑。

#### 1. 创建工具函数

```typescript
// src/utils/agentNameUtils.ts

import type { AgentInfo } from '@/types/groupchat'
import type { Channel } from '@/components/AgentChat/agentUtils'

/**
 * 获取智能体的标准名称
 * 优先级：display_name > name > agent_name > '未命名智能体'
 */
export function getAgentName(agent: Partial<AgentInfo | Channel>): string {
  if (!agent) {
    return '未命名智能体'
  }

  // 尝试各种可能的名称字段
  const name = 
    (agent as any).display_name ||
    (agent as any).name ||
    (agent as any).agent_name ||
    '未命名智能体'

  return name.trim() || '未命名智能体'
}

/**
 * 从 Channel 获取智能体名称
 */
export function getChannelName(channel: Channel): string {
  return channel.name || channel.display_name || '未命名智能体'
}

/**
 * 标准化智能体数据，确保名称字段一致
 */
export function normalizeAgent(agent: Partial<AgentInfo | Channel>): AgentInfo {
  const name = getAgentName(agent)
  
  return {
    id: (agent as any).id || (agent as any).did || (agent as any).cid || '',
    name,
    display_name: name,
    mode: (agent as any).mode || 'agent',
    avatar: (agent as any).avatar || (agent as any).avatar_url,
    did: (agent as any).did,
    cid: (agent as any).cid,
    ipns: (agent as any).ipns,
  }
}
```

#### 2. 更新 GroupChatPanel 使用统一名称

```jsx
// src/components/agent/GroupChatPanel.jsx

import { getAgentName } from '@/utils/agentNameUtils'

// 在 AgentAvatar 组件中
const AgentAvatar = ({ agent, onAgentClick, t }) => {
  const agentId = agent.id || agent.agent_id || agent.did
  const avatar = resolveAgentAvatar(agent)
  const agentName = getAgentName(agent) // 使用统一名称

  return (
    <button
      key={agentId}
      className="agent-avatar-button"
      onClick={() => onAgentClick?.(agent)}
      title={`${agentName} - 点击打开对话`} // 使用统一名称
    >
      <div className="agent-avatar-small">
        {avatar ? (
          <img src={avatar} alt={agentName} />
        ) : (
          <div style={{
            width: '32px',
            height: '32px',
            borderRadius: '50%',
            background: '#ccc',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            fontSize: '12px',
            color: '#666'
          }}>
            {agentName.charAt(0).toUpperCase() || '?'}
          </div>
        )}
      </div>
      {agent.mode && (
        <div className="agent-mode-badge">{agent.mode === 'agent' ? 'A' : 'L'}</div>
      )}
    </button>
  )
}
```

#### 3. 更新 useAgentInvite 标准化智能体数据

```typescript
// src/components/AgentChat/useAgentInvite.ts

import { normalizeAgent, getAgentName } from '@/utils/agentNameUtils'

// 在 handleInviteSubmit 中
const handleInviteSubmit = useCallback(async (channel: Channel, agents: Agent[], source: string) => {
  // 标准化所有智能体数据，确保名称一致
  const normalizedAgents = agents.map(agent => normalizeAgent(agent))
  
  // 使用标准化后的智能体数据
  const agentNames = normalizedAgents.map(a => getAgentName(a)).join(', ')
  
  // ... 后续逻辑使用 normalizedAgents 而不是 agents
}, [])
```

### 方案 2: 从 agentStore 同步智能体名称

如果智能体已经存储在 `agentStore` 中，可以从 store 中获取标准名称。

```typescript
// src/components/agent/GroupChatPanel.jsx

import useAgentStore from '@/stores/agentStore'

const GroupChatPanel = ({ agents = [], ... }) => {
  const getAgent = useAgentStore(state => state.getAgent)
  
  // 增强智能体数据，从 store 中获取标准名称
  const enhancedAgents = useMemo(() => {
    return agents.map(agent => {
      // 尝试从 store 获取完整信息
      const storedAgent = getAgent(agent.id) || getAgent(agent.did)
      
      if (storedAgent) {
        return {
          ...agent,
          name: storedAgent.name || storedAgent.display_name || agent.name,
          display_name: storedAgent.display_name || storedAgent.name || agent.display_name,
        }
      }
      
      return agent
    })
  }, [agents, getAgent])
  
  // 使用 enhancedAgents 而不是 agents
  const actualAgents = useMemo(() => {
    return activeGroup?.agents || enhancedAgents || []
  }, [activeGroup, enhancedAgents])
}
```

### 方案 3: 在邀请时传递完整的智能体信息

确保邀请智能体时传递完整的名称信息。

```typescript
// src/components/AgentChat/useAgentInvite.ts

const handleInviteSubmit = useCallback(async (channel: Channel, agents: Agent[], source: string) => {
  // 确保智能体数据包含所有名称字段
  const agentsWithNames = agents.map(agent => ({
    ...agent,
    // 确保所有名称字段都存在
    name: agent.display_name || agent.name || '未命名智能体',
    display_name: agent.name || agent.display_name || '未命名智能体',
    agent_name: agent.name || agent.display_name || '未命名智能体',
  }))
  
  // 使用 agentsWithNames 进行后续操作
}, [])
```

## 实施步骤

1. **创建工具函数** - 添加 `src/utils/agentNameUtils.ts`
2. **更新 GroupChatPanel** - 使用统一名称函数
3. **更新 useAgentInvite** - 标准化智能体数据
4. **测试验证** - 确保左侧栏和群聊显示相同名称

## 测试场景

1. 创建智能体，检查左侧栏显示名称
2. 邀请智能体到群聊，检查群聊中显示的名称
3. 修改智能体名称，检查两侧是否同步更新

## 相关文件

- `src/components/agent/AgentSidebarLeft.jsx` - 左侧栏组件
- `src/components/agent/GroupChatPanel.jsx` - 群聊面板组件
- `src/components/AgentChat/useAgentInvite.ts` - 智能体邀请逻辑
- `src/stores/agentStore.ts` - 智能体存储
- `src/types/groupchat.ts` - 群聊类型定义
