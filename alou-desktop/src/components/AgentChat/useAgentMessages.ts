import { useCallback, useState, useMemo, useEffect, useRef, RefObject } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import agentService from '@/services/agentService'
import { getActiveApiConfig } from '@/hooks/useApiConfig'
import useAgentStore from '@/stores/agentStore'
import { getSystemPromptForAgent } from './utils/agentPrompts'
import { getMessageHistory } from './utils/messageUtils'
import { useAsyncTaskPolling } from './hooks/useAsyncTaskPolling'
import { useAgentCreation } from './hooks/useAgentCreation'
import type { ClusterActionStore } from '@/stores/clusterActionStore.types'

// ── Tauri 进度事件类型 ──────────────────────────────────────────
interface AgentProgressPayload {
  type: 'started' | 'thinking' | 'tool_calling' | 'tool_done' | 'tools_pending' | 'completed' | 'failed'
  task_id: string
  // thinking
  content?: string
  // tool_calling
  tool_name?: string
  // tool_done
  success?: boolean
  preview?: string
  error?: string
  // tools_pending
  count?: number
  // completed
  result?: string
}

/** 将进度事件转换为可读的中文消息 */
function progressToText(e: AgentProgressPayload): string | null {
  switch (e.type) {
    case 'started':
      return '🚀 开始执行任务...'
    case 'thinking':
      // 只在有工具调用中间过程时显示思考内容（最终回复由主流程处理）
      return e.content ? `💭 ${e.content}` : null
    case 'tools_pending':
      return `🔧 准备调用 ${e.count} 个工具...`
    case 'tool_calling':
      return `⚙️ 调用工具：**${e.tool_name}**`
    case 'tool_done':
      if (e.success) {
        const preview = e.preview ? `\n\`\`\`\n${e.preview.slice(0, 200)}\n\`\`\`` : ''
        return `✅ 工具 **${e.tool_name}** 完成${preview}`
      } else {
        return `❌ 工具 **${e.tool_name}** 失败：${e.error || '未知错误'}`
      }
    case 'failed':
      return `❌ 任务失败：${e.error || '未知错误'}`
    default:
      return null
  }
}

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
  
  // 获取钱包地址（在组件级别获取，供多个函数使用）
  const walletAddress = typeof window !== 'undefined' ? localStorage.getItem('wallet_address') : null
  
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
  // 当没有选中智能体时，显示 'welcome' 虚拟频道的消息（用于对话式创建流程）
  const messages = useMemo(() => {
    if (activeChannelId) {
      return messagesByChannel[activeChannelId] || []
    }
    return messagesByChannel['welcome'] || []
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
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  const { pollAsyncTask: _pollAsyncTask, cancelPolling } = useAsyncTaskPolling({
    appendMessage: (message: unknown, agentId: string) => appendMessage(message as Message, agentId),
    scrollToBottom,
    setAgentLoading,
    setMessagesByChannel: setMessagesByChannel as React.Dispatch<React.SetStateAction<Record<string, unknown[]>>>,
    walletAddress,
    chain: activeChain,
  })

  const { parseAgentCreationCommandWithAIDirect } = useAgentCreation({
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    appendMessage: (message: any) => appendMessage(message as Message),
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

      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const cid = await agentService.uploadMessagesToIpfs?.(newMessages as any[], agentId)

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
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        setMessagesForChannel(channelId, data.messages as any as Message[])
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

    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    const contextSnapshot = contextEventsRef.current?.splice(0, contextEventsRef.current.length) || []
    void contextSnapshot // clear context events even though not passed to local AI

    // 直接调用后端 API 进行单智能体聊天
    try {
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

      // ── 本地 API 模式：通过 Tauri invoke 直接调用 AI，不依赖 Workers 后端 ──

      // 1. 读取本地已保存的 API 配置
      const localApiConfig = await getActiveApiConfig()

      if (!localApiConfig) {
        // 未配置 API Key，提示用户去配置
        appendMessage({
          id: `no_api_${Date.now()}`,
          type: 'assistant',
          content: '⚙️ **请先配置 API Key**\n\n点击右上角设置图标 → API 配置，填入你的 API Key（支持 DeepSeek / OpenAI / Claude / Kimi），然后即可直接使用工具和自主循环。',
          timestamp: Date.now(),
          source: 'system',
          agentId: targetAgentId,
        }, targetAgentId)
        setAgentLoading(targetAgentId, false)
        return
      }

      // 2. 构建包含上下文的正确消息数组（system + history + 当前 user）
      const agentInfo = targetAgent || selectedAgent
      const systemPrompt = getSystemPromptForAgent(agentInfo, currentMode, walletAddress, activeChain ?? null)
      const history = getMessageHistory(targetAgentId, messagesByChannel)

      // 构建正确格式的 messages 数组发送给 Rust
      const messagesArray: Array<{ role: string; content: string }> = []
      if (systemPrompt) {
        messagesArray.push({ role: 'system', content: systemPrompt })
      }
      // 最近 10 条历史记录
      for (const m of history.slice(-10)) {
        messagesArray.push({ role: m.role, content: m.content })
      }
      // 当前用户消息
      messagesArray.push({ role: 'user', content: text.trim() })

      console.log('[useAgentMessages] 调用本地 AI，provider:', localApiConfig.provider, 'model:', localApiConfig.model, '消息数:', messagesArray.length)

      // 3. 通过 Tauri invoke 执行 AI 对话（本地 Rust 直接调用 AI API）
      // Rust 返回 { success, result: TaskFinalResult, execution_mode, timestamp }
      // TaskFinalResult = { task_id, success, result: string, error, iteration_count }
      const tauri_result = await invoke<{
        success: boolean
        result?: {
          task_id?: string
          success?: boolean
          result?: string      // AI 最终回复的文本
          error?: string
          iteration_count?: number
          // 兼容旧字段
          final_answer?: string
          content?: string
          summary?: string
        }
        execution_mode?: string
        timestamp?: number
      }>('execute_ai_conversation', {
        agentConfig: {
          id: localApiConfig.id || 'primary',
          provider: localApiConfig.provider,
          api_key: localApiConfig.api_key,
          // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
          base_url: localApiConfig.base_url != null ? localApiConfig.base_url : null,
          // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
          model: localApiConfig.model != null ? localApiConfig.model : null,
          is_active: true,
        },
        message: text.trim(),       // fallback 单条消息
        messages: messagesArray,    // 完整上下文数组（优先使用）
        options: { stream: false },
      })

      console.log('[useAgentMessages] Tauri AI 响应:', tauri_result)

      if (tauri_result?.success) {
        // 提取响应内容
        // TaskFinalResult.result 是 AI 最终回复的字符串
        const result = tauri_result.result
        const content =
          result?.result ||        // TaskFinalResult.result (主字段)
          result?.final_answer ||  // 兼容旧格式
          result?.content ||
          result?.summary ||
          '✅ 任务已完成'

        const assistantMessage: Message = {
          id: `assistant_${Date.now()}_${targetAgentId}`,
          type: 'assistant',
          content: String(content),
          timestamp: tauri_result.timestamp ? tauri_result.timestamp * 1000 : Date.now(),
          source: 'local',
          agentId: targetAgentId,
        }
        appendMessage(assistantMessage, targetAgentId)
      } else {
        appendMessage({
          id: `error_${Date.now()}`,
          type: 'assistant',
          content: '❌ AI 执行失败，请检查 API Key 是否正确，或查看日志获取详情。',
          timestamp: Date.now(),
          source: 'error',
          agentId: targetAgentId,
        }, targetAgentId)
      }
    } catch (error) {
      const err = error as { name?: string; message?: string }

      // 用户主动取消
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

      const errorMessage = err?.message || '未知错误'
      console.error('[useAgentMessages] 本地 AI 执行错误:', errorMessage)

      appendMessage({
        id: `error_${Date.now()}`,
        type: 'assistant',
        content: `❌ 执行出错：${errorMessage}\n\n请检查 API Key 配置是否正确。`,
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
    messagesByChannel,
    onRateLimitExceeded,
    recordInteraction,
    scrollToBottom,
    selectedAgent,
    sessionId,
    sessionsByAgent,
    setAgentLoading,
    setSessionId,
    setSessionReady,
    currentMode,
    walletAddress,
  ])


  // 向后兼容的 sendMessage（发送到当前活动智能体）
  // 当没有选中智能体时，任何文本消息都会触发"用对话创建智能体"流程
  const sendMessage = useCallback(async () => {
    const text = currentMessage.trim()
    if (!text || isLoading) {
      return
    }

    // ── 没有活动频道（未选中任何智能体）→ 进入对话式创建流程 ──
    if (!activeChannelId) {
      setCurrentMessage('')

      // 显示用户消息（在 'welcome' 虚拟频道，让用户感受到"有对话"）
      appendMessage({
        id: `user_${Date.now()}`,
        type: 'user',
        content: text,
        timestamp: Date.now(),
        source: 'user',
      }, 'welcome')

      // 如果有自动创建回调（父组件支持），用 AI 解析描述 → 自动创建
      if (onAutoCreateAgent) {
        appendMessage({
          id: `system_thinking_${Date.now()}`,
          type: 'system',
          content: '🤔 正在理解你的需求，准备创建智能体...',
          timestamp: Date.now(),
          source: 'system',
        }, 'welcome')

        try {
          const agentInfo = await parseAgentCreationCommandWithAIDirect(text)
          console.log('[sendMessage] 解析的智能体信息:', agentInfo)

          appendMessage({
            id: `system_creating_${Date.now()}`,
            type: 'system',
            content: `🔄 正在创建智能体 **"${agentInfo.name}"**...`,
            timestamp: Date.now(),
            source: 'system',
          }, 'welcome')

          await onAutoCreateAgent(agentInfo as unknown as AgentInfo)

          appendMessage({
            id: `system_done_${Date.now()}`,
            type: 'assistant',
            content: `✅ 智能体 **"${agentInfo.name}"** 已创建！点击左侧频道开始对话。`,
            timestamp: Date.now(),
            source: 'system',
          }, 'welcome')
        } catch (err) {
          appendMessage({
            id: `system_err_${Date.now()}`,
            type: 'assistant',
            content: `❌ 创建失败：${(err as Error).message || '未知错误'}`,
            timestamp: Date.now(),
            source: 'error',
          }, 'welcome')
        }
      } else if (onCreateAgent) {
        // 父组件只支持打开模态框
        appendMessage({
          id: `system_modal_${Date.now()}`,
          type: 'assistant',
          content: '📝 即将打开创建表单...',
          timestamp: Date.now(),
          source: 'system',
        }, 'welcome')
        try {
          await onCreateAgent()
        } catch (e) {
          console.error('[sendMessage] 打开创建模态框失败:', e)
        }
      } else {
        // 兜底：提示用户点击 "+" 按钮
        appendMessage({
          id: `system_hint_${Date.now()}`,
          type: 'assistant',
          content: '👈 点击左侧 **"+"** 按钮创建你的第一个智能体，或者试试输入：\n\n> `创建一个擅长写代码的助手`',
          timestamp: Date.now(),
          source: 'system',
        }, 'welcome')
      }

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
          const { addGroupChatMessage } = store.default
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

  // 用 ref 持有最新的 onAutoCreateAgent 回调，避免 useEffect([]) 的陈旧闭包
  // 同时防止回调变化时重新注册 Tauri listener（会导致重复监听）
  const onAutoCreateAgentRef = useRef(onAutoCreateAgent)
  useEffect(() => {
    onAutoCreateAgentRef.current = onAutoCreateAgent
  }, [onAutoCreateAgent])

  // ── 监听 Rust 发来的 agent:created 事件（agent_creator 工具创建成功后触发）────
  // Rust executor 在 agent_creator create 成功后 emit "agent:created"
  // 前端收到后调用 onAutoCreateAgent 将新 Agent 写入 Zustand 并显示在侧边栏
  useEffect(() => {
    let unlisten: (() => void) | null = null

    // cancelled flag：防止 React 18 Strict Mode 双重挂载导致注册两个监听器
    // Strict Mode: mount → cleanup(cancel) → mount。async listen 的 resolve 可能在 cleanup 之后，
    // 所以用 cancelled 在 resolve 时立即 unlisten，确保只有最新的监听器存活
    let cancelled = false

    const setupAgentCreatedListener = async () => {
      try {
        const fn = await listen<{ name: string; role_description?: string }>('agent:created', async (event) => {
          const payload = event.payload
          console.log('[useAgentMessages] 收到 agent:created 事件:', payload)

          // 通过 ref 获取最新回调，避免陈旧闭包导致 "Should have a queue" React 错误
          const cb = onAutoCreateAgentRef.current
          if (cb && payload?.name) {
            try {
              await cb({
                name: payload.name,
                // 同时传两种字段命名，兼容 useAutoAgentCreator (roleDescription) 和其他消费者 (role_description)
                role_description: payload.role_description ?? '',
                roleDescription: payload.role_description ?? '',
              })
              console.log('[useAgentMessages] Agent 已自动添加到侧边栏:', payload.name)
            } catch (err) {
              console.error('[useAgentMessages] 自动创建 Agent 失败:', err)
            }
          }
        })
        if (cancelled) {
          // Strict Mode 的第一次挂载已经被取消，立即释放
          fn()
          console.log('[useAgentMessages] agent:created listener 已取消（Strict Mode cleanup）')
        } else {
          unlisten = fn
        }
      } catch (e) {
        console.warn('[useAgentMessages] agent:created listen 不可用（非桌面环境）:', e)
      }
    }

    setupAgentCreatedListener()

    return () => {
      cancelled = true
      if (unlisten) {
        unlisten()
        unlisten = null
      }
    }
  // 只挂载一次；通过 onAutoCreateAgentRef 访问最新回调
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  // ── 监听 Rust 发来的 agent:progress 进度事件 ──────────────────────────────
  // 在智能体执行期间，Rust 会通过 AppHandle 发送工具调用进度
  // 我们在当前活动频道插入进度消息（source='progress'），让用户实时可见
  useEffect(() => {
    let unlisten: (() => void) | null = null

    const setupListener = async () => {
      try {
        unlisten = await listen<AgentProgressPayload>('agent:progress', (event) => {
          const payload = event.payload
          console.log('[useAgentMessages] 收到进度事件:', payload)

          // 只显示工具相关进度（tool_calling / tool_done / tools_pending）
          // thinking/started/completed 由主流程负责，避免重复
          const showTypes: AgentProgressPayload['type'][] = ['tools_pending', 'tool_calling', 'tool_done']
          if (!showTypes.includes(payload.type)) return

          const text = progressToText(payload)
          if (!text) return

          // 向当前活动频道插入进度消息
          // 用 activeChannelIdRef 避免闭包陈旧
          const channelId = activeChannelIdRef.current
          if (!channelId) return

          setMessagesByChannel((prev) => {
            const channelMessages = prev[channelId] || []
            const progressMsg: Message = {
              id: `progress_${Date.now()}_${Math.random().toString(36).slice(2, 6)}`,
              type: 'system',
              content: text,
              timestamp: Date.now(),
              source: 'progress',
              agentId: channelId,
              metadata: { progressType: payload.type, toolName: payload.tool_name },
            }
            return { ...prev, [channelId]: [...channelMessages, progressMsg] }
          })

          // 滚动到底部
          requestAnimationFrame(() => {
            conversationOverlayRef.current?.scrollToBottom?.()
          })
        })
      } catch (e) {
        console.warn('[useAgentMessages] Tauri listen 不可用（非桌面环境）:', e)
      }
    }

    setupListener()

    return () => {
      if (unlisten) unlisten()
    }
  // 只需要挂载一次，通过 ref 访问最新的 activeChannelId
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [conversationOverlayRef])

  // 用 ref 持有最新的 activeChannelId，供 listen 闭包使用（避免陈旧闭包）
  const activeChannelIdRef = useRef<string | null>(activeChannelId)
  useEffect(() => {
    activeChannelIdRef.current = activeChannelId
  }, [activeChannelId])

  // ── 监听 Rust 发来的 document:updated 事件（agent_document update 工具触发）──
  // AI 在 Ralph Loop 中调用 agent_document({ action: "update", ... }) 后，
  // Rust 会 emit "document:updated"，前端负责持久化到 agentStore
  useEffect(() => {
    let unlisten: (() => void) | null = null
    let cancelled = false

    const setupDocumentUpdatedListener = async () => {
      try {
        const fn = await listen<{ document_type: string; new_content: string; reason?: string }>(
          'document:updated',
          (event) => {
            const { document_type, new_content, reason } = event.payload
            console.log('[useAgentMessages] 收到 document:updated 事件:', document_type, '原因:', reason)

            // 用 activeChannelIdRef 获取当前活动智能体（无陈旧闭包问题）
            const agentId = activeChannelIdRef.current
            if (!agentId) {
              console.warn('[useAgentMessages] document:updated: 无活动智能体，跳过更新')
              return
            }

            // 调用 agentStore.updateAgentDocument 持久化文档变更
            const updateDocFn = useAgentStore.getState().updateAgentDocument
            if (updateDocFn) {
              const updated = updateDocFn(agentId, document_type, new_content)
              if (updated) {
                console.log(`[useAgentMessages] 智能体 '${agentId}' 的 ${document_type} 文档已更新，新长度: ${new_content.length}`)
              } else {
                console.warn(`[useAgentMessages] 未找到智能体 '${agentId}'，无法更新文档`)
              }
            }
          }
        )
        if (cancelled) {
          fn()
          return
        }
        unlisten = fn
      } catch (e) {
        console.warn('[useAgentMessages] document:updated listen 不可用（非桌面环境）:', e)
      }
    }

    setupDocumentUpdatedListener()

    return () => {
      cancelled = true
      if (unlisten) {
        unlisten()
        unlisten = null
      }
    }
  // 只挂载一次；通过 ref 获取最新 activeChannelId
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

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
