/**
 * 智能体名称统一工具函数
 * 
 * 提供统一的智能体名称获取和标准化功能，确保所有组件显示一致的名称
 * 
 * @module utils/agentNameUtils
 * @author Alou Team
 */

import type { AgentInfo } from '@/types/groupchat'
import type { Agent, Channel } from '@/components/AgentChat/agentUtils'

/**
 * 智能体数据标准化接口
 */
export interface NormalizedAgent extends AgentInfo {
  /** 标准名称（所有名称字段统一为此值） */
  standardName: string
  /** 名称来源字段 */
  nameSource: 'display_name' | 'name' | 'agent_name' | 'id'
}

/**
 * 获取智能体的标准名称
 * 
 * 优先级规则：
 * 1. display_name - 显示名称（最优先）
 * 2. name - 基础名称
 * 3. agent_name - 智能体名称
 * 4. id 的简短形式 - 如果以上都没有，使用 ID 的简短形式
 * 5. '未命名智能体' - 默认值
 * 
 * @param agent - 智能体对象（支持 AgentInfo、Agent、Channel 等类型）
 * @returns 标准化的智能体名称
 */
export function getAgentName(agent: Partial<AgentInfo | Agent | Channel | null | undefined>): string {
  if (!agent) {
    return '未命名智能体'
  }

  // 尝试各种可能的名称字段，按优先级排序
  const name = 
    (agent as any).display_name ||
    (agent as any).name ||
    (agent as any).agent_name ||
    (agent as any).standardName ||
    null

  // 如果有名称，返回修剪后的值
  if (name && typeof name === 'string' && name.trim()) {
    return name.trim()
  }

  // 如果没有名称，尝试使用 ID 的简短形式
  const id = (agent as any).id || (agent as any).did || (agent as any).cid || (agent as any).ipns
  if (id && typeof id === 'string') {
    // 如果是 DID，提取最后一部分
    if (id.startsWith('did:')) {
      const didParts = id.split(':')
      if (didParts.length > 2) {
        const lastPart = didParts[didParts.length - 1]
        return lastPart.length > 16 ? `${lastPart.substring(0, 8)}...` : lastPart
      }
    }
    // 如果是 IPNS，提取名称部分
    if (id.startsWith('ipns/') || id.startsWith('/ipns/')) {
      return id.replace(/^\/?ipns\//, '').substring(0, 42)
    }
    // 其他 ID 类型，截取前 8 个字符
    return id.substring(0, 8)
  }

  return '未命名智能体'
}

/**
 * 从 Channel 获取智能体名称
 * @param channel - Channel 对象
 * @returns 频道名称
 */
export function getChannelName(channel: Channel | null | undefined): string {
  if (!channel) {
    return '未命名频道'
  }
  return channel.name || channel.display_name || '未命名频道'
}

/**
 * 标准化智能体数据
 * 
 * 确保所有名称字段（name、display_name、agent_name）都使用相同的值，
 * 避免不同组件显示不同名称的问题。
 * 
 * @param agent - 原始智能体数据
 * @returns 标准化后的智能体数据
 */
export function normalizeAgent(agent: Partial<AgentInfo | Agent>): NormalizedAgent {
  const standardName = getAgentName(agent)
  
  // 确定名称来源
  let nameSource: NormalizedAgent['nameSource'] = 'name'
  if ((agent as any).display_name && (agent as any).display_name === standardName) {
    nameSource = 'display_name'
  } else if ((agent as any).agent_name && (agent as any).agent_name === standardName) {
    nameSource = 'agent_name'
  } else if (!agent.id && !agent.did && !agent.cid) {
    nameSource = 'id'
  }

  return {
    id: (agent as any).id || (agent as any).did || (agent as any).cid || '',
    name: standardName,
    display_name: standardName,
    mode: (agent as any).mode || 'agent',
    avatar: (agent as any).avatar || (agent as any).avatar_url,
    avatar_url: (agent as any).avatar_url || (agent as any).avatar,
    avatar_cid: (agent as any).avatar_cid,
    did: (agent as any).did,
    cid: (agent as any).cid,
    ipns: (agent as any).ipns,
    // 添加标准化标记
    standardName,
    nameSource,
  }
}

/**
 * 批量标准化智能体数组
 * @param agents - 智能体数组
 * @returns 标准化后的智能体数组
 */
export function normalizeAgents(agents: Array<Partial<AgentInfo | Agent>>): NormalizedAgent[] {
  return agents.map(agent => normalizeAgent(agent))
}

/**
 * 确保智能体的所有名称字段一致
 * 
 * 如果智能体的 name、display_name、agent_name 不一致，统一为标准名称
 * 
 * @param agent - 智能体对象
 * @returns 名称字段一致的智能体对象
 */
export function ensureConsistentAgentNames(agent: Partial<AgentInfo | Agent>): Partial<AgentInfo | Agent> {
  const standardName = getAgentName(agent)
  
  return {
    ...agent,
    name: standardName,
    display_name: standardName,
    agent_name: standardName,
  }
}

/**
 * 比较两个智能体名称是否一致
 * @param agent1 - 第一个智能体
 * @param agent2 - 第二个智能体
 * @returns 名称是否一致
 */
export function areAgentNamesEqual(agent1: Partial<AgentInfo | Agent>, agent2: Partial<AgentInfo | Agent>): boolean {
  return getAgentName(agent1) === getAgentName(agent2)
}

/**
 * 格式化智能体名称用于显示
 * 
 * 对于过长的名称进行截断，添加省略号
 * 
 * @param agent - 智能体对象
 * @param maxLength - 最大长度（默认 20）
 * @returns 格式化后的名称
 */
export function formatAgentNameForDisplay(agent: Partial<AgentInfo | Agent>, maxLength: number = 20): string {
  const name = getAgentName(agent)
  
  if (name.length <= maxLength) {
    return name
  }
  
  return `${name.substring(0, maxLength - 3)}...`
}

/**
 * 从 agentStore 数据中提取标准名称
 * 
 * 优先使用 agentStore 中存储的名称，确保跨组件一致性
 * 
 * @param agentId - 智能体 ID
 * @param agentStore - agentStore 实例
 * @returns 标准名称，如果未找到则返回 null
 */
export function getAgentNameFromStore(
  agentId: string, 
  agentStore: { getAgent: (id: string) => any }
): string | null {
  const storedAgent = agentStore.getAgent(agentId)
  if (!storedAgent) {
    return null
  }
  
  return getAgentName(storedAgent)
}

/**
 * 增强智能体数据 - 从 store 中补充缺失的名称信息
 * 
 * 如果传入的智能体缺少名称字段，尝试从 agentStore 中获取
 * 
 * @param agent - 智能体对象
 * @param agentStore - agentStore 实例
 * @returns 增强后的智能体对象
 */
export function enhanceAgentWithStoreData(
  agent: Partial<AgentInfo | Agent>,
  agentStore: { getAgent: (id: string) => any }
): Partial<AgentInfo | Agent> {
  // 如果没有名称，尝试从 store 获取
  if (!agent.name && !agent.display_name && agent.id) {
    const storedAgent = agentStore.getAgent(agent.id)
    if (storedAgent) {
      return {
        ...agent,
        name: storedAgent.name || storedAgent.display_name,
        display_name: storedAgent.display_name || storedAgent.name,
      }
    }
  }
  
  return agent
}
