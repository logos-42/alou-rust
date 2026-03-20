import { useCallback, useState, useMemo, useEffect, useRef, RefObject } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import agentService from '@/services/agentService'
import agentDocumentService from '@/services/agentDocumentService'  // ← 新增
import { getActiveApiConfig } from '@/hooks/useApiConfig'
import useAgentStore from '@/stores/agentStore'
import { getSystemPromptForAgent } from '../utils/agentPrompts'
import { getMessageHistory } from '../utils/messageUtils'
import { useAsyncTaskPolling } from '../hooks/useAsyncTaskPolling'
import { useAgentCreation } from '../hooks/useAgentCreation'
import clusterActionStore from '@/stores/clusterActionStore'
import { isAgentMentioned } from '@/utils/mentionParser'
import { useSessionId } from '@/context/SessionContext'
import { enqueueMessage, setSessionHandler, getQueueStatus, type QueueMessage } from '@/services/sessionMessageQueue'

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
      console.log(`[progressToText] tools_pending:`, {
        count: e.count,
        task_id: e.task_id
      });
      return `🔧 准备调用 ${e.count} 个工具...`
    case 'tool_calling':
      // 添加详细日志来捕获工具调用参数
      console.log(`[tool_calling] 工具调用详情:`, {
        task_id: e.task_id,
        tool_name: e.tool_name,
        // 尝试从其他地方获取参数
      });
      console.log(`[progressToText] tool_calling:`, {
        tool_name: e.tool_name,
        preview: e.preview
      });
      if (!e.tool_name) {
        console.warn('[progressToText] tool_calling 缺少 tool_name 字段:', e)
        return `⚙️ 调用工具中...`
      }
      return `⚙️ 调用工具：**${e.tool_name}**`
    case 'tool_done':
      console.log(`[progressToText] tool_done:`, {
        tool_name: e.tool_name,
        success: e.success,
        error: e.error,
        preview: e.preview
      });
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
  // @提及相关字段
  rawContent?: string // 原始消息内容（包含@）
  isMentioned?: boolean // 是否有@提及
  mentionedAgentIds?: string[] // 被@的智能体ID列表
}

// Hook参数类型
export interface ChannelAgent {
  id: string
  name: string
  did?: string
  ipns?: string
  cid?: string
  role_description?: string
  avatar?: string
}

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
  onAutoCreateAgent?: (agentInfo: AgentInfo) => Promise<boolean>
  channelAgents?: ChannelAgent[]  // Channel 中的其他智能体列表
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
  channelAgents,  // Channel 中的其他智能体列表
}: UseAgentMessagesProps) => {
  // 按频道存储消息：Map<channelId, Message[]>
  const [messagesByChannel, setMessagesByChannel] = useState<Record<string, Message[]>>({})
  const [currentMessage, setCurrentMessage] = useState('')
  // 全局 loading 状态（向后兼容）
  const [isLoading, setIsLoading] = useState(false)
  // 🔥 按 agent 存储 loading 状态：每个 channel 的 agent 独立执行
  // 由于每个 channel 对应一个不同的 agent，所以用 agentId 作为 key 即可实现隔离
  const [loadingByAgent, setLoadingByAgent] = useState<Record<string, boolean>>({})
  // 🔥 按 agent 存储 session：每个 channel 的 agent 有独立的 session
  // key: agentId (也是 channelId，因为一一对应)
  const [sessionsByAgent, setSessionsByAgent] = useState<Record<string, string>>({})
  // 按智能体缓存系统提示词（包含记忆注入）：Map<channelId, string>
  const [systemPromptCache, setSystemPromptCache] = useState<Record<string, string>>({})
  const [isConversationVisible, setConversationVisible] = useState(false)
  
  // 跟踪已保存过的消息数量，避免重复保存
  const savedMessageCountRef = useRef<Record<string, number>>({})
  
  // 获取钱包地址（在组件级别获取，供多个函数使用）
  const walletAddress = typeof window !== 'undefined' ? localStorage.getItem('wallet_address') : null

  // 预加载系统提示词（包含记忆注入）
  useEffect(() => {
    if (!activeChannelId || !selectedAgent?.id) return

    const prefetchSystemPrompt = async () => {
      try {
        // 预加载完整版本（injectAll=true）- 用于首次激活时的上下文建立
        const fullCacheKey = `${activeChannelId}_full`
        const cachedFullPrompt = systemPromptCache[fullCacheKey]
        
        // 预加载精简版本（injectAll=false）- 用于后续对话，节省 token
        const liteCacheKey = `${activeChannelId}_lite`
        const cachedLitePrompt = systemPromptCache[liteCacheKey]
        
        if (cachedFullPrompt && cachedLitePrompt) {
          console.log('[useAgentMessages] 使用缓存的系统提示词（full 和 lite 都已缓存）')
          return
        }

        // 首次激活时生成完整提示词（包含所有7个文档）
        if (!cachedFullPrompt) {
          console.log('[useAgentMessages] 预加载系统提示词（初次激活，注入所有文档）...')
          const fullPrompt = await getSystemPromptForAgent(selectedAgent, currentMode, walletAddress, activeChain ?? null, true, channelAgents)  // injectAll=true
          
          if (fullPrompt) {
            setSystemPromptCache(prev => ({
              ...prev,
              [fullCacheKey]: fullPrompt
            }))
            console.log('[useAgentMessages] 系统提示词预加载完成（full），长度:', fullPrompt.length)
          }
        }

        // 同时预加载精简版本（只包含 MEMORY.md）
        if (!cachedLitePrompt) {
          console.log('[useAgentMessages] 预加载系统提示词（lite，只注入记忆）...')
          const litePrompt = await getSystemPromptForAgent(selectedAgent, currentMode, walletAddress, activeChain ?? null, false, channelAgents)  // injectAll=false
          
          if (litePrompt) {
            setSystemPromptCache(prev => ({
              ...prev,
              [liteCacheKey]: litePrompt
            }))
            console.log('[useAgentMessages] 系统提示词预加载完成（lite），长度:', litePrompt.length)
          }
        }
      } catch (error) {
        console.warn('[useAgentMessages] 预加载系统提示词失败:', error)
      }
    }

    prefetchSystemPrompt()
  }, [activeChannelId, selectedAgent?.id, currentMode, walletAddress, activeChain, systemPromptCache])

  // 按智能体存储 AbortController：Map<channelId, AbortController>
  const abortControllersByAgent = useRef<Record<string, AbortController>>({})
  
  // 获取 agentStore 方法
  const updateAgent = useAgentStore((state) => state.updateAgent)
  
  // 获取指定智能体的 loading 状态
  // 🔥 获取指定智能体的 loading 状态（每个 channel 的 agent 独立）
  const isAgentLoading = useCallback((agentId: string) => {
    return loadingByAgent[agentId] || false
  }, [loadingByAgent])

  // 🔥 设置指定智能体的 loading 状态（每个 channel 的 agent 独立）
  const setAgentLoading = useCallback((agentId: string, loading: boolean) => {
    console.log('[useAgentMessages.setAgentLoading]', {
      agentId,
      activeChannelId,
      loading,
      before: loadingByAgent[agentId],
    })
    setLoadingByAgent(prev => ({ ...prev, [agentId]: loading }))
    // 同时更新全局 loading（如果是当前活动智能体）
    if (agentId === activeChannelId) {
      setIsLoading(loading)
    }
  }, [activeChannelId])

  // 当前频道的消息
  // 当没有选中智能体时，显示 'welcome' 虚拟频道的消息（用于对话式创建流程）
  const messages = useMemo(() => {
    const result = activeChannelId
      ? messagesByChannel[activeChannelId] || []
      : messagesByChannel['welcome'] || []
    console.log('[useAgentMessages.messages] 消息更新:', {
      activeChannelId,
      messagesCount: result.length,
      allChannels: Object.keys(messagesByChannel),
    })
    return result
  }, [messagesByChannel, activeChannelId])

  // 添加消息到指定频道
  const appendMessage = useCallback(
    (message: Message, channelId: string | null = activeChannelId) => {
      console.log('[useAgentMessages.appendMessage] 添加消息:', {
        messageId: message.id,
        channelId,
        activeChannelId,
        messageType: message.type,
        content: message.content.slice(0, 50),
      })
      if (!channelId) {
        console.warn('[useAgentMessages] 无法添加消息：没有活动频道')
        return
      }

      setMessagesByChannel((prev) => {
        const channelMessages = prev[channelId] || []
        const newMessages = [...channelMessages, message]
        console.log('[useAgentMessages.appendMessage] 更新消息列表:', {
          channelId,
          beforeCount: channelMessages.length,
          afterCount: newMessages.length,
        })
        return {
          ...prev,
          [channelId]: newMessages,
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
        
        // ✅ 新增：更新 IPFS.md 文档
        await updateIpfsDocument(agentId, {
          cid,
          timestamp: Date.now(),
          messageCount: newMessages.length,
          totalMessages: channelMessages.length,
        })      }
      
      // 更新已保存计数
      savedMessageCountRef.current[channelId] = channelMessages.length
      
      return cid
    } catch (error) {
      console.error('[useAgentMessages] 保存消息到 IPFS 失败:', error)
      return null
    }
  }, [messagesByChannel, updateAgent])

    // ✅ 新增：更新 IPFS.md 文档
  const updateIpfsDocument = useCallback(async (agentId: string, sessionInfo: {
    cid: string;
    timestamp: number;
    messageCount: number;
    totalMessages: number;
  }): Promise<void> => {
    try {
      console.log('[useAgentMessages] 更新 IPFS.md 文档:', agentId)
      
      // 读取当前 IPFS.md
      const currentDoc = await agentDocumentService.getAgentDocument(agentId, 'ipfs').catch(() => null)
      
      const now = new Date(sessionInfo.timestamp)
      const dateStr = now.toLocaleString('zh-CN')
      
      // 生成新的会话记录
      const sessionRecord = `
### 会话：${dateStr}
- **CID**: \`${sessionInfo.cid}\`
- **时间**: ${dateStr}
- **消息数**: ${sessionInfo.messageCount} 条（本会话）/ ${sessionInfo.totalMessages} 条（总计）
- **主题**: 对话会话
`
      
      // 更新文档内容
      let updatedContent = currentDoc || '# IPFS 对话历史索引\n\n这里记录了存储在 IPFS 上的重要对话会话。\n\n'
      
      // 查找"## 最近会话"位置
      const recentSection = updatedContent.indexOf('## 最近会话')
      if (recentSection !== -1) {
        // 在"## 最近会话"后插入
        const insertPos = updatedContent.indexOf('\n', recentSection + 6) + 1
        updatedContent = updatedContent.slice(0, insertPos) + sessionRecord + updatedContent.slice(insertPos)
      } else {
        // 如果没有"## 最近会话"部分，添加到末尾
        updatedContent += '\n\n## 最近会话\n' + sessionRecord
      }
      
      // 更新统计信息
      updatedContent = updatedContent.replace(/- 总会话数：\d+/, (match) => {
        const count = parseInt(match.match(/\d+/)?.[0] || '0') + 1
        return `- 总会话数：${count}`
      })
      updatedContent = updatedContent.replace(/- 总消息数：\d+/, (match) => {
        const count = parseInt(match.match(/\d+/)?.[0] || '0') + sessionInfo.messageCount
        return `- 总消息数：${count}`
      })
      
      // 写回
      await agentDocumentService.updateAgentDocument(agentId, 'ipfs', updatedContent)
      console.log('[useAgentMessages] IPFS.md 文档已更新')
    } catch (err) {
      console.warn('[useAgentMessages] 更新 IPFS.md 失败:', err)
    }
  }, [agentDocumentService])

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
  const sendMessageToAgent = useCallback(async (targetAgentId: string, text: string, targetAgent: Agent | null = null, options?: { groupId?: string; isGroupChat?: boolean; originalMessage?: string }) => {
    console.log('[useAgentMessages.sendMessageToAgent] ========== 开始发送消息 ==========')
    console.log('[useAgentMessages.sendMessageToAgent] 参数:', {
      targetAgentId,
      text: text.slice(0, 50),
      options,
    })
    console.log('[useAgentMessages.sendMessageToAgent] 当前状态:', {
      activeChannelId,
      sessionId,
      sessionsByAgent: Object.keys(sessionsByAgent),
      loadingByAgent: Object.keys(loadingByAgent),
    })

    if (!text?.trim() || !targetAgentId) {
      console.warn('[useAgentMessages] 无法发送消息：缺少文本或目标智能体')
      return
    }

    // 如果是群聊消息，设置 metadata
    const isGroupChat = options?.isGroupChat || false
    const groupId = options?.groupId

    // 🔥 检查该智能体是否正在执行（每个 channel 的 agent 独立）
    // 使用 isAgentLoading 函数而不是直接访问 loadingByAgent，避免闭包问题
    const currentLoadingState = isAgentLoading(targetAgentId)
    console.log('[useAgentMessages] 检查 loading 状态:', {
      targetAgentId,
      activeChannelId,
      currentLoadingState,
      willSkip: !!currentLoadingState,
    })
    if (currentLoadingState) {
      console.log('[useAgentMessages] 智能体正在执行中，跳过:', targetAgentId)
      return
    }

    const userMessage: Message = {
      id: `user_${Date.now()}_${targetAgentId}`,
      type: 'user',
      content: String(text).trim(),
      timestamp: Date.now(),
      metadata: isGroupChat ? {
        isGroupChatMessage: true,
        groupId: groupId,
        originalMessage: options?.originalMessage
      } : undefined
    }

    appendMessage(userMessage, targetAgentId)
    setAgentLoading(targetAgentId, true)
    recordInteraction('user_message', { content: text, agentId: targetAgentId })
    // 移除这里的 scrollToBottom - AgentConversationOverlay 的 useEffect 会自动滚动
    // 重复调用会导致容器上下跳动

     
    const contextSnapshot = contextEventsRef.current?.splice(0, contextEventsRef.current.length) || []
    void contextSnapshot // clear context events even though not passed to local AI

    // 直接调用后端 API 进行单智能体聊天
    try {
      // 🔥 获取或创建该智能体的 session（每个 channel 的 agent 独立）
      let agentSessionId = sessionsByAgent[targetAgentId]

      // 如果没有有效的 session，创建一个新的
      if (!agentSessionId || agentSessionId.startsWith('frontend_')) {
        try {
          console.log('[useAgentMessages] 为智能体创建新会话:', targetAgentId, 'channel:', activeChannelId)
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

      // 🔥 保存该智能体的 session
      setSessionsByAgent(prev => ({ ...prev, [targetAgentId]: agentSessionId }))

      console.log('[useAgentMessages] 发送消息，sessionId:', agentSessionId, '智能体:', targetAgentId, 'channel:', activeChannelId)

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
      
      // 优先使用缓存的系统提示词（已包含记忆注入）
      // 注意：我们使用 lite 版本（injectAll=false）来节省 token
      const cacheKey = `${activeChannelId}_lite`
      let systemPrompt = systemPromptCache[cacheKey]
      
      // 如果缓存不存在或为空，动态生成并更新缓存（只注入记忆）
      if (!systemPrompt || systemPrompt.length === 0) {
        console.log('[useAgentMessages] 缓存未命中，动态生成系统提示词（只注入记忆）')
        systemPrompt = await getSystemPromptForAgent(agentInfo, currentMode, walletAddress, activeChain ?? null, false, channelAgents)  // injectAll=false
        
        if (systemPrompt) {
          setSystemPromptCache(prev => ({
            ...prev,
            [cacheKey]: systemPrompt
          }))
        }
      } else {
        console.log('[useAgentMessages] 使用缓存的系统提示词（lite模式，长度:', systemPrompt.length, ')')
      }
      
      const history = getMessageHistory(targetAgentId, messagesByChannel)

      // 调试日志：检查消息历史
      console.log('[useAgentMessages] 发送消息前检查上下文:', {
        targetAgentId,
        messagesInChannel: messagesByChannel[targetAgentId]?.length || 0,
        historyLength: history.length,
      })
      console.log('[useAgentMessages] 历史消息内容:', history)

      // 构建正确格式的 messages 数组发送给 Rust
      const messagesArray: Array<{ role: string; content: string }> = []
      if (systemPrompt) {
        messagesArray.push({ role: 'system', content: systemPrompt })
      }
      // 动态计算保留多少条历史（不超过 token 限制）
      // 估算：每条消息平均 100 tokens，system ≈ 500 tokens，预留 1000 tokens 给响应
      // 可用 tokens = maxTokens - system - response_buffer
      // 可用消息数 = 可用 tokens / 平均每条消息 tokens
      const maxTokens = selectedAgent?.maxTokens || 4000
      const avgTokensPerMessage = 100
      const systemTokens = systemPrompt?.length || 0
      const responseBuffer = 1000
      const availableTokens = maxTokens - systemTokens - responseBuffer
      const maxHistoryMessages = Math.floor(availableTokens / avgTokensPerMessage)

      // 至少保留 5 条，最多保留 50 条
      const safeHistory = history.slice(-Math.max(3, Math.min(50, maxHistoryMessages)))

      console.log('[useAgentMessages] 动态计算上下文:', {
        maxTokens,
        systemTokens,
        availableTokens,
        maxHistoryMessages,
        actualHistoryLength: safeHistory.length,
      })

      // 添加历史消息
      for (const m of safeHistory) {
        messagesArray.push({ role: m.role, content: m.content })
      }
      // 当前用户消息
      messagesArray.push({ role: 'user', content: text.trim() })

      console.log('[useAgentMessages] 发送给 LLM 的 messagesArray:', messagesArray)
      console.log('[useAgentMessages] 调用本地 AI，provider:', localApiConfig.provider, 'model:', localApiConfig.model, '消息数:', messagesArray.length)

      // 3. 通过 Tauri invoke 执行 AI 对话（本地 Rust 直接调用 AI API）
      // Rust 返回 { success, result: TaskFinalResult, execution_mode, timestamp }
      // TaskFinalResult = { task_id, success, result: string, error, iteration_count }
      
      // 修复：确保 base_url 不为空字符串，否则会导致 "relative URL without a base" 错误
      const baseUrl = localApiConfig.base_url?.trim()
      const sanitizedBaseUrl = baseUrl && baseUrl.length > 0 ? baseUrl : null
      
      console.log('[useAgentMessages] API 配置:', {
        provider: localApiConfig.provider,
        model: localApiConfig.model,
        base_url: sanitizedBaseUrl || '(使用默认)',
      })
      
      console.log('[useAgentMessages] 调用 execute_ai_conversation:', {
        agentId: agentInfo?.id || targetAgentId,
        sessionId: sessionId,  // 全局 session（来自 SessionContext）
        agentSessionId: agentSessionId,  // ← 这是每个 agent 独立的 session
        targetAgentId,
      })

      const tauri_result = await invoke<{
        success: boolean
        result?: {
          task_id?: string
          success?: boolean
          result?: string      // AI 最终回复的文本
          error?: string
          iteration_count?: number
          session_id?: string  // 新增：session_id
          // 兼容旧字段
          final_answer?: string
          content?: string
          summary?: string
        }
        execution_mode?: string
        timestamp?: number
        session_id?: string    // 新增：顶层 session_id
      }>('execute_ai_conversation', {
        agentConfig: {
          id: localApiConfig.id || 'primary',
          provider: localApiConfig.provider,
          api_key: localApiConfig.api_key,
          base_url: sanitizedBaseUrl,
          model: localApiConfig.model != null ? localApiConfig.model : null,
          is_active: true,
        },
        message: text.trim(),       // fallback 单条消息
        messages: messagesArray,    // 完整上下文数组（优先使用）
        options: { stream: false },
        agentId: agentInfo?.id || targetAgentId,  // 传递智能体 ID
        sessionId: agentSessionId,  // ← 修复：传递每个 agent 独立的 session，而不是全局 sessionId
        // 🔥 传递系统提示到后端
        systemPrompt,
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
          // 🔥 关键修复：继承群聊 metadata，这样同步逻辑才能检测到
          metadata: isGroupChat ? {
            isGroupChatMessage: true,
            groupId: groupId,
            originalMessage: options?.originalMessage,
            agentId: targetAgentId
          } : undefined
        }
        appendMessage(assistantMessage, targetAgentId)
        
        // ✅ 关键修复：对话结束后立即保存消息到 IPFS，保存上下文
        if (selectedAgent?.id && activeChannelId) {
          saveMessagesToIpfs(activeChannelId, selectedAgent.id, false).then((cid) => {
            if (cid) {
              console.log('[useAgentMessages] 消息已保存到 IPFS，CID:', cid)
              // 更新 agent 的 messages_cid
              updateAgent(selectedAgent.id, { messages_cid: cid })
            }
          }).catch(err => {
            console.warn('[useAgentMessages] 保存消息失败:', err)
          })
        }
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
      // 处理 Tauri 错误格式
      let errName: string | undefined
      let errMessage: string | undefined
      
      if (typeof error === 'string') {
        errMessage = error
      } else if (error && typeof error === 'object') {
        const err = error as { name?: string; message?: string; error?: string }
        errName = err.name
        errMessage = err.message || err.error || JSON.stringify(error)
      }

      // 用户主动取消
      if (errName === 'AbortError' || errName === 'CanceledError') {
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

      const errorMessage = errMessage || '未知错误'
      console.error('[useAgentMessages] 本地 AI 执行错误:', errorMessage, error)

      // 根据错误类型提供友好的提示
      let userFriendlyMessage = `❌ 执行出错：${errorMessage}`
      
      if (errorMessage.includes('401') || errorMessage.includes('Unauthorized') || errorMessage.includes('Authentication Fails')) {
        userFriendlyMessage = `🔑 **API Key 无效**

您的 DeepSeek API Key 认证失败，可能原因：
1. API Key 填写错误或已失效
2. 账户余额不足或已欠费
3. Key 被删除或禁用

**解决方法：**
• 前往 [DeepSeek 开放平台](https://platform.deepseek.com/) 检查 API Key 状态
• 确认账户有足够余额
• 重新生成 API Key 并在设置中更新`}
      else if (errorMessage.includes('429') || errorMessage.includes('Rate limit')) {
        userFriendlyMessage = `⏳ **请求太频繁**

已达到 API 速率限制，请稍后再试。`
      }
      else if (errorMessage.includes('Network') || errorMessage.includes('relative URL')) {
        userFriendlyMessage = `🌐 **网络错误**

${errorMessage}

请检查网络连接或 API 配置。`}
      else {
        userFriendlyMessage += '\n\n请检查 API Key 配置是否正确，或查看控制台获取详细错误信息。'
      }

      appendMessage({
        id: `error_${Date.now()}`,
        type: 'assistant',
        content: userFriendlyMessage,
        timestamp: Date.now(),
        source: 'error',
      }, targetAgentId)
    } finally {
      // 清理 AbortController
      delete abortControllersByAgent.current[targetAgentId]
      setAgentLoading(targetAgentId, false)
      // 移除这里的 scrollToBottom - AgentConversationOverlay 的 useEffect 会自动滚动
      // 重复调用会导致容器上下跳动
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
    isAgentLoading,  // 使用函数而不是直接的 loadingByAgent
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

  // 更新 ref，使事件处理器可以访问最新的 sendMessageToAgent
  useEffect(() => {
    sendMessageToAgentRef.current = sendMessageToAgent
  }, [sendMessageToAgent])


  // ============================================================================
  // 异步并发消息队列初始化 - 支持多 session 并行执行
  // ============================================================================

  /**
   * 初始化 sessionMessageQueue 的处理器
   * 每个 session 有独立的消息队列，支持 maxConcurrent=30 的并发处理
   */
  useEffect(() => {
    // 消息队列处理器 - 由 sessionMessageQueue 调用
    const handler = async (message: QueueMessage) => {
      console.log('[sessionMessageQueue handler] 处理消息，agentId:', message.agentId, 'sessionId:', message.sessionId, 'isGroupChat:', message.isGroupChat)

      try {
        // 显示 loading 状态
        setAgentLoading(message.agentId, true)

        // 调用 sendMessageToAgent 处理消息（使用已有的同步版本）
        if (sendMessageToAgentRef.current) {
          await sendMessageToAgentRef.current(
            message.agentId,
            message.content,
            null,
            {
              groupId: message.groupId,
              isGroupChat: message.isGroupChat,
              originalMessage: message.originalMessage,
            }
          )
        } else {
          throw new Error('sendMessageToAgentRef 未初始化')
        }
      } catch (error) {
        console.error('[sessionMessageQueue handler] 消息处理失败:', error)
        // 显示错误消息
        appendMessage({
          id: `error_${message.id}`,
          type: 'assistant',
          content: `❌ 执行失败：${error instanceof Error ? error.message : '未知错误'}`,
          timestamp: Date.now(),
          source: 'error',
          agentId: message.agentId,
        }, message.agentId)
      } finally {
        setAgentLoading(message.agentId, false)
      }
    }

    // 为当前 session 设置处理器
    if (sessionId) {
      console.log('[useAgentMessages] 初始化主 session handler，sessionId:', sessionId)
      setSessionHandler(sessionId, handler)
    }

    // 🔥 为所有 agent 的 session 设置处理器（群聊消息需要）
    Object.entries(sessionsByAgent).forEach(([agentId, agentSessionId]) => {
      if (agentSessionId && agentSessionId !== sessionId) {
        console.log('[useAgentMessages] 为 agent session 设置 handler，agentId:', agentId, 'sessionId:', agentSessionId)
        setSessionHandler(agentSessionId, handler)
      }
    })

    return () => {
      // 清理 handler（可选）
      console.log('[useAgentMessages] 清理 sessionMessageQueue handler')
    }
  }, [sessionId, sessionsByAgent, appendMessage, setAgentLoading])

  // 向后兼容的 sendMessage（发送到当前活动智能体）
  // 修改：接收 text 参数而不是依赖内部 currentMessage 状态
  const sendMessage = useCallback(async (text?: string) => {
    console.log('[useAgentMessages.sendMessage] 被调用:', {
      text,
      activeChannelId,
      selectedAgent: selectedAgent?.id,
      isLoading,
      isAgentLoadingResult: isAgentLoading(selectedAgent?.id || activeChannelId),
    })
    // 如果没有传入 text，使用内部 currentMessage（向后兼容）
    const messageText = (text !== undefined ? text : currentMessage).trim()
    // 🔥 修复：检查当前 agent 是否正在执行，而不是全局 isLoading
    const currentAgentId = selectedAgent?.id || activeChannelId
    if (!messageText || isAgentLoading(currentAgentId)) {
      console.log('[useAgentMessages.sendMessage] 跳过：消息为空或 agent 正在执行', {
        messageText: !!messageText,
        isAgentLoading: isAgentLoading(currentAgentId),
        currentAgentId,
      })
      return
    }

    // ── 没有活动频道（未选中任何智能体）→ 进入对话式创建流程 ──
    if (!activeChannelId) {
      setCurrentMessage('')

      // 显示用户消息 - 直接在当前可见区域显示，而不是 'welcome' 虚拟频道
      // 这样用户可以看到自己的输入和后续的系统响应
      appendMessage({
        id: `user_${Date.now()}`,
        type: 'user',
        content: messageText,
        timestamp: Date.now(),
        source: 'user',
      }) // 不指定 channelId，使用当前默认频道

      // 如果有自动创建回调（父组件支持），用 AI 解析描述 → 自动创建
      if (wrappedAutoCreateAgent) {
        appendMessage({
          id: `system_thinking_${Date.now()}`,
          type: 'system',
          content: '🤔 正在理解你的需求，准备创建智能体...',
          timestamp: Date.now(),
          source: 'system',
        })

        try {
          const agentInfo = await parseAgentCreationCommandWithAIDirect(messageText)
          console.log('[sendMessage] 解析的智能体信息:', agentInfo)

          appendMessage({
            id: `system_creating_${Date.now()}`,
            type: 'system',
            content: `🔄 正在创建智能体 **"${agentInfo.name}"**...`,
            timestamp: Date.now(),
            source: 'system',
          })

          await wrappedAutoCreateAgent(agentInfo as unknown as AgentInfo)

          appendMessage({
            id: `system_done_${Date.now()}`,
            type: 'assistant',
            content: `✅ 智能体 **"${agentInfo.name}"** 已创建！点击左侧频道开始对话。`,
            timestamp: Date.now(),
            source: 'system',
          })
        } catch (err) {
          appendMessage({
            id: `system_err_${Date.now()}`,
            type: 'assistant',
            content: `❌ 创建失败：${(err as Error).message || '未知错误'}`,
            timestamp: Date.now(),
            source: 'error',
          })
        }
      } else if (onCreateAgent) {
        // 父组件只支持打开模态框
        appendMessage({
          id: `system_modal_${Date.now()}`,
          type: 'assistant',
          content: '📝 即将打开创建表单...',
          timestamp: Date.now(),
          source: 'system',
        })
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
        })
      }

      return
    }

    setCurrentMessage('')
    await sendMessageToAgent(activeChannelId, text ?? '', selectedAgent)
  }, [
    activeChannelId,
    currentMessage,
    isLoading,
    selectedAgent,
    sendMessageToAgent,
    appendMessage,
    onAutoCreateAgent,
    onCreateAgent,
    parseAgentCreationCommandWithAIDirect,
  ])

  const openConversationPanel = useCallback(() => {
    setConversationVisible(true)
    // 移除这里的 scrollToBottom - AgentConversationOverlay 的 useEffect 会自动滚动
  }, [])

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
  
  // 创建 ref 来存储 sendMessageToAgent，以便在事件处理器中使用
  const sendMessageToAgentRef = useRef<((targetAgentId: string, text: string, targetAgent: Agent | null, options?: { groupId?: string; isGroupChat?: boolean; originalMessage?: string }) => Promise<void>) | null>(null)

  // 监听群聊消息事件（带 session 过滤）
  useEffect(() => {
    const handleAgentGroupMessage = async (event: Event) => {
      const customEvent = event as CustomEvent<{ agentId: string; message: GroupChatMessage; sessionId?: string }>
      const { agentId, message, sessionId: eventSessionId } = customEvent.detail

      // Session 过滤：只处理当前 session 的消息
      // 使用 sessionId prop（来自组件）
      if (eventSessionId && eventSessionId !== sessionId) {
        console.log('[useAgentMessages] Session 不匹配，跳过消息:', {
          eventSessionId,
          currentSessionId: sessionId,
          agentId,
        })
        return
      }

      // 关键修复：移除 selectedAgent 检查，让所有智能体都能接收群聊消息
      // 即使智能体未被选中，也应该能够处理群聊消息
      console.log('[useAgentMessages] 智能体收到群聊消息:', agentId, message)

      const messageContent = message.content || message.text || ''
      if (!messageContent.trim()) {
        console.warn('[useAgentMessages] 群聊消息内容为空，跳过')
        return
      }
      
      // 检查消息是否包含@提及
      if (message.isMentioned !== undefined && message.isMentioned) {
        // 消息有@提及，检查当前智能体是否在被@列表中
        const mentionedIds = message.mentionedAgentIds || []
        if (!mentionedIds.includes(agentId)) {
          console.log('[useAgentMessages] 智能体未被@，跳过处理:', agentId, '被@的智能体:', mentionedIds)
          return
        }
        console.log('[useAgentMessages] 智能体被@，继续处理:', agentId)
      } else {
        // 如果没有@提及标记，检查原始消息是否包含@（兼容旧消息格式）
        // 尝试从store获取智能体列表
        let agentsList: any[] = []
        try {
          const { getActions } = clusterActionStore.getState()
          const actions = getActions('') || []
          for (const action of actions) {
            if (action.agents && Array.isArray(action.agents)) {
              agentsList = [...agentsList, ...action.agents]
            }
          }
        } catch (e) {
          // ignore
        }
        
        if (agentsList.length > 0 && messageContent.includes('@')) {
          // 消息包含@，检查是否@了当前智能体
          if (!isAgentMentioned(agentId, messageContent, agentsList)) {
            console.log('[useAgentMessages] 智能体未被@（兼容检查），跳过处理:', agentId)
            return
          }
        }
      }
      
      // 检查智能体是否正在执行
      if (isAgentLoading(agentId)) {
        console.log('[useAgentMessages] 智能体正在执行中，跳过群聊消息:', agentId)
        return
      }

      // 使用异步消息队列处理群聊消息（非阻塞，支持并行）
      console.log('[useAgentMessages] 触发智能体处理群聊消息（异步并行）:', agentId, messageContent.slice(0, 50))

      // 🔥 获取智能体的 session（每个 channel 的 agent 独立）
      let agentSessionId = sessionsByAgent[agentId] || sessionId
      if (!agentSessionId || agentSessionId.startsWith('frontend_')) {
        agentSessionId = sessionId
      }

      // 将群聊消息放入异步队列（fire-and-forget，支持 maxConcurrent=30 并发）
      const success = enqueueMessage(agentId, messageContent, agentSessionId, {
        isGroupChat: true,
        groupId: message.groupId,
        originalMessage: messageContent,
        priority: 5, // 群聊消息优先级稍低
      })

      if (!success) {
        console.warn('[useAgentMessages] 群聊消息入队失败，队列可能已满')
      } else {
        console.log('[useAgentMessages] 群聊消息已入队，sessionId:', agentSessionId, 'queue status:', getQueueStatus(agentSessionId))
      }
    }

    window.addEventListener('agent-group-message', handleAgentGroupMessage)

    return () => {
      window.removeEventListener('agent-group-message', handleAgentGroupMessage)
    }
  }, [isAgentLoading, sessionId, sessionsByAgent])

  // 监听智能体消息变化，如果是群聊消息的回复，则同步到群聊
  useEffect(() => {
    if (!messages.length) return

    const lastMessage = messages[messages.length - 1]

    // 🔥 关键修复：检查消息是否来自群聊智能体响应
    if (lastMessage &&
        lastMessage.type === 'assistant' &&
        (lastMessage.metadata?.isGroupChatMessage || lastMessage.metadata?.groupId)) {

      const groupId = lastMessage.metadata?.groupId as string
      const agentId = lastMessage.agentId || selectedAgent?.id
      
      if (groupId && agentId) {
        console.log('[useAgentMessages] 检测到群聊智能体回复，同步到群聊:', {
          groupId,
          agentId,
          content: lastMessage.content.slice(0, 50)
        })

        try {
          const { addGroupChatMessage, getActions } = clusterActionStore.getState()
          
          // 获取智能体信息
          const allActions = Object.values(get().actionsByChannel || {}).flat()
          let agentInfo = null
          for (const action of allActions) {
            const agent = action.agents?.find((a: any) => a.id === agentId)
            if (agent) {
              agentInfo = agent
              break
            }
          }

          const groupReplyMessage = {
            id: `agent_reply_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`,
            type: 'agent' as const,
            from: agentId,
            fromName: agentInfo?.name || lastMessage.agentId || '智能体',
            avatar: agentInfo?.avatar,
            content: lastMessage.content,
            timestamp: lastMessage.timestamp,
            metadata: {
              agentId: agentId,
              replyTo: lastMessage.metadata?.originalMessage,
              isReply: true
            }
          }

          addGroupChatMessage(groupId, groupReplyMessage)
          console.log('[useAgentMessages] 群聊消息已同步:', groupId)
        } catch (error) {
          console.warn('[useAgentMessages] 同步智能体回复到群聊失败:', error)
        }
      }
    }
  }, [messages])

  // 用 ref 持有最新的 onAutoCreateAgent 回调，避免 useEffect([]) 的陈旧闭包
  // 同时防止回调变化时重新注册 Tauri listener（会导致重复监听）
  const onAutoCreateAgentRef = useRef(onAutoCreateAgent)
  useEffect(() => {
    onAutoCreateAgentRef.current = onAutoCreateAgent
  }, [onAutoCreateAgent])

  // 用 ref 跟踪前端主动触发的创建请求，避免 agent:created 事件重复处理
  const pendingAgentCreatesRef = useRef<Set<string>>(new Set())

  // 包装 onAutoCreateAgent，在调用前标记为 pending
  const wrappedAutoCreateAgent = useCallback(async (agentInfo: AgentInfo): Promise<boolean> => {
    const agentKey = `${agentInfo.name}_${agentInfo.roleDescription}`.toLowerCase().trim()
    
    // 标记为 pending
    pendingAgentCreatesRef.current.add(agentKey)
    console.log('[useAgentMessages] 标记前端创建请求:', agentKey)
    
    try {
      if (!onAutoCreateAgentRef.current) {
        throw new Error('onAutoCreateAgent 回调未提供')
      }
      
      const result = await onAutoCreateAgentRef.current(agentInfo)
      
      // 创建完成后，延迟移除 pending 标记（给后端事件处理留出时间窗口）
      setTimeout(() => {
        pendingAgentCreatesRef.current.delete(agentKey)
        console.log('[useAgentMessages] 移除前端创建标记:', agentKey)
      }, 5000) // 5 秒后移除，覆盖后端事件到达的时间窗口
      
      return result
    } catch (error) {
      // 失败时立即移除标记
      pendingAgentCreatesRef.current.delete(agentKey)
      throw error
    }
  }, [])

  // ── 监听 Rust 发来的 agent:created 事件（agent_creator 工具创建成功后触发）────
  // Rust executor 在 agent_creator create 成功后 emit "agent:created"
  // 前端收到后调用 onAutoCreateAgent 将新 Agent 写入 Zustand 并显示在侧边栏
  useEffect(() => {
    let unlisten: (() => void) | null = null
    let cancelled = false

    const setupAgentCreatedListener = async () => {
      try {
        const fn = await listen<{ name: string; role_description?: string; id?: string; autoActivate?: boolean }>('agent:created', async (event) => {
          const payload = event.payload
          console.log('[useAgentMessages] ========== 收到 agent:created 事件 ==========')
          console.log('[useAgentMessages] payload:', payload)
          console.log('[useAgentMessages] onAutoCreateAgentRef.current:', onAutoCreateAgentRef.current ? '存在' : '不存在')

          // 检查是否是前端刚触发的创建（避免重复处理）
          const agentKey = `${payload.name}_${payload.role_description || ''}`.toLowerCase().trim()
          if (pendingAgentCreatesRef.current.has(agentKey)) {
            console.log('[useAgentMessages] 跳过前端已触发的创建:', payload.name)
            return
          }

          // 通过 ref 获取最新回调，避免陈旧闭包导致 "Should have a queue" React 错误
          const cb = onAutoCreateAgentRef.current
          console.log('[useAgentMessages] 检查回调和payload:', { 
            hasCallback: !!cb, 
            hasName: !!payload?.name,
            payloadId: payload?.id,
            payloadName: payload?.name 
          })
          
          if (cb && payload?.name) {
            try {
              // 只添加智能体到侧边栏，不要自动激活（autoActivate 默认为 false）
              const shouldAutoActivate = payload.autoActivate ?? false
              console.log('[useAgentMessages] Agent 已自动添加到侧边栏:', payload.name, '自动激活:', shouldAutoActivate)

              // 调用回调添加智能体
              const agentInfo = {
                name: payload.name,
                // 同时传两种字段命名，兼容 useAutoAgentCreator (roleDescription) 和其他消费者 (role_description)
                role_description: payload.role_description ?? '',
                roleDescription: payload.role_description ?? '',
                id: payload.id,
              }
              console.log('[useAgentMessages] 调用cb前的agentInfo:', agentInfo)
              
              await cb(agentInfo)
              
              console.log('[useAgentMessages] onAutoCreateAgent 调用完成')
            } catch (err) {
              console.error('[useAgentMessages] 自动创建 Agent 失败:', err)
            }
          } else {
            console.warn('[useAgentMessages] onAutoCreateAgent 回调不存在或 payload 缺少 name')
          }
        })
        if (cancelled) {
          // Strict Mode 的第一次挂载已经被取消，立即释放
          fn()
          console.log('[useAgentMessages] agent:created listener 已取消（Strict Mode cleanup）')
        } else {
          unlisten = fn
          console.log('[useAgentMessages] agent:created 监听器已设置')
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
  }, [])

  // ── 监听 Rust 发来的 tool:created 事件（tool_creation 工具创建成功后触发）────
  useEffect(() => {
    let unlisten: (() => void) | null = null
    let cancelled = false

    const setupToolCreatedListener = async () => {
      try {
        const fn = await listen<{ tool_name: string; description?: string; status?: string }>('tool:created', async (event) => {
          const payload = event.payload
          console.log('[useAgentMessages] 收到 tool:created 事件:', payload)

          // 在对话框中显示工具创建成功的消息
          appendMessage({
            id: `tool_created_${Date.now()}`,
            type: 'assistant',
            content: `✅ 工具 **${payload.tool_name}** 创建成功！\n\n${payload.description || ''}`,
            timestamp: Date.now(),
            source: 'system',
          })
        })
        if (cancelled) {
          fn()
          console.log('[useAgentMessages] tool:created listener 已取消（Strict Mode cleanup）')
        } else {
          unlisten = fn
        }
      } catch (e) {
        console.warn('[useAgentMessages] tool:created listen 不可用（非桌面环境）:', e)
      }
    }

    // 🔥 监听 tool:log 事件（通用工具日志）
    const setupToolLogListener = async () => {
      try {
        const fn = await listen<{ tool: string; action: string; success?: boolean; message: string; args?: any }>('tool:log', (event) => {
          const payload = event.payload
          console.log('[useAgentMessages] 🔧 tool:log:', payload.action, payload.tool, payload.message)
        })
        if (cancelled) {
          fn()
        } else {
          unlisten = fn
        }
      } catch (e) {
        console.warn('[useAgentMessages] tool:log listen 不可用（非桌面环境）:', e)
      }
    }

    setupToolCreatedListener()

    return () => {
      cancelled = true
      if (unlisten) {
        unlisten()
        unlisten = null
      }
    }
  }, [appendMessage]) // 依赖 appendMessage 以显示消息

  // ── 监听 Rust 发来的 agent:progress 进度事件 ──────────────────────────────
  // 在智能体执行期间，Rust 会通过 AppHandle 发送工具调用进度
  // 我们在当前活动频道插入进度消息（source='progress'），让用户实时可见
  useEffect(() => {
    let unlisten: (() => void) | null = null
    // cancelled flag：防止 React 18 Strict Mode 双重挂载导致注册两个监听器
    let cancelled = false
    // 跟踪上一个进度消息的 ID，用于更新而不是添加新消息
    let lastProgressMsgId: string | null = null

    const setupListener = async () => {
      try {
        const fn = await listen<AgentProgressPayload>('agent:progress', (event) => {
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

            // 如果是 tool_calling 或 tools_pending，更新最后一条进度消息（避免频繁添加新消息）
            // 如果是 tool_done，添加新消息（表示一个工具已完成）
            const isUpdate = payload.type === 'tool_calling' || payload.type === 'tools_pending'
            let newMessages: Message[]

            if (isUpdate && lastProgressMsgId) {
              // 更新现有的进度消息 - 直接修改数组，避免创建新数组导致重新渲染
              const updatedMessages = [...channelMessages]
              const msgIndex = updatedMessages.findIndex(msg => msg.id === lastProgressMsgId)
              if (msgIndex !== -1) {
                updatedMessages[msgIndex] = {
                  ...updatedMessages[msgIndex],
                  content: text,
                  timestamp: Date.now()
                }
              }
              newMessages = updatedMessages
            } else {
              // 添加新的进度消息
              const progressMsg: Message = {
                id: `progress_${Date.now()}_${Math.random().toString(36).slice(2, 6)}`,
                type: 'system',
                content: text,
                timestamp: Date.now(),
                source: 'progress',
                agentId: channelId,
                metadata: { progressType: payload.type, toolName: payload.tool_name },
              }
              lastProgressMsgId = progressMsg.id
              newMessages = [...channelMessages, progressMsg]
            }

            return { ...prev, [channelId]: newMessages }
          })

          // 移除自动滚动 - AgentConversationOverlay 的 useEffect 已经处理了滚动
          // 反复调用 scrollToBottom 会导致容器上下跳动
        })

        // Strict Mode 的第一次挂载已经被取消，立即释放
        if (cancelled) {
          fn()
        } else {
          unlisten = fn
        }
      } catch (e) {
        console.warn('[useAgentMessages] Tauri listen 不可用（非桌面环境）:', e)
      }
    }

    setupListener()

    return () => {
      cancelled = true
      if (unlisten) unlisten()
      unlisten = null
    }
  // 只需要挂载一次，通过 ref 访问最新的 activeChannelId
     
  }, [conversationOverlayRef])

  // 用 ref 持有最新的 activeChannelId，供 listen 闭包使用（避免陈旧闭包）
  const activeChannelIdRef = useRef<string | null>(activeChannelId)
  useEffect(() => {
    activeChannelIdRef.current = activeChannelId
  }, [activeChannelId])

  // ── 监听 Rust 发来的 document:updated 事件（agent_document update 工具触发）──
  // Rust 会 emit "document:updated"，包含正确的 agent_id，前端负责更新 agentStore
  useEffect(() => {
    let unlisten: (() => void) | null = null
    let cancelled = false

    const setupDocumentUpdatedListener = async () => {
      try {
        const fn = await listen<{ document_type: string; new_content: string; reason?: string; agent_id: string }>(
          'document:updated',
          (event) => {
            const { document_type, new_content, reason, agent_id } = event.payload
            console.log('[useAgentMessages] 收到 document:updated 事件:', document_type, '原因:', reason, 'agent_id:', agent_id)

            // 使用 Rust 事件中携带的正确 agent_id
            if (!agent_id) {
              console.warn('[useAgentMessages] document:updated: 事件中无 agent_id，跳过更新')
              return
            }

            // 调用 agentStore.updateAgentDocument 持久化文档变更
            const updateDocFn = useAgentStore.getState().updateAgentDocument
            if (updateDocFn) {
              const updated = updateDocFn(agent_id, document_type, new_content)
              if (updated) {
                console.log(`[useAgentMessages] 智能体 '${agent_id}' 的 ${document_type} 文档已更新，新长度: ${new_content.length}`)
              } else {
                console.warn(`[useAgentMessages] 未找到智能体 '${agent_id}'，无法更新文档`)
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
   
  }, [])

  /**
   * 终止指定智能体的执行
   */
  const cancelAgentExecution = useCallback((agentId: string) => {
    console.log("[useAgentMessages.cancelAgentExecution] 开始终止:", { agentId, activeChannelId })
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
