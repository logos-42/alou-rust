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

/**
 * 转换参数格式为 Rust 后端期望的格式
 *
 * Rust 后端期望的格式：
 * - FileSystem: { operation: "list"|"read"|"write"|..., path: string, ... }
 * - Bash: { operation: "execute", shell: "bash"|"cmd"|"powershell", command: string, ... }
 * - Search: { operation: "grep"|"search", pattern: string, directory: string, ... }
 */
function normalizeToolArguments(
  toolId: string,
  args: Record<string, unknown>
): Record<string, unknown> {
  const normalizedArgs: Record<string, unknown> = { ...args }

  // 如果已经有 operation 字段，说明格式已经正确，但仍需确保其他字段完整
  const hasOperation = normalizedArgs.operation !== undefined

  // FileSystem Tool 参数转换
  if (toolId === 'filesystem') {
    // 确保必要的默认值
    if (normalizedArgs.recursive === undefined) {
      normalizedArgs.recursive = false
    }
    if (normalizedArgs.create_dirs === undefined) {
      normalizedArgs.create_dirs = false
    }

    return normalizedArgs
  }

  // Bash Tool 参数转换
  if (toolId === 'bash') {
    // 确保有 operation 字段 - 使用 Rust 枚举格式（小写）
    if (normalizedArgs.operation === undefined) {
      normalizedArgs.operation = 'execute'
    } else {
      // 转换 operation 为小写
      normalizedArgs.operation = String(normalizedArgs.operation).toLowerCase()
    }

    // 确保有 shell 字段 - 转换为 Rust 枚举小写格式
    if (normalizedArgs.shell === undefined) {
      normalizedArgs.shell = 'bash'
    } else if (typeof normalizedArgs.shell === 'string') {
      const shellMap: Record<string, string> = {
        'bash': 'bash',
        'cmd': 'cmd',
        'powershell': 'powershell',
        'python': 'python',
        'node': 'node'
      }
      normalizedArgs.shell = shellMap[normalizedArgs.shell.toLowerCase()] || 'bash'
    }

    // 确保有 command 字段
    if (normalizedArgs.command === undefined) {
      // 尝试从其他字段推断
      if ((normalizedArgs as any).cmd !== undefined) {
        normalizedArgs.command = (normalizedArgs as any).cmd
      } else if ((normalizedArgs as any).script !== undefined) {
        normalizedArgs.command = (normalizedArgs as any).script
      }
    }

    // 确保有 timeout_seconds 字段
    if (normalizedArgs.timeout_seconds === undefined) {
      normalizedArgs.timeout_seconds = 30
    }

    // 确保 environment 是数组
    if (normalizedArgs.environment === undefined) {
      normalizedArgs.environment = []
    }

    return normalizedArgs
  }

  // Search Tool 参数转换
  if (toolId === 'search') {
    // 如果没有 operation 字段，根据参数推断 - 使用 Rust 枚举格式（小写）
    if (!hasOperation) {
      normalizedArgs.operation = 'grep'
    } else {
      // 转换 operation 为小写
      normalizedArgs.operation = String(normalizedArgs.operation).toLowerCase()
    }

    // 将 query 转换为 pattern
    if (normalizedArgs.pattern === undefined && (normalizedArgs as any).query !== undefined) {
      normalizedArgs.pattern = (normalizedArgs as any).query
      delete (normalizedArgs as any).query
    }

    // 将 path 转换为 directory
    if (normalizedArgs.directory === undefined && (normalizedArgs as any).path !== undefined) {
      normalizedArgs.directory = (normalizedArgs as any).path
      delete (normalizedArgs as any).path
    }

    // 确保有 directory 字段（默认为当前目录）
    if (normalizedArgs.directory === undefined) {
      normalizedArgs.directory = '.'
    }

    return normalizedArgs
  }

  // 其他工具保持原样
  return normalizedArgs
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
  const toolRetryCounts = useRef<Record<string, number>>({})

  const executeToolCallsAndSubmitResults = useCallback(async (taskId: string, toolCalls: ToolCall[], agentId: string, retryCount: number = 0) => {
    const MAX_RETRY_COUNT = 3 // 最大重试次数
    console.log(`[pollAsyncTask] 执行 ${toolCalls.length} 个工具调用 (重试次数：${retryCount})`, toolCalls)

    try {
      const toolResults: ToolResult[] = []
      let hasFailure = false
      
      for (const toolCall of toolCalls) {
        // 检查是否是媒体工具
        const isMediaTool = ['generate_image', 'generate_audio', 'generate_video', 'get_video_status'].includes(toolCall.tool)

        if (isMediaTool) {
          // 使用 HTTP API 执行媒体工具
          try {
            console.log(`[MediaTool] 执行媒体工具：${toolCall.tool}`, toolCall.arguments)
            
            const response = await apiClient.post('/api/media/generate', {
              tool: toolCall.tool,
              args: toolCall.arguments,
              timeout: 300000 // 媒体生成可能需要更长时间
            })
            
            const result = {
              tool: toolCall.tool,
              success: response.data.success || false,
              result: response.data.result || response.data,
              error: response.data.error,
              arguments: toolCall.arguments,
              timestamp: Date.now(),
              tool_call_id: toolCall.id,
            }
            
            if (!result.success) {
              hasFailure = true
              console.error(`[MediaTool] 媒体工具执行失败：${toolCall.tool}`, result.error)
            }
            
            toolResults.push(result)
            continue // 继续下一个工具
          } catch (mediaError) {
            console.error(`[MediaTool] 媒体工具执行失败：${toolCall.tool}`, mediaError)
            toolResults.push({
              tool: toolCall.tool,
              success: false,
              error: (mediaError as Error).message,
              arguments: toolCall.arguments,
              timestamp: Date.now(),
              tool_call_id: toolCall.id,
            })
            continue
          }
        }

        try {
          // 使用 normalizeToolArguments 转换参数格式
          const normalizedArgs = normalizeToolArguments(toolCall.tool, toolCall.arguments)
          
          console.log(`[pollAsyncTask] 执行工具: ${toolCall.tool}`)
          console.log(`[pollAsyncTask] 原始参数:`, JSON.stringify(toolCall.arguments, null, 2))
          console.log(`[pollAsyncTask] 转换后参数:`, JSON.stringify(normalizedArgs, null, 2))

          const { invoke } = await import('@tauri-apps/api/core')
          const toolResponse = await invoke<LocalToolResult>('execute_tool', {
            toolId: toolCall.tool,
            args: JSON.stringify(normalizedArgs),
            timeout: 30000
          })

          const result = {
            tool: toolCall.tool,
            success: toolResponse.success || false,
            result: toolResponse.data || toolResponse.output || '工具执行完成',
            error: toolResponse.error,
            arguments: normalizedArgs,
            timestamp: Date.now(),
            tool_call_id: toolCall.id,
          }
          
          if (!result.success) {
            hasFailure = true
            console.error(`[pollAsyncTask] 工具执行失败：${toolCall.tool}`, result.error)
          }
          
          toolResults.push(result)
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

      // 如果所有工具都失败且超过最大重试次数，不再重试
      if (hasFailure && retryCount >= MAX_RETRY_COUNT) {
        console.error(`[pollAsyncTask] 达到最大重试次数 (${MAX_RETRY_COUNT})，停止重试`)
        toolResults.push({
          tool: 'system',
          success: false,
          error: `工具执行失败，已达到最大重试次数 ${MAX_RETRY_COUNT}。请检查参数或尝试其他方法。`,
          arguments: {},
          timestamp: Date.now(),
        })
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
              // 获取当前重试次数
              const currentRetryCount = toolRetryCounts.current[taskId] || 0
              console.log(`[pollAsyncTask] 工具调用重试次数：${currentRetryCount}`)
              await executeToolCallsAndSubmitResults(taskId, toolCalls, agentId, currentRetryCount)
              
              // 增加重试计数
              toolRetryCounts.current[taskId] = currentRetryCount + 1
            }
          } catch (toolError) {
            console.log(`[pollAsyncTask] 获取待处理工具调用失败:`, (toolError as Error).message)
          }
        }

        // 任务完成或失败
        if (status === 'completed' || status === 'failed') {
          delete pollingTimeoutsByAgent.current[agentId]
          delete pollingStatusByAgent.current[agentId]
          delete toolRetryCounts.current[taskId]

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
    
    // 同时清理 window 上的 timeout 引用（如果有）
    if (typeof window !== 'undefined') {
      const windowTimeoutId = (window as any)[`polling_${agentId}`]
      if (windowTimeoutId) {
        clearTimeout(windowTimeoutId)
        delete (window as any)[`polling_${agentId}`]
      }
    }
  }, [])

  return {
    pollAsyncTask,
    cancelPolling,
  }
}

export default useAsyncTaskPolling
