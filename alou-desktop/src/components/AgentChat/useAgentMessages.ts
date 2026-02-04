import { useCallback, useState, useMemo, useEffect, useRef, RefObject } from 'react'
import apiClient from '@/services/api'
import agentService from '@/services/agentService'
import useAgentStore from '@/stores/agentStore'
import { getToolCategoriesByMode, getToolsByCategories } from './agentUtils'
import { getSystemPromptForAgent } from './utils/agentPrompts'
import { getMessageHistory } from './utils/messageUtils'
import { useAsyncTaskPolling } from './hooks/useAsyncTaskPolling'
import { useAgentCreation } from './hooks/useAgentCreation'
import LoadingIcon from '@/assets/加载0.2.png'
import type { ClusterActionStore } from '@/stores/clusterActionStore.types'

// 消息类型
export interface Message {
  id: string
  type: 'user' | 'assistant' | 'system' | 'error'
  content: string
  timestamp: number
  source?: string
  agentId?: string
  isLoading?: boolean
  metadata?: Record<string, unknown>
}

// 群聊消息类型
export interface GroupChatMessage {
  groupId?: string
  fromName?: string
  content?: string
  text?: string
  timestamp?: number
}

// Hook参数类型
export interface UseAgentMessagesProps {
  sessionId: string
  setSessionId: (id: string) => void
  activeChain?: string
  activeChannelId: string | null
  selectedAgent: Agent | null
  isSessionReady: boolean
  createSession: () => Promise<void>
  setSessionReady: (ready: boolean) => void
  recordInteraction: (type: string, data: unknown) => void
  handleToolCalls: (toolCalls: ToolCall[]) => Promise<void>
  conversationOverlayRef: RefObject<{ scrollToBottom: () => void } | null>
  consoleDockRef: RefObject<{ adjustInputHeight: () => void } | null>
  contextEventsRef: RefObject<unknown[]>
  currentMode?: 'agent' | 'alou'
  onRateLimitExceeded?: (data: { remainingRequests: number; resetTime: string | null }) => void
  onCreateAgent?: () => Promise<void>
  onAutoCreateAgent?: (agentInfo: AgentInfo) => Promise<void>
}

// 智能体类型
export interface Agent {
  id: string
  display_name?: string
  name?: string
  role_description?: string
  customPrompt?: string
  customInstructions?: string
  did?: string
  ipns?: string
  mcpTools?: string[]
  mcp_tools?: string[]
  avatar?: string
  model?: string
  maxTokens?: number
  temperature?: number
  metadata?: Record<string, unknown>
}

// 智能体信息类型
export interface AgentInfo {
  name: string
  role_description?: string
  customPrompt?: string
  [key: string]: unknown
}

// 工具调用类型
export interface ToolCall {
  name: string
  arguments: Record<string, unknown>
}

// API响应类型
interface ApiResponse {
  task_id?: string
  taskId?: string
  toolCalls?: ToolCall[]
  tool_calls?: ToolCall[]
  response?: string
  content?: string
  timestamp?: number
  source?: string
  session_id?: string
}

// 错误响应类型
interface ApiError {
  response?: {
    status: number
    data: {
      error?: string
      remaining_requests?: number
      reset_time?: string | null
    } | string
  }
  name?: string
}

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
  // isSessionReady,  // Currently unused but kept for future use
  createSession,
  setSessionReady,
  recordInteraction,
  handleToolCalls,
  conversationOverlayRef,
  consoleDockRef,
  contextEventsRef,
  currentMode = 'agent',
  onRateLimitExceeded,
  onCreateAgent,
  onAutoCreateAgent,
}: UseAgentMessagesProps) => {
  // 按频道存储消息：Map<channelId, Message[]>
  const [messagesByChannel, setMessagesByChannel] = useState<Record<string, Message[]>>({})
  const [currentMessage, setCurrentMessage] = useState('')
  // 全局 loading 状态（向后兼容）
  const [isLoading, setIsLoading] = useState(false)
  // 按智能体存储 loading 状态：Map<channelId, boolean>
  const [loadingByAgent, setLoadingByAgent] = useState<Record<string, boolean>>({})
  // 按智能体存储 session：Map<channelId, sessionId>
  const [sessionsByAgent, setSessionsByAgent] = useState<Record<string, string>>({})
  const [isConversationVisible, setConversationVisible] = useState(false)
  
  // 跟踪已保存过的消息数量，避免重复保存
  const savedMessageCountRef = useRef<Record<string, number>>({})
  
  // 按智能体存储 AbortController：Map<channelId, AbortController>
  const abortControllersByAgent = useRef<Record<string, AbortController>>({})
  
  // 获取 agentStore 方法
  const updateAgent = useAgentStore((state) => state.updateAgent)
  
  // 获取指定智能体的 loading 状态
  const isAgentLoading = useCallback((agentId: string) => {
    return loadingByAgent[agentId] || false
  }, [loadingByAgent])
  
  // 设置指定智能体的 loading 状态
  const setAgentLoading = useCallback((agentId: string, loading: boolean) => {
    setLoadingByAgent(prev => ({ ...prev, [agentId]: loading }))
    // 同时更新全局 loading（如果是当前活动智能体）
    if (agentId === activeChannelId) {
      setIsLoading(loading)
    }
  }, [activeChannelId])

  // 当前频道的消息
  const messages = useMemo(() => {
    return messagesByChannel[activeChannelId || ''] || []
  }, [messagesByChannel, activeChannelId])

  // 添加消息到指定频道
  const appendMessage = useCallback(
    (message: Message, channelId: string | null = activeChannelId) => {
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
  const setMessagesForChannel = useCallback((channelId: string, msgs: Message[]) => {
    if (!channelId) return
    
    setMessagesByChannel((prev) => ({
      ...prev,
      [channelId]: msgs,
    }))
    
    // 更新已保存计数
    savedMessageCountRef.current[channelId] = msgs.length
  }, [])

  // 清空指定频道的消息
  const clearMessagesForChannel = useCallback((channelId: string) => {
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
    appendMessage: (message: unknown, agentId: string) => appendMessage(message as Message, agentId),
    scrollToBottom,
    setAgentLoading,
    setMessagesByChannel: setMessagesByChannel as React.Dispatch<React.SetStateAction<Record<string, unknown[]>>>,
  })

  const { parseAgentCreationCommandWithAIDirect } = useAgentCreation({
    appendMessage: (message: string | Message) => appendMessage(message as Message),
  })

  // Suppress unused variable warning
  void parseAgentCreationCommandWithAIDirect

  // 保存消息到 IPFS
  const saveMessagesToIpfs = useCallback(async (channelId: string, agentId: string, force = false): Promise<string | null> => {
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

      const cid = await agentService.uploadMessagesToIpfs?.(newMessages, agentId)

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
  const loadMessagesFromIpfs = useCallback(async (channelId: string, messagesCid: string): Promise<boolean> => {
    if (!channelId || !messagesCid) return false
    
    try {
      console.log(`[useAgentMessages] 从 IPFS 加载消息，CID: ${messagesCid}`)

      const data = await agentService.loadMessagesFromIpfs?.(messagesCid)

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
  const sendMessageToAgent = useCallback(async (targetAgentId: string, text: string, targetAgent: Agent | null = null) => {
    if (!text?.trim() || !targetAgentId) {
      console.warn('[useAgentMessages] 无法发送消息：缺少文本或目标智能体')
      return
    }

    // 检查该智能体是否正在执行
    if (loadingByAgent[targetAgentId]) {
      console.log('[useAgentMessages] 智能体正在执行中，跳过:', targetAgentId)
      return
    }

    const userMessage: Message = {
      id: `user_${Date.now()}_${targetAgentId}`,
      type: 'user',
      content: String(text).trim(),
      timestamp: Date.now(),
    }

    appendMessage(userMessage, targetAgentId)
    setAgentLoading(targetAgentId, true)
    recordInteraction('user_message', { content: text, agentId: targetAgentId })
    scrollToBottom()

    const contextSnapshot = contextEventsRef.current?.splice(0, contextEventsRef.current.length) || []

    // 直接调用后端 API 进行单智能体聊天
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
        systemPrompt: systemPrompt,
        history: getMessageHistory(targetAgentId, messagesByChannel),
        agentInfo: {
          session_id: agentSessionId,
          wallet_address: walletAddress || null,
          chain: activeChain || undefined,
          context_events: contextSnapshot,
          agent_id: targetAgentId,
          name: agentInfo?.display_name || agentInfo?.name,
          role_description: agentInfo?.role_description,
          custom_prompt: agentInfo?.customPrompt,
          mode: currentMode,
          custom_instructions: agentInfo?.customInstructions,
          did: agentInfo?.did,
          ipns: agentInfo?.ipns,
          mcp_tools: agentInfo?.mcpTools || agentInfo?.mcp_tools || [],
          allowed_tools: tools.map((t: { name: string }) => t.name),
          ...(agentInfo?.metadata || {}),
        },
        tools: tools,
        model: agentInfo?.model || "deepseek-chat",
        maxTokens: agentInfo?.maxTokens || 4096,
        temperature: agentInfo?.temperature || 0.7,
        taskType: "async",
        timeout: 30000,
      }

      const data: ApiResponse = await apiClient
        .post('/ai-task/init-and-start', claudeSdkRequest, {
          signal: abortController.signal,
        })
        .then((response) => response.data)

      // 检查是否为异步任务（包含 task_id）
      const taskId = data.task_id || data.taskId
      
      if (taskId) {
        // 是异步任务，启动轮询
        console.log(`[useAgentMessages] 检测到异步任务: ${taskId}`)
        
        // 显示加载消息
        const loadingMessage: Message = {
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
        const toolCalls = data.toolCalls || data.tool_calls || []
        const content = data.response || data.content || ""
        
        if (toolCalls.length > 0) {
          await handleToolCalls(toolCalls)
        }

        const assistantMessage: Message = {
          id: `assistant_${Date.now()}_${targetAgentId}`,
          type: 'assistant',
          content: String(content || '收到响应'),
          timestamp: data.timestamp || Date.now(),
          source: data.source || 'alou-edge',
          agentId: targetAgentId,
        }
        appendMessage(assistantMessage, targetAgentId)

        if (data.session_id) {
          setSessionsByAgent(prev => ({ ...prev, [targetAgentId]: data.session_id! }))
          if (targetAgentId === activeChannelId) {
            setSessionId(data.session_id!)
          }
        }
      }
    } catch (error) {
      const err = error as ApiError
      // 如果是用户主动取消，不显示错误消息
      if (err.name === 'AbortError' || err.name === 'CanceledError') {
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
      const statusCode = err?.response?.status
      
      // 尝试从响应中提取详细错误信息
      let detailedError = errorMessage
      if (err?.response?.data) {
        const errorData = err.response.data
        if (typeof errorData === 'object' && errorData.error) {
          detailedError = errorData.error
        } else if (typeof errorData === 'string') {
          detailedError = errorData
        }
      }
      
      console.error('[useAgentMessages] 后端 API 错误:', {
        status: statusCode,
        message: errorMessage,
        detailedError,
        response: err?.response?.data,
      })
      
      // 处理 429 错误（限额超限）
      if (statusCode === 429) {
        const errorData = err?.response?.data as { remaining_requests?: number; reset_time?: string | null } | undefined
        const remainingRequests = errorData?.remaining_requests ?? 0
        const resetTime = errorData?.reset_time ?? null
        
        if (onRateLimitExceeded) {
          onRateLimitExceeded({
            remainingRequests,
            resetTime,
          })
        }
        return
      }
      
      let friendlyMessage = `❌ 抱歉，发生了错误：${detailedError}`
      if (statusCode === 404) {
        friendlyMessage = '❌ 会话已过期，请刷新页面重试。'
        setSessionReady(false)
      } else if (statusCode === 500) {
        friendlyMessage = `❌ 服务器内部错误：${detailedError}\n\n请检查后端服务是否正常运行，或查看控制台获取更多信息。`
      } else if (!err?.response) {
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
        const agentInfo = await parseAgentCreationCommandWithAIDirect(text)
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
            await onAutoCreateAgent(agentInfo)
            console.log('[useAgentMessages] 智能体自动创建命令已处理')
            
            appendMessage({
              id: `system_${Date.now()}_success`,
              type: 'assistant',
              content: `✅ 智能体 "${agentInfo.name}" 创建成功！已添加到频道栏。`,
              timestamp: Date.now(),
              source: 'system',
            }, 'system')
          } catch (error) {
            console.error('[useAgentMessages] 自动创建智能体失败:', error)
            appendMessage({
              id: `system_${Date.now()}_error`,
              type: 'assistant',
              content: `❌ 创建智能体失败: ${(error as Error).message || '未知错误'}`,
              timestamp: Date.now(),
              source: 'system',
            }, 'system')
          }
        } else if (onCreateAgent) {
          try {
            await onCreateAgent()
            console.log('[useAgentMessages] 智能体创建命令已处理（打开模态框）')
          } catch (error) {
            console.error('[useAgentMessages] 打开创建模态框失败:', error)
          }
        } else {
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
    parseAgentCreationCommandWithAIDirect,
  ])

  const openConversationPanel = useCallback(() => {
    setConversationVisible(true)
    scrollToBottom()
  }, [scrollToBottom])

  const closeConversationPanel = useCallback(() => {
    setConversationVisible(false)
  }, [])

  // 当关闭对话面板或切换频道时，保存消息到 IPFS
  const previousChannelRef = useRef<string | null>(activeChannelId)
  
  useEffect(() => {
    const prevChannel = previousChannelRef.current
    
    // 如果频道发生变化，保存之前频道的消息
    if (prevChannel && prevChannel !== activeChannelId) {
      const prevMessages = messagesByChannel[prevChannel] || []
      const savedCount = savedMessageCountRef.current[prevChannel] || 0
      
      if (prevMessages.length > savedCount) {
        const agentId = prevChannel
        saveMessagesToIpfs(prevChannel, agentId).catch((err: Error) => {
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
      
      if (channelMessages.length > savedCount) {
        console.log(`[useAgentMessages] 自动保存 ${channelMessages.length - savedCount} 条新消息`)
        saveMessagesToIpfs(activeChannelId, selectedAgent.id).catch((err: Error) => {
          console.error('[useAgentMessages] 自动保存失败:', err)
        })
      }
    }, 15 * 60 * 1000)
    
    return () => clearInterval(autoSaveInterval)
  }, [activeChannelId, selectedAgent, messagesByChannel, saveMessagesToIpfs])

  // 监听消息数量变化，达到阈值时自动保存（每10条新消息）
  useEffect(() => {
    if (!activeChannelId || !selectedAgent) return
    
    const channelMessages = messagesByChannel[activeChannelId] || []
    const savedCount = savedMessageCountRef.current[activeChannelId] || 0
    const newMessageCount = channelMessages.length - savedCount
    
    if (newMessageCount >= 10) {
      console.log(`[useAgentMessages] 检测到 ${newMessageCount} 条新消息，触发自动保存`)
      saveMessagesToIpfs(activeChannelId, selectedAgent.id).catch((err: Error) => {
        console.error('[useAgentMessages] 阈值保存失败:', err)
      })
    }
  }, [messagesByChannel, activeChannelId, selectedAgent, saveMessagesToIpfs])
  

  // 监听群聊消息事件
  useEffect(() => {
    const handleAgentGroupMessage = (event: Event) => {
      const customEvent = event as CustomEvent<{ agentId: string; message: GroupChatMessage }>
      const { agentId, message } = customEvent.detail
      
      if (selectedAgent && selectedAgent.id === agentId) {
        console.log('[useAgentMessages] 智能体收到群聊消息:', agentId, message)
        
        const agentMessage: Message = {
          id: `group_msg_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`,
          type: 'user',
          content: message.content || message.text || '',
          timestamp: message.timestamp || Date.now(),
          source: 'group-chat',
          metadata: {
            groupId: message.groupId,
            fromName: message.fromName,
            originalMessage: message,
            isGroupChatMessage: true,
          }
        }
        
        appendMessage(agentMessage, agentId)
        
        if (!isAgentLoading(agentId)) {
          setTimeout(() => {
            console.log('[useAgentMessages] 触发智能体处理群聊消息:', agentId)
          }, 100)
        }
      }
    }
    
    window.addEventListener('agent-group-message', handleAgentGroupMessage)
    
    return () => {
      window.removeEventListener('agent-group-message', handleAgentGroupMessage)
    }
  }, [selectedAgent, appendMessage, isAgentLoading])

  // 监听智能体消息变化，如果是群聊消息的回复，则同步到群聊
  useEffect(() => {
    if (!selectedAgent || !messages.length) return

    const lastMessage = messages[messages.length - 1]
    
    if (lastMessage && 
        lastMessage.type === 'assistant' && 
        lastMessage.metadata?.isGroupChatMessage) {
      
      const groupId = lastMessage.metadata.groupId as string
      if (groupId) {
        console.log('[useAgentMessages] 智能体回复群聊消息，同步到群聊:', groupId, lastMessage)
        
        try {
          const store = require('@/stores/clusterActionStore') as { default: ClusterActionStore }
          const { addGroupChatMessage, getState } = store.default
          const state = getState()
          const groupReplyMessage = {
            id: `agent_reply_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`,
            type: 'agent' as const,
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
  const cancelAgentExecution = useCallback((agentId: string) => {
    const controller = abortControllersByAgent.current[agentId]
    if (controller) {
      console.log('[useAgentMessages] 终止智能体HTTP请求:', agentId)
      controller.abort()
      delete abortControllersByAgent.current[agentId]
    }

    cancelPolling(agentId)
    setAgentLoading(agentId, false)

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
    setMessages: (msgs: Message[]) => {
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
    loadingByAgent,
    isAgentLoading,
    setAgentLoading,
    sessionsByAgent,
    isConversationVisible,
    setConversationVisible,
    appendMessage,
    scrollToBottom,
    sendMessage,
    sendMessageToAgent,
    cancelAgentExecution,
    openConversationPanel,
    closeConversationPanel,
    saveMessagesToIpfs,
    loadMessagesFromIpfs,
  }
}
