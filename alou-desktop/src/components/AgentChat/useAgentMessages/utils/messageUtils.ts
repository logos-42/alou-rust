/**
 * 消息工具函数
 * 
 * @module components/AgentChat/useAgentMessages/utils/messageUtils
 */

import { Message, AgentInfo } from '../types'

/**
 * 检查智能体是否被@
 */
export function isAgentMentioned(
  agentId: string,
  messageContent: string,
  agentsList: AgentInfo[]
): boolean {
  const agent = agentsList.find(a => a.id === agentId)
  if (!agent) return false

  const agentName = agent.display_name || agent.name
  if (!agentName) return false

  // 检查是否包含@智能体名称
  return messageContent.includes(`@${agentName}`)
}

/**
 * 创建用户消息
 */
export function createUserMessage(
  content: string,
  agentId: string,
  metadata?: Record<string, unknown>
): Message {
  return {
    id: `user_${Date.now()}_${agentId}`,
    type: 'user',
    content: content.trim(),
    timestamp: Date.now(),
    metadata,
  }
}

/**
 * 创建助手消息
 */
export function createAssistantMessage(
  content: string,
  agentId: string,
  source: string = 'local'
): Message {
  return {
    id: `assistant_${Date.now()}_${agentId}`,
    type: 'assistant',
    content,
    timestamp: Date.now(),
    source,
    agentId,
  }
}

/**
 * 创建系统消息
 */
export function createSystemMessage(
  content: string,
  agentId?: string
): Message {
  return {
    id: `system_${Date.now()}`,
    type: 'system',
    content,
    timestamp: Date.now(),
    source: 'system',
    agentId,
  }
}

/**
 * 创建错误消息
 */
export function createErrorMessage(
  content: string,
  agentId?: string
): Message {
  return {
    id: `error_${Date.now()}`,
    type: 'error',
    content,
    timestamp: Date.now(),
    source: 'error',
    agentId,
  }
}
