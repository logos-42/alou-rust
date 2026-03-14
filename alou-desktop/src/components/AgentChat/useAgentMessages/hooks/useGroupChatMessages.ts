/**
 * 群聊消息 Hook - 处理群聊相关的消息收发
 * 
 * @module components/AgentChat/useAgentMessages/hooks/useGroupChatMessages
 */

import { useCallback } from 'react'
import { enqueueMessage, getQueueStatus } from '@/services/sessionMessageQueue'
import { GroupChatMessage, Message } from '../types'

interface UseGroupChatMessagesConfig {
  sessionId: string
  sessionsByAgent: Record<string, string>
  isAgentLoading: (agentId: string) => boolean
  setAgentLoading: (agentId: string, loading: boolean) => void
  appendMessage: (message: Message, channelId?: string) => void
}

interface UseGroupChatMessagesReturn {
  handleGroupChatMessage: (agentId: string, message: GroupChatMessage) => void
  sendGroupChatMessage: (
    agentId: string,
    content: string,
    groupId: string,
    originalMessage?: string
  ) => boolean
}

/**
 * 群聊消息 Hook
 */
export function useGroupChatMessages({
  sessionId,
  sessionsByAgent,
  isAgentLoading,
  setAgentLoading,
  appendMessage,
}: UseGroupChatMessagesConfig): UseGroupChatMessagesReturn {
  /**
   * 处理收到的群聊消息
   */
  const handleGroupChatMessage = useCallback((
    agentId: string,
    message: GroupChatMessage
  ) => {
    const messageContent = message.content || message.text || ''
    
    if (!messageContent.trim()) {
      console.warn('[handleGroupChatMessage] 群聊消息内容为空，跳过')
      return
    }

    // 检查智能体是否正在执行
    if (isAgentLoading(agentId)) {
      console.log('[handleGroupChatMessage] 智能体正在执行中，跳过群聊消息:', agentId)
      return
    }

    // 检查@提及
    if (message.isMentioned !== undefined && message.isMentioned) {
      const mentionedIds = message.mentionedAgentIds || []
      if (!mentionedIds.includes(agentId)) {
        console.log('[handleGroupChatMessage] 智能体未被@，跳过处理:', agentId)
        return
      }
    }

    // 获取智能体的 session
    let agentSessionId = sessionsByAgent[agentId] || sessionId
    if (!agentSessionId || agentSessionId.startsWith('frontend_')) {
      agentSessionId = sessionId
    }

    // 使用异步消息队列处理（非阻塞，支持并行）
    console.log('[handleGroupChatMessage] 触发智能体处理群聊消息（异步并行）:', agentId)

    const success = enqueueMessage(agentId, messageContent, agentSessionId, {
      isGroupChat: true,
      groupId: message.groupId,
      originalMessage: messageContent,
      priority: 5, // 群聊消息优先级稍低
    })

    if (!success) {
      console.warn('[handleGroupChatMessage] 群聊消息入队失败，队列可能已满')
    } else {
      console.log(
        '[handleGroupChatMessage] 群聊消息已入队，sessionId:',
        agentSessionId,
        'queue status:',
        getQueueStatus(agentSessionId)
      )
    }
  }, [sessionId, sessionsByAgent, isAgentLoading, setAgentLoading, appendMessage])

  /**
   * 发送群聊消息
   */
  const sendGroupChatMessage = useCallback((
    agentId: string,
    content: string,
    groupId: string,
    originalMessage?: string
  ): boolean => {
    if (!content?.trim() || !agentId || !groupId) {
      console.warn('[sendGroupChatMessage] 无法发送消息：缺少必要参数')
      return false
    }

    // 获取智能体的 session
    let agentSessionId = sessionsByAgent[agentId] || sessionId
    if (!agentSessionId || agentSessionId.startsWith('frontend_')) {
      agentSessionId = sessionId
    }

    // 显示用户消息（立即显示，不等待执行）
    const userMessage: Message = {
      id: `user_${Date.now()}_${agentId}`,
      type: 'user',
      content: content.trim(),
      timestamp: Date.now(),
      metadata: {
        isGroupChatMessage: true,
        groupId,
        originalMessage,
      },
    }

    appendMessage(userMessage, agentId)
    setAgentLoading(agentId, true)

    // 将消息放入异步队列
    const success = enqueueMessage(agentId, content, agentSessionId, {
      isGroupChat: true,
      groupId,
      originalMessage,
      priority: 10, // 普通消息优先级较高
    })

    if (!success) {
      console.warn('[sendGroupChatMessage] 消息入队失败，队列可能已满')
      setAgentLoading(agentId, false)
      appendMessage({
        id: `error_${Date.now()}`,
        type: 'error',
        content: '⚠️ 消息队列已满，请稍后重试',
        timestamp: Date.now(),
        source: 'error',
        agentId,
      }, agentId)
    } else {
      console.log(
        '[sendGroupChatMessage] 消息已入队，sessionId:',
        agentSessionId,
        'queue status:',
        getQueueStatus(agentSessionId)
      )
    }

    return success
  }, [sessionId, sessionsByAgent, appendMessage, setAgentLoading])

  return {
    handleGroupChatMessage,
    sendGroupChatMessage,
  }
}
