import { useCallback, useRef } from 'react'
import apiClient from '@/services/api'
import LoadingIcon from '@/assets/加载0.2.png'

// Tauri invoke 类型
interface LocalToolResult {
  success: boolean;
  data?: any;
  error?: string;
  execution_time_ms?: number;
  output?: string;
  warnings?: string[];
}

// 工具调用类型
interface ToolCall {
  id: string;
  tool: string;
  arguments: Record<string, unknown>;
}

// 工具结果类型
interface ToolResult {
  tool: string;
  success: boolean;
  result?: unknown;
  error?: string;
  arguments: Record<string, unknown>;
  timestamp: number;
  tool_call_id?: string;
}

// 任务状态类型
type TaskStatus = 'queued' | 'pending' | 'processing' | 'running' | 'completed' | 'failed';

// 任务结果类型
interface TaskResult {
  status: TaskStatus;
  progress?: number;
  current_step?: string;
  error?: string;
  result?: {
    response?: unknown;
  };
}

// 轮询选项类型
interface PollingOptions {
  interval?: number;
  timeout?: number;
}

// Hook参数类型
interface UseAsyncTaskPollingParams {
  appendMessage: (message: unknown, agentId: string) => void;
  scrollToBottom: () => void;
  setAgentLoading: (agentId: string, loading: boolean) => void;
  setMessagesByChannel: React.Dispatch<React.SetStateAction<Record<string, unknown[]>>>;
  walletAddress?: string | null;
  chain?: string | null;
}

// 轮询状态类型
interface PollingStatus {
  taskId: string;
  messageId: string;
  startTime: number;
  fastMode?: boolean;
}

export const useAsyncTaskPolling = ({
  appendMessage,
  scrollToBottom,
  setAgentLoading,
  setMessagesByChannel,
}: UseAsyncTaskPollingParams) => {
  const pollingTimeoutsByAgent = useRef<Record<string, number>>({})
  const pollingStatusByAgent = useRef<Record<string, PollingStatus>>({})

  const executeToolCallsAndSubmitResults = useCallback(async (taskId: string, toolCalls: ToolCall[], agentId: string) => {
    console.log(`[pollAsyncTask] 执行 ${toolCalls.length} 个工具调用`, toolCalls)

    try {
      const toolResults: ToolResult[] = []
      for (const toolCall of toolCalls) {
        try {
          console.log(`[pollAsyncTask] 执行工具: ${toolCall.tool}`, toolCall.arguments)

          const { invoke } = await import('@tauri-apps/api/core')
          const toolResponse = await invoke<LocalToolResult>('execute_tool', {
            toolId: toolCall.tool,
            args: JSON.stringify(toolCall.arguments),
            timeout: 30000
          })

          toolResults.push({
            tool: toolCall.tool,
            success: toolResponse.success || false,
            result: toolResponse.data || toolResponse.output || '工具执行完成',
            error: toolResponse.error,
            arguments: toolCall.arguments,
            timestamp: Date.now(),
            tool_call_id: toolCall.id,
          })
        } catch (toolError) {
          console.error(`[pollAsyncTask] 工具执行失败: ${toolCall.tool}`, toolError)
          toolResults.push({
            tool: toolCall.tool,
            success: false,
            error: (toolError as Error).message,
            arguments: toolCall.arguments,
            timestamp: Date.now(),
          })
        }
      }

      console.log(`[pollAsyncTask] 提交工具结果到任务 ${taskId}`)
      await apiClient.post(`/ai-task/${taskId}/tool-result`, {
        results: toolResults,
        timestamp: Date.now(),
      })

      console.log(`[pollAsyncTask] 工具结果已提交，启用快速轮询模式`)
      
      const status = pollingStatusByAgent.current[agentId]
      if (status) {
        status.fastMode = true
      }
    } catch (error) {
      console.error(`[pollAsyncTask] 提交工具结果失败:`, error)
      throw error
    }
  }, [])

  const pollAsyncTask = useCallback((taskId: string, progressMessageId: string | null, agentId: string, options: PollingOptions = {}) => {
    const { interval = 1000, timeout = 120000 } = options

    // 清理之前的轮询
    const existingTimeout = pollingTimeoutsByAgent.current[agentId]
    if (existingTimeout) {
      clearTimeout(existingTimeout)
      delete pollingTimeoutsByAgent.current[agentId]
    }

    const startTime = Date.now()
    let pollCount = 0

    pollingStatusByAgent.current[agentId] = {
      taskId,
      messageId: progressMessageId || '',
      startTime,
      fastMode: false,
    }

    console.log(`[pollAsyncTask] 开始轮询任务 ${taskId}，智能体: ${agentId}`)

    const doPoll = async () => {
      try {
        if (Date.now() - startTime > timeout) {
          throw new Error('任务执行超时')
        }

        const currentStatus = pollingStatusByAgent.current[agentId]
        if (!currentStatus || currentStatus.taskId !== taskId) {
          console.log(`[pollAsyncTask] 任务 ${taskId} 已停止轮询`)
          return
        }

        const apiResponse = await apiClient.get(`/ai-task/${taskId}/status`)
        const result = apiResponse.data as TaskResult

        const { status, progress = 0, current_step, error, result: taskResult } = result
        const taskResponse = taskResult?.response

        console.log(`[pollAsyncTask] 第 ${++pollCount} 次轮询 - 状态: ${status}, 进度: ${progress}%, 步骤: ${current_step}`)

        // 更新进度消息
        if (progressMessageId) {
          setMessagesByChannel((prev) => {
            const channelMessages = prev[agentId] || []
            const messageIndex = channelMessages.findIndex((msg: unknown) => 
              typeof msg === 'object' && msg !== null && 'id' in msg && (msg as { id: string }).id === progressMessageId
            )

            if (messageIndex !== -1) {
              const updatedMessages = [...channelMessages]
              const progressMessage = { ...updatedMessages[messageIndex] as Record<string, unknown> }

              let content = `<img src="${LoadingIcon}" alt="加载中" class="loading-icon" />`
              if (current_step) {
                content += ` ${current_step}`
              } else {
                content += ' 任务正在执行中'
              }
              if (progress > 0) {
                content += ` (${Math.round(progress * 100)}%)`
              }

              progressMessage.content = content
              progressMessage.progress = progress
              progressMessage.status = status
              progressMessage.currentStep = current_step

              updatedMessages[messageIndex] = progressMessage
              return { ...prev, [agentId]: updatedMessages }
            }
            return prev
          })
        }

        // 检查是否有待处理的工具调用
        if (status === 'processing') {
          try {
            const toolCallsApiResponse = await apiClient.get(`/ai-task/${taskId}/pending-tools`)
            const toolCalls = (toolCallsApiResponse.data as { toolCalls?: ToolCall[] })?.toolCalls || []

            if (toolCalls.length > 0) {
              console.log(`[pollAsyncTask] 发现 ${toolCalls.length} 个待处理工具调用`)
              await executeToolCallsAndSubmitResults(taskId, toolCalls, agentId)
            }
          } catch (toolError) {
            console.log(`[pollAsyncTask] 获取待处理工具调用失败:`, (toolError as Error).message)
          }
        }

        // 任务完成或失败
        if (status === 'completed' || status === 'failed') {
          delete pollingTimeoutsByAgent.current[agentId]
          delete pollingStatusByAgent.current[agentId]

          if (status === 'completed' && taskResponse) {
            if (progressMessageId) {
              setMessagesByChannel((prev) => {
                const channelMessages = prev[agentId] || []
                const messageIndex = channelMessages.findIndex((msg: unknown) =>
                  typeof msg === 'object' && msg !== null && 'id' in msg && (msg as { id: string }).id === progressMessageId
                )

                if (messageIndex !== -1) {
                  const updatedMessages = [...channelMessages]
                  const completedMessage = { ...updatedMessages[messageIndex] as Record<string, unknown> }
                  completedMessage.content = String(taskResponse)
                  completedMessage.progress = 1
                  completedMessage.status = 'completed'
                  updatedMessages[messageIndex] = completedMessage
                  return { ...prev, [agentId]: updatedMessages }
                }
                return prev
              })
            } else {
              appendMessage({
                id: `assistant_${Date.now()}_${agentId}`,
                type: 'assistant',
                content: String(taskResponse),
                timestamp: Date.now(),
                source: 'alou-edge',
                agentId: agentId,
              }, agentId)
            }
          } else if (status === 'failed') {
            const errorContent = `❌ 任务执行失败: ${error || '未知错误'}`
            if (progressMessageId) {
              setMessagesByChannel((prev) => {
                const channelMessages = prev[agentId] || []
                const messageIndex = channelMessages.findIndex((msg: unknown) =>
                  typeof msg === 'object' && msg !== null && 'id' in msg && (msg as { id: string }).id === progressMessageId
                )

                if (messageIndex !== -1) {
                  const updatedMessages = [...channelMessages]
                  const failedMessage = { ...updatedMessages[messageIndex] as Record<string, unknown> }
                  failedMessage.content = errorContent
                  failedMessage.progress = 1
                  failedMessage.status = 'failed'
                  failedMessage.error = error
                  updatedMessages[messageIndex] = failedMessage
                  return { ...prev, [agentId]: updatedMessages }
                }
                return prev
              })
            } else {
              appendMessage({
                id: `error_${Date.now()}_${agentId}`,
                type: 'assistant',
                content: errorContent,
                timestamp: Date.now(),
                source: 'error',
                agentId: agentId,
              }, agentId)
            }
          }

          setAgentLoading(agentId, false)
          scrollToBottom()
          return
        }

        // 继续轮询
        const isFastMode = currentStatus?.fastMode
        const nextDelay = isFastMode ? 200 : interval
        
        if (status !== 'processing') {
          currentStatus.fastMode = false
        }
        
        const timeoutId = window.setTimeout(doPoll, nextDelay)
        pollingTimeoutsByAgent.current[agentId] = timeoutId
      } catch (error) {
        console.error(`[pollAsyncTask] 轮询失败:`, error)
        delete pollingTimeoutsByAgent.current[agentId]
        delete pollingStatusByAgent.current[agentId]
        setAgentLoading(agentId, false)
        scrollToBottom()
      }
    }

    // 开始第一次轮询
    const initialTimeoutId = window.setTimeout(doPoll, 500)
    pollingTimeoutsByAgent.current[agentId] = initialTimeoutId
  }, [appendMessage, scrollToBottom, setAgentLoading, setMessagesByChannel, executeToolCallsAndSubmitResults])

  const cancelPolling = useCallback((agentId: string) => {
    const timeoutId = pollingTimeoutsByAgent.current[agentId]
    if (timeoutId) {
      console.log('[useAsyncTaskPolling] 终止智能体轮询:', agentId)
      clearTimeout(timeoutId)
      delete pollingTimeoutsByAgent.current[agentId]
      delete pollingStatusByAgent.current[agentId]
    }
  }, [])

  return {
    pollAsyncTask,
    cancelPolling,
  }
}

export default useAsyncTaskPolling
