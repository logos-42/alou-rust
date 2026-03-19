/**
 * 智能体执行 Hook - AI 对话和工具调用
 * 
 * @module components/AgentChat/useAgentMessages/hooks/useAgentExecution
 */

import { useCallback, useRef } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { Message, AgentInfo } from '../types'

interface ChannelAgent {
  id: string
  name: string
  did?: string
  ipns?: string
  cid?: string
  role_description?: string
  avatar?: string
}

interface UseAgentExecutionConfig {
  sessionId: string
  appendMessage: (message: Message, channelId?: string) => void
  setAgentLoading: (agentId: string, loading: boolean) => void
  getActiveApiConfig: () => Promise<{
    id?: string
    provider: string
    api_key: string
    base_url?: string | null
    model?: string | null
    is_active: boolean
  }>
  getSystemPromptForAgent: (
    agent: AgentInfo | null,
    mode: string,
    walletAddress: string | null,
    chain: string | null,
    injectAll: boolean,
    channelAgents?: ChannelAgent[]
  ) => Promise<string>
  getMessageHistory: (agentId: string, messagesByChannel: unknown) => unknown[]
  systemPromptCache: Record<string, string>
  setSystemPromptCache: React.Dispatch<React.SetStateAction<Record<string, string>>>
  selectedAgent: AgentInfo | null
  currentMode: string
  walletAddress: string | null
  activeChain: string | null
  maxTokens?: number
  channelAgents?: ChannelAgent[]  // Channel 中的其他智能体列表
}

interface UseAgentExecutionReturn {
  sendMessageToAgent: (
    targetAgentId: string,
    text: string,
    targetAgent: AgentInfo | null,
    options?: { groupId?: string; isGroupChat?: boolean; originalMessage?: string }
  ) => Promise<void>
  cancelAgentExecution: (agentId: string) => void
}

/**
 * 智能体执行 Hook
 */
export function useAgentExecution({
  sessionId,
  appendMessage,
  setAgentLoading,
  getActiveApiConfig,
  getSystemPromptForAgent,
  getMessageHistory,
  systemPromptCache,
  setSystemPromptCache,
  selectedAgent,
  currentMode,
  walletAddress,
  activeChain,
  maxTokens = 4000,
  channelAgents,  // Channel 中的其他智能体列表
}: UseAgentExecutionConfig): UseAgentExecutionReturn {
  const abortControllersByAgent = useRef<Record<string, AbortController>>({})

  /**
   * 发送消息到智能体（核心执行逻辑）
   */
  const sendMessageToAgent = useCallback(async (
    targetAgentId: string,
    text: string,
    targetAgent: AgentInfo | null = null,
    options?: { groupId?: string; isGroupChat?: boolean; originalMessage?: string }
  ): Promise<void> => {
    if (!text?.trim() || !targetAgentId) {
      console.warn('[sendMessageToAgent] 无法发送消息：缺少文本或目标智能体')
      return
    }

    const isGroupChat = options?.isGroupChat || false
    const groupId = options?.groupId

    // 创建用户消息
    const userMessage: Message = {
      id: `user_${Date.now()}_${targetAgentId}`,
      type: 'user',
      content: text.trim(),
      timestamp: Date.now(),
      metadata: isGroupChat ? {
        isGroupChatMessage: true,
        groupId,
        originalMessage: options?.originalMessage,
      } : undefined,
    }

    appendMessage(userMessage, targetAgentId)
    setAgentLoading(targetAgentId, true)

    try {
      // 获取 API 配置
      const localApiConfig = await getActiveApiConfig()

      if (!localApiConfig) {
        appendMessage({
          id: `no_api_${Date.now()}`,
          type: 'assistant',
          content: '⚙️ **请先配置 API Key**\n\n点击右上角设置图标 → API 配置',
          timestamp: Date.now(),
          source: 'system',
          agentId: targetAgentId,
        }, targetAgentId)
        setAgentLoading(targetAgentId, false)
        return
      }

      // 获取系统提示词
      const agentInfo = targetAgent || selectedAgent
      const cacheKey = `system_${targetAgentId}_lite`
      let systemPrompt = systemPromptCache[cacheKey]

      if (!systemPrompt || systemPrompt.length === 0) {
        systemPrompt = await getSystemPromptForAgent(
          agentInfo,
          currentMode,
          walletAddress,
          activeChain,
          false,
          channelAgents
        )
        if (systemPrompt) {
          setSystemPromptCache(prev => ({ ...prev, [cacheKey]: systemPrompt }))
        }
      }

      // 获取历史消息
      const history = getMessageHistory(targetAgentId, {})
      const avgTokensPerMessage = 100
      const systemTokens = systemPrompt?.length || 0
      const responseBuffer = 1000
      const availableTokens = maxTokens - systemTokens - responseBuffer
      const maxHistoryMessages = Math.floor(availableTokens / avgTokensPerMessage)
      const safeHistory = (history as unknown[]).slice(
        -Math.max(3, Math.min(50, maxHistoryMessages))
      )

      // 构建消息数组
      const messagesArray: Array<{ role: string; content: string }> = []
      if (systemPrompt) {
        messagesArray.push({ role: 'system', content: systemPrompt })
      }
      for (const m of safeHistory as unknown[]) {
        const msg = m as { role: string; content: string }
        messagesArray.push({ role: msg.role, content: msg.content })
      }
      messagesArray.push({ role: 'user', content: text.trim() })

      // 创建 AbortController
      const abortController = new AbortController()
      abortControllersByAgent.current[targetAgentId] = abortController

      // 调用 AI
      const tauriResult = await invoke<{
        success: boolean
        result?: {
          result?: string
          final_answer?: string
          content?: string
          summary?: string
        }
        timestamp?: number
      }>('execute_ai_conversation', {
        agentConfig: {
          id: localApiConfig.id || 'primary',
          provider: localApiConfig.provider,
          api_key: localApiConfig.api_key,
          base_url: localApiConfig.base_url ?? null,
          model: localApiConfig.model ?? null,
          is_active: true,
        },
        message: text.trim(),
        messages: messagesArray,
        options: { stream: false },
        agentId: agentInfo?.id || targetAgentId,
        sessionId,
        // 🔥 传递系统提示到后端（优先使用单独的参数）
        systemPrompt,
      })

      // 处理响应
      if (tauriResult?.success) {
        const result = tauriResult.result
        const content =
          result?.result ||
          result?.final_answer ||
          result?.content ||
          result?.summary ||
          '✅ 任务已完成'

        const assistantMessage: Message = {
          id: `assistant_${Date.now()}_${targetAgentId}`,
          type: 'assistant',
          content,
          timestamp: tauriResult.timestamp ? tauriResult.timestamp * 1000 : Date.now(),
          source: 'local',
          agentId: targetAgentId,
        }
        appendMessage(assistantMessage, targetAgentId)
      } else {
        appendMessage({
          id: `error_${Date.now()}`,
          type: 'assistant',
          content: '❌ AI 执行失败',
          timestamp: Date.now(),
          source: 'error',
          agentId: targetAgentId,
        }, targetAgentId)
      }
    } catch (error) {
      const err = error as { name?: string; message?: string }

      if (err.name === 'AbortError' || err.name === 'CanceledError') {
        appendMessage({
          id: `cancel_${Date.now()}`,
          type: 'assistant',
          content: '⏹️ 执行已终止',
          timestamp: Date.now(),
          source: 'cancel',
        }, targetAgentId)
        return
      }

      appendMessage({
        id: `error_${Date.now()}`,
        type: 'assistant',
        content: `❌ 执行出错：${err.message || '未知错误'}`,
        timestamp: Date.now(),
        source: 'error',
      }, targetAgentId)
    } finally {
      delete abortControllersByAgent.current[targetAgentId]
      setAgentLoading(targetAgentId, false)
    }
  }, [
    sessionId,
    appendMessage,
    setAgentLoading,
    getActiveApiConfig,
    getSystemPromptForAgent,
    getMessageHistory,
    systemPromptCache,
    setSystemPromptCache,
    selectedAgent,
    currentMode,
    walletAddress,
    activeChain,
    maxTokens,
  ])

  /**
   * 取消智能体执行
   */
  const cancelAgentExecution = useCallback((agentId: string) => {
    const controller = abortControllersByAgent.current[agentId]
    if (controller) {
      controller.abort()
      delete abortControllersByAgent.current[agentId]
      console.log('[cancelAgentExecution] 已取消智能体执行:', agentId)
    }
    
    // 同时取消可能的轮询任务（如果有）
    if (typeof window !== 'undefined') {
      // 清理可能的 setTimeout
      const timeoutId = (window as any)[`polling_${agentId}`]
      if (timeoutId) {
        clearTimeout(timeoutId)
        delete (window as any)[`polling_${agentId}`]
      }
    }
    
    console.log('[cancelAgentExecution] 已清理智能体相关任务:', agentId)
  }, [])

  return {
    sendMessageToAgent,
    cancelAgentExecution,
  }
}
