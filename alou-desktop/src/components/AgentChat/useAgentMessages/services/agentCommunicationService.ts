/**
 * 智能体通信服务
 * 
 * @module components/AgentChat/useAgentMessages/services/agentCommunicationService
 */

import { invoke } from '@tauri-apps/api/core'
import { AgentExecutionResult } from '../types'

interface AgentCommunicationServiceConfig {
  sessionId: string
  maxTokens?: number
}

interface APIConfig {
  id?: string
  provider: string
  api_key: string
  base_url?: string | null
  model?: string | null
  is_active: boolean
}

interface TauriResult {
  success: boolean
  result?: {
    result?: string
    final_answer?: string
    content?: string
    summary?: string
    error?: string
    task_id?: string
  }
  timestamp?: number
  execution_mode?: string
}

/**
 * 智能体通信服务
 */
export function createAgentCommunicationService({
  sessionId,
  maxTokens = 4000,
}: AgentCommunicationServiceConfig) {
  const abortControllers = new Map<string, AbortController>()

  /**
   * 构建消息数组
   */
  function buildMessagesArray(
    systemPrompt: string,
    history: unknown[],
    userMessage: string
  ): Array<{ role: string; content: string }> {
    const messages: Array<{ role: string; content: string }> = []

    if (systemPrompt) {
      messages.push({ role: 'system', content: systemPrompt })
    }

    // 添加历史消息
    const safeHistory = Array.isArray(history)
      ? history.slice(-Math.max(3, Math.min(50, Math.floor((maxTokens - 1500) / 100))))
      : []

    for (const msg of safeHistory) {
      const m = msg as { role?: string; content?: string }
      if (m.role && m.content) {
        messages.push({ role: m.role, content: m.content })
      }
    }

    // 添加当前用户消息
    messages.push({ role: 'user', content: userMessage })

    return messages
  }

  /**
   * 执行 AI 对话
   */
  async function executeConversation(
    apiConfig: APIConfig,
    messages: Array<{ role: string; content: string }>,
    agentId: string,
    // 🔥 新增：系统提示参数
    systemPrompt?: string
  ): Promise<AgentExecutionResult> {
    const controller = new AbortController()
    abortControllers.set(agentId, controller)

    try {
      const result = await invoke<TauriResult>('execute_ai_conversation', {
        agentConfig: {
          id: apiConfig.id || 'primary',
          provider: apiConfig.provider,
          api_key: apiConfig.api_key,
          base_url: apiConfig.base_url ?? null,
          model: apiConfig.model ?? null,
          is_active: true,
        },
        messages,
        options: { stream: false },
        agentId,
        sessionId,
        // 🔥 传递系统提示到后端
        systemPrompt,
      })

      if (result?.success) {
        const content =
          result.result?.result ||
          result.result?.final_answer ||
          result.result?.content ||
          result.result?.summary ||
          '✅ 任务已完成'

        return {
          success: true,
          content,
          taskId: result.result?.task_id,
        }
      } else {
        return {
          success: false,
          error: 'AI 执行失败',
        }
      }
    } catch (error) {
      const err = error as { name?: string; message?: string }

      if (err.name === 'AbortError' || err.name === 'CanceledError') {
        return {
          success: false,
          error: '执行已取消',
        }
      }

      return {
        success: false,
        error: err.message || '未知错误',
      }
    } finally {
      abortControllers.delete(agentId)
    }
  }

  /**
   * 取消执行
   */
  function cancelExecution(agentId: string): boolean {
    const controller = abortControllers.get(agentId)
    if (controller) {
      controller.abort()
      abortControllers.delete(agentId)
      console.log('[AgentCommunicationService] 已取消执行:', agentId)
      return true
    }
    return false
  }

  /**
   * 清理所有执行
   */
  function cleanup(): void {
    for (const [agentId, controller] of abortControllers.entries()) {
      controller.abort()
      console.log('[AgentCommunicationService] 清理执行:', agentId)
    }
    abortControllers.clear()
  }

  return {
    buildMessagesArray,
    executeConversation,
    cancelExecution,
    cleanup,
  }
}
