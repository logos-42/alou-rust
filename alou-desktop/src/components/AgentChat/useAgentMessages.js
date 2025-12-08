import { useCallback, useState, useMemo, useEffect, useRef } from 'react'
import apiClient from '@/services/api'
import agentService from '@/services/agentService'
import useAgentStore from '@/stores/agentStore'

/**
 * Hook for managing messages and conversation
 * 按频道分开存储消息，支持 IPFS 持久化
 */
export const useAgentMessages = ({
  sessionId,
  setSessionId,
  activeChain,
  activeChannelId,
  selectedAgent,
  isSessionReady,
  createSession,
  setSessionReady,
  recordInteraction,
  handleToolCalls,
  conversationOverlayRef,
  consoleDockRef,
  contextEventsRef,
}) => {
  // 按频道存储消息：Map<channelId, Message[]>
  const [messagesByChannel, setMessagesByChannel] = useState({})
  const [currentMessage, setCurrentMessage] = useState('')
  const [isLoading, setIsLoading] = useState(false)
  const [isConversationVisible, setConversationVisible] = useState(false)
  
  // 跟踪已保存过的消息数量，避免重复保存
  const savedMessageCountRef = useRef({})
  
  // 获取 agentStore 方法
  const updateAgent = useAgentStore((state) => state.updateAgent)

  // 当前频道的消息
  const messages = useMemo(() => {
    return messagesByChannel[activeChannelId] || []
  }, [messagesByChannel, activeChannelId])

  // 添加消息到指定频道
  const appendMessage = useCallback(
    (message, channelId = activeChannelId) => {
      if (!channelId) {
        console.warn('[useAgentMessages] 无法添加消息：没有活动频道')
        return
      }
      
      setMessagesByChannel((prev) => {
        const channelMessages = prev[channelId] || []
        return {
          ...prev,
          [channelId]: [...channelMessages, message],
        }
      })
      
      if (!isConversationVisible) {
        setConversationVisible(true)
      }
    },
    [activeChannelId, isConversationVisible],
  )

  // 设置指定频道的所有消息（用于从 IPFS 加载）
  const setMessagesForChannel = useCallback((channelId, messages) => {
    if (!channelId) return
    
    setMessagesByChannel((prev) => ({
      ...prev,
      [channelId]: messages,
    }))
    
    // 更新已保存计数
    savedMessageCountRef.current[channelId] = messages.length
  }, [])

  // 清空指定频道的消息
  const clearMessagesForChannel = useCallback((channelId) => {
    if (!channelId) return
    
    setMessagesByChannel((prev) => {
      const newMap = { ...prev }
      delete newMap[channelId]
      return newMap
    })
    
    // 重置保存计数
    delete savedMessageCountRef.current[channelId]
  }, [])

  const scrollToBottom = useCallback(() => {
    requestAnimationFrame(() => {
      conversationOverlayRef.current?.scrollToBottom?.()
    })
  }, [conversationOverlayRef])

  // 保存消息到 IPFS
  const saveMessagesToIpfs = useCallback(async (channelId, agentId) => {
    const channelMessages = messagesByChannel[channelId] || []
    const savedCount = savedMessageCountRef.current[channelId] || 0
    
    // 如果没有新消息，跳过保存
    if (channelMessages.length === 0 || channelMessages.length <= savedCount) {
      return null
    }
    
    try {
      console.log(`[useAgentMessages] 保存 ${channelMessages.length} 条消息到 IPFS，频道: ${channelId}`)
      
      const cid = await agentService.uploadMessagesToIpfs(channelMessages, agentId)
      
      // 更新 agentStore 中的 messages_cid
      if (cid && agentId) {
        updateAgent(agentId, { messages_cid: cid })
        console.log(`[useAgentMessages] 消息已保存到 IPFS，CID: ${cid}`)
      }
      
      // 更新已保存计数
      savedMessageCountRef.current[channelId] = channelMessages.length
      
      return cid
    } catch (error) {
      console.error('[useAgentMessages] 保存消息到 IPFS 失败:', error)
      return null
    }
  }, [messagesByChannel, updateAgent])

  // 从 IPFS 加载消息
  const loadMessagesFromIpfs = useCallback(async (channelId, messagesCid) => {
    if (!channelId || !messagesCid) return false
    
    try {
      console.log(`[useAgentMessages] 从 IPFS 加载消息，CID: ${messagesCid}`)
      
      const data = await agentService.loadMessagesFromIpfs(messagesCid)
      
      if (data && data.messages && Array.isArray(data.messages)) {
        setMessagesForChannel(channelId, data.messages)
        console.log(`[useAgentMessages] 已加载 ${data.messages.length} 条消息`)
        return true
      }
      
      return false
    } catch (error) {
      console.error('[useAgentMessages] 从 IPFS 加载消息失败:', error)
      return false
    }
  }, [setMessagesForChannel])

  const sendMessage = useCallback(async () => {
    const text = currentMessage.trim()
    if (!text || isLoading) {
      return
    }

    if (!activeChannelId) {
      console.warn('[useAgentMessages] 无法发送消息：没有活动频道')
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
        }, activeChannelId)
        return
      }
    }

    const userMessage = {
      id: `user_${Date.now()}`,
      type: 'user',
      content: text,
      timestamp: Date.now(),
    }

    appendMessage(userMessage, activeChannelId)
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
      console.log('[useAgentMessages] 发送消息，sessionId:', actualSessionId, '频道:', activeChannelId)

      const data = await apiClient
        .post('/agent/chat', {
          session_id: actualSessionId,
          message: text,
          wallet_address: walletAddress || undefined,
          chain: activeChain || undefined,
          context_events: contextSnapshot,
          // 传递智能体信息，让后端使用正确的 prompt
          agent_id: activeChannelId,
          agent_info: selectedAgent ? {
            name: selectedAgent.display_name || selectedAgent.name,
            role_description: selectedAgent.role_description,
            custom_prompt: selectedAgent.customPrompt,
          } : undefined,
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
      appendMessage(assistantMessage, activeChannelId)

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
      }, activeChannelId)
    } finally {
      setIsLoading(false)
      scrollToBottom()
      consoleDockRef.current?.adjustInputHeight?.()
    }
  }, [
    activeChain,
    activeChannelId,
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
    selectedAgent,
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

  // 当关闭对话面板或切换频道时，保存消息到 IPFS
  const previousChannelRef = useRef(activeChannelId)
  useEffect(() => {
    const prevChannel = previousChannelRef.current
    
    // 如果频道发生变化，保存之前频道的消息
    if (prevChannel && prevChannel !== activeChannelId) {
      const prevMessages = messagesByChannel[prevChannel] || []
      const savedCount = savedMessageCountRef.current[prevChannel] || 0
      
      if (prevMessages.length > savedCount) {
        // 异步保存，不阻塞
        const agentId = prevChannel
        saveMessagesToIpfs(prevChannel, agentId).catch(err => {
          console.error('[useAgentMessages] 切换频道时保存消息失败:', err)
        })
      }
    }
    
    previousChannelRef.current = activeChannelId
  }, [activeChannelId, messagesByChannel, saveMessagesToIpfs])

  return {
    messages,
    messagesByChannel,
    setMessages: (msgs) => {
      if (activeChannelId) {
        setMessagesForChannel(activeChannelId, msgs)
      }
    },
    setMessagesForChannel,
    clearMessagesForChannel,
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
    saveMessagesToIpfs,
    loadMessagesFromIpfs,
  }
}
