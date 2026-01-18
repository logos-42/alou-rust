import { useCallback, useState, useMemo, useEffect, useRef } from 'react'
import apiClient from '@/services/api'
import agentService from '@/services/agentService'
import useAgentStore from '@/stores/agentStore'
import { getToolCategoriesByMode, getToolsByCategories } from './agentUtils'
import { getSystemPromptForAgent } from './utils/agentPrompts'
import { getMessageHistory } from './utils/messageUtils'
import { useAsyncTaskPolling } from './hooks/useAsyncTaskPolling'
import { useAgentCreation } from './hooks/useAgentCreation'
import LoadingIcon from '@/assets/加载0.2.png'

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
  onRateLimitExceeded, // 回调函数：当遇到 429 错误时调用
  onCreateAgent, // 新增：创建智能体的回调函数
  onAutoCreateAgent, // 新增：自动创建智能体的回调函数
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

  // 子Hook将在所有依赖函数定义之后调用

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

  // 使用子Hook（在所有依赖函数定义之后）
  const { pollAsyncTask, cancelPolling } = useAsyncTaskPolling({
    appendMessage,
    scrollToBottom,
    setAgentLoading,
    setMessagesByChannel,
  })

  const { parseAgentCreationCommandWithAI } = useAgentCreation({
    appendMessage,
  })

  // 保存消息到 IPFS
  const saveMessagesToIpfs = useCallback(async (channelId, agentId, force = false) => {
    const channelMessages = messagesByChannel[channelId] || []
    const savedCount = savedMessageCountRef.current[channelId] || 0
    
    // 检查是否需要保存
    const hasNewMessages = channelMessages.length > savedCount
    const shouldSave = force || hasNewMessages
    
    if (!shouldSave || channelMessages.length === 0) {
      return null
    }
    
    try {
      console.log(`[useAgentMessages] 保存 ${channelMessages.length} 条消息到 IPFS，频道: ${channelId}，新消息: ${hasNewMessages}`)
      
      // 只保存新消息（增量保存）
      const newMessages = hasNewMessages 
        ? channelMessages.slice(savedCount)
        : channelMessages
      
      const cid = await agentService.uploadMessagesToIpfs(newMessages, agentId)
      
      // 更新 agentStore 中的 messages_cid
      if (cid && agentId) {
        updateAgent(agentId, { 
          messages_cid: cid,
          last_saved_at: Date.now()
        })
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
      content: String(text).trim(),
      timestamp: Date.now(),
    }

    appendMessage(userMessage, targetAgentId)
    setAgentLoading(targetAgentId, true)
    recordInteraction('user_message', { content: text, agentId: targetAgentId })
    scrollToBottom()

    const contextSnapshot = contextEventsRef.current.splice(0, contextEventsRef.current.length)

    // 直接调用后端 API 进行单智能体聊天
    // 注意：多智能体协作（集群行动）只在用户主动创建邀请时创建（见 useAgentInvite.js）
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
      
      // 构建 Claude SDK 格式的请求
      console.log('[sendMessageToAgent] 构建请求，参数:', {
        currentMode,
        hasAgentInfo: !!agentInfo,
        walletAddress,
        activeChain,
        agentInfoKeys: agentInfo ? Object.keys(agentInfo) : []
      });
      
      // 使用原有的系统提示词生成逻辑
      const systemPrompt = getSystemPromptForAgent(agentInfo, currentMode, walletAddress, activeChain);
      console.log('[sendMessageToAgent] 生成的 systemPrompt:', systemPrompt ? `有内容，长度: ${systemPrompt.length}` : 'undefined');
      
      // 获取工具配置（但不覆盖系统提示词）
      const toolCategories = getToolCategoriesByMode(currentMode, agentInfo);
      console.log('[sendMessageToAgent] 工具类别:', toolCategories);
      
      // 只获取工具配置，不生成新的系统提示词
      const tools = getToolsByCategories(toolCategories);
      console.log('[sendMessageToAgent] 可用工具数量:', tools.length);
      
      const claudeSdkRequest = {
        apiKey: "alou-backend-default-token",
        prompt: text.trim(),
        systemPrompt: systemPrompt, // 使用原有的系统提示词
        history: getMessageHistory(targetAgentId, messagesByChannel), // 获取消息历史
        agentInfo: {
          session_id: agentSessionId,
          wallet_address: walletAddress || undefined,
          chain: activeChain || undefined,
          context_events: contextSnapshot,
          agent_id: targetAgentId,
          name: agentInfo?.display_name || agentInfo?.name,
          role_description: agentInfo?.role_description,
          custom_prompt: agentInfo?.customPrompt,
          mode: currentMode,
          // 添加其他可选字段
          custom_instructions: agentInfo?.customInstructions,
          did: agentInfo?.did,
          ipns: agentInfo?.ipns,
          // MCP 工具配置
          mcp_tools: agentInfo?.mcpTools || agentInfo?.mcp_tools || [],
          // 工具配置
          allowed_tools: tools.map(t => t.name),
          // 其他自定义配置
          ...(agentInfo?.metadata || {}),
        },
        tools: tools, // 添加工具配置
        model: agentInfo?.model || "deepseek-chat",
        maxTokens: agentInfo?.maxTokens || 4096,
        temperature: agentInfo?.temperature || 0.7,
        // 添加其他可选参数
        taskType: "async", // 异步任务 - 强制使用异步模式以支持工具循环
        timeout: 30000, // 30秒超时
      }

      const data = await apiClient
        .post('/ai-task/init-and-start', claudeSdkRequest, {
          signal: abortController.signal, // 添加 abort signal 用于终止请求
        })
        .then((response) => response.data)

      // 检查是否为异步任务（包含 task_id）
      const taskId = data.task_id || data.taskId
      
      if (taskId) {
        // 是异步任务，启动轮询
        console.log(`[useAgentMessages] 检测到异步任务: ${taskId}`)
        
        // 显示加载消息
        const loadingMessage = {
          id: `task_${taskId}`,
          type: 'assistant',
          content: `<img src="${LoadingIcon}" alt="加载中" class="loading-icon" /> 正在处理中...`,
          timestamp: Date.now(),
          source: 'task-progress',
          agentId: targetAgentId,
          isLoading: true,
        }
        appendMessage(loadingMessage, targetAgentId)
        
        // 启动轮询
        pollAsyncTask(taskId, loadingMessage.id, targetAgentId)
      } else {
        // 同步任务，直接处理响应
        // 处理 Claude SDK 响应格式
        const toolCalls = data.toolCalls || data.tool_calls || []
        const content = data.response || data.content || ""
        
        if (toolCalls.length > 0) {
          await handleToolCalls(toolCalls)
        }

        const assistantMessage = {
          id: `assistant_${Date.now()}_${targetAgentId}`,
          type: 'assistant',
          content: String(content || '收到响应'),
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
      
      // 尝试从响应中提取详细错误信息
      let detailedError = errorMessage
      if (error?.response?.data) {
        const errorData = error.response.data
        if (errorData.error) {
          detailedError = errorData.error
        } else if (typeof errorData === 'string') {
          detailedError = errorData
        }
      }
      
      console.error('[useAgentMessages] 后端 API 错误:', {
        status: statusCode,
        message: errorMessage,
        detailedError,
        response: error?.response?.data,
      })
      
      // 处理 429 错误（限额超限）
      if (statusCode === 429) {
        const errorData = error?.response?.data
        const remainingRequests = errorData?.remaining_requests ?? 0
        const resetTime = errorData?.reset_time ?? null
        
        // 调用回调函数显示弹窗
        if (onRateLimitExceeded) {
          onRateLimitExceeded({
            remainingRequests,
            resetTime,
          })
        }
        
        // 不添加错误消息到对话中，因为已经有弹窗了
        return
      }
      
      let friendlyMessage = `❌ 抱歉，发生了错误：${detailedError}`
      if (statusCode === 404) {
        friendlyMessage = '❌ 会话已过期，请刷新页面重试。'
        setSessionReady(false)
      } else if (statusCode === 500) {
        friendlyMessage = `❌ 服务器内部错误：${detailedError}\n\n请检查后端服务是否正常运行，或查看控制台获取更多信息。`
      } else if (!error?.response) {
        friendlyMessage = '❌ 无法连接到服务器，请检查网络连接。'
      }
      
      appendMessage({
        id: `error_${Date.now()}`,
        type: 'assistant',
        content: String(friendlyMessage),
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
    onRateLimitExceeded,
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

    // 如果没有活动频道，检查是否为创建智能体的命令
    if (!activeChannelId) {
      const createCommands = ['创建智能体', '新建智能体', 'create agent', 'new agent', '/create', '/new']
      const isCreateCommand = createCommands.some(cmd => 
        text.toLowerCase().includes(cmd.toLowerCase())
      )
      
      if (isCreateCommand) {
        console.log('[useAgentMessages] 检测到创建智能体命令:', text)
        setCurrentMessage('')
        
        // 解析指令并生成智能体信息
        const agentInfo = await parseAgentCreationCommandWithAI(text)
        console.log('[useAgentMessages] 解析的智能体信息:', agentInfo)
        
        // 显示创建中的消息
        appendMessage({
          id: `system_${Date.now()}`,
          type: 'assistant',
          content: `🔄 正在创建智能体 "${agentInfo.name}"...`,
          timestamp: Date.now(),
          source: 'system',
        }, 'system')
        
        // 调用自动创建智能体的逻辑
        if (onAutoCreateAgent) {
          try {
            // 调用自动创建智能体回调
            await onAutoCreateAgent(agentInfo)
            console.log('[useAgentMessages] 智能体自动创建命令已处理')
            
            // 显示创建成功消息
            appendMessage({
              id: `system_${Date.now()}_success`,
              type: 'assistant',
              content: `✅ 智能体 "${agentInfo.name}" 创建成功！已添加到频道栏。`,
              timestamp: Date.now(),
              source: 'system',
            }, 'system')
          } catch (error) {
            console.error('[useAgentMessages] 自动创建智能体失败:', error)
            // 显示错误消息
            appendMessage({
              id: `system_${Date.now()}_error`,
              type: 'assistant',
              content: `❌ 创建智能体失败: ${error.message || '未知错误'}`,
              timestamp: Date.now(),
              source: 'system',
            }, 'system')
          }
        } else if (onCreateAgent) {
          // 如果没有自动创建回调，但有创建回调，则打开模态框
          try {
            await onCreateAgent()
            console.log('[useAgentMessages] 智能体创建命令已处理（打开模态框）')
          } catch (error) {
            console.error('[useAgentMessages] 打开创建模态框失败:', error)
          }
        } else {
          // 如果没有提供任何创建回调，显示提示
          setTimeout(() => {
            appendMessage({
              id: `system_${Date.now()}_2`,
              type: 'assistant',
              content: '⚠️ 智能体创建功能需要从左侧"+"按钮启动。请点击左侧的"+"按钮创建智能体。',
              timestamp: Date.now(),
              source: 'system',
            }, 'system')
          }, 1000)
        }
        
        return
      }
      
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
    appendMessage,
    onCreateAgent,
    onAutoCreateAgent,
    parseAgentCreationCommandWithAI,
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

  // 自动定期保存消息到 IPFS（每15分钟）
  useEffect(() => {
    if (!activeChannelId || !selectedAgent) return
    
    const autoSaveInterval = setInterval(() => {
      const channelMessages = messagesByChannel[activeChannelId] || []
      const savedCount = savedMessageCountRef.current[activeChannelId] || 0
      
      // 检查是否有新消息需要保存
      if (channelMessages.length > savedCount) {
        console.log(`[useAgentMessages] 自动保存 ${channelMessages.length - savedCount} 条新消息`)
        saveMessagesToIpfs(activeChannelId, selectedAgent.id).catch(err => {
          console.error('[useAgentMessages] 自动保存失败:', err)
        })
      }
    }, 15 * 60 * 1000) // 每15分钟
    
    return () => clearInterval(autoSaveInterval)
  }, [activeChannelId, selectedAgent, messagesByChannel, saveMessagesToIpfs])

  // 监听消息数量变化，达到阈值时自动保存（每10条新消息）
  useEffect(() => {
    if (!activeChannelId || !selectedAgent) return
    
    const channelMessages = messagesByChannel[activeChannelId] || []
    const savedCount = savedMessageCountRef.current[activeChannelId] || 0
    const newMessageCount = channelMessages.length - savedCount
    
    // 当新消息达到10条时自动保存
    if (newMessageCount >= 10) {
      console.log(`[useAgentMessages] 检测到 ${newMessageCount} 条新消息，触发自动保存`)
      saveMessagesToIpfs(activeChannelId, selectedAgent.id).catch(err => {
        console.error('[useAgentMessages] 阈值保存失败:', err)
      })
    }
  }, [messagesByChannel, activeChannelId, selectedAgent, saveMessagesToIpfs])
  

  // 监听群聊消息事件
  useEffect(() => {
    const handleAgentGroupMessage = (event) => {
      const { agentId, message } = event.detail
      
      // 只处理当前选中的智能体的群聊消息
      if (selectedAgent && selectedAgent.id === agentId) {
        console.log('[useAgentMessages] 智能体收到群聊消息:', agentId, message)
        
        // 将群聊消息转换为智能体消息格式
        const agentMessage = {
          id: `group_msg_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`,
          type: 'user', // 群聊用户消息作为用户输入处理
          content: message.content || message.text || '',
          timestamp: message.timestamp || Date.now(),
          source: 'group-chat',
          metadata: {
            groupId: message.groupId,
            fromName: message.fromName,
            originalMessage: message,
            isGroupChatMessage: true, // 标记这是群聊消息
          }
        }
        
        // 添加到智能体的消息列表中
        appendMessage(agentMessage, agentId)
        
        // 触发智能体处理消息（如果智能体处于空闲状态）
        if (!isAgentLoading(agentId)) {
          // 延迟一下，确保消息完全添加后再触发处理
          setTimeout(() => {
            // 这里可以触发智能体的自动回复逻辑
            console.log('[useAgentMessages] 触发智能体处理群聊消息:', agentId)
          }, 100)
        }
      }
    }
    
    // 添加事件监听器
    window.addEventListener('agent-group-message', handleAgentGroupMessage)
    
    // 清理函数
    return () => {
      window.removeEventListener('agent-group-message', handleAgentGroupMessage)
    }
  }, [selectedAgent, appendMessage, isAgentLoading])

  // 监听智能体消息变化，如果是群聊消息的回复，则同步到群聊
  useEffect(() => {
    if (!selectedAgent || !messages.length) return

    const lastMessage = messages[messages.length - 1]
    
    // 检查是否是智能体对群聊消息的回复
    if (lastMessage && 
        lastMessage.type === 'assistant' && 
        lastMessage.metadata?.isGroupChatMessage) {
      
      const groupId = lastMessage.metadata.groupId
      if (groupId) {
        console.log('[useAgentMessages] 智能体回复群聊消息，同步到群聊:', groupId, lastMessage)
        
        // 将智能体回复同步到群聊
        try {
          const { addGroupChatMessage } = require('@/stores/clusterActionStore').default.getState()
          const groupReplyMessage = {
            id: `agent_reply_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`,
            type: 'agent',
            from: selectedAgent.id,
            fromName: selectedAgent.display_name || selectedAgent.name || '智能体',
            avatar: selectedAgent.avatar,
            content: lastMessage.content,
            timestamp: lastMessage.timestamp,
            metadata: {
              agentId: selectedAgent.id,
              replyTo: lastMessage.metadata.originalMessage,
              isReply: true
            }
          }
          
          addGroupChatMessage(groupId, groupReplyMessage)
        } catch (error) {
          console.warn('[useAgentMessages] 同步智能体回复到群聊失败:', error)
        }
      }
    }
  }, [messages, selectedAgent])

  
  /**
   * 终止指定智能体的执行
   */
  const cancelAgentExecution = useCallback((agentId) => {
    // 终止HTTP请求
    const controller = abortControllersByAgent.current[agentId]
    if (controller) {
      console.log('[useAgentMessages] 终止智能体HTTP请求:', agentId)
      controller.abort()
      delete abortControllersByAgent.current[agentId]
    }

    // 终止轮询（通过子Hook）
    cancelPolling(agentId)

    setAgentLoading(agentId, false)

    // 添加终止消息
    appendMessage({
      id: `cancel_${Date.now()}`,
      type: 'assistant',
      content: '⏹️ 执行已终止',
      timestamp: Date.now(),
      source: 'cancel',
    }, agentId)
  }, [appendMessage, setAgentLoading, cancelPolling])

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
