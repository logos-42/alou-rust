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

    if (!isSessionReady) {
      await createSession()
      setSessionReady(true)
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

      const data = await apiClient
        .post('/agent/chat', {
          session_id: sessionId,
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
      appendMessage({
        id: `error_${Date.now()}`,
        type: 'assistant',
        content: `❌ 抱歉，发生了错误：${error instanceof Error ? error.message : '未知错误'}`,
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
