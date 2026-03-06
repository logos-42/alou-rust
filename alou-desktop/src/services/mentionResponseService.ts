/**
 * @ 提令响应服务
 * 当智能体收到 @ 提令时自动响应
 */

import { parseMentions, isMentioned } from '@/utils/groupchat/mentionParser'

export interface MentionResponseOptions {
  /** 智能体ID */
  agentId: string
  /** 智能体名称 */
  agentName: string
  /** 收到提令时的回调 */
  onMentioned?: (message: string, mentionedBy: string) => Promise<void>
  /** 是否自动响应 */
  autoRespond?: boolean
  /** 自动响应延迟（毫秒） */
  autoRespondDelay?: number
}

export interface IncomingMessage {
  /** 消息ID */
  id: string
  /** 消息内容 */
  content: string
  /** 发送者ID */
  senderId: string
  /** 发送者名称 */
  senderName: string
  /** 群聊ID */
  groupId: string
  /** 原始消息数据 */
  raw?: any
}

/**
 * 创建 @ 提令响应处理器
 * 
 * @param options - 配置选项
 * @returns 消息处理函数
 */
export function createMentionResponseHandler(options: MentionResponseOptions) {
  const {
    agentId,
    agentName,
    onMentioned,
    autoRespond = true,
    autoRespondDelay = 1000
  } = options

  /**
   * 处理接收到的消息
   * 如果消息提令了当前智能体，触发响应
   * 
   * @param message - 接收到的消息
   */
  return async function handleIncomingMessage(message: IncomingMessage): Promise<boolean> {
    // 跳过自己发送的消息
    if (message.senderId === agentId) {
      return false
    }

    // 检查消息是否提令了当前智能体
    const mentioned = isMentionions(
      message.content,
      agentId,
      [agentId],
      agentName
    )

    if (mentioned) {
      console.log(`[MentionResponse] 智能体 ${agentName} (${agentId}) 收到提令:`, {
        messageId: message.id,
        content: message.content.slice(0, 50),
        from: message.senderName
      })

      // 如果有提令回调，调用它
      if (onMentioned) {
        try {
          await onMentioned(message.content, message.senderName)
        } catch (error) {
          console.error(`[MentionResponse] 提令回调失败:`, error)
        }
      }

      // 如果启用自动响应
      if (autoRespond) {
        setTimeout(() => {
          triggerAutoResponse(message, agentId, agentName)
        }, autoRespondDelay)
      }

      return true
    }

    return false
  }
}

/**
 * 检查消息是否提令了指定智能体
 */
function isMentionions(
  content: string,
  agentId: string,
  agentIds: string[],
  agentName: string
): boolean {
  // 检查 @agentName 格式
  const mentionPattern = new RegExp(`@${escapeRegex(agentName)}`, 'i')
  if (mentionPattern.test(content)) {
    return true
  }

  // 检查 @agentId 格式
  for (const id of agentIds) {
    const idPattern = new RegExp(`@${escapeRegex(id)}`, 'i')
    if (idPattern.test(content)) {
      return true
    }
  }

  return false
}

/**
 * 转义正则表达式特殊字符
 */
function escapeRegex(string: string): string {
  return string.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
}

/**
 * 触发自动响应
 */
async function triggerAutoResponse(
  message: IncomingMessage,
  agentId: string,
  agentName: string
): Promise<void> {
  console.log(`[MentionResponse] 智能体 ${agentName} 准备自动响应`)
  
  // 这里可以添加实际的 AI 响应逻辑
  // 例如：调用 AI 服务生成回复，然后发送回群聊
  
  // 示例：构建响应消息
  const responseContent = generateAutoResponse(message.content, agentName)
  
  // 发送响应（需要通过外部调用）
  console.log(`[MentionResponse] 自动响应内容:`, responseContent)
  
  // 可以通过事件或回调发送响应
  if (typeof window !== 'undefined') {
    window.dispatchEvent(new CustomEvent('agent-mention-response', {
      detail: {
        agentId,
        agentName,
        originalMessage: message,
        responseContent,
        groupId: message.groupId
      }
    }))
  }
}

/**
 * 生成自动响应内容
 * 这里可以接入实际的 AI 服务
 */
function generateAutoResponse(mentionMessage: string, agentName: string): string {
  // 提取提令后的实际内容（去除 @agentName）
  const actualContent = mentionMessage
    .replace(new RegExp(`@${escapeRegex(agentName)}`, 'gi'), '')
    .trim()

  // 示例响应，实际应该调用 AI
  return `[${agentName}] 收到您的消息: "${actualContent}"

我正在处理您的请求，请稍候...

💡 提示：这是自动回复。要实现真正的 AI 响应，需要接入 AI 服务。`
}

/**
 * 批量创建多个智能体的提令响应处理器
 * 
 * @param agents - 智能体列表
 * @param onMessage - 消息回调
 * @returns 处理器映射
 */
export function createMentionHandlersForAgents(
  agents: Array<{ id: string; name: string }>,
  options: {
    onMentioned?: MentionResponseOptions['onMentioned']
    autoRespond?: boolean
    autoRespondDelay?: number
  } = {}
): Map<string, (message: IncomingMessage) => Promise<boolean>> {
  const handlers = new Map<string, (message: IncomingMessage) => Promise<boolean>>()

  for (const agent of agents) {
    const handler = createMentionResponseHandler({
      agentId: agent.id,
      agentName: agent.name,
      onMentioned: options.onMentioned,
      autoRespond: options.autoRespond ?? true,
      autoRespondDelay: options.autoRespondDelay ?? 1000
    })

    handlers.set(agent.id, handler)
  }

  return handlers
}

/**
 * 监听群聊消息并分发提令响应
 * 
 * @param handlers - 智能体处理器映射
 * @param message - 收到的消息
 */
export async function dispatchMentionResponse(
  handlers: Map<string, (message: IncomingMessage) => Promise<boolean>>,
  message: IncomingMessage
): Promise<void> {
  // 并行检查所有智能体是否被提令
  const results = await Promise.all(
    Array.from(handlers.values()).map(handler => handler(message))
  )

  const respondedCount = results.filter(Boolean).length
  if (respondedCount > 0) {
    console.log(`[MentionResponse] 消息触发了 ${respondedCount} 个智能体响应`)
  }
}
