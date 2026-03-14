/**
 * 消息队列服务
 * 
 * @module components/AgentChat/useAgentMessages/services/messageQueueService
 */

import { enqueueMessage, setSessionHandler, getQueueStatus, type QueueMessage } from '@/services/sessionMessageQueue'

interface MessageQueueServiceConfig {
  sessionId: string
  onMessageProcess?: (message: QueueMessage) => Promise<void>
  onMessageError?: (message: QueueMessage, error: Error) => void
}

interface MessageQueueServiceReturn {
  initialize: () => void
  destroy: () => void
  sendAgentMessage: (
    agentId: string,
    content: string,
    options?: {
      groupId?: string
      isGroupChat?: boolean
      originalMessage?: string
      priority?: number
    }
  ) => boolean
  getQueueStatus: () => any
}

/**
 * 消息队列服务
 */
export function createMessageQueueService({
  sessionId,
  onMessageProcess,
  onMessageError,
}: MessageQueueServiceConfig): MessageQueueServiceReturn {
  let initialized = false

  /**
   * 初始化消息队列处理器
   */
  const initialize = () => {
    if (initialized) {
      console.log('[MessageQueueService] 已初始化，跳过')
      return
    }

    // 设置消息处理器
    const handler = async (message: QueueMessage) => {
      console.log('[MessageQueueService] 处理消息，agentId:', message.agentId)

      try {
        if (onMessageProcess) {
          await onMessageProcess(message)
        }
      } catch (error) {
        console.error('[MessageQueueService] 消息处理失败:', error)
        if (onMessageError) {
          onMessageError(message, error as Error)
        }
      }
    }

    setSessionHandler(sessionId, handler)
    initialized = true
    console.log('[MessageQueueService] 初始化完成，sessionId:', sessionId)
  }

  /**
   * 销毁消息队列
   */
  const destroy = () => {
    console.log('[MessageQueueService] 销毁消息队列')
    initialized = false
  }

  /**
   * 发送消息到智能体
   */
  const sendAgentMessage = (
    agentId: string,
    content: string,
    options?: {
      groupId?: string
      isGroupChat?: boolean
      originalMessage?: string
      priority?: number
    }
  ): boolean => {
    if (!content?.trim() || !agentId) {
      console.warn('[MessageQueueService] 无法发送消息：缺少必要参数')
      return false
    }

    const success = enqueueMessage(agentId, content, sessionId, {
      groupId: options?.groupId,
      isGroupChat: options?.isGroupChat,
      originalMessage: options?.originalMessage,
      priority: options?.priority ?? 10,
    })

    if (!success) {
      console.warn('[MessageQueueService] 消息入队失败，队列可能已满')
    } else {
      const status = getQueueStatus(sessionId)
      console.log('[MessageQueueService] 消息已入队，queue status:', status)
    }

    return success
  }

  /**
   * 获取队列状态
   */
  const getQueueStatusWrapper = () => {
    return getQueueStatus(sessionId)
  }

  return {
    initialize,
    destroy,
    sendAgentMessage,
    getQueueStatus: getQueueStatusWrapper,
  }
}
