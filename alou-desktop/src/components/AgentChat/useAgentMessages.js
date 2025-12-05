import { useCallback, useState } from 'react'
import apiClient from '@/services/api'
import { resolveBackendChain } from '@/hooks/useAgentChat'

/**
 * Hook for managing messages and conversation
 */
export const useAgentMessages = ({
  sessionId,
  setSessionId,
  activeChain,
  isSessionReady,
  createSession,
  setSessionReady,
  recordInteraction,
  handleToolCalls,
  conversationOverlayRef,
  consoleDockRef,
  contextEventsRef,
}) => {
  const [messages, setMessages] = useState([])
  const [currentMessage, setCurrentMessage] = useState('')
  const [isLoading, setIsLoading] = useState(false)
  const [isConversationVisible, setConversationVisible] = useState(false)

  const appendMessage = useCallback(
    (message) => {
      setMessages((prev) => [...prev, message])
      if (!isConversationVisible) {
        setConversationVisible(true)
      }
    },
    [isConversationVisible],
  )

  const scrollToBottom = useCallback(() => {
    requestAnimationFrame(() => {
      conversationOverlayRef.current?.scrollToBottom?.()
    })
  }, [conversationOverlayRef])

  const sendMessage = useCallback(async () => {
    const text = currentMessage.trim()
    if (!text || isLoading) {
      return
    }

    // 确保 session 创建成功
    let currentSessionId = sessionId
    if (!isSessionReady || sessionId.startsWith('frontend_')) {
      try {
        console.log('[useAgentMessages] 创建新会话...')
        await createSession()
        setSessionReady(true)
        // 等待一下让 sessionId 更新
        await new Promise((resolve) => setTimeout(resolve, 100))
      } catch (sessionErr) {
        console.error('[useAgentMessages] 会话创建失败:', sessionErr)
        appendMessage({
          id: `error_${Date.now()}`,
          type: 'assistant',
          content: '❌ 无法连接到后端服务，请检查网络连接或稍后重试。',
          timestamp: Date.now(),
          source: 'error',
        })
        return
      }
    }

    const userMessage = {
      id: `user_${Date.now()}`,
      type: 'user',
      content: text,
      timestamp: Date.now(),
    }

    appendMessage(userMessage)
    setCurrentMessage('')
    setIsLoading(true)
    recordInteraction('user_message', { content: text })
    scrollToBottom()

    const contextSnapshot = contextEventsRef.current.splice(0, contextEventsRef.current.length)

    try {
      const walletAddress =
        typeof window !== 'undefined' ? localStorage.getItem('wallet_address') : null

      // 使用最新的 sessionId（可能在 createSession 后更新了）
      const actualSessionId = sessionId.startsWith('frontend_') ? currentSessionId : sessionId
      console.log('[useAgentMessages] 发送消息，sessionId:', actualSessionId)

      const data = await apiClient
        .post('/agent/chat', {
          session_id: actualSessionId,
          message: text,
          wallet_address: walletAddress || undefined,
          chain: activeChain || undefined,
          context_events: contextSnapshot,
        })
        .then((response) => response.data)

      if (data.tool_calls) {
        await handleToolCalls(data.tool_calls)
      }

      const assistantMessage = {
        id: `assistant_${Date.now()}`,
        type: 'assistant',
        content: data.content || data.response || '收到响应',
        timestamp: data.timestamp || Date.now(),
        source: data.source || 'alou-edge',
      }
      appendMessage(assistantMessage)

      if (data.session_id) {
        setSessionId(data.session_id)
      }
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : '未知错误'
      const statusCode = error?.response?.status
      
      let friendlyMessage = `❌ 抱歉，发生了错误：${errorMessage}`
      if (statusCode === 404) {
        friendlyMessage = '❌ 会话已过期，请刷新页面重试。'
        // 重置 session 状态
        setSessionReady(false)
      } else if (statusCode === 500) {
        friendlyMessage = '❌ 服务器内部错误，请稍后重试。'
      } else if (!error?.response) {
        friendlyMessage = '❌ 无法连接到服务器，请检查网络连接。'
      }
      
      appendMessage({
        id: `error_${Date.now()}`,
        type: 'assistant',
        content: friendlyMessage,
        timestamp: Date.now(),
        source: 'error',
      })
    } finally {
      setIsLoading(false)
      scrollToBottom()
      consoleDockRef.current?.adjustInputHeight?.()
    }
  }, [
    activeChain,
    appendMessage,
    consoleDockRef,
    contextEventsRef,
    createSession,
    currentMessage,
    handleToolCalls,
    isLoading,
    isSessionReady,
    recordInteraction,
    scrollToBottom,
    sessionId,
    setSessionId,
    setSessionReady,
  ])

  const openConversationPanel = useCallback(() => {
    setConversationVisible(true)
    scrollToBottom()
  }, [scrollToBottom])

  const closeConversationPanel = useCallback(() => {
    setConversationVisible(false)
  }, [])

  return {
    messages,
    setMessages,
    currentMessage,
    setCurrentMessage,
    isLoading,
    setIsLoading,
    isConversationVisible,
    setConversationVisible,
    appendMessage,
    scrollToBottom,
    sendMessage,
    openConversationPanel,
    closeConversationPanel,
  }
}
