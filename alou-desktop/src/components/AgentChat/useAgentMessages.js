import { useCallback, useState, useMemo, useEffect, useRef } from 'react'
import apiClient from '@/services/api'
import agentService from '@/services/agentService'
import clusterActionService from '@/services/clusterActionService'
import useAgentStore from '@/stores/agentStore'
import useClusterActionStore from '@/stores/clusterActionStore'

/**
 * Hook for managing messages and conversation
 * 按频道分开存储消息，支持 IPFS 持久化
 * 支持多智能体独立执行空间
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
  currentMode = 'agent', // 当前模式：'agent' 或 'alou'
}) => {
  // 按频道存储消息：Map<channelId, Message[]>
  const [messagesByChannel, setMessagesByChannel] = useState({})
  const [currentMessage, setCurrentMessage] = useState('')
  // 全局 loading 状态（向后兼容）
  const [isLoading, setIsLoading] = useState(false)
  // 按智能体存储 loading 状态：Map<channelId, boolean>
  const [loadingByAgent, setLoadingByAgent] = useState({})
  // 按智能体存储 session：Map<channelId, sessionId>
  const [sessionsByAgent, setSessionsByAgent] = useState({})
  const [isConversationVisible, setConversationVisible] = useState(false)
  
  // 跟踪已保存过的消息数量，避免重复保存
  const savedMessageCountRef = useRef({})
  
  // 按智能体存储 AbortController：Map<channelId, AbortController>
  const abortControllersByAgent = useRef({})
  
  // 获取 agentStore 方法
  const updateAgent = useAgentStore((state) => state.updateAgent)
  
  // 获取指定智能体的 loading 状态
  const isAgentLoading = useCallback((agentId) => {
    return loadingByAgent[agentId] || false
  }, [loadingByAgent])
  
  // 设置指定智能体的 loading 状态
  const setAgentLoading = useCallback((agentId, loading) => {
    setLoadingByAgent(prev => ({ ...prev, [agentId]: loading }))
    // 同时更新全局 loading（如果是当前活动智能体）
    if (agentId === activeChannelId) {
      setIsLoading(loading)
    }
  }, [activeChannelId])

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

  /**
   * 发送消息到指定智能体
   * 支持多智能体独立执行空间
   * 自动检测是否需要集群行动
   */
  const sendMessageToAgent = useCallback(async (targetAgentId, text, targetAgent = null) => {
    if (!text?.trim() || !targetAgentId) {
      console.warn('[useAgentMessages] 无法发送消息：缺少文本或目标智能体')
      return
    }

    // 检查该智能体是否正在执行
    if (loadingByAgent[targetAgentId]) {
      console.log('[useAgentMessages] 智能体正在执行中，跳过:', targetAgentId)
      return
    }

    const userMessage = {
      id: `user_${Date.now()}_${targetAgentId}`,
      type: 'user',
      content: text.trim(),
      timestamp: Date.now(),
    }

    appendMessage(userMessage, targetAgentId)
    setAgentLoading(targetAgentId, true)
    recordInteraction('user_message', { content: text, agentId: targetAgentId })
    scrollToBottom()

    const contextSnapshot = contextEventsRef.current.splice(0, contextEventsRef.current.length)

    // 尝试自动创建集群行动（如果任务需要多智能体协作）
    try {
      const { addAction, setActiveAction } = useClusterActionStore.getState()
      const { getAgents } = useAgentStore.getState()
      const availableAgents = getAgents().map((a) => a.sessionId || a.id).filter(Boolean)

      if (availableAgents.length > 1) {
        // 分析任务是否需要集群行动
        const analysis = await clusterActionService.analyzeTask(text.trim(), availableAgents)

        if (analysis?.analysis?.needs_cluster_action) {
          console.log('[useAgentMessages] 检测到需要集群行动，创建中...', analysis)

          // 创建集群行动
          const userId = typeof window !== 'undefined' ? localStorage.getItem('user_id') || 'user' : 'user'
          const createResult = await clusterActionService.createClusterAction(
            text.trim(),
            userId,
            { auto_created: true, original_message: text.trim() },
          )

          if (createResult?.action) {
            const action = createResult.action
            addAction(action)

            // 执行集群行动
            const walletAddress =
              typeof window !== 'undefined' ? localStorage.getItem('wallet_address') : null
            await clusterActionService.executeClusterAction(
              action.action_id,
              walletAddress,
              activeChain || undefined,
            )

            // 设置为活跃行动并打开群聊
            setActiveAction(action.action_id)
            // 注意：showGroupChat 状态需要在父组件中管理，这里通过事件通知
            if (typeof window !== 'undefined') {
              window.dispatchEvent(
                new CustomEvent('cluster-action-created', {
                  detail: { actionId: action.action_id },
                }),
              )
            }

            // 添加系统消息提示
            appendMessage(
              {
                id: `system_${Date.now()}`,
                type: 'assistant',
                content: `🤖 检测到需要多智能体协作，已创建集群行动 #${action.action_id.slice(-8)}。正在执行中...`,
                timestamp: Date.now(),
                source: 'system',
              },
              targetAgentId,
            )

            setAgentLoading(targetAgentId, false)
            return // 集群行动已创建，不再执行单智能体逻辑
          }
        }
      }
    } catch (error) {
      console.error('[useAgentMessages] 自动创建集群行动失败，继续单智能体处理:', error)
      // 失败时继续执行单智能体逻辑
    }

    try {
      const walletAddress =
        typeof window !== 'undefined' ? localStorage.getItem('wallet_address') : null

      // 获取或创建该智能体的 session
      let agentSessionId = sessionsByAgent[targetAgentId] || sessionId
      
      // 如果没有有效的 session，创建一个新的
      if (!agentSessionId || agentSessionId.startsWith('frontend_')) {
        try {
          console.log('[useAgentMessages] 为智能体创建新会话:', targetAgentId)
          await createSession()
          setSessionReady(true)
          await new Promise((resolve) => setTimeout(resolve, 100))
          agentSessionId = sessionId
        } catch (sessionErr) {
          console.error('[useAgentMessages] 会话创建失败:', sessionErr)
          appendMessage({
            id: `error_${Date.now()}`,
            type: 'assistant',
            content: '❌ 无法连接到后端服务，请检查网络连接或稍后重试。',
            timestamp: Date.now(),
            source: 'error',
          }, targetAgentId)
          setAgentLoading(targetAgentId, false)
          return
        }
      }

      // 保存该智能体的 session
      setSessionsByAgent(prev => ({ ...prev, [targetAgentId]: agentSessionId }))

      console.log('[useAgentMessages] 发送消息，sessionId:', agentSessionId, '智能体:', targetAgentId)

      // 创建 AbortController 用于终止请求
      const abortController = new AbortController()
      abortControllersByAgent.current[targetAgentId] = abortController

      const agentInfo = targetAgent || selectedAgent
      const data = await apiClient
        .post('/agent/chat', {
          session_id: agentSessionId,
          message: text.trim(),
          wallet_address: walletAddress || undefined,
          chain: activeChain || undefined,
          context_events: contextSnapshot,
          agent_id: targetAgentId,
          agent_info: agentInfo ? {
            name: agentInfo.display_name || agentInfo.name,
            role_description: agentInfo.role_description,
            custom_prompt: agentInfo.customPrompt,
            mode: currentMode, // 传递当前模式
          } : undefined,
        }, {
          signal: abortController.signal, // 添加 abort signal 用于终止请求
        })
        .then((response) => response.data)

      if (data.tool_calls) {
        await handleToolCalls(data.tool_calls)
      }

      const assistantMessage = {
        id: `assistant_${Date.now()}_${targetAgentId}`,
        type: 'assistant',
        content: data.content || data.response || '收到响应',
        timestamp: data.timestamp || Date.now(),
        source: data.source || 'alou-edge',
        agentId: targetAgentId,
      }
      appendMessage(assistantMessage, targetAgentId)

      if (data.session_id) {
        setSessionsByAgent(prev => ({ ...prev, [targetAgentId]: data.session_id }))
        if (targetAgentId === activeChannelId) {
          setSessionId(data.session_id)
        }
      }
    } catch (error) {
      // 如果是用户主动取消，不显示错误消息
      if (error.name === 'AbortError' || error.name === 'CanceledError') {
        console.log('[useAgentMessages] 用户取消了智能体执行:', targetAgentId)
        appendMessage({
          id: `cancel_${Date.now()}`,
          type: 'assistant',
          content: '⏹️ 执行已终止',
          timestamp: Date.now(),
          source: 'cancel',
        }, targetAgentId)
        return
      }
      
      const errorMessage = error instanceof Error ? error.message : '未知错误'
      const statusCode = error?.response?.status
      
      let friendlyMessage = `❌ 抱歉，发生了错误：${errorMessage}`
      if (statusCode === 404) {
        friendlyMessage = '❌ 会话已过期，请刷新页面重试。'
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
      }, targetAgentId)
    } finally {
      // 清理 AbortController
      delete abortControllersByAgent.current[targetAgentId]
      setAgentLoading(targetAgentId, false)
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
    handleToolCalls,
    loadingByAgent,
    recordInteraction,
    scrollToBottom,
    selectedAgent,
    sessionId,
    sessionsByAgent,
    setAgentLoading,
    setSessionId,
    setSessionReady,
  ])

  // 向后兼容的 sendMessage（发送到当前活动智能体）
  const sendMessage = useCallback(async () => {
    const text = currentMessage.trim()
    if (!text || isLoading) {
      return
    }

    if (!activeChannelId) {
      console.warn('[useAgentMessages] 无法发送消息：没有活动频道')
      return
    }

    setCurrentMessage('')
    await sendMessageToAgent(activeChannelId, text, selectedAgent)
  }, [
    activeChannelId,
    currentMessage,
    isLoading,
    selectedAgent,
    sendMessageToAgent,
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

  /**
   * 终止指定智能体的执行
   */
  const cancelAgentExecution = useCallback((agentId) => {
    const controller = abortControllersByAgent.current[agentId]
    if (controller) {
      console.log('[useAgentMessages] 终止智能体执行:', agentId)
      controller.abort()
      delete abortControllersByAgent.current[agentId]
      setAgentLoading(agentId, false)
      
      // 添加终止消息
      appendMessage({
        id: `cancel_${Date.now()}`,
        type: 'assistant',
        content: '⏹️ 执行已终止',
        timestamp: Date.now(),
        source: 'cancel',
      }, agentId)
    }
  }, [appendMessage, setAgentLoading])

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
    // 多智能体独立执行空间
    loadingByAgent,
    isAgentLoading,
    setAgentLoading,
    sessionsByAgent,
    isConversationVisible,
    setConversationVisible,
    appendMessage,
    scrollToBottom,
    sendMessage,
    sendMessageToAgent, // 新增：发送到指定智能体
    cancelAgentExecution, // 新增：终止智能体执行
    openConversationPanel,
    closeConversationPanel,
    saveMessagesToIpfs,
    loadMessagesFromIpfs,
  }
}
