import { useCallback, useRef } from 'react'
import apiClient from '@/services/api'
import LoadingIcon from '@/assets/加载0.2.png'

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
}

// 轮询状态类型
interface PollingStatus {
  taskId: string;
  messageId: string;
  startTime: number;
}

export const useAsyncTaskPolling = ({
  appendMessage,
  scrollToBottom,
  setAgentLoading,
  setMessagesByChannel,
}: UseAsyncTaskPollingParams) => {
  // 按智能体存储轮询定时器：Map<agentId, intervalId>
  const pollingIntervalsByAgent = useRef<Record<string, number>>({})

  // 按智能体存储轮询状态：Map<agentId, { taskId, messageId, startTime }>
  const pollingStatusByAgent = useRef<Record<string, PollingStatus>>({})

  /**
   * 执行工具调用并提交结果
   */
  const executeToolCallsAndSubmitResults = useCallback(async (taskId: string, toolCalls: ToolCall[], agentId: string) => {
    console.log(`[pollAsyncTask] 执行 ${toolCalls.length} 个工具调用`, toolCalls)

    try {
      // 执行所有工具调用
      const toolResults: ToolResult[] = []
      for (const toolCall of toolCalls) {
        try {
          console.log(`[pollAsyncTask] 执行工具: ${toolCall.tool}`, toolCall.arguments)

          // 调用实际的工具执行API
          const toolResponse = await apiClient.post('/mcp/execute-tool', {
            tool_id: toolCall.tool,
            args: toolCall.arguments,
            session_id: `task_${taskId}`,
            timeout_seconds: 30,
          })

          const toolResult: ToolResult = {
            tool: toolCall.tool,
            success: toolResponse.data.success || false,
            result: toolResponse.data.data || toolResponse.data.result || '工具执行完成',
            error: toolResponse.data.error,
            arguments: toolCall.arguments,
            timestamp: Date.now(),
            tool_call_id: toolCall.id,
          }

          toolResults.push(toolResult)
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

      // 提交工具结果
      console.log(`[pollAsyncTask] 提交工具结果到任务 ${taskId}`)
      await apiClient.post(`/ai-task/${taskId}/tool-result`, {
        results: toolResults,
        timestamp: Date.now(),
      })

      console.log(`[pollAsyncTask] 工具结果已提交`)
    } catch (error) {
      console.error(`[pollAsyncTask] 提交工具结果失败:`, error)
      throw error
    }
  }, [])

  /**
   * 轮询异步任务状态
   */
  const pollAsyncTask = useCallback((taskId: string, progressMessageId: string | null, agentId: string, options: PollingOptions = {}) => {
    const {
      interval = 1000,
      timeout = 120000,
    } = options

    // 清理该智能体之前的轮询
    const existingInterval = pollingIntervalsByAgent.current[agentId]
    if (existingInterval) {
      clearInterval(existingInterval)
      delete pollingIntervalsByAgent.current[agentId]
    }

    const startTime = Date.now()
    let pollCount = 0

    // 记录轮询状态
    pollingStatusByAgent.current[agentId] = {
      taskId,
      messageId: progressMessageId || '',
      startTime,
    }

    console.log(`[pollAsyncTask] 开始轮询任务 ${taskId}，智能体: ${agentId}`)

    const intervalId = window.setInterval(async () => {
      try {
        // 检查超时
        if (Date.now() - startTime > timeout) {
          throw new Error('任务执行超时')
        }

        // 查询任务状态
        const response = await apiClient.get(`/ai-task/${taskId}/status`)
        const result: TaskResult = response.data

        const { status, progress = 0, current_step, error, result: taskResult } = result
        const taskResponse = taskResult?.response

        console.log(`[pollAsyncTask] 第 ${++pollCount} 次轮询 - 状态: ${status}, 进度: ${progress}%, 步骤: ${current_step}, 错误: ${error}`)

        // 检查是否有待处理的工具调用
        if (status === 'processing') {
           try {
             const toolCallsResponse = await apiClient.get(`/ai-task/${taskId}/pending-tools`)
             const toolCalls: ToolCall[] = toolCallsResponse.data?.toolCalls || []

             if (toolCalls.length > 0) {
               console.log(`[pollAsyncTask] 发现 ${toolCalls.length} 个待处理工具调用:`, toolCalls)
               await executeToolCallsAndSubmitResults(taskId, toolCalls, agentId)
               console.log(`[pollAsyncTask] 工具结果已提交，立即进行下次状态检查`)
             }
           } catch (toolError) {
             console.log(`[pollAsyncTask] 获取待处理工具调用失败，继续轮询:`, (toolError as Error).message)
           }
         }

        // 更新进度消息
        setMessagesByChannel((prev) => {
          const channelMessages = prev[agentId] || []
          const messageIndex = channelMessages.findIndex((msg: { id: string }) => msg.id === progressMessageId)

          if (messageIndex !== -1) {
            const updatedMessages = [...channelMessages]
            const progressMessage = { ...updatedMessages[messageIndex] }

            progressMessage.progress = progress
            progressMessage.status = status
            progressMessage.currentStep = current_step

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

            updatedMessages[messageIndex] = progressMessage
            return {
              ...prev,
              [agentId]: updatedMessages,
            }
          }

          return prev
        })

        console.log(`[POLL_DEBUG] === 开始处理状态 ===`)
        console.log(`[POLL_DEBUG] 任务ID: ${taskId}`)
        console.log(`[POLL_DEBUG] 状态: "${status}" (类型: ${typeof status})`)
        console.log(`[POLL_DEBUG] 进度: ${progress}`)
        console.log(`[POLL_DEBUG] 轮询计数: ${pollCount}`)
        console.log(`[POLL_DEBUG] AgentID: ${agentId}`)

        switch (status) {
           case 'queued':
           case 'pending':
           case 'processing':
           case 'running':
             console.log(`[POLL_DEBUG] 状态 "${status}" 继续轮询`)
             
             if (progressMessageId) {
               setMessagesByChannel((prev) => {
                 const channelMessages = prev[agentId] || []
                 const messageIndex = channelMessages.findIndex((msg: { id: string }) => msg.id === progressMessageId)
                 
                 if (messageIndex !== -1 && current_step) {
                   const updatedMessages = [...channelMessages]
                   const progressMessage = { ...updatedMessages[messageIndex] }
                   
                   progressMessage.content = `🔄 ${current_step}${progress > 0 ? ` (${Math.round(progress * 100)}%)` : ''}`
                   
                   updatedMessages[messageIndex] = progressMessage
                   return {
                     ...prev,
                     [agentId]: updatedMessages,
                   }
                 }
                 
                 return prev
               })
             }
             break

           case 'completed':
             console.log(`[POLL_DEBUG] 🎯 检测到完成状态，执行清理逻辑`)
             clearInterval(intervalId)
             delete pollingIntervalsByAgent.current[agentId]
             delete pollingStatusByAgent.current[agentId]
             console.log(`[POLL_DEBUG] ✅ 任务完成: ${taskId}`)

            if (progressMessageId && taskResponse) {
              setMessagesByChannel((prev) => {
                const channelMessages = prev[agentId] || []
                const messageIndex = channelMessages.findIndex((msg: { id: string }) => msg.id === progressMessageId)

                if (messageIndex !== -1) {
                  const updatedMessages = [...channelMessages]
                  const completedMessage = { ...updatedMessages[messageIndex] }

                  completedMessage.content = String(taskResponse)
                  completedMessage.progress = 1
                  completedMessage.status = 'completed'

                  updatedMessages[messageIndex] = completedMessage
                  return {
                    ...prev,
                    [agentId]: updatedMessages,
                  }
                }

                return prev
              })
            } else if (taskResponse) {
              const responseMessage = {
                id: `assistant_${Date.now()}_${agentId}`,
                type: 'assistant',
                content: String(taskResponse),
                timestamp: Date.now(),
                source: 'alou-edge',
                agentId: agentId,
              }
              appendMessage(responseMessage, agentId)
            }

            setAgentLoading(agentId, false)
            scrollToBottom()
            break

          case 'failed':
            console.log(`[POLL_DEBUG] ❌ 检测到失败状态，执行清理逻辑`)
            clearInterval(intervalId)
            delete pollingIntervalsByAgent.current[agentId]
            delete pollingStatusByAgent.current[agentId]
            console.error(`[pollAsyncTask] 任务失败: ${taskId}, 错误: ${error}`)
            console.log(`[POLL_DEBUG] ❌ 任务失败: ${taskId}`)

            if (progressMessageId) {
              setMessagesByChannel((prev) => {
                const channelMessages = prev[agentId] || []
                const messageIndex = channelMessages.findIndex((msg: { id: string }) => msg.id === progressMessageId)

                if (messageIndex !== -1) {
                  const updatedMessages = [...channelMessages]
                  const failedMessage = { ...updatedMessages[messageIndex] }

                  failedMessage.content = `❌ 任务执行失败: ${error || '未知错误'}`
                  failedMessage.progress = 1
                  failedMessage.status = 'failed'
                  failedMessage.error = error

                  updatedMessages[messageIndex] = failedMessage
                  return {
                    ...prev,
                    [agentId]: updatedMessages,
                  }
                }

                return prev
              })
            } else {
              appendMessage({
                id: `error_${Date.now()}_${agentId}`,
                type: 'assistant',
                content: `❌ 任务执行失败: ${error || '未知错误'}`,
                timestamp: Date.now(),
                source: 'error',
                agentId: agentId,
              }, agentId)
            }

            setAgentLoading(agentId, false)
            scrollToBottom()
            break

          default:
            console.warn(`[pollAsyncTask] 未知任务状态: ${status}`)
        }
      } catch (error) {
        console.error(`[pollAsyncTask] 轮询失败:`, error)

        clearInterval(intervalId)
        delete pollingIntervalsByAgent.current[agentId]
        delete pollingStatusByAgent.current[agentId]

        if (progressMessageId) {
          setMessagesByChannel((prev) => {
            const channelMessages = prev[agentId] || []
            const messageIndex = channelMessages.findIndex((msg: { id: string }) => msg.id === progressMessageId)

            if (messageIndex !== -1) {
              const updatedMessages = [...channelMessages]
              const errorMessage = { ...updatedMessages[messageIndex] }

              errorMessage.content = `❌ 任务执行出错: ${(error as Error).message}`
              errorMessage.progress = 1
              errorMessage.status = 'error'
              errorMessage.error = (error as Error).message

              updatedMessages[messageIndex] = errorMessage
              return {
                ...prev,
                [agentId]: updatedMessages,
              }
            }

            return prev
          })
        } else {
          appendMessage({
            id: `error_${Date.now()}_${agentId}`,
            type: 'assistant',
            content: `❌ 任务执行出错: ${(error as Error).message}`,
            timestamp: Date.now(),
            source: 'error',
            agentId: agentId,
          }, agentId)
        }

        setAgentLoading(agentId, false)
        scrollToBottom()
      }
    }, interval)

    pollingIntervalsByAgent.current[agentId] = intervalId

    const immediatePoll = window.setTimeout(async () => {
      try {
        const response = await apiClient.get(`/ai-task/${taskId}/status`)
        const result: TaskResult = response.data

        if (result.status) {
          const { status, progress = 0 } = result
          console.log(`[pollAsyncTask] 首次轮询 - 状态: ${status}, 进度: ${progress}%`)
        }
      } catch (error) {
        console.error(`[pollAsyncTask] 首次轮询失败:`, error)
      }
    }, 100)
    
    pollingIntervalsByAgent.current[`${agentId}_immediate`] = immediatePoll
  }, [appendMessage, scrollToBottom, setAgentLoading, setMessagesByChannel, executeToolCallsAndSubmitResults])

  /**
   * 终止指定智能体的轮询
   */
  const cancelPolling = useCallback((agentId: string) => {
    const intervalId = pollingIntervalsByAgent.current[agentId]
    if (intervalId) {
      console.log('[useAsyncTaskPolling] 终止智能体轮询:', agentId)
      clearInterval(intervalId)
      delete pollingIntervalsByAgent.current[agentId]
      delete pollingStatusByAgent.current[agentId]
    }
  }, [])

  return {
    pollAsyncTask,
    cancelPolling,
  }
}

export default useAsyncTaskPolling
